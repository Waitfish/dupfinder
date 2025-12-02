# DupFinder 发布指南

## 📦 发布信息

**版本**: v0.2.0  
**发布日期**: 2025-12-02  
**发布状态**: ✅ 成功

---

## 🎯 发布内容

### 已完成项

- ✅ **代码推送到 GitHub**
  - Repository: https://github.com/Waitfish/dupfinder
  - 分支: master
  - 提交: 12ad801

- ✅ **发布到 crates.io**
  - 版本: v0.2.0
  - 包名: dupfinder
  - 链接: https://crates.io/crates/dupfinder
  - 安装: `cargo install dupfinder`

- ✅ **创建 GitHub Release**
  - 标签: v0.2.0
  - 自动构建多平台二进制文件
  - 链接: https://github.com/Waitfish/dupfinder/releases

- ✅ **GitHub Actions 配置**
  - CI 工作流: 自动检查、构建和 lint
  - Release 工作流: 自动构建多平台二进制文件
  - 链接: https://github.com/Waitfish/dupfinder/actions

---

## 🚀 支持的平台

GitHub Release 自动构建以下平台的二进制文件：

1. **Linux x86_64 (GNU)**
   - 文件: `dupfinder-linux-x86_64`
   - 适用于大多数 Linux 发行版

2. **Linux x86_64 (musl)**
   - 文件: `dupfinder-linux-x86_64-musl`
   - 静态链接，兼容性更好

3. **macOS x86_64**
   - 文件: `dupfinder-macos-x86_64`
   - Intel Mac

4. **macOS aarch64**
   - 文件: `dupfinder-macos-aarch64`
   - Apple Silicon (M1/M2/M3)

5. **Windows x86_64**
   - 文件: `dupfinder-windows-x86_64.exe`
   - 64位 Windows

---

## 📥 安装方式

### 方式 1: 从 crates.io 安装（推荐）

```bash
cargo install dupfinder
```

**优点**:
- 自动编译优化版本
- 自动安装到 `~/.cargo/bin`
- 容易更新

### 方式 2: 从 GitHub Release 下载

1. 访问: https://github.com/Waitfish/dupfinder/releases/tag/v0.2.0
2. 下载适合您系统的二进制文件
3. 赋予执行权限并移动到 PATH

**Linux/macOS:**
```bash
chmod +x dupfinder-*
sudo mv dupfinder-* /usr/local/bin/dupfinder
```

**Windows:**
- 下载 `.exe` 文件
- 移动到 PATH 中的目录

### 方式 3: 从源码编译

```bash
git clone https://github.com/Waitfish/dupfinder.git
cd dupfinder
cargo build --release
sudo cp target/release/dupfinder /usr/local/bin/
```

---

## 📄 Cargo.toml 元数据

```toml
[package]
name = "dupfinder"
version = "0.2.0"
edition = "2021"
authors = ["waitfish <daiwj2024@outlook.com>"]
description = "A fast duplicate file finder with JSON export and safe deletion scripts"
license = "MIT"
repository = "https://github.com/Waitfish/dupfinder"
homepage = "https://github.com/Waitfish/dupfinder"
documentation = "https://github.com/Waitfish/dupfinder"
readme = "README.md"
keywords = ["duplicate", "file-finder", "fdupes", "deduplication", "cli"]
categories = ["command-line-utilities", "filesystem"]
```

---

## 🛠️ 发布工具

### publish.sh 脚本

项目包含智能发布脚本 `publish.sh`，支持：

1. **版本检查**: 自动检查 crates.io 上的现有版本
2. **版本升级**: 智能建议新版本号（patch/minor/major）
3. **自动提交**: 自动提交版本更新到 Git
4. **多种发布方式**:
   - 测试发布（dry-run）
   - 只发布到 crates.io
   - 只创建 GitHub Release
   - 全部执行

**使用方法**:
```bash
./publish.sh
```

---

## 📊 项目统计

- **代码行数**: 803 行
- **文档文件**: 5 个（README, USAGE, CHEATSHEET, NEW_FEATURES, PROJECT_SUMMARY）
- **依赖数量**: 8 个
- **二进制大小**: 758KB（优化后）
- **编译时间**: ~5 秒

---

## 🔗 重要链接

| 资源 | 链接 |
|------|------|
| **crates.io** | https://crates.io/crates/dupfinder |
| **GitHub 仓库** | https://github.com/Waitfish/dupfinder |
| **GitHub Releases** | https://github.com/Waitfish/dupfinder/releases |
| **GitHub Actions** | https://github.com/Waitfish/dupfinder/actions |

---

## 🎯 下次发布流程

### 1. 更新代码

```bash
# 进行代码修改...
git add .
git commit -m "feat: Add new feature"
git push
```

### 2. 运行发布脚本

```bash
./publish.sh
```

脚本会自动：
- 检查版本冲突
- 建议新版本号
- 更新 Cargo.toml
- 提交版本更新
- 发布到 crates.io
- 创建 GitHub 标签
- 触发 GitHub Release 构建

### 3. 验证发布

```bash
# 检查 crates.io
cargo search dupfinder --registry crates-io

# 检查 GitHub Release
# 访问: https://github.com/Waitfish/dupfinder/releases
```

---

## 📝 版本规范

遵循语义化版本（SemVer）：

- **MAJOR** (x.0.0): 不兼容的 API 变更
- **MINOR** (0.x.0): 向后兼容的新功能
- **PATCH** (0.0.x): 向后兼容的 bug 修复

**示例**:
- `0.2.0` → `0.2.1`: 修复 bug
- `0.2.0` → `0.3.0`: 添加新功能
- `0.2.0` → `1.0.0`: 重大变更

---

## ✅ 发布检查清单

发布前检查：

- [ ] 代码已编译通过 (`cargo build --release`)
- [ ] 测试已通过（如果有）
- [ ] 文档已更新（README, USAGE 等）
- [ ] CHANGELOG 已更新（如果维护）
- [ ] 版本号已更新
- [ ] Git 工作目录干净
- [ ] 所有更改已推送到 GitHub

发布后验证：

- [ ] crates.io 上可见新版本
- [ ] 可以通过 `cargo install` 安装
- [ ] GitHub Release 已创建
- [ ] 多平台二进制文件已构建
- [ ] CI/CD 工作流全部通过

---

## 🐛 问题排查

### 问题 1: crates.io 版本冲突

```
error: crate dupfinder@0.2.0 already exists
```

**解决方案**: 使用发布脚本，它会自动检测并建议新版本号。

### 问题 2: GitHub Actions 失败

**检查**:
- 访问: https://github.com/Waitfish/dupfinder/actions
- 查看失败的工作流
- 检查日志

**常见原因**:
- 权限不足（需要 `contents: write`）
- 标签已存在
- 构建依赖问题

### 问题 3: cargo search 找不到包

```bash
# 使用正确的 registry
cargo search dupfinder --registry crates-io
```

### 问题 4: 二进制文件未构建

- GitHub Release 工作流由 tag push 触发
- 检查 `.github/workflows/release.yml`
- 确保标签格式正确（`v*`）

---

## 🎊 总结

DupFinder v0.2.0 已成功发布到：

✅ **crates.io** - Rust 社区可以通过 `cargo install` 安装  
✅ **GitHub Releases** - 提供多平台预编译二进制文件  
✅ **GitHub Repository** - 完整的源代码和文档

用户现在可以轻松安装和使用 DupFinder 来查找和清理重复文件！

---

**Happy duplicate hunting! 🔍**

