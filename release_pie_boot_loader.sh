#!/bin/bash

# 自动化发布 pie-boot-loader-aarch64 release 脚本
# 使用 GitHub Token 和 RESTful API
# 作者: GitHub Copilot
# 创建时间: 2025-09-19

set -e  # 如果任何命令失败，立即退出

# GitHub API 配置
GITHUB_OWNER="rcore-os"
GITHUB_REPO="somehal"
GITHUB_API_BASE="https://api.github.com"

# 全局变量
CURRENT_TAG=""

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 打印带颜色的消息
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 检查必要的工具是否存在
check_dependencies() {
    print_info "检查必要的工具..."
    
    if ! command -v git &> /dev/null; then
        print_error "git 未安装或不在PATH中"
        exit 1
    fi
    
    if ! command -v curl &> /dev/null; then
        print_error "curl 未安装或不在PATH中"
        exit 1
    fi
    
    if ! command -v jq &> /dev/null; then
        print_error "jq 未安装或不在PATH中"
        print_info "请安装 jq: sudo apt install jq"
        exit 1
    fi
    
    if ! command -v cargo &> /dev/null; then
        print_error "cargo 未安装或不在PATH中"
        exit 1
    fi
    
    # 检查 GitHub Token
    if [ -z "$GITHUB_TOKEN" ] && [ ! -f ~/.github_token ]; then
        print_error "GITHUB_TOKEN 环境变量未设置且未找到 ~/.github_token 文件"
        print_info "请设置 GitHub Personal Access Token:"
        print_info "export GITHUB_TOKEN=your_token_here"
        print_info "或创建 ~/.github_token 文件"
        exit 1
    fi
    
    print_success "所有依赖工具检查通过"
}

# 获取 GitHub Token
get_github_token() {
    if [ -n "$GITHUB_TOKEN" ]; then
        echo "$GITHUB_TOKEN"
    elif [ -f ~/.github_token ]; then
        cat ~/.github_token | tr -d '\n'
    else
        print_error "未找到 GitHub Token"
        print_info "请设置 GITHUB_TOKEN 环境变量或创建 ~/.github_token 文件"
        exit 1
    fi
}

# GitHub API 调用函数
github_api_call() {
    local method="$1"
    local endpoint="$2"
    local data="$3"
    local token=$(get_github_token)
    
    local curl_args=(
        -s
        -w "HTTP_CODE:%{http_code}\n"
        -H "Authorization: token $token"
        -H "Accept: application/vnd.github.v3+json"
        -X "$method"
    )
    
    if [ -n "$data" ]; then
        curl_args+=(-H "Content-Type: application/json" -d "$data")
    fi
    
    local response=$(curl "${curl_args[@]}" "$GITHUB_API_BASE$endpoint")
    local http_code=$(echo "$response" | grep "HTTP_CODE:" | cut -d: -f2)
    local body=$(echo "$response" | grep -v "HTTP_CODE:")
    
    # 如果是调试模式，显示详细信息
    if [ "$DEBUG" == "1" ]; then
        echo "[DEBUG] API调用: $method $endpoint"
        echo "[DEBUG] HTTP状态码: $http_code"
        echo "[DEBUG] 响应: $body" | head -5
    fi
    
    # 检查HTTP状态码
    if [[ "$http_code" =~ ^2[0-9][0-9]$ ]]; then
        echo "$body"
    else
        print_error "API调用失败 ($method $endpoint): HTTP $http_code"
        echo "$body" | jq . 2>/dev/null || echo "$body"
        exit 1
    fi
}

# 步骤1: 执行构建脚本
build_binary() {
    print_info "执行 ./loader/pie-boot-loader-aarch64/build.sh"
    
    if [ ! -f "./loader/pie-boot-loader-aarch64/build.sh" ]; then
        print_error "构建脚本不存在: ./loader/pie-boot-loader-aarch64/build.sh"
        exit 1
    fi
    
    ./loader/pie-boot-loader-aarch64/build.sh
    
    # 检查二进制文件是否生成成功
    binary_path="target/aarch64-unknown-none-softfloat/release/pie-boot-loader-aarch64.bin"
    if [ ! -f "$binary_path" ]; then
        print_error "构建失败: 未找到二进制文件 $binary_path"
        exit 1
    fi
    
    print_success "构建完成，二进制文件生成: $binary_path"
}

# 步骤2: 检查当前标签（修改为检查所有标签中是否存在 pie-boot-loader-aarch64-* 格式）
check_current_tag() {
    print_info "检查当前标签..."
    
    # 获取当前提交的所有标签
    local all_tags=$(git tag --points-at HEAD)
    
    if [ -z "$all_tags" ]; then
        print_error "当前HEAD没有对应的标签"
        print_info "请先创建标签，例如: git tag pie-boot-loader-aarch64-v0.3.0"
        exit 1
    fi
    
    print_info "当前HEAD的所有标签:"
    echo "$all_tags"
    
    # 在所有标签中查找符合 pie-boot-loader-aarch64-* 格式的标签
    local target_tag=""
    while IFS= read -r tag; do
        if [[ "$tag" =~ ^pie-boot-loader-aarch64-.* ]]; then
            target_tag="$tag"
            print_success "找到符合条件的标签: $target_tag"
            break
        fi
    done <<< "$all_tags"
    
    if [ -z "$target_tag" ]; then
        print_error "当前HEAD没有符合 pie-boot-loader-aarch64-* 格式的标签"
        print_info "当前标签: $all_tags"
        print_info "请创建符合格式的标签，例如: git tag pie-boot-loader-aarch64-v0.3.0"
        exit 1
    fi
    
    # 使用全局变量而不是echo返回值
    CURRENT_TAG="$target_tag"
}

# 步骤3: 检查远程是否存在同样标签
check_remote_tag() {
    local tag="$1"
    print_info "检查远程仓库是否存在标签: $tag"
    
    # 使用 GitHub API 检查标签，需要URL编码标签名
    local encoded_tag=$(echo "$tag" | sed 's|/|%2F|g')
    local response=$(github_api_call "GET" "/repos/$GITHUB_OWNER/$GITHUB_REPO/git/refs/tags/$encoded_tag")
    local ref_exists=$(echo "$response" | jq -r '.ref // empty')
    
    if [ -n "$ref_exists" ] && [ "$ref_exists" != "null" ]; then
        print_success "远程仓库存在标签: $tag"
        return 0
    else
        # 检查是否是权限问题或其他错误
        local error_message=$(echo "$response" | jq -r '.message // empty')
        if [ -n "$error_message" ] && [ "$error_message" != "null" ]; then
            print_error "GitHub API 错误: $error_message"
        else
            print_warning "远程仓库不存在标签: $tag"
            print_info "请先推送标签到远程仓库: git push origin $tag"
        fi
        exit 1
    fi
}

# 步骤4: 检查标签的release中是否存在二进制文件
check_release_assets() {
    local tag="$1"
    print_info "检查标签 $tag 的 release 中是否存在 pie-boot-loader-aarch64.bin"
    
    # 使用 GitHub API 获取 release 信息
    local response=$(github_api_call "GET" "/repos/$GITHUB_OWNER/$GITHUB_REPO/releases/tags/$tag")
    local release_id=$(echo "$response" | jq -r '.id // empty')
    
    if [ -z "$release_id" ] || [ "$release_id" == "null" ]; then
        print_warning "标签 $tag 的 release 不存在"
        print_info "将创建新的 release..."
        return 1
    fi
    
    # 检查 release 的 assets
    local assets=$(echo "$response" | jq -r '.assets[].name')
    
    if echo "$assets" | grep -q "pie-boot-loader-aarch64.bin"; then
        print_warning "pie-boot-loader-aarch64.bin 已存在于 release $tag 中"
        print_info "跳过上传"
        return 0
    else
        print_info "pie-boot-loader-aarch64.bin 不存在于 release $tag 中，需要上传"
        return 1
    fi
}

# 步骤5: 上传二进制文件到release
upload_binary_to_release() {
    local tag="$1"
    local binary_path="target/aarch64-unknown-none-softfloat/release/pie-boot-loader-aarch64.bin"
    
    print_info "准备上传二进制文件到 release..."
    
    # 检查二进制文件是否存在
    if [ ! -f "$binary_path" ]; then
        print_error "二进制文件不存在: $binary_path"
        exit 1
    fi
    
    # 首先检查 release 是否存在
    local response=$(github_api_call "GET" "/repos/$GITHUB_OWNER/$GITHUB_REPO/releases/tags/$tag")
    local release_id=$(echo "$response" | jq -r '.id // empty')
    
    if [ -z "$release_id" ] || [ "$release_id" == "null" ]; then
        print_info "创建新的 release: $tag"
        
        # 创建 release
        local create_data=$(jq -n \
            --arg tag "$tag" \
            --arg name "$tag" \
            --arg body "自动发布 $tag

## 包含文件
- pie-boot-loader-aarch64.bin: ARM64 架构的 PIE 引导加载器

## 构建信息
- 目标架构: aarch64-unknown-none-softfloat
- 构建模式: release
- PIC/PIE 模式: 启用

## 使用方法
下载 pie-boot-loader-aarch64.bin 文件即可直接使用。" \
            '{
                tag_name: $tag,
                target_commitish: $tag,
                name: $name,
                body: $body,
                draft: false,
                prerelease: false
            }')
        
        local create_response=$(github_api_call "POST" "/repos/$GITHUB_OWNER/$GITHUB_REPO/releases" "$create_data")
        release_id=$(echo "$create_response" | jq -r '.id')
        
        if [ -z "$release_id" ] || [ "$release_id" == "null" ]; then
            print_error "创建 release 失败"
            echo "$create_response" | jq .
            exit 1
        fi
        
        print_success "Release 创建成功，ID: $release_id"
    fi
    
    # 上传二进制文件
    print_info "上传二进制文件: $binary_path"
    
    local upload_url="https://uploads.github.com/repos/$GITHUB_OWNER/$GITHUB_REPO/releases/$release_id/assets"
    local token=$(get_github_token)
    
    local upload_response=$(curl -s \
        -w "HTTP_CODE:%{http_code}\n" \
        -H "Authorization: token $token" \
        -H "Content-Type: application/octet-stream" \
        --data-binary @"$binary_path" \
        "$upload_url?name=pie-boot-loader-aarch64.bin&label=pie-boot-loader-aarch64.bin")
    
    local http_code=$(echo "$upload_response" | grep "HTTP_CODE:" | cut -d: -f2)
    local upload_body=$(echo "$upload_response" | grep -v "HTTP_CODE:")
    
    if [ "$http_code" == "201" ]; then
        local asset_id=$(echo "$upload_body" | jq -r '.id')
        print_success "二进制文件上传完成，Asset ID: $asset_id"
    else
        print_error "上传二进制文件失败，HTTP码: $http_code"
        echo "$upload_body" | jq . 2>/dev/null || echo "$upload_body"
        exit 1
    fi
    
    # 显示 release 信息
    print_info "Release 详情:"
    local final_response=$(github_api_call "GET" "/repos/$GITHUB_OWNER/$GITHUB_REPO/releases/tags/$tag")
    echo "$final_response" | jq -r '.html_url'
}

# 主函数
main() {
    print_info "开始 pie-boot-loader-aarch64 release 发布流程..."
    
    # 检查依赖
    check_dependencies
    
    # 步骤1: 构建二进制文件
    build_binary
    
    # 步骤2: 检查当前标签
    check_current_tag
    print_info "使用标签: $CURRENT_TAG"
    
    # 步骤3: 检查远程标签
    check_remote_tag "$CURRENT_TAG"
    
    # 步骤4: 检查release资源
    if check_release_assets "$CURRENT_TAG"; then
        print_success "pie-boot-loader-aarch64.bin 已存在，无需重复上传"
        exit 0
    fi
    
    # 步骤5: 上传二进制文件
    upload_binary_to_release "$CURRENT_TAG"
    
    print_success "pie-boot-loader-aarch64 release 发布完成!"
    print_info "访问以下链接查看 release: https://github.com/rcore-os/somehal/releases/tag/$CURRENT_TAG"
}

# 错误处理
trap 'print_error "脚本执行失败，退出码: $?"' ERR

# 帮助信息
if [ "$1" == "--help" ] || [ "$1" == "-h" ]; then
    echo "用法: $0 [选项]"
    echo ""
    echo "自动化发布 pie-boot-loader-aarch64 release 的脚本"
    echo ""
    echo "功能:"
    echo "  1. 执行 ./loader/pie-boot-loader-aarch64/build.sh 构建二进制文件"
    echo "  2. 检查当前HEAD的标签中是否存在 pie-boot-loader-aarch64-* 格式的标签"
    echo "  3. 使用 GitHub API 检查远程仓库是否存在相同标签"
    echo "  4. 检查该标签的 release 中是否已存在 pie-boot-loader-aarch64.bin"
    echo "  5. 如果不存在，则上传二进制文件到 release"
    echo ""
    echo "选项:"
    echo "  -h, --help    显示此帮助信息"
    echo "  --debug       启用调试模式，显示详细的API调用信息"
    echo ""
    echo "环境变量:"
    echo "  GITHUB_TOKEN  GitHub Personal Access Token (必需)"
    echo "  DEBUG=1       启用调试模式"
    echo ""
    echo "先决条件:"
    echo "  - 安装 Git, curl, jq"
    echo "  - 安装 Rust 工具链"
    echo "  - 设置 GITHUB_TOKEN 环境变量或创建 ~/.github_token 文件"
    echo "  - 当前HEAD必须有 pie-boot-loader-aarch64-* 格式的标签"
    echo ""
    echo "GitHub Token 设置:"
    echo "  export GITHUB_TOKEN=your_token_here"
    echo "  或者: echo 'your_token_here' > ~/.github_token"
    echo ""
    echo "调试模式:"
    echo "  DEBUG=1 ./release_pie_boot_loader.sh"
    exit 0
fi

# 检查调试模式
if [ "$1" == "--debug" ]; then
    export DEBUG=1
    shift
fi

# 执行主函数
main