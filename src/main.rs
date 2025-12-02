// ============================================================================
// DupFinder - 重复文件查找工具
// 模仿 fdupes 的 4 层验证流程
// ============================================================================

use chrono::Local;
use clap::Parser;
use colored::*;
use globset::{Glob, GlobSet, GlobSetBuilder};
use regex::Regex;
use same_file::is_same_file;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

// ============================================================================
// 【Rust 概念 1: 命令行参数结构】
// ============================================================================
#[derive(Parser, Debug)]
#[command(
    name = "dupfinder",
    about = "快速查找重复文件 - 4 层验证",
    long_about = "使用多层验证策略快速准确地查找重复文件：\n\
                  1. 文件大小比较\n\
                  2. 部分内容哈希\n\
                  3. 完整 MD5 校验\n\
                  4. 逐字节比较"
)]
struct Args {
    /// 要扫描的目录路径
    #[arg(default_value = ".")]
    path: PathBuf,

    /// 递归扫描子目录（默认递归）
    #[arg(short, long, default_value_t = true)]
    recursive: bool,

    /// 不递归扫描（只扫描当前目录）
    #[arg(short = 'n', long = "no-recursive", conflicts_with = "recursive")]
    no_recursive: bool,

    /// 显示详细信息（显示验证过程）
    #[arg(short, long)]
    verbose: bool,

    /// 显示文件大小
    #[arg(short = 'S', long)]
    size: bool,

    /// 包含硬链接（默认跳过硬链接）
    #[arg(short = 'H', long)]
    hardlinks: bool,

    /// 显示相对路径（默认显示绝对路径）
    #[arg(short = 'R', long = "relative")]
    relative_path: bool,

    /// 输出 JSON 格式到文件
    #[arg(long, value_name = "FILE")]
    json: Option<PathBuf>,

    /// 生成删除脚本
    #[arg(long, value_name = "FILE")]
    delete_script: Option<PathBuf>,

    /// 文件名 glob 模式过滤（可多次使用）
    /// 
    /// 示例:
    ///   -p "*.pdf"                    只检测 PDF 文件
    ///   -p "*.jpg" -p "*.png"         检测图片文件
    ///   -p "backup*"                  检测 backup 开头的文件
    #[arg(short = 'p', long = "pattern", value_name = "GLOB")]
    patterns: Vec<String>,

    /// 文件名正则表达式过滤
    /// 
    /// 示例:
    ///   --regex ".*\\.pdf$"                              PDF 文件
    ///   --regex "photo_[0-9]+\\.jpg"                     photo_数字.jpg
    ///   --regex ".*\\.(txt|pdf|docx?|xlsx?|pptx?|csv)$"  Office 文件
    ///   --regex ".*\\.(txt|pdf|doc|docx|xls|xlsx|ppt|pptx|csv|xmind)$"  所有文档
    #[arg(long = "regex", value_name = "REGEX")]
    regex_pattern: Option<String>,
}

// ============================================================================
// 【Rust 概念 2: 文件信息结构体】
// ============================================================================
#[derive(Debug, Clone, Serialize, Deserialize)]
struct FileInfo {
    path: PathBuf,
    size: u64,
    partial_hash: Option<String>,  // 部分内容的哈希
    full_hash: Option<String>,     // 完整文件的哈希
}

impl FileInfo {
    fn new(path: PathBuf, size: u64) -> Self {
        FileInfo {
            path,
            size,
            partial_hash: None,
            full_hash: None,
        }
    }
}

// ============================================================================
// 【Rust 概念 3: 主逻辑结构】
// ============================================================================
struct DupFinder {
    verbose: bool,
    show_size: bool,
    include_hardlinks: bool,
    relative_path: bool,
    base_path: PathBuf,
    glob_set: Option<GlobSet>,
    regex: Option<Regex>,
}

impl DupFinder {
    fn new(
        verbose: bool,
        show_size: bool,
        include_hardlinks: bool,
        relative_path: bool,
        base_path: PathBuf,
        glob_set: Option<GlobSet>,
        regex: Option<Regex>,
    ) -> Self {
        DupFinder {
            verbose,
            show_size,
            include_hardlinks,
            relative_path,
            base_path,
            glob_set,
            regex,
        }
    }
    
    /// 检查文件是否应该被包含在扫描中
    fn should_include_file(&self, path: &Path) -> bool {
        // 如果没有指定任何过滤条件，包含所有文件
        if self.glob_set.is_none() && self.regex.is_none() {
            return true;
        }
        
        let filename = match path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name,
            None => return false,
        };
        
        // 检查 glob 模式
        if let Some(ref globset) = self.glob_set {
            if globset.is_match(filename) {
                return true;
            }
        }
        
        // 检查正则表达式
        if let Some(ref regex) = self.regex {
            if regex.is_match(filename) {
                return true;
            }
        }
        
        false
    }
    
    /// 格式化路径显示（绝对路径或相对路径）
    fn format_path(&self, path: &Path) -> String {
        if self.relative_path {
            // 显示相对于扫描目录的路径
            if let Ok(rel_path) = path.strip_prefix(&self.base_path) {
                format!("./{}", rel_path.display())
            } else {
                path.display().to_string()
            }
        } else {
            // 显示绝对路径
            if let Ok(abs_path) = path.canonicalize() {
                abs_path.display().to_string()
            } else {
                path.display().to_string()
            }
        }
    }

    // ========================================================================
    // 第 1 层：按文件大小分组
    // ========================================================================
    fn group_by_size(&self, paths: Vec<PathBuf>) -> HashMap<u64, Vec<FileInfo>> {
        if self.verbose {
            println!("{}", "🔍 第 1 层：按文件大小分组...".cyan());
        }

        let mut size_groups: HashMap<u64, Vec<FileInfo>> = HashMap::new();

        for path in paths {
            if let Ok(metadata) = fs::metadata(&path) {
                let size = metadata.len();
                
                // 跳过空文件
                if size == 0 {
                    continue;
                }

                let file_info = FileInfo::new(path, size);
                size_groups
                    .entry(size)
                    .or_insert_with(Vec::new)
                    .push(file_info);
            }
        }

        // 只保留大小相同的文件（潜在重复）
        size_groups.retain(|_size, files| files.len() > 1);

        if self.verbose {
            let potential = size_groups.values().map(|v| v.len()).sum::<usize>();
            println!(
                "  ✓ 找到 {} 组可能重复的文件（{} 个文件）",
                size_groups.len(),
                potential
            );
        }

        size_groups
    }

    // ========================================================================
    // 第 2 层：计算部分内容哈希（前 8KB）
    // ========================================================================
    fn calculate_partial_hash(&self, path: &Path) -> io::Result<String> {
        let mut file = File::open(path)?;
        let mut buffer = vec![0u8; 8192]; // 读取前 8KB
        let bytes_read = file.read(&mut buffer)?;
        buffer.truncate(bytes_read);

        let digest = md5::compute(&buffer);
        Ok(format!("{:x}", digest))
    }

    fn group_by_partial_hash(
        &self,
        size_groups: HashMap<u64, Vec<FileInfo>>,
    ) -> HashMap<String, Vec<FileInfo>> {
        if self.verbose {
            println!("{}", "🔍 第 2 层：计算部分内容哈希...".cyan());
        }

        let mut hash_groups: HashMap<String, Vec<FileInfo>> = HashMap::new();
        let mut checked = 0;

        for (_size, mut files) in size_groups {
            for file_info in &mut files {
                if let Ok(hash) = self.calculate_partial_hash(&file_info.path) {
                    file_info.partial_hash = Some(hash.clone());
                    hash_groups
                        .entry(hash)
                        .or_insert_with(Vec::new)
                        .push(file_info.clone());
                    checked += 1;
                }
            }
        }

        // 只保留哈希相同的文件
        hash_groups.retain(|_hash, files| files.len() > 1);

        if self.verbose {
            let potential = hash_groups.values().map(|v| v.len()).sum::<usize>();
            println!(
                "  ✓ 检查了 {} 个文件，找到 {} 组部分哈希相同（{} 个文件）",
                checked,
                hash_groups.len(),
                potential
            );
        }

        hash_groups
    }

    // ========================================================================
    // 第 3 层：计算完整文件 MD5
    // ========================================================================
    fn calculate_full_hash(&self, path: &Path) -> io::Result<String> {
        let mut file = File::open(path)?;
        let mut context = md5::Context::new();
        let mut buffer = vec![0u8; 8192];

        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            context.consume(&buffer[..bytes_read]);
        }

        Ok(format!("{:x}", context.compute()))
    }

    fn group_by_full_hash(
        &self,
        partial_groups: HashMap<String, Vec<FileInfo>>,
    ) -> HashMap<String, Vec<FileInfo>> {
        if self.verbose {
            println!("{}", "🔍 第 3 层：计算完整文件 MD5...".cyan());
        }

        let mut full_hash_groups: HashMap<String, Vec<FileInfo>> = HashMap::new();
        let mut checked = 0;

        for (_partial, mut files) in partial_groups {
            for file_info in &mut files {
                if let Ok(hash) = self.calculate_full_hash(&file_info.path) {
                    file_info.full_hash = Some(hash.clone());
                    full_hash_groups
                        .entry(hash)
                        .or_insert_with(Vec::new)
                        .push(file_info.clone());
                    checked += 1;
                }
            }
        }

        // 只保留完整哈希相同的文件
        full_hash_groups.retain(|_hash, files| files.len() > 1);

        if self.verbose {
            let potential = full_hash_groups.values().map(|v| v.len()).sum::<usize>();
            println!(
                "  ✓ 检查了 {} 个文件，找到 {} 组完整 MD5 相同（{} 个文件）",
                checked,
                full_hash_groups.len(),
                potential
            );
        }

        full_hash_groups
    }

    // ========================================================================
    // 第 4 层：逐字节比较（最终确认）
    // ========================================================================
    fn byte_compare(&self, path1: &Path, path2: &Path) -> io::Result<bool> {
        // 检查是否是硬链接（同一个文件）
        // 硬链接指向同一个 inode，删除一个不会节省空间
        if !self.include_hardlinks && is_same_file(path1, path2).unwrap_or(false) {
            if self.verbose {
                println!(
                    "  {} 跳过硬链接: {} <-> {}",
                    "↪".dimmed(),
                    path1.display(),
                    path2.display()
                );
            }
            return Ok(false);  // 不算重复
        }

        let mut file1 = File::open(path1)?;
        let mut file2 = File::open(path2)?;

        let mut buffer1 = vec![0u8; 8192];
        let mut buffer2 = vec![0u8; 8192];

        loop {
            let bytes1 = file1.read(&mut buffer1)?;
            let bytes2 = file2.read(&mut buffer2)?;

            if bytes1 != bytes2 {
                return Ok(false);
            }

            if bytes1 == 0 {
                return Ok(true);
            }

            if buffer1[..bytes1] != buffer2[..bytes2] {
                return Ok(false);
            }
        }
    }

    fn verify_duplicates(
        &self,
        hash_groups: HashMap<String, Vec<FileInfo>>,
    ) -> Vec<Vec<FileInfo>> {
        if self.verbose {
            println!("{}", "🔍 第 4 层：逐字节比较验证...".cyan());
        }

        let mut verified_groups = Vec::new();
        let mut comparisons = 0;

        for (_hash, files) in hash_groups {
            // 使用图的方式验证：如果 A == B 且 B == C，则 A == B == C
            let mut duplicate_group = vec![files[0].clone()];

            for i in 1..files.len() {
                if let Ok(true) = self.byte_compare(&files[0].path, &files[i].path) {
                    duplicate_group.push(files[i].clone());
                    comparisons += 1;
                }
            }

            if duplicate_group.len() > 1 {
                verified_groups.push(duplicate_group);
            }
        }

        if self.verbose {
            let total = verified_groups.iter().map(|g| g.len()).sum::<usize>();
            println!(
                "  ✓ 进行了 {} 次字节比较，确认 {} 组完全重复（{} 个文件）",
                comparisons,
                verified_groups.len(),
                total
            );
        }

        verified_groups
    }

    // ========================================================================
    // 显示结果
    // ========================================================================
    fn display_results(&self, groups: &[Vec<FileInfo>]) {
        if groups.is_empty() {
            println!("{}", "✅ 未发现重复文件".green());
            return;
        }

        println!("\n{}", "=" .repeat(70));
        println!("{}", format!("📊 发现 {} 组重复文件", groups.len()).yellow().bold());
        println!("{}", "=".repeat(70));

        for (i, group) in groups.iter().enumerate() {
            println!("\n{}", format!("组 {}:", i + 1).bright_blue().bold());
            
            if self.show_size {
                println!(
                    "  {}",
                    format!("文件大小: {} bytes", group[0].size).dimmed()
                );
            }

            for file_info in group {
                let path_display = self.format_path(&file_info.path);
                println!("  {}", path_display);
            }
        }

        let total_files: usize = groups.iter().map(|g| g.len()).sum();
        let can_save: usize = groups.iter().map(|g| g.len() - 1).sum();
        
        println!("\n{}", "=".repeat(70));
        println!("{}", format!("📈 统计信息:").cyan().bold());
        println!("  总重复文件数: {}", total_files);
        println!("  可删除文件数: {} (保留每组 1 个)", can_save);
        
        if self.show_size {
            let total_size: u64 = groups.iter()
                .map(|g| g[0].size * (g.len() as u64 - 1))
                .sum();
            println!(
                "  可节省空间: {} ({} bytes)",
                format_size(total_size),
                total_size
            );
        }
        println!("{}", "=".repeat(70));
    }

    // ========================================================================
    // JSON 输出
    // ========================================================================
    fn export_json(&self, groups: &[Vec<FileInfo>], output_path: &Path) -> io::Result<()> {
        // 构建 JSON 数据结构
        #[derive(Serialize)]
        struct DuplicateReport {
            scan_info: ScanInfo,
            duplicate_groups: Vec<DuplicateGroup>,
            statistics: Statistics,
        }

        #[derive(Serialize)]
        struct ScanInfo {
            base_path: String,
            total_groups: usize,
            timestamp: String,
        }

        #[derive(Serialize)]
        struct DuplicateGroup {
            group_id: usize,
            file_size: u64,
            file_count: usize,
            md5_hash: Option<String>,
            files: Vec<FileEntry>,
        }

        #[derive(Serialize)]
        struct FileEntry {
            path: String,
            absolute_path: String,
        }

        #[derive(Serialize)]
        struct Statistics {
            total_duplicate_files: usize,
            deletable_files: usize,
            potential_space_savings: u64,
        }

        // 准备数据
        let duplicate_groups: Vec<DuplicateGroup> = groups
            .iter()
            .enumerate()
            .map(|(i, group)| {
                let files = group
                    .iter()
                    .map(|f| {
                        let path_display = self.format_path(&f.path);
                        let abs_path = f.path
                            .canonicalize()
                            .unwrap_or_else(|_| f.path.clone())
                            .display()
                            .to_string();
                        FileEntry {
                            path: path_display,
                            absolute_path: abs_path,
                        }
                    })
                    .collect();

                DuplicateGroup {
                    group_id: i + 1,
                    file_size: group[0].size,
                    file_count: group.len(),
                    md5_hash: group[0].full_hash.clone(),
                    files,
                }
            })
            .collect();

        let total_files: usize = groups.iter().map(|g| g.len()).sum();
        let deletable: usize = groups.iter().map(|g| g.len() - 1).sum();
        let space_savings: u64 = groups
            .iter()
            .map(|g| g[0].size * (g.len() as u64 - 1))
            .sum();

        let report = DuplicateReport {
            scan_info: ScanInfo {
                base_path: self.base_path.display().to_string(),
                total_groups: groups.len(),
                timestamp: Local::now().to_rfc3339(),
            },
            duplicate_groups,
            statistics: Statistics {
                total_duplicate_files: total_files,
                deletable_files: deletable,
                potential_space_savings: space_savings,
            },
        };

        // 写入文件
        let json = serde_json::to_string_pretty(&report)?;
        let mut file = File::create(output_path)?;
        file.write_all(json.as_bytes())?;

        println!(
            "\n{} {}",
            "✅ JSON 报告已保存到:".green(),
            output_path.display()
        );

        Ok(())
    }

    // ========================================================================
    // 生成删除脚本
    // ========================================================================
    fn generate_delete_script(&self, groups: &[Vec<FileInfo>], output_path: &Path) -> io::Result<()> {
        // 运行时检测操作系统，决定生成哪种脚本
        // 使用 std::env::consts::OS 而不是编译时 cfg
        let is_windows = std::env::consts::OS == "windows";

        let script = if is_windows {
            self.generate_powershell_script(groups, output_path)?
        } else {
            self.generate_bash_script(groups, output_path)?
        };

        // 写入文件
        let mut file = File::create(output_path)?;
        file.write_all(script.as_bytes())?;

        // 设置执行权限（Unix 系统）
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(output_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(output_path, perms)?;
        }

        println!(
            "\n{} {}",
            "✅ 删除脚本已生成:".green(),
            output_path.display()
        );
        
        if is_windows {
            println!("{}", "   请仔细检查后执行！".yellow());
            println!("{}", "   执行方式：".cyan());
            println!("{}", "     PowerShell -ExecutionPolicy Bypass -File <脚本文件>".cyan());
            println!("{}", "     或右键脚本 -> 使用 PowerShell 运行".cyan());
        } else {
            println!("{}", "   请仔细检查后执行！".yellow());
            println!("{}", "   执行方式：bash <脚本文件>".cyan());
        }

        Ok(())
    }

    // ========================================================================
    // 生成 Bash 脚本（Linux/macOS）
    // ========================================================================
    fn generate_bash_script(&self, groups: &[Vec<FileInfo>], output_path: &Path) -> io::Result<String> {
        let mut script = String::new();

        // 脚本头部
        script.push_str("#!/bin/bash\n");
        script.push_str("# ============================================================================\n");
        script.push_str("# DupFinder 自动生成的删除脚本\n");
        script.push_str(&format!("# 生成时间: {}\n", Local::now().format("%Y-%m-%d %H:%M:%S")));
        script.push_str(&format!("# 扫描路径: {}\n", self.base_path.display()));
        script.push_str(&format!("# 重复组数: {}\n", groups.len()));
        script.push_str("# ============================================================================\n");
        script.push_str("#\n");
        script.push_str("# ⚠️  警告：此脚本将删除重复文件！\n");
        script.push_str("#    每组重复文件会保留第一个，删除其他的。\n");
        script.push_str("#    请仔细检查后再执行！\n");
        script.push_str("#\n");
        script.push_str("# 使用方法:\n");
        script.push_str("#   1. 仔细检查下面的删除命令\n");
        script.push_str("#   2. 如果需要保留其他文件，请注释掉对应的删除行\n");
        script.push_str(&format!("#   3. 添加执行权限: chmod +x {}\n", output_path.display()));
        script.push_str(&format!("#   4. 执行脚本: ./{}\n", output_path.file_name().unwrap().to_string_lossy()));
        script.push_str("# ============================================================================\n\n");

        // 安全检查
        script.push_str("set -e  # 遇到错误立即退出\n");
        script.push_str("set -u  # 使用未定义变量时报错\n\n");

        // 交互式确认
        script.push_str("# 确认提示\n");
        script.push_str("echo \"⚠️  警告: 即将删除重复文件！\"\n");
        script.push_str(&format!("echo \"扫描路径: {}\"\n", self.base_path.display()));
        script.push_str(&format!("echo \"重复组数: {}\"\n", groups.len()));
        
        let deletable: usize = groups.iter().map(|g| g.len() - 1).sum();
        let space_savings: u64 = groups
            .iter()
            .map(|g| g[0].size * (g.len() as u64 - 1))
            .sum();
        
        script.push_str(&format!("echo \"将删除文件数: {}\"\n", deletable));
        script.push_str(&format!("echo \"可节省空间: {}\"\n", format_size(space_savings)));
        script.push_str("echo \"\"\n");
        script.push_str("read -p \"确认要继续吗? (yes/no): \" confirm\n");
        script.push_str("if [ \"$confirm\" != \"yes\" ]; then\n");
        script.push_str("    echo \"❌ 已取消删除操作\"\n");
        script.push_str("    exit 0\n");
        script.push_str("fi\n\n");

        // 统计变量
        script.push_str("# 统计变量\n");
        script.push_str("deleted_count=0\n");
        script.push_str("deleted_size=0\n");
        script.push_str("failed_count=0\n\n");

        // 为每组生成删除命令
        for (i, group) in groups.iter().enumerate() {
            script.push_str(&format!("\n# ============================================================================\n"));
            script.push_str(&format!("# 组 {}: {} 个重复文件 (大小: {} bytes)\n", 
                i + 1, group.len(), group[0].size));
            script.push_str(&format!("# ============================================================================\n"));
            
            // 显示保留的文件
            let keep_path = if let Ok(abs) = group[0].path.canonicalize() {
                abs.display().to_string()
            } else {
                group[0].path.display().to_string()
            };
            script.push_str(&format!("# 保留: {}\n", keep_path));
            
            // 删除其他文件
            for (j, file) in group.iter().skip(1).enumerate() {
                let file_path = if let Ok(abs) = file.path.canonicalize() {
                    abs.display().to_string()
                } else {
                    file.path.display().to_string()
                };
                
                script.push_str(&format!("\n# 删除文件 {}/{}\n", j + 1, group.len() - 1));
                script.push_str(&format!("if [ -f \"{}\" ]; then\n", file_path));
                script.push_str(&format!("    echo \"删除: {}\"\n", file_path));
                script.push_str(&format!("    if rm \"{}\"; then\n", file_path));
                script.push_str(&format!("        deleted_count=$((deleted_count + 1))\n"));
                script.push_str(&format!("        deleted_size=$((deleted_size + {}))\n", file.size));
                script.push_str("    else\n");
                script.push_str(&format!("        echo \"❌ 删除失败: {}\"\n", file_path));
                script.push_str("        failed_count=$((failed_count + 1))\n");
                script.push_str("    fi\n");
                script.push_str("else\n");
                script.push_str(&format!("    echo \"⚠️  文件不存在: {}\"\n", file_path));
                script.push_str("fi\n");
            }
        }

        // 脚本结尾 - 显示统计信息
        script.push_str("\n# ============================================================================\n");
        script.push_str("# 删除完成，显示统计信息\n");
        script.push_str("# ============================================================================\n");
        script.push_str("echo \"\"\n");
        script.push_str("echo \"==============================================================================\"\n");
        script.push_str("echo \"📊 删除统计:\"\n");
        script.push_str("echo \"==============================================================================\"\n");
        script.push_str("echo \"✅ 成功删除: $deleted_count 个文件\"\n");
        script.push_str("echo \"❌ 失败数量: $failed_count 个文件\"\n");
        script.push_str("echo \"💾 节省空间: $(numfmt --to=iec-i --suffix=B $deleted_size 2>/dev/null || echo \\\"$deleted_size bytes\\\")\"\n");
        script.push_str("echo \"==============================================================================\"\n");

        Ok(script)
    }

    // ========================================================================
    // 生成 PowerShell 脚本（Windows）
    // ========================================================================
    fn generate_powershell_script(&self, groups: &[Vec<FileInfo>], output_path: &Path) -> io::Result<String> {
        let mut script = String::new();

        let deletable: usize = groups.iter().map(|g| g.len() - 1).sum();
        let space_savings: u64 = groups
            .iter()
            .map(|g| g[0].size * (g.len() as u64 - 1))
            .sum();

        // 脚本头部
        script.push_str("# ============================================================================\n");
        script.push_str("# DupFinder 自动生成的删除脚本 (PowerShell)\n");
        script.push_str(&format!("# 生成时间: {}\n", Local::now().format("%Y-%m-%d %H:%M:%S")));
        script.push_str(&format!("# 扫描路径: {}\n", self.base_path.display()));
        script.push_str(&format!("# 重复组数: {}\n", groups.len()));
        script.push_str("# ============================================================================\n");
        script.push_str("#\n");
        script.push_str("# ⚠️  警告：此脚本将删除重复文件！\n");
        script.push_str("#    每组重复文件会保留第一个，删除其他的。\n");
        script.push_str("#    请仔细检查后再执行！\n");
        script.push_str("#\n");
        script.push_str("# 使用方法:\n");
        script.push_str("#   1. 仔细检查下面的删除命令\n");
        script.push_str("#   2. 如果需要保留其他文件，请注释掉对应的删除行\n");
        script.push_str(&format!("#   3. 执行脚本: PowerShell -ExecutionPolicy Bypass -File {}\n", output_path.file_name().unwrap().to_string_lossy()));
        script.push_str("#   4. 或右键 -> 使用 PowerShell 运行\n");
        script.push_str("# ============================================================================\n\n");

        // 安全检查
        script.push_str("# 设置错误处理\n");
        script.push_str("$ErrorActionPreference = \"Stop\"\n\n");

        // 交互式确认
        script.push_str("# 确认提示\n");
        script.push_str("Write-Host \"⚠️  警告: 即将删除重复文件！\" -ForegroundColor Yellow\n");
        script.push_str(&format!("Write-Host \"扫描路径: {}\"\n", self.base_path.display()));
        script.push_str(&format!("Write-Host \"重复组数: {}\"\n", groups.len()));
        script.push_str(&format!("Write-Host \"将删除文件数: {}\"\n", deletable));
        script.push_str(&format!("Write-Host \"可节省空间: {}\"\n", format_size(space_savings)));
        script.push_str("Write-Host \"\"\n");
        script.push_str("$confirm = Read-Host \"确认要继续吗? (yes/no)\"\n");
        script.push_str("if ($confirm -ne \"yes\") {\n");
        script.push_str("    Write-Host \"❌ 已取消删除操作\" -ForegroundColor Red\n");
        script.push_str("    exit 0\n");
        script.push_str("}\n\n");

        // 统计变量
        script.push_str("# 统计变量\n");
        script.push_str("$deletedCount = 0\n");
        script.push_str("$deletedSize = 0\n");
        script.push_str("$failedCount = 0\n\n");

        // 为每组生成删除命令
        for (i, group) in groups.iter().enumerate() {
            script.push_str("\n# ============================================================================\n");
            script.push_str(&format!("# 组 {}: {} 个重复文件 (大小: {} bytes)\n", 
                i + 1, group.len(), group[0].size));
            script.push_str("# ============================================================================\n");
            
            // 显示保留的文件
            let keep_path = if let Ok(abs) = group[0].path.canonicalize() {
                abs.display().to_string()
            } else {
                group[0].path.display().to_string()
            };
            script.push_str(&format!("# 保留: {}\n", keep_path));
            
            // 删除其他文件
            for (j, file) in group.iter().skip(1).enumerate() {
                let file_path = if let Ok(abs) = file.path.canonicalize() {
                    abs.display().to_string()
                } else {
                    file.path.display().to_string()
                };
                
                script.push_str(&format!("\n# 删除文件 {}/{}\n", j + 1, group.len() - 1));
                script.push_str(&format!("if (Test-Path \"{}\") {{\n", file_path));
                script.push_str(&format!("    Write-Host \"删除: {}\"\n", file_path));
                script.push_str("    try {\n");
                script.push_str(&format!("        Remove-Item \"{}\" -Force\n", file_path));
                script.push_str("        $deletedCount++\n");
                script.push_str(&format!("        $deletedSize += {}\n", file.size));
                script.push_str("    } catch {\n");
                script.push_str(&format!("        Write-Host \"❌ 删除失败: {}\" -ForegroundColor Red\n", file_path));
                script.push_str("        $failedCount++\n");
                script.push_str("    }\n");
                script.push_str("} else {\n");
                script.push_str(&format!("    Write-Host \"⚠️  文件不存在: {}\" -ForegroundColor Yellow\n", file_path));
                script.push_str("}\n");
            }
        }

        // 脚本结尾 - 显示统计信息
        script.push_str("\n# ============================================================================\n");
        script.push_str("# 删除完成，显示统计信息\n");
        script.push_str("# ============================================================================\n");
        script.push_str("Write-Host \"\"\n");
        script.push_str("Write-Host \"==============================================================================\" -ForegroundColor Cyan\n");
        script.push_str("Write-Host \"📊 删除统计:\" -ForegroundColor Cyan\n");
        script.push_str("Write-Host \"==============================================================================\" -ForegroundColor Cyan\n");
        script.push_str("Write-Host \"✅ 成功删除: $deletedCount 个文件\" -ForegroundColor Green\n");
        script.push_str("Write-Host \"❌ 失败数量: $failedCount 个文件\" -ForegroundColor Red\n");
        script.push_str("$sizeInMB = [math]::Round($deletedSize / 1MB, 2)\n");
        script.push_str("if ($sizeInMB -gt 0) {\n");
        script.push_str("    Write-Host \"💾 节省空间: $sizeInMB MB ($deletedSize bytes)\" -ForegroundColor Green\n");
        script.push_str("} else {\n");
        script.push_str("    Write-Host \"💾 节省空间: $deletedSize bytes\" -ForegroundColor Green\n");
        script.push_str("}\n");
        script.push_str("Write-Host \"==============================================================================\" -ForegroundColor Cyan\n");
        script.push_str("\n# 暂停，等待用户按键\n");
        script.push_str("Write-Host \"\"\n");
        script.push_str("Write-Host \"按任意键退出...\" -ForegroundColor Gray\n");
        script.push_str("$null = $Host.UI.RawUI.ReadKey(\"NoEcho,IncludeKeyDown\")\n");

        Ok(script)
    }

    // ========================================================================
    // 主查找流程
    // ========================================================================
    fn find_duplicates(&self, root: &Path, recursive: bool) -> Vec<Vec<FileInfo>> {
        // 收集所有文件路径
        let mut paths = Vec::new();
        
        let walker = if recursive {
            WalkDir::new(root).into_iter()
        } else {
            WalkDir::new(root).max_depth(1).into_iter()
        };

        for entry in walker.filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                let path = entry.path();
                // 应用文件名过滤
                if self.should_include_file(path) {
                    paths.push(path.to_path_buf());
                }
            }
        }

        if paths.is_empty() {
            if self.glob_set.is_some() || self.regex.is_some() {
                println!("{}", "⚠️  未找到匹配的文件".yellow());
            }
            return Vec::new();
        }

        println!(
            "{}",
            format!("🔎 开始扫描 {} 个文件...\n", paths.len()).green()
        );

        // 执行 4 层验证
        let size_groups = self.group_by_size(paths);
        let partial_groups = self.group_by_partial_hash(size_groups);
        let full_groups = self.group_by_full_hash(partial_groups);
        let duplicates = self.verify_duplicates(full_groups);

        duplicates
    }
}

// ============================================================================
// 辅助函数
// ============================================================================
fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

// ============================================================================
// Main 函数
// ============================================================================
fn main() {
    let args = Args::parse();

    println!(
        "{}",
        "🔍 DupFinder - 重复文件查找工具".bright_cyan().bold()
    );
    
    // 构建 GlobSet
    let glob_set = if !args.patterns.is_empty() {
        let mut builder = GlobSetBuilder::new();
        for pattern in &args.patterns {
            match Glob::new(pattern) {
                Ok(glob) => {
                    builder.add(glob);
                }
                Err(e) => {
                    eprintln!("{} {}: {}", "❌ 无效的 glob 模式".red(), pattern, e);
                    std::process::exit(1);
                }
            }
        }
        match builder.build() {
            Ok(set) => Some(set),
            Err(e) => {
                eprintln!("{} {}", "❌ 构建 glob 集合失败:".red(), e);
                std::process::exit(1);
            }
        }
    } else {
        None
    };
    
    // 构建 Regex
    let regex = if let Some(ref pattern) = args.regex_pattern {
        match Regex::new(pattern) {
            Ok(re) => Some(re),
            Err(e) => {
                eprintln!("{} {}: {}", "❌ 无效的正则表达式".red(), pattern, e);
                std::process::exit(1);
            }
        }
    } else {
        None
    };
    
    // 获取绝对路径作为基准路径
    let base_path = args.path.canonicalize().unwrap_or_else(|_| args.path.clone());
    
    println!(
        "{}",
        format!("📂 扫描路径: {}", args.path.display()).dimmed()
    );
    
    // 显示过滤条件
    if !args.patterns.is_empty() {
        println!(
            "{}",
            format!("🔍 Glob 模式: {}", args.patterns.join(", ")).dimmed()
        );
    }
    if let Some(ref regex_pattern) = args.regex_pattern {
        println!(
            "{}",
            format!("🔍 正则表达式: {}", regex_pattern).dimmed()
        );
    }
    
    // 处理递归选项（默认递归，除非指定 --no-recursive）
    let do_recursive = !args.no_recursive && args.recursive;
    
    if do_recursive {
        println!("{}", "🔄 递归模式: 开启".dimmed());
    } else {
        println!("{}", "🔄 递归模式: 关闭（仅扫描当前目录）".dimmed());
    }
    
    if args.relative_path {
        println!("{}", "📍 路径显示: 相对路径".dimmed());
    }
    
    if args.verbose {
        println!("{}", "📋 详细模式: 开启".dimmed());
    }
    
    println!();

    let finder = DupFinder::new(
        args.verbose,
        args.size,
        args.hardlinks,
        args.relative_path,
        base_path.clone(),
        glob_set,
        regex,
    );
    let duplicates = finder.find_duplicates(&args.path, do_recursive);
    finder.display_results(&duplicates);

    // JSON 输出
    if let Some(json_path) = args.json {
        if let Err(e) = finder.export_json(&duplicates, &json_path) {
            eprintln!("{} {}", "❌ JSON 输出失败:".red(), e);
        }
    }

    // 生成删除脚本
    if let Some(script_path) = args.delete_script {
        if let Err(e) = finder.generate_delete_script(&duplicates, &script_path) {
            eprintln!("{} {}", "❌ 删除脚本生成失败:".red(), e);
        }
    }
}

// ============================================================================
// Rust 学习要点总结
// ============================================================================
// 
// 1. 所有权和借用：
//    - &Path 借用路径，不获取所有权
//    - &mut 可变借用用于修改数据
//
// 2. 错误处理：
//    - Result<T, E> 强制处理错误
//    - ? 操作符简化错误传播
//
// 3. 集合类型：
//    - HashMap 用于分组
//    - Vec 用于存储列表
//
// 4. 迭代器：
//    - filter_map, map 等函数式操作
//    - 零成本抽象，性能等同于手写循环
//
// 5. 模式匹配：
//    - match 和 if let 优雅处理 Option
//
// 6. 结构体和方法：
//    - impl 块组织相关功能
//    - &self 借用，self 获取所有权
//
// ============================================================================
