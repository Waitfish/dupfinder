# DupFinder 新功能说明

## 🎉 新增功能

### 1. JSON 输出 (`--json`)

将扫描结果导出为结构化的 JSON 格式，便于其他程序处理或自动化分析。

#### 使用方法

```bash
# 基本用法
dupfinder /path/to/directory --json report.json

# 组合使用
dupfinder -v -S /path/to/directory --json report.json
```

#### JSON 格式示例

```json
{
  "scan_info": {
    "base_path": "/path/to/directory",
    "total_groups": 2,
    "timestamp": "2025-12-02T16:30:00+08:00"
  },
  "duplicate_groups": [
    {
      "group_id": 1,
      "file_size": 1048576,
      "file_count": 3,
      "md5_hash": "5d41402abc4b2a76b9719d911017c592",
      "files": [
        {
          "path": "/path/to/file1.txt",
          "absolute_path": "/path/to/file1.txt"
        },
        {
          "path": "/path/to/file2.txt",
          "absolute_path": "/path/to/file2.txt"
        }
      ]
    }
  ],
  "statistics": {
    "total_duplicate_files": 5,
    "deletable_files": 3,
    "potential_space_savings": 3145728
  }
}
```

#### JSON 数据结构

- **scan_info**: 扫描元信息
  - `base_path`: 扫描的基础路径
  - `total_groups`: 重复文件组数
  - `timestamp`: 扫描时间（RFC3339 格式）

- **duplicate_groups**: 重复文件组列表
  - `group_id`: 组编号
  - `file_size`: 文件大小（字节）
  - `file_count`: 该组文件数量
  - `md5_hash`: MD5 哈希值
  - `files`: 文件列表
    - `path`: 显示路径（根据 --relative 参数）
    - `absolute_path`: 绝对路径

- **statistics**: 统计信息
  - `total_duplicate_files`: 总重复文件数
  - `deletable_files`: 可删除文件数（保留每组一个）
  - `potential_space_savings`: 可节省空间（字节）

#### 应用场景

1. **自动化处理**
```bash
# 使用 jq 解析 JSON
deletable=$(jq '.statistics.deletable_files' report.json)
echo "可删除 $deletable 个文件"
```

2. **CI/CD 集成**
```bash
dupfinder . --json report.json
deletable=$(jq '.statistics.deletable_files' report.json)
if [ "$deletable" -gt 0 ]; then
    echo "⚠️  发现 $deletable 个重复文件"
    exit 1
fi
```

3. **数据分析**
```python
import json

with open('report.json') as f:
    data = json.load(f)

print(f"扫描路径: {data['scan_info']['base_path']}")
print(f"重复文件组: {data['scan_info']['total_groups']}")
print(f"可节省空间: {data['statistics']['potential_space_savings'] / 1024 / 1024:.2f} MB")

for group in data['duplicate_groups']:
    print(f"\n组 {group['group_id']}:")
    for file in group['files']:
        print(f"  - {file['path']}")
```

---

### 2. 删除脚本生成 (`--delete-script`)

自动识别操作系统，生成相应的删除脚本（Bash 或 PowerShell）。每组重复文件保留第一个，删除其他的。

#### 支持的平台

- **Linux/macOS**: 生成 Bash 脚本 (.sh)
- **Windows**: 生成 PowerShell 脚本 (.ps1)

#### 使用方法

**Linux/macOS:**
```bash
# 1. 生成删除脚本
dupfinder /path/to/directory --delete-script delete_dups.sh

# 2. 查看脚本内容（可以手动编辑）
cat delete_dups.sh

# 3. 执行删除（需要手动确认）
bash delete_dups.sh
```

**Windows:**
```powershell
# 1. 生成删除脚本
dupfinder C:\path\to\directory --delete-script delete_dups.ps1

# 2. 查看脚本内容（可以手动编辑）
Get-Content delete_dups.ps1

# 3. 执行删除（需要手动确认）
PowerShell -ExecutionPolicy Bypass -File delete_dups.ps1
# 或右键 -> 使用 PowerShell 运行
```

#### 脚本特点

✅ **跨平台支持**
- 自动识别操作系统（Linux、macOS、Windows）
- Linux/macOS: 生成 Bash 脚本，自动设置执行权限
- Windows: 生成 PowerShell 脚本，支持彩色输出

✅ **安全特性**
- 需要手动输入 `yes` 确认才会执行
- Bash: 使用 `set -e` 和 `set -u` 错误处理
- PowerShell: 使用 `$ErrorActionPreference = "Stop"`
- 每个文件删除前检查是否存在
- 详细的错误处理和统计

✅ **可编辑性**
- 清晰的注释说明保留和删除的文件
- 可以手动注释掉不想删除的行
- 可以修改保留哪个文件

✅ **执行信息**
- 删除前显示警告和统计信息
- 删除过程显示实时进度
- 删除完成显示详细统计

#### 脚本示例

```bash
#!/bin/bash
# ============================================================================
# DupFinder 自动生成的删除脚本
# 生成时间: 2025-12-02 16:30:00
# 扫描路径: /home/user/Downloads
# 重复组数: 2
# ============================================================================
#
# ⚠️  警告：此脚本将删除重复文件！
#    每组重复文件会保留第一个，删除其他的。
#    请仔细检查后再执行！
#
# 使用方法:
#   1. 仔细检查下面的删除命令
#   2. 如果需要保留其他文件，请注释掉对应的删除行
#   3. 添加执行权限: chmod +x delete_dups.sh
#   4. 执行脚本: ./delete_dups.sh
# ============================================================================

set -e  # 遇到错误立即退出
set -u  # 使用未定义变量时报错

# 确认提示
echo "⚠️  警告: 即将删除重复文件！"
echo "扫描路径: /home/user/Downloads"
echo "重复组数: 2"
echo "将删除文件数: 3"
echo "可节省空间: 5.00 MB"
echo ""
read -p "确认要继续吗? (yes/no): " confirm
if [ "$confirm" != "yes" ]; then
    echo "❌ 已取消删除操作"
    exit 0
fi

# 统计变量
deleted_count=0
deleted_size=0
failed_count=0

# ============================================================================
# 组 1: 3 个重复文件 (大小: 1048576 bytes)
# ============================================================================
# 保留: /home/user/Downloads/file1.txt

# 删除文件 1/2
if [ -f "/home/user/Downloads/file2.txt" ]; then
    echo "删除: /home/user/Downloads/file2.txt"
    if rm "/home/user/Downloads/file2.txt"; then
        deleted_count=$((deleted_count + 1))
        deleted_size=$((deleted_size + 1048576))
    else
        echo "❌ 删除失败: /home/user/Downloads/file2.txt"
        failed_count=$((failed_count + 1))
    fi
else
    echo "⚠️  文件不存在: /home/user/Downloads/file2.txt"
fi

# ... 更多删除命令 ...

# ============================================================================
# 删除完成，显示统计信息
# ============================================================================
echo ""
echo "=============================================================================="
echo "📊 删除统计:"
echo "=============================================================================="
echo "✅ 成功删除: $deleted_count 个文件"
echo "❌ 失败数量: $failed_count 个文件"
echo "💾 节省空间: $(numfmt --to=iec-i --suffix=B $deleted_size 2>/dev/null || echo \"$deleted_size bytes\")"
echo "=============================================================================="
```

**PowerShell 脚本示例（Windows）：**

```powershell
# ============================================================================
# DupFinder 自动生成的删除脚本 (PowerShell)
# 生成时间: 2025-12-02 16:30:00
# 扫描路径: C:\Users\user\Downloads
# 重复组数: 2
# ============================================================================
#
# ⚠️  警告：此脚本将删除重复文件！
#    每组重复文件会保留第一个，删除其他的。
#    请仔细检查后再执行！
#
# 使用方法:
#   1. 仔细检查下面的删除命令
#   2. 如果需要保留其他文件，请注释掉对应的删除行
#   3. 执行脚本: PowerShell -ExecutionPolicy Bypass -File delete_dups.ps1
#   4. 或右键 -> 使用 PowerShell 运行
# ============================================================================

# 设置错误处理
$ErrorActionPreference = "Stop"

# 确认提示
Write-Host "⚠️  警告: 即将删除重复文件！" -ForegroundColor Yellow
Write-Host "扫描路径: C:\Users\user\Downloads"
Write-Host "重复组数: 2"
Write-Host "将删除文件数: 3"
Write-Host "可节省空间: 3.00 MB"
Write-Host ""
$confirm = Read-Host "确认要继续吗? (yes/no)"
if ($confirm -ne "yes") {
    Write-Host "❌ 已取消删除操作" -ForegroundColor Red
    exit 0
}

# 统计变量
$deletedCount = 0
$deletedSize = 0
$failedCount = 0

# ============================================================================
# 组 1: 3 个重复文件 (大小: 1048576 bytes)
# ============================================================================
# 保留: C:\Users\user\Downloads\file1.txt

# 删除文件 1/2
if (Test-Path "C:\Users\user\Downloads\file2.txt") {
    Write-Host "删除: C:\Users\user\Downloads\file2.txt"
    try {
        Remove-Item "C:\Users\user\Downloads\file2.txt" -Force
        $deletedCount++
        $deletedSize += 1048576
    } catch {
        Write-Host "❌ 删除失败: C:\Users\user\Downloads\file2.txt" -ForegroundColor Red
        $failedCount++
    }
} else {
    Write-Host "⚠️  文件不存在: C:\Users\user\Downloads\file2.txt" -ForegroundColor Yellow
}

# ... 更多删除命令 ...

# ============================================================================
# 删除完成，显示统计信息
# ============================================================================
Write-Host ""
Write-Host "==============================================================================" -ForegroundColor Cyan
Write-Host "📊 删除统计:" -ForegroundColor Cyan
Write-Host "==============================================================================" -ForegroundColor Cyan
Write-Host "✅ 成功删除: $deletedCount 个文件" -ForegroundColor Green
Write-Host "❌ 失败数量: $failedCount 个文件" -ForegroundColor Red
$sizeInMB = [math]::Round($deletedSize / 1MB, 2)
Write-Host "💾 节省空间: $sizeInMB MB ($deletedSize bytes)" -ForegroundColor Green
Write-Host "==============================================================================" -ForegroundColor Cyan

# 暂停，等待用户按键
Write-Host ""
Write-Host "按任意键退出..." -ForegroundColor Gray
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
```

#### 执行示例

```bash
$ bash delete_dups.sh
⚠️  警告: 即将删除重复文件！
扫描路径: /home/user/Downloads
重复组数: 2
将删除文件数: 3
可节省空间: 5.00 MB

确认要继续吗? (yes/no): yes
删除: /home/user/Downloads/file2.txt
删除: /home/user/Downloads/file3.txt
删除: /home/user/Downloads/file5.txt

==============================================================================
📊 删除统计:
==============================================================================
✅ 成功删除: 3 个文件
❌ 失败数量: 0 个文件
💾 节省空间: 5.00MB
==============================================================================
```

#### 应用场景

1. **批量清理**
```bash
# 扫描多个目录，生成删除脚本
dupfinder ~/Downloads --delete-script clean_downloads.sh
dupfinder ~/Pictures --delete-script clean_pictures.sh
dupfinder ~/Videos --delete-script clean_videos.sh

# 查看所有脚本，决定执行哪些
cat clean_downloads.sh
cat clean_pictures.sh
cat clean_videos.sh

# 执行清理
bash clean_downloads.sh
```

2. **自定义保留逻辑**
```bash
# 生成脚本
dupfinder ~/backup --delete-script delete.sh

# 编辑脚本，调整保留哪个文件
# 例如：保留最新的而不是第一个
vim delete.sh

# 执行自定义的删除
bash delete.sh
```

3. **定期清理**
```bash
# 添加到 crontab，每周清理一次
# 0 2 * * 0 /path/to/cleanup_script.sh

#!/bin/bash
# cleanup_script.sh
dupfinder ~/Downloads --delete-script /tmp/cleanup.sh
# 自动确认（谨慎使用！）
echo "yes" | bash /tmp/cleanup.sh
```

---

### 3. 组合使用

JSON 输出和删除脚本可以同时使用，实现完整的扫描-分析-清理流程：

```bash
# 一次性生成 JSON 报告和删除脚本
dupfinder -v -S ~/Downloads \
    --json report.json \
    --delete-script delete_dups.sh

# 查看 JSON 报告，分析重复文件
cat report.json | jq '.statistics'

# 查看删除脚本，确认要删除的文件
cat delete_dups.sh | grep "保留:"
cat delete_dups.sh | grep "删除:"

# 决定是否执行删除
bash delete_dups.sh
```

---

## 📊 技术实现

### Rust 依赖

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = "0.4"
```

### 核心特性

- ✅ 使用 `serde` 实现序列化/反序列化
- ✅ 支持 `#[derive(Serialize)]` 自动生成序列化代码
- ✅ 使用 `chrono` 生成 RFC3339 格式时间戳
- ✅ Shell 脚本自动设置执行权限（Unix 系统）
- ✅ 脚本包含完整的错误处理和统计

---

## 🚀 性能影响

新增的 JSON 和删除脚本功能对扫描性能**没有影响**：

- JSON 序列化在扫描完成后进行
- 删除脚本生成同样在扫描完成后进行
- 二进制大小从 651KB 增加到 758KB（增加约 100KB）

---

## 📝 更新日志

**v0.2.0** (2025-12-02)
- ✨ 新增：JSON 输出功能 (`--json`)
- ✨ 新增：删除脚本生成功能 (`--delete-script`)
- 📦 新增依赖：`serde`, `serde_json`, `chrono`
- 📄 更新文档：README.md, USAGE.md, CHEATSHEET.md

---

## 🎯 未来计划

- [ ] 交互式删除模式（无需生成脚本）
- [ ] 支持 CSV 输出格式
- [ ] 支持自定义 JSON 模板
- [ ] 删除脚本支持 PowerShell（Windows）
- [ ] 删除前自动备份功能

---

**Happy duplicate hunting! 🔍**

