# 常见问题

## 打开与安装

### macOS 提示「已损坏，无法打开」/「无法验证开发者」

未签名的应用都会这样提示，不代表文件真的损坏。右键点击 App → **打开**，或执行：

```bash
xattr -cr /Applications/BiuViewer.app
```

详见 [INSTALL.md](INSTALL.md)。

### Windows 提示「Windows 已保护你的电脑」

点击**更多信息** → **仍要运行**。若使用公司电脑且被策略拦截，需要 IT 管理员放行。

### Linux 启动报缺少 `.so` 文件

安装运行库：

```bash
sudo apt install libxkbcommon-x11-0 libwayland-client0 libvulkan1 \
    libx11-xcb1 libfontconfig1 libfreetype6
```

如果系统没有 Vulkan，请确认已安装 Mesa 驱动（`mesa-vulkan-drivers`）。

### 双击图片没有用 BiuViewer 打开

BiuViewer 不会自动抢占文件关联。macOS 用「显示简介 → 打开方式 → 全部更改」，Windows 用「打开方式 → 始终使用」，Linux 安装后在文件管理器里选择。

## 使用

### 拖拽文件到窗口没反应

Wayland 下部分合成器对跨应用拖拽支持不完整，可改用「打开图片 / 打开文件夹」对话框或命令行传入路径。

### 方向键不能翻页

窗口需要先获得焦点。点击一下图片区域即可；如果正在使用输入法，先按下 `Esc` 退出候选状态。

### 图片方向不对（躺倒 / 镜像）

BiuViewer 会读取 EXIF Orientation 并在解码时自动校正。若某张图方向仍然错误，多半是文件本身没有写入 EXIF 信息，可用 `R`（顺时针）、`⇧R`（逆时针）、`H`/`V` 临时旋转查看。

### 手动旋转后点「重置」为什么没恢复原样

「重置」只清除你在本次查看中做的旋转与翻转，不会撤销 EXIF 校正——那才是照片的真实方向。

### GIF 能暂停吗

可以。按 `Space` 暂停 / 继续，暂停后用 `,` / `.` 逐帧前后查看；工具栏在打开动画时也会出现对应按钮。暂停只是停止计时，不会重置到第一帧。

### 支持 HEIC / AVIF / RAW 吗

支持 HEIC/HEIF 与 AVIF，使用纯 Rust 解码器。RAW 暂不支持。当前完整格式列表：PNG、JPEG、GIF、BMP、WebP、TIFF、ICO、HEIC/HEIF、AVIF。

缩略图条会优先复用 HEIC/AVIF 文件内嵌的 `thmb` 缩略图，只解码这一小张；文件没有内嵌缩略图时（常见于部分 AVIF）才完整解码后再缩小，因此那个缩略图会慢一些。

## 性能

### 打开大文件夹很慢

文件夹扫描是同步的，1000 张图片通常 1 秒内完成。如果所在目录在机械硬盘或网络盘上，首个缩略图出现会慢一些；主图仍是优先加载的，不依赖缩略图。

### 内存占用看起来很高

缓存预算上限是 512MB（主图）+ 64MB（缩略图），浏览大图或长动画时会接近这个上限，属于正常范围。单张 GIF 解码后的总大小上限为 256MB，超过会提示错误而不是继续占用内存。停止翻页后不会继续增长；如需立即释放，关闭窗口重新打开即可。

### 超大图片打不开

单边超过 32768 像素或总像素超过 1 亿的图片会被拒绝解码，以避免内存耗尽。这类图片一般需要专门的工具处理。

## 其他

### 有没有设置项 / 配置文件

没有可视化设置页，但会保存少量查看偏好：窗口位置与大小、缩放模式（适应 / 1:1）、缩略图开关。配置文件位置：

- Windows：`%APPDATA%\BiuViewer\config.txt`
- macOS：`~/Library/Application Support/BiuViewer/config.txt`
- Linux：`$XDG_CONFIG_HOME/biuviewer/config.txt`（默认 `~/.config/biuviewer/config.txt`）

删除该文件即可恢复默认；不记录浏览历史。

### 如何反馈问题

在 [Issues](https://github.com/tagecode/biu-viewer/issues) 提交，并附上：操作系统与版本、BiuViewer 版本（`biuviewer --version`）、出问题的图片格式、复现步骤。能提供样本文件更好。
