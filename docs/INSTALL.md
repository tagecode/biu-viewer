# 安装与首次运行

BiuViewer 当前未做代码签名与公证，三个平台的系统都会在首次运行时给出安全提示。这是预期行为，不是文件损坏。

## macOS

**要求**：macOS 10.15 Catalina 或更高。产物是通用二进制，Intel 与 Apple Silicon 均可原生运行。

1. 下载 `BiuViewer-vX.Y.Z-macOS-universal.zip` 并解压。
2. 把 `BiuViewer.app` 拖进「应用程序」文件夹。
3. 首次打开时若提示「无法打开，因为 Apple 无法检查其是否包含恶意软件」，用以下任一方式放行：
   - 在「访达」中**右键点击** App → **打开** → 在弹窗里再点一次**打开**；
   - 或打开「系统设置 → 隐私与安全性」，在页面底部点击**仍要打开**；
   - 或执行：
     ```bash
     xattr -cr /Applications/BiuViewer.app
     ```

已知提示文案包括「已损坏，无法打开」「无法验证开发者」——都是 Gatekeeper 拦截未签名应用的统一说法，按上面任一方式处理即可。

**设为默认打开方式**：右键任一图片 → 显示简介 → 打开方式选 BiuViewer → 点击「全部更改」。

## Windows

**要求**：Windows 10 1809 或更高，64 位。

1. 下载 `BiuViewer-vX.Y.Z-windows-x64.zip` 并解压到任意目录（便携版，无需安装）。
2. 双击 `biuviewer.exe`。
3. 若出现「Windows 已保护你的电脑」提示，点击**更多信息** → **仍要运行**。

**设为默认打开方式**：右键图片 → 打开方式 → 选择其他应用 → 找到 `biuviewer.exe` → 勾选「始终使用此应用」。

**拖入打开**：把图片或文件夹直接拖到窗口上即可。

## Linux

**要求**：x86_64 架构，glibc 2.31+。需要系统具备 Vulkan 或 OpenGL 驱动与中文字体。

```bash
tar -xzf BiuViewer-vX.Y.Z-linux-x86_64.tar.gz
cd biuviewer
./install.sh            # 默认安装到 ~/.local
# 或指定前缀
PREFIX=/usr/local sudo ./install.sh
```

安装脚本会：

- 把可执行文件放到 `$PREFIX/bin/biuviewer`
- 把图标与 `.desktop` 文件注册到应用菜单
- 刷新 desktop database 与 icon cache（工具存在时）

卸载：

```bash
rm -f ~/.local/bin/biuviewer
rm -f ~/.local/share/applications/biuviewer.desktop
rm -f ~/.local/share/icons/hicolor/256x256/apps/biuviewer.png
```

**缺少依赖时**（Debian/Ubuntu）：

```bash
sudo apt install libxkbcommon-x11-0 libwayland-client0 libvulkan1 \
    libx11-xcb1 libfontconfig1 libfreetype6
```

**设为默认打开方式**：安装后 `.desktop` 已声明 `image/*` 的 MimeType，在文件管理器里右键图片 →「打开方式」即可选择 BiuViewer。

## 从源码构建

```bash
git clone https://github.com/tagecode/biu-viewer.git
cd biu-viewer
cargo build --release
./target/release/biuviewer
```

Linux 构建前先安装开发依赖：

```bash
sudo apt install libxkbcommon-dev libxkbcommon-x11-dev libwayland-dev \
    libvulkan-dev libx11-xcb-dev libxcb1-dev libfontconfig-dev libfreetype-dev libssl-dev
```
