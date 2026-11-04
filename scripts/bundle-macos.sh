#!/usr/bin/env bash
# 生成 macOS .app 与 BiuViewer-v{version}-macOS-universal.zip。
# CI 会先把两个架构 lipo 成 target/release/biuviewer；本地构建则直接用当前架构。

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=scripts/utils.sh
source "${SCRIPT_DIR}/utils.sh"

APP_NAME="BiuViewer"
VERSION="$(get_version)"
BINARY="$(binary_path "" biuviewer)"
STAGE_DIR="${BUNDLE_DIR}/macos"
APP_DIR="${STAGE_DIR}/${APP_NAME}.app"
OUTPUT="${BUNDLE_DIR}/${APP_NAME}-v${VERSION}-macOS-universal.zip"

if [ ! -f "${BINARY}" ]; then
    echo "找不到可执行文件：${BINARY}，请先运行 cargo build --release" >&2
    exit 1
fi

echo "==> 打包 macOS ${VERSION}"
rm -rf "${STAGE_DIR}" "${OUTPUT}"
mkdir -p "${APP_DIR}/Contents/MacOS" "${APP_DIR}/Contents/Resources"

cp "${BINARY}" "${APP_DIR}/Contents/MacOS/${APP_NAME}"
chmod +x "${APP_DIR}/Contents/MacOS/${APP_NAME}"
cp "${ICON_DIR}/icon.icns" "${APP_DIR}/Contents/Resources/icon.icns"

cat > "${APP_DIR}/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>zh_CN</string>
    <key>CFBundleExecutable</key>
    <string>${APP_NAME}</string>
    <key>CFBundleIconFile</key>
    <string>icon.icns</string>
    <key>CFBundleIdentifier</key>
    <string>com.biuviewer.app</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>${APP_NAME}</string>
    <key>CFBundleDisplayName</key>
    <string>博览 BiuViewer</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>${VERSION}</string>
    <key>CFBundleVersion</key>
    <string>${VERSION}</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.15</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>CFBundleDocumentTypes</key>
    <array>
        <dict>
            <key>CFBundleTypeName</key>
            <string>Image</string>
            <key>CFBundleTypeRole</key>
            <string>Viewer</string>
            <key>LSHandlerRank</key>
            <string>Alternate</string>
            <key>LSItemContentTypes</key>
            <array>
                <string>public.image</string>
                <string>public.png</string>
                <string>public.jpeg</string>
                <string>com.compuserve.gif</string>
                <string>public.tiff</string>
                <string>com.microsoft.bmp</string>
                <string>org.webmproject.webp</string>
                <string>com.microsoft.ico</string>
            </array>
        </dict>
    </array>
</dict>
</plist>
EOF

printf 'APPL????' > "${APP_DIR}/Contents/PkgInfo"

mkdir -p "${BUNDLE_DIR}"
make_zip "${STAGE_DIR}" "${OUTPUT}"
rm -rf "${STAGE_DIR}"

echo "==> 已生成 ${OUTPUT}"
