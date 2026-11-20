# Changelog

本项目遵循 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/) 与
[语义化版本](https://semver.org/lang/zh-CN/)。

## [0.2.0] - 2026-11-21（候选）

### 新增

- 支持 HEIC/HEIF 与 AVIF 静态图解码，使用纯 Rust 解码器，不引入平台原生依赖。
- HEIC/AVIF 缩略图优先读取文件内嵌的 `thmb` 条目，避免为缩略图完整解码大图；没有内嵌缩略图时回退完整解码。
- GIF 暂停/继续、上一帧/下一帧，对应 `Space`、`,`、`.`，工具栏与状态栏同步显示。
- 查看偏好持久化：窗口位置与大小、适应/1:1 缩放模式、缩略图开关，保存到平台配置目录的 `config.txt`。
- 相邻图片预加载（前后各一张），串行后台解码，不占用主图缓存 pin。
- 取消令牌：切换图片时丢弃过期解码结果，并在 GIF 逐帧解码与主图加载的关键阶段检查取消。
- JPEG 缩略图使用 `jpeg-decoder` 按 1/8、1/4、1/2 降采样，减少缩略图条的全尺寸解码。
- 缩略图并发按文件大小折算槽位，避免同时解码多张大图。

### 变更

- 主图像素上限从 2 亿收紧到 1 亿，超出时返回不支持提示。
- GIF 增加 1000 帧、256 MiB 解码总量上限，超限返回明确错误。
- 空文件夹改为居中空状态，并且只在用户主动选择空文件夹后显示。

### 修复

- 修复启动或自动打开空文件夹时顶部出现「该文件夹中没有可显示的图片」横条的问题。
- 修复点击「打开」时文件对话框阻塞 GPUI 事件循环导致的借用崩溃，对话框改为异步。
- 修复空状态仍渲染空白缩略图条的问题。

### 验证

- `cargo test --offline --workspace --jobs 1`：110 passed。
- `cargo clippy --offline --workspace --all-targets -- -D warnings` 通过。
- `cargo fmt --all -- --check` 通过。
- CI 三平台构建通过。

### 已知限制

- macOS 与 Linux 的 0.2.0 发布产物尚未完成实机验收；Linux 仍待真实桌面执行第 1–16 项。
- AVIF 没有内嵌 `thmb` 时仍需完整解码后缩小。
- HEIC/AVIF 的 EXIF 方向未单独处理，`irot`/`imir` 交由解码器处理。
- 当前未做代码签名；Windows SmartScreen、macOS Gatekeeper 首次运行提示仍存在。
