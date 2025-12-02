#!/bin/bash
# 智能发布脚本

set -e

echo "🔍 DupFinder - 发布助手"
echo "================================"
echo ""

# 检查是否在正确的目录
if [ ! -f "Cargo.toml" ]; then
    echo "❌ 错误: 请在项目根目录运行此脚本"
    exit 1
fi

# 获取包名
PACKAGE_NAME=$(grep '^name' Cargo.toml | head -n 1 | cut -d '"' -f 2)

# 获取当前版本
CURRENT_VERSION=$(grep '^version' Cargo.toml | head -n 1 | cut -d '"' -f 2)
echo "📦 当前本地版本: $CURRENT_VERSION"

# 函数：检查版本是否存在于 crates.io
check_version_exists() {
    local version=$1
    echo "🔍 检查 crates.io 上的版本..."
    
    # 方法1: 尝试 dry-run 发布来检测版本冲突
    if cargo publish --registry crates-io --dry-run 2>&1 | grep -q "already exists"; then
        echo "⚠️  版本 $version 已存在于 crates.io"
        
        # 尝试获取最新版本
        if cargo search "$PACKAGE_NAME" --limit 1 2>/dev/null | grep -q "^$PACKAGE_NAME"; then
            LATEST_VERSION=$(cargo search "$PACKAGE_NAME" --limit 1 | grep "^$PACKAGE_NAME" | sed 's/.*= "\(.*\)".*/\1/')
            echo "📦 crates.io 最新版本: $LATEST_VERSION"
        fi
        return 0  # 版本已存在
    fi
    
    # 方法2: 通过 cargo search 检查
    if cargo search "$PACKAGE_NAME" --limit 1 2>/dev/null | grep -q "^$PACKAGE_NAME"; then
        LATEST_VERSION=$(cargo search "$PACKAGE_NAME" --limit 1 | grep "^$PACKAGE_NAME" | sed 's/.*= "\(.*\)".*/\1/')
        echo "📦 crates.io 最新版本: $LATEST_VERSION"
        
        if [ "$version" = "$LATEST_VERSION" ] || [ "$version" \< "$LATEST_VERSION" ]; then
            return 0  # 版本已存在或更旧
        fi
    else
        echo "ℹ️  包尚未发布到 crates.io（或索引未更新）"
    fi
    
    return 1  # 版本不存在
}

# 函数：递增版本号
increment_version() {
    local version=$1
    local part=${2:-patch}  # patch, minor, major
    
    IFS='.' read -r -a parts <<< "$version"
    local major="${parts[0]}"
    local minor="${parts[1]}"
    local patch="${parts[2]}"
    
    case $part in
        patch)
            patch=$((patch + 1))
            ;;
        minor)
            minor=$((minor + 1))
            patch=0
            ;;
        major)
            major=$((major + 1))
            minor=0
            patch=0
            ;;
    esac
    
    echo "$major.$minor.$patch"
}

# 函数：更新 Cargo.toml 中的版本
update_version() {
    local new_version=$1
    sed -i "0,/^version = \".*\"/s//version = \"$new_version\"/" Cargo.toml
    echo "✅ 已更新 Cargo.toml 版本为: $new_version"
}

# 检查版本是否已存在
if check_version_exists "$CURRENT_VERSION"; then
    echo ""
    echo "⚠️  版本 $CURRENT_VERSION 已存在于 crates.io！"
    echo ""
    echo "建议的新版本号："
    
    PATCH_VERSION=$(increment_version "$CURRENT_VERSION" "patch")
    MINOR_VERSION=$(increment_version "$CURRENT_VERSION" "minor")
    MAJOR_VERSION=$(increment_version "$CURRENT_VERSION" "major")
    
    echo "  1) $PATCH_VERSION (修复bug/小改进)"
    echo "  2) $MINOR_VERSION (新功能，向后兼容)"
    echo "  3) $MAJOR_VERSION (重大变更，可能不兼容)"
    echo "  4) 手动输入版本号"
    echo "  5) 取消发布"
    echo ""
    read -p "请选择 (1-5): " version_choice
    
    case $version_choice in
        1)
            NEW_VERSION=$PATCH_VERSION
            ;;
        2)
            NEW_VERSION=$MINOR_VERSION
            ;;
        3)
            NEW_VERSION=$MAJOR_VERSION
            ;;
        4)
            read -p "请输入新版本号 (格式: x.y.z): " NEW_VERSION
            # 简单验证
            if ! [[ $NEW_VERSION =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
                echo "❌ 版本号格式无效"
                exit 1
            fi
            ;;
        5)
            echo "❌ 已取消"
            exit 0
            ;;
        *)
            echo "❌ 无效选择"
            exit 1
            ;;
    esac
    
    echo ""
    echo "📝 准备将版本从 $CURRENT_VERSION 更新到 $NEW_VERSION"
    read -p "确认继续？(y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "❌ 已取消"
        exit 1
    fi
    
    # 更新版本号
    update_version "$NEW_VERSION"
    CURRENT_VERSION=$NEW_VERSION
    
    # 立即提交版本更新
    if [ -d ".git" ]; then
        echo ""
        echo "📝 提交版本更新到 Git..."
        git add Cargo.toml Cargo.lock
        git commit -m "Bump version to $NEW_VERSION"
        
        read -p "是否推送到远程仓库？(Y/n) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Nn]$ ]]; then
            git push
            echo "✅ 已推送到远程仓库"
        fi
    fi
    
    echo ""
fi

# 再次检查 Git 状态（以防有其他未提交的更改）
if [ -d ".git" ]; then
    if [ -n "$(git status --porcelain)" ]; then
        echo "📝 发现未提交的更改："
        git status --short
        echo ""
        read -p "是否自动提交这些更改？(Y/n) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Nn]$ ]]; then
            # 自动提交所有更改
            echo "📝 自动提交更改..."
            git add -A
            
            # 生成提交信息
            COMMIT_MSG="feat: Add glob/regex filtering for v$CURRENT_VERSION

- Add --pattern/-p flag for glob pattern filtering
- Add --regex flag for regex pattern filtering
- Update documentation with examples
- Bump version to $CURRENT_VERSION"
            
            git commit -m "$COMMIT_MSG"
            
            read -p "是否推送到远程仓库？(Y/n) " -n 1 -r
            echo
            if [[ ! $REPLY =~ ^[Nn]$ ]]; then
                git push
                echo "✅ 已推送到远程仓库"
            fi
        else
            echo "❌ 已取消发布"
            exit 1
        fi
    fi
fi

echo ""
echo "📦 即将发布版本: $CURRENT_VERSION"
echo ""

# 询问发布类型
echo "选择发布方式:"
echo "  1) 测试发布 (dry-run)"
echo "  2) 发布到 crates.io"
echo "  3) 创建 GitHub Release (需要先推送代码)"
echo "  4) 全部执行 (crates.io + GitHub tag)"
echo ""
read -p "请选择 (1-4): " choice

case $choice in
    1)
        echo "🧪 执行测试发布..."
        cargo publish --registry crates-io --dry-run
        echo ""
        echo "✅ 测试完成！如果没有错误，可以执行正式发布"
        ;;
    2)
        echo "📤 发布到 crates.io..."
        
        # 构建
        echo "🔨 构建项目..."
        cargo build --release
        
        # 运行测试（如果有）
        if cargo test --dry-run 2>&1 | grep -q "test"; then
            echo "🧪 运行测试..."
            cargo test
        else
            echo "ℹ️  未找到测试，跳过测试步骤"
        fi
        
        # 检查并提交 Cargo.lock 更新
        if [ -d ".git" ]; then
            if git diff --quiet Cargo.lock 2>/dev/null; then
                echo "ℹ️  Cargo.lock 无变化"
            else
                echo "📝 提交 Cargo.lock 更新..."
                git add Cargo.lock
                git commit -m "chore: Update Cargo.lock for v$CURRENT_VERSION"
                git push
                echo "✅ Cargo.lock 已更新并推送"
            fi
        fi
        
        # 发布
        echo "📤 发布中..."
        cargo publish --registry crates-io
        
        echo ""
        echo "🎉 发布成功！"
        echo "用户现在可以通过以下命令安装:"
        echo "  cargo install dupfinder"
        ;;
    3)
        echo "🏷️  创建 GitHub Release..."
        
        if [ ! -d ".git" ]; then
            echo "❌ 错误: 这不是一个 Git 仓库"
            exit 1
        fi
        
        # 创建标签
        TAG="v$CURRENT_VERSION"
        echo "创建标签: $TAG"
        
        # 检查标签是否已存在
        if git rev-parse "$TAG" >/dev/null 2>&1; then
            echo "⚠️  标签 $TAG 已存在"
            read -p "是否删除旧标签并重新创建？(y/N) " -n 1 -r
            echo
            if [[ $REPLY =~ ^[Yy]$ ]]; then
                git tag -d "$TAG"
                git push origin ":refs/tags/$TAG" 2>/dev/null || true
            else
                exit 1
            fi
        fi
        
        git tag -a "$TAG" -m "Release $TAG"
        git push origin "$TAG"
        
        echo ""
        echo "✅ 标签已推送！GitHub Actions 将自动构建并创建 Release"
        echo "查看进度: https://github.com/Waitfish/dupfinder/actions"
        ;;
    4)
        echo "🚀 执行完整发布流程..."
        
        # 1. 构建和测试
        echo "🔨 构建项目..."
        cargo build --release
        
        if cargo test --dry-run 2>&1 | grep -q "test"; then
            echo "🧪 运行测试..."
            cargo test
        fi
        
        # 1.5 检查并提交 Cargo.lock 更新
        if [ -d ".git" ]; then
            if git diff --quiet Cargo.lock 2>/dev/null; then
                echo "ℹ️  Cargo.lock 无变化"
            else
                echo "📝 提交 Cargo.lock 更新..."
                git add Cargo.lock
                git commit -m "chore: Update Cargo.lock for v$CURRENT_VERSION"
                git push
                echo "✅ Cargo.lock 已更新并推送"
            fi
        fi
        
        # 2. 发布到 crates.io
        echo "📤 发布到 crates.io..."
        cargo publish --registry crates-io
        
        echo ""
        echo "⏳ 等待 crates.io 索引更新（30秒）..."
        sleep 30
        
        # 3. 创建 Git 标签（版本已经在前面提交过了）
        if [ -d ".git" ]; then
            TAG="v$CURRENT_VERSION"
            echo "🏷️  创建标签: $TAG"
            
            # 删除已存在的标签
            if git rev-parse "$TAG" >/dev/null 2>&1; then
                git tag -d "$TAG"
                git push origin ":refs/tags/$TAG" 2>/dev/null || true
                sleep 2
            fi
            
            git tag -a "$TAG" -m "Release $TAG"
            git push origin "$TAG"
        fi
        
        echo ""
        echo "🎉 发布完成！"
        echo ""
        echo "📋 已完成:"
        echo "  ✅ 发布到 crates.io (https://crates.io/crates/dupfinder)"
        echo "  ✅ 创建 Git 标签 v$CURRENT_VERSION"
        echo "  ✅ 触发 GitHub Release 构建"
        echo ""
        echo "🔗 查看链接:"
        echo "  📦 crates.io: https://crates.io/crates/dupfinder"
        echo "  🏷️  GitHub Actions: https://github.com/Waitfish/dupfinder/actions"
        echo "  📥 Releases: https://github.com/Waitfish/dupfinder/releases"
        ;;
    *)
        echo "❌ 无效选择"
        exit 1
        ;;
esac

echo ""
echo "🎊 感谢使用 DupFinder!"

