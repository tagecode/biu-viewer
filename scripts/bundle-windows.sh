#!/usr/bin/env bash
# 生成 BiuViewer-v{version}-windows-x64.zip（exe + README.txt）。

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=scripts/utils.sh
source "${SCRIPT_DIR}/utils.sh"

APP_NAME="BiuViewer"
VERSION="$(get_version)"
BINARY="$(binary_path x86_64-pc-windows-msvc biuviewer.exe)"
STAGE_DIR="${BUNDLE_DIR}/windows"
OUTPUT="${BUNDLE_DIR}/${APP_NAME}-v${VERSION}-windows-x64.zip"

if [ ! -f "${BINARY}" ]; then
    echo "找不到可执行文件：${BINARY}，请先运行 cargo build --release --target x86_64-pc-windows-msvc" >&2
    exit 1
fi

echo "==> 打包 Windows ${VERSION}"
rm -rf "${STAGE_DIR}" "${OUTPUT}"
mkdir -p "${STAGE_DIR}"

cp "${BINARY}" "${STAGE_DIR}/biuviewer.exe"
cp "${ICON_DIR}/icon.ico" "${STAGE_DIR}/icon.ico"

cat > "${STAGE_DIR}/README.txt" <<EOF
BiuViewer ${VERSION}
====================

轻量跨平台图片查看器 / Lightweight cross-platform image viewer.

使用说明
--------
1. 双击 biuviewer.exe 启动。
2. 首次运行若出现 Windows SmartScreen 提示，点击「更多信息」→「仍要运行」。
3. 也可以把图片或文件夹拖到 biuviewer.exe 图标上直接打开。

快捷键
------
← / →      上一张 / 下一张
Home / End 首张 / 末张
+ / -      放大 / 缩小
0          适应窗口
R / ⇧R     顺时针 / 逆时针旋转
H / V      水平 / 垂直翻转
F          全屏
Ctrl+O     打开图片
Ctrl+⇧+O   打开文件夹

支持格式
--------
PNG、JPEG、GIF、BMP、WebP、TIFF、ICO
EOF

mkdir -p "${BUNDLE_DIR}"
make_zip "${STAGE_DIR}" "${OUTPUT}"
rm -rf "${STAGE_DIR}"

echo "==> 已生成 ${OUTPUT}"
