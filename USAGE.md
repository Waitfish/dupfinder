# DupFinder 使用指南

## 🎯 功能总结

DupFinder 是一个高性能重复文件查找工具，完全模仿 fdupes 的 4 层验证策略。

### ✨ 核心特性

- ✅ **4 层验证**：确保 100% 准确
- ✅ **默认递归**：自动扫描子目录
- ✅ **智能硬链接检测**：避免误报
- ✅ **路径显示选项**：绝对路径或相对路径
- ✅ **彩色输出**：清晰易读
- ✅ **JSON 输出**：导出结构化报告
- ✅ **删除脚本生成**：安全删除重复文件
- ✅ **性能极佳**：毫秒级扫描

---

## 📖 使用示例

### 示例 1：快速扫描（默认选项）

```bash
# 扫描当前目录及所有子目录（默认递归）
dupfinder

# 输出（绝对路径）：
# ======================================================================
# 📊 发现 2 组重复文件
# ======================================================================
# 
# 组 1:
#   /home/user/photos/img1.jpg
#   /home/user/backup/img1.jpg
```

### 示例 2：显示详细过程

```bash
dupfinder -v ~/Downloads

# 输出：
# 🔍 第 1 层：按文件大小分组...
#   ✓ 找到 15 组可能重复的文件（50 个文件）
# 🔍 第 2 层：计算部分内容哈希...
#   ✓ 检查了 50 个文件，找到 8 组部分哈希相同（20 个文件）
# 🔍 第 3 层：计算完整文件 MD5...
#   ✓ 检查了 20 个文件，找到 5 组完整 MD5 相同（15 个文件）
# 🔍 第 4 层：逐字节比较验证...
#   ↪ 跳过硬链接: file1 <-> file1_link
#   ✓ 进行了 10 次字节比较，确认 5 组完全重复（15 个文件）
```

### 示例 3：查看可节省空间

```bash
dupfinder -S ~/Documents

# 输出：
# 组 1:
#   文件大小: 1048576 bytes
#   /home/user/Documents/report.pdf
#   /home/user/Documents/backup/report.pdf
# 
# ======================================================================
# 📈 统计信息:
#   总重复文件数: 12
#   可删除文件数: 6 (保留每组 1 个)
#   可节省空间: 125.50 MB (131621888 bytes)
# ======================================================================
```

### 示例 4：相对路径显示

```bash
cd ~/project
dupfinder -R .

# 输出（相对路径，更简洁）：
# 组 1:
#   ./src/utils.rs
#   ./backup/src/utils.rs
#   ./old/utils.rs
```

### 示例 5：只扫描当前目录（不递归）

```bash
dupfinder -n /tmp

# 只扫描 /tmp 目录本身，不扫描子目录
```

### 示例 6：包含硬链接

```bash
dupfinder -H /data

# 硬链接也会被列为重复文件
# 注意：删除硬链接不会节省空间（指向同一个 inode）
```

### 示例 7：完整功能组合

```bash
dupfinder -v -S -R ~/Downloads

# -v: 显示详细验证过程
# -S: 显示文件大小和可节省空间
# -R: 使用相对路径显示
```

### 示例 8：导出 JSON 报告

```bash
dupfinder ~/Documents --json report.json

# 输出：
# ✅ JSON 报告已保存到: report.json

# 查看 JSON 内容
cat report.json | jq '.statistics'
# {
#   "total_duplicate_files": 10,
#   "deletable_files": 7,
#   "potential_space_savings": 15728640
# }
```

### 示例 9：生成删除脚本

```bash
# 1. 生成删除脚本
dupfinder ~/Downloads --delete-script delete_dups.sh

# 输出：
# ✅ 删除脚本已生成: delete_dups.sh
#    请仔细检查后执行！

# 2. 查看脚本内容
cat delete_dups.sh
# #!/bin/bash
# # 保留: /home/user/Downloads/file1.txt
# # 删除文件 1/2
# if [ -f "/home/user/Downloads/file2.txt" ]; then
#     echo "删除: /home/user/Downloads/file2.txt"
#     rm "/home/user/Downloads/file2.txt"
# ...

# 3. 执行删除（需要确认）
bash delete_dups.sh
# ⚠️  警告: 即将删除重复文件！
# 确认要继续吗? (yes/no): yes
# 删除: /home/user/Downloads/file2.txt
# ✅ 成功删除: 2 个文件
```

### 示例 10：一次性生成报告和删除脚本

```bash
dupfinder -v -S ~/Downloads \
    --json report.json \
    --delete-script delete_dups.sh

# 同时生成 JSON 报告和删除脚本
# 可以先查看报告，再决定是否执行删除
```

---

## 🎯 典型使用场景

### 场景 1：清理下载文件夹

```bash
dupfinder -S ~/Downloads

# 查看可以删除哪些重复的下载文件
# 可节省空间一目了然
```

### 场景 2：代码仓库去重

```bash
cd ~/my-project
dupfinder -R .

# 使用相对路径，方便看清楚项目结构中的重复文件
```

### 场景 3：备份文件检查

```bash
dupfinder -v ~/backup

# 详细模式查看验证过程
# 确保备份文件的完整性
```

### 场景 4：大文件去重

```bash
dupfinder -S /media/storage

# 找出占用空间的重复大文件
# 可节省大量磁盘空间
```

### 场景 5：快速检查单个目录

```bash
dupfinder -n -S /var/log

# 只检查 /var/log 本身，不递归
# 快速查找当前目录的重复日志
```

### 场景 6：批量清理+记录

```bash
# 扫描、生成报告和删除脚本
dupfinder ~/Downloads --json report.json --delete-script delete.sh

# 查看报告（可以用 jq 处理 JSON）
cat report.json | jq '.statistics'

# 决定是否执行删除
bash delete.sh
```

### 场景 7：CI/CD 集成

```bash
# 在 CI 中检查代码仓库的重复文件
dupfinder . --json dup_report.json

# 解析 JSON，如果发现重复文件，提示警告
deletable=$(jq '.statistics.deletable_files' dup_report.json)
if [ "$deletable" -gt 0 ]; then
    echo "⚠️  发现 $deletable 个重复文件，请清理"
    exit 1
fi
```

---

## 🔍 参数详解

### 路径相关

| 参数 | 说明 | 示例 |
|------|------|------|
| `<PATH>` | 扫描目录（默认 `.`） | `dupfinder /home/user` |
| `-R, --relative` | 显示相对路径 | `dupfinder -R .` |

### 扫描选项

| 参数 | 说明 | 默认值 |
|------|------|--------|
| `-r, --recursive` | 递归扫描子目录 | ✅ **开启** |
| `-n, --no-recursive` | 只扫描当前目录 | 关闭 |
| `-H, --hardlinks` | 包含硬链接 | 关闭（默认跳过） |

### 显示选项

| 参数 | 说明 |
|------|------|
| `-v, --verbose` | 显示 4 层验证详细过程 |
| `-S, --size` | 显示文件大小和可节省空间 |
| `-R, --relative` | 显示相对路径而非绝对路径 |

---

## 💡 使用技巧

### 技巧 1：管道配合其他命令

```bash
# 导出到文件
dupfinder /data > duplicates.txt

# 统计重复文件数
dupfinder /data | grep "总重复文件数" | awk '{print $2}'
```

### 技巧 2：结合 find 限制文件类型

```bash
# 先用 dupfinder 找出所有重复
dupfinder -R ~/photos

# 如果只想查找图片重复，可以：
dupfinder -R ~/photos | grep -E '\.(jpg|png|jpeg)$'
```

### 技巧 3：大目录扫描

```bash
# 对于大目录，建议先用普通模式快速查看
dupfinder /large/directory

# 如果需要详细信息，再使用 -v
dupfinder -v -S /large/directory
```

### 技巧 4：定期清理脚本

```bash
#!/bin/bash
# cleanup-duplicates.sh

echo "扫描下载目录..."
dupfinder -S ~/Downloads > /tmp/dup_report.txt

if grep -q "发现.*组重复文件" /tmp/dup_report.txt; then
    echo "发现重复文件！"
    cat /tmp/dup_report.txt
else
    echo "未发现重复文件"
fi
```

---

## 📊 性能基准

测试环境：
- CPU: Intel Core i5
- 文件数：1000 个
- 总大小：500 MB

| 操作 | 耗时 |
|------|------|
| 扫描 1000 个小文件 | ~50ms |
| 第 1 层：大小分组 | ~2ms |
| 第 2 层：部分哈希 | ~15ms |
| 第 3 层：完整 MD5 | ~20ms |
| 第 4 层：字节比较 | ~10ms |

---

## 🆚 与 fdupes 对比

| 特性 | dupfinder | fdupes |
|------|-----------|--------|
| 4 层验证 | ✅ | ✅ |
| 硬链接检测 | ✅ 智能跳过 | ✅ |
| 彩色输出 | ✅ | ❌ |
| 默认递归 | ✅ | ❌ |
| 相对路径选项 | ✅ | ❌ |
| 详细验证过程 | ✅ | ❌ |
| 可执行文件大小 | 651KB | ~50KB |
| 性能 | ⚡ 极快 | ⚡ 快 |

---

## ❓ 常见问题

### Q: 为什么默认跳过硬链接？
**A:** 硬链接指向同一个 inode（同一份数据），删除一个硬链接不会节省空间。只有删除所有硬链接后，数据才会被真正释放。

### Q: 什么时候使用 -H 参数？
**A:** 当你想查看所有指向相同内容的文件时，包括硬链接。但要注意，删除硬链接不会节省空间。

### Q: 绝对路径和相对路径有什么区别？
**A:** 
- 绝对路径：显示完整路径，适合跨目录操作
- 相对路径：显示相对于扫描目录的路径，更简洁，适合项目内查看

### Q: 默认是递归还是不递归？
**A:** **默认递归**。如果只想扫描当前目录，使用 `-n` 或 `--no-recursive`。

### Q: 能删除重复文件吗？
**A:** 可以！使用 `--delete-script` 生成删除脚本：
```bash
dupfinder /path/to/dir --delete-script delete.sh
bash delete.sh  # 需要手动确认
```
脚本会保留每组的第一个文件，删除其他的。你可以编辑脚本来选择保留哪个文件。

### Q: 如何导出结果供其他程序使用？
**A:** 使用 `--json` 输出 JSON 格式：
```bash
dupfinder /path/to/dir --json report.json
```
JSON 包含完整的扫描信息、重复文件组和统计数据。

### Q: 删除脚本安全吗？
**A:** 非常安全！
- ✅ 需要手动输入 `yes` 确认
- ✅ 详细的注释和文件列表
- ✅ 错误处理和统计信息
- ✅ 可以手动编辑脚本
- ✅ 使用 `set -e` 遇到错误立即停止

---

## 🚀 下一步功能（即将实现）

- [ ] 并行处理大文件
- [ ] 进度条显示
- [ ] 更多哈希算法（SHA256, xxHash）
- [ ] 支持排除模式（.gitignore 风格）
- [ ] 交互式删除模式（无需生成脚本）

---

## 🔗 相关资源

- **GitHub**: https://github.com/Waitfish/dupfinder
- **Issue 反馈**: https://github.com/Waitfish/dupfinder/issues
- **fdupes 官方**: https://github.com/adrianlopezroche/fdupes

---

**Happy duplicate hunting! 🔍**

