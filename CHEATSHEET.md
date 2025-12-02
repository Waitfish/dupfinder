# DupFinder 快速参考

## 📋 常用命令

```bash
# 最常用：扫描目录，显示大小和相对路径
dupfinder -S -R /path/to/dir

# 详细模式查看验证过程
dupfinder -v -S /path/to/dir

# 只扫描当前目录（不递归）
dupfinder -n .

# 包含硬链接
dupfinder -H /data

# 导出 JSON 报告
dupfinder /path/to/dir --json report.json

# 生成删除脚本
dupfinder /path/to/dir --delete-script delete.sh

# 一次性：扫描 + JSON + 删除脚本
dupfinder -v -S /path/to/dir --json report.json --delete-script delete.sh
```

## 🎯 参数速查

| 参数 | 简写 | 功能 | 默认 |
|------|------|------|------|
| `--recursive` | `-r` | 递归扫描 | ✅ 开启 |
| `--no-recursive` | `-n` | 不递归 | 关闭 |
| `--verbose` | `-v` | 详细模式 | 关闭 |
| `--size` | `-S` | 显示大小 | 关闭 |
| `--relative` | `-R` | 相对路径 | 关闭（默认绝对路径） |
| `--hardlinks` | `-H` | 包含硬链接 | 关闭（默认跳过） |
| `--json <FILE>` | - | JSON 输出 | 无 |
| `--delete-script <FILE>` | - | 生成删除脚本 | 无 |

## 🚀 快速场景

### 清理下载文件夹
```bash
dupfinder -S ~/Downloads
```

### 检查代码仓库
```bash
cd ~/project
dupfinder -R .
```

### 深度扫描（详细信息）
```bash
dupfinder -v -S -R /data
```

### 只扫描当前目录
```bash
dupfinder -n .
```

### 生成报告和删除脚本
```bash
# 扫描 + 生成 JSON 和删除脚本
dupfinder ~/Downloads --json report.json --delete-script delete.sh

# 查看报告
cat report.json | jq '.statistics'

# 执行删除（需要确认）
bash delete.sh
```

### CI/CD 检查
```bash
# 在 CI 中检查重复文件
dupfinder . --json dup_report.json
deletable=$(jq '.statistics.deletable_files' dup_report.json)
[ "$deletable" -gt 0 ] && echo "发现重复文件" && exit 1
```

## 📊 4 层验证流程

```
文件大小比较 (毫秒级)
    ↓
部分哈希 8KB (秒级)
    ↓
完整 MD5 (秒到分钟级)
    ↓
逐字节比较 (秒到分钟级)
    ↓
100% 准确结果
```

## 💡 Pro Tips

- 💻 **大目录先不加 -v**：快速看结果
- 📁 **项目目录用 -R**：相对路径更清晰
- 💾 **查看空间用 -S**：了解可节省多少
- 🔍 **调试问题用 -v**：查看验证细节
- ⚡ **Release 版本更快**：比 debug 快 10 倍

## 🎨 输出格式

### 默认输出（绝对路径）
```
组 1:
  /home/user/file1.txt
  /home/user/backup/file1.txt
```

### 相对路径 (-R)
```
组 1:
  ./file1.txt
  ./backup/file1.txt
```

### 带大小 (-S)
```
组 1:
  文件大小: 1048576 bytes
  /home/user/file.dat
  /home/user/copy.dat
  
可节省空间: 1.00 MB (1048576 bytes)
```

## 🔧 高级用法

### 组合命令

```bash
# 查找大于 1MB 的重复文件
find . -type f -size +1M -exec dirname {} \; | sort -u | while read dir; do
    dupfinder -n -S "$dir"
done

# 生成报告
dupfinder -S /data > report.txt
```

---

**快速安装**：
```bash
cargo install dupfinder
# 或
wget <release-url> && chmod +x dupfinder && sudo mv dupfinder /usr/local/bin/
```

