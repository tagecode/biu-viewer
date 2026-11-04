#!/usr/bin/env bash
# 生成 BiuViewer-v{version}-linux-x86_64.tar.gz（二进制、桌面文件、图标、安装脚本）。

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=scripts/utils.sh
source "${SCRIPT_DIR}/utils.sh"

APP_NAME="biuviewer"
VERSION="$(get_version)"
BINARY="$(binary_path x86_64-unknown-linux-gnu biuviewer)"
STAGE_ROOT="${BUNDLE_DIR}/linux"
STAGE_DIR="${STAGE_ROOT}/${APP_NAME}"
OUTPUT="${BUNDLE_DIR}/BiuViewer-v${VERSION}-linux-x86_64.tar.gz"

if [ ! -f "${BINARY}" ]; then
    echo "找不到可执行文件：${BINARY}，请先运行 cargo build --release --target x86_64-unknown-linux-gnu" >&2
    exit 1
fi

echo "==> 打包 Linux ${VERSION}"
rm -rf "${STAGE_ROOT}" "${OUTPUT}"
mkdir -p \
    "${STAGE_DIR}/bin" \
    "${STAGE_DIR}/share/applications" \
    "${STAGE_DIR}/share/icons/hicolor/256x256/apps" \
    "${STAGE_DIR}/share/doc/${APP_NAME}"

cp "${BINARY}" "${STAGE_DIR}/bin/${APP_NAME}"
chmod +x "${STAGE_DIR}/bin/${APP_NAME}"
cp "${ICON_DIR}/icon.png" "${STAGE_DIR}/share/icons/hicolor/256x256/apps/${APP_NAME}.png"

cat > "${STAGE_DIR}/share/applications/${APP_NAME}.desktop" <<EOF
[Desktop Entry]
Version=1.0
Type=Application
Name=BiuViewer
GenericName=Image Viewer
GenericName[zh_CN]=图片查看器
Comment=Lightweight cross-platform image viewer
Comment[zh_CN]=轻量跨平台图片查看器
Exec=${APP_NAME} %f
Icon=${APP_NAME}
Terminal=false
Categories=Graphics;Viewer;
MimeType=image/png;image/jpeg;image/gif;image/bmp;image/webp;image/tiff;image/x-icon;
StartupNotify=true
EOF

cat > "${STAGE_DIR}/install.sh" <<'EOF'
#!/usr/bin/env bash

set -euo pipefail

PREFIX="${PREFIX:-$HOME/.local}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "正在安装 BiuViewer 到 ${PREFIX}"

install -Dm755 "${SCRIPT_DIR}/bin/biuviewer" "${PREFIX}/bin/biuviewer"
install -Dm644 "${SCRIPT_DIR}/share/applications/biuviewer.desktop" \
    "${PREFIX}/share/applications/biuviewer.desktop"
install -Dm644 "${SCRIPT_DIR}/share/icons/hicolor/256x256/apps/biuviewer.png" \
    "${PREFIX}/share/icons/hicolor/256x256/apps/biuviewer.png"

if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "${PREFIX}/share/applications" >/dev/null 2>&1 || true
fi
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache "${PREFIX}/share/icons/hicolor" >/dev/null 2>&1 || true
fi

if ! printf '%s' "${PATH}" | grep -q "${PREFIX}/bin"; then
    echo ""
    echo "提示：${PREFIX}/bin 不在 PATH 中，请将其加入 shell 配置后重新登录。"
fi

echo ""
echo "安装完成，运行 'biuviewer' 或从应用菜单启动。"
EOF
chmod +x "${STAGE_DIR}/install.sh"

cat > "${STAGE_DIR}/README.md" <<EOF
# BiuViewer ${VERSION}

轻量跨平台图片查看器。

## 系统依赖

GPUI 需要以下运行库（Ubuntu/Debian）：

\`\`\`bash
sudo apt-get install -y libxkbcommon-x11-0 libwayland-client0 libvulkan1 \\
  libfontconfig1 libfreetype6 libxcb1
\`\`\`

Fedora：

\`\`\`bash
sudo dnf install -y libxkbcommon-x11 wayland-libs-client vulkan-loader \\
  fontconfig freetype libxcb
\`\`\`

## 安装

\`\`\`bash
./install.sh                     # 安装到 ~/.local（推荐）
sudo PREFIX=/usr ./install.sh    # 系统级安装
./bin/biuviewer                  # 或直接运行
\`\`\`

## 卸载

\`\`\`bash
rm -f ~/.local/bin/biuviewer
rm -f ~/.local/share/applications/biuviewer.desktop
rm -f ~/.local/share/icons/hicolor/256x256/apps/biuviewer.png
# 系统级安装时把 ~/.local 换成 /usr
\`\`\`
EOF

mkdir -p "${BUNDLE_DIR}"
cp "${STAGE_DIR}/README.md" "${STAGE_DIR}/share/doc/${APP_NAME}/README.md"
(cd "${STAGE_ROOT}" && tar czf "${OUTPUT}" "${APP_NAME}")
rm -rf "${STAGE_ROOT}"

echo "==> 已生成 ${OUTPUT}"
