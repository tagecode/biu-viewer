# 博览(BiuViewer)技术架构文档

## 一、架构概述

### 1.1 架构目标

博览(BiuViewer)采用现代化的 Rust + GPUI-Kit 技术栈,旨在构建一款高性能、跨平台的轻量级图片查看器。架构设计遵循以下核心原则:

- **高性能**: GPU 加速渲染,异步 I/O,智能缓存策略
- **可扩展**: 模块化设计,可插拔的解码后端
- **跨平台**: 统一代码库,平台特定适配最小化
- **轻量级**: 最小化依赖,快速启动,低内存占用
- **可维护**: 清晰的分层架构,职责明确的模块划分

### 1.2 技术栈选型

| 层级 | 技术选型 | 理由 |
|:---|:---|:---|
| 编程语言 | Rust 1.70+ | 内存安全、零成本抽象、优秀的并发支持 |
| UI 框架 | GPUI-Kit | GPU 加速、声明式 UI、跨平台支持 |
| 图像解码 | image crate | 纯 Rust 实现、格式支持全面、性能优秀 |
| 异步运行时 | tokio | 成熟稳定、生态丰富、性能优异 |
| 序列化 | serde + serde_json | Rust 标准序列化方案 |
| 日志 | tracing + tracing-subscriber | 结构化日志、异步友好 |
| 错误处理 | thiserror + anyhow | 类型安全的错误定义和传播 |

### 1.3 架构风格

采用**分层架构 + 事件驱动**的混合模式:

```
┌─────────────────────────────────────────────────────────┐
│                     Presentation Layer                   │  UI 展示层
│              (GPUI Components & Views)                   │
├─────────────────────────────────────────────────────────┤
│                     Application Layer                    │  应用逻辑层
│         (State Management & Business Logic)              │
├─────────────────────────────────────────────────────────┤
│                      Domain Layer                        │  领域层
│     (Image Model, Viewer Logic, Cache Strategy)          │
├─────────────────────────────────────────────────────────┤
│                   Infrastructure Layer                   │  基础设施层
│  (Decoder, File I/O, Texture Management, Platform API)   │
└─────────────────────────────────────────────────────────┘
```

---

## 二、系统架构

### 2.1 整体架构图

```
┌──────────────────────────────────────────────────────────────────┐
│                          Application Entry                        │
│                        (main.rs + app.rs)                         │
└────────────────────────────┬─────────────────────────────────────┘
                             │
                ┌────────────┴────────────┐
                │    GPUI Application     │
                │   (Event Loop & Window) │
                └────────────┬────────────┘
                             │
        ┌────────────────────┼────────────────────┐
        │                    │                    │
   ┌────▼─────┐       ┌─────▼──────┐      ┌─────▼──────┐
   │   View   │       │  Toolbar   │      │  Thumbnail │
   │  Manager │       │  Component │      │    Panel   │
   └────┬─────┘       └─────┬──────┘      └─────┬──────┘
        │                   │                    │
        └───────────────────┼────────────────────┘
                            │
                  ┌─────────▼─────────┐
                  │   State Manager   │  (Application State)
                  │  - Current Image  │
                  │  - Image List     │
                  │  - View State     │
                  └─────────┬─────────┘
                            │
        ┌───────────────────┼───────────────────┐
        │                   │                   │
   ┌────▼──────┐     ┌─────▼──────┐     ┌─────▼──────┐
   │  Decoder  │     │   Cache    │     │ File System│
   │  Manager  │     │  Manager   │     │   Watcher  │
   └────┬──────┘     └─────┬──────┘     └─────┬──────┘
        │                  │                   │
   ┌────▼──────┐     ┌─────▼──────┐     ┌─────▼──────┐
   │  Image    │     │   Memory   │     │  Platform  │
   │  Decoder  │     │   Cache    │     │    API     │
   │  Backend  │     │ + Disk Cache│     │  (Native)  │
   └───────────┘     └────────────┘     └────────────┘
```

### 2.2 模块划分

#### 2.2.1 核心模块

| 模块 | 职责 | 主要组件 |
|:---|:---|:---|
| **app** | 应用入口和生命周期管理 | `App`, `AppState`, `Config` |
| **ui** | UI 组件和视图管理 | `ViewerWindow`, `Toolbar`, `ThumbnailPanel` |
| **viewer** | 图片查看器核心逻辑 | `ImageViewer`, `ViewportManager`, `TransformState` |
| **image** | 图片数据模型和操作 | `Image`, `ImageMetadata`, `ImageList` |
| **decoder** | 图像解码引擎 | `DecoderManager`, `FormatDetector`, `DecoderBackend` |
| **cache** | 缓存管理 | `MemoryCache`, `DiskCache`, `CachePolicy` |
| **io** | 文件系统操作 | `FileLoader`, `FolderScanner`, `FileWatcher` |
| **platform** | 平台适配层 | `Clipboard`, `FileDialog`, `SystemTheme` |
| **utils** | 工具函数 | `Logger`, `Config`, `Error` |

#### 2.2.2 模块依赖关系

```
app
 ├── ui (视图层)
 │   └── viewer (查看器逻辑)
 │       ├── image (图片模型)
 │       ├── decoder (解码)
 │       ├── cache (缓存)
 │       └── io (文件 I/O)
 └── platform (平台适配)
     └── utils (工具库)
```

---

## 三、核心子系统设计

### 3.1 图像解码子系统

#### 3.1.1 架构设计

```rust
// 解码器接口
trait ImageDecoder: Send + Sync {
    fn supports(&self, format: ImageFormat) -> bool;
    fn decode(&self, data: &[u8]) -> Result<DecodedImage>;
    fn decode_metadata(&self, data: &[u8]) -> Result<ImageMetadata>;
}

// 解码管理器
struct DecoderManager {
    decoders: Vec<Box<dyn ImageDecoder>>,
    format_detector: FormatDetector,
}
```

#### 3.1.2 支持的格式与解码器

| 格式 | 解码器实现 | 优先级 | 说明 |
|:---|:---|:---|:---|
| PNG/JPEG/BMP/GIF | `image::ImageDecoder` | P0 | 默认通过 image crate |
| WebP | `image::webp::Decoder` | P0 | image crate 内置 |
| TIFF | `image::tiff::Decoder` | P0 | image crate 内置 |
| ICO | `image::ico::Decoder` | P0 | image crate 内置 |
| GIF 动画 | `AnimatedGifDecoder` | P1 | 自定义实现,支持帧提取 |
| APNG | `ApngDecoder` (apng crate) | P1 | 可选依赖 |
| AVIF | `libavif-rs` | P1 | 动态加载 |
| HEIC/HEIF | `heif-oxide` | P1/P2 | 动态加载 |
| SVG | `resvg` | P2 | 可选,转为位图 |

#### 3.1.3 解码流程

```
┌─────────────┐
│ Load File   │
└──────┬──────┘
       │
┌──────▼──────────┐
│ Detect Format   │  (通过魔数/扩展名)
└──────┬──────────┘
       │
┌──────▼──────────┐
│ Select Decoder  │  (选择对应解码器)
└──────┬──────────┘
       │
┌──────▼──────────┐
│ Async Decode    │  (后台线程解码)
│  - Read EXIF    │
│  - Decode Pixels│
│  - Apply Orient │
└──────┬──────────┘
       │
┌──────▼──────────┐
│ Create Texture  │  (上传到 GPU)
└──────┬──────────┘
       │
┌──────▼──────────┐
│ Update View     │  (触发 UI 刷新)
└─────────────────┘
```

#### 3.1.4 动画处理

```rust
struct AnimatedImage {
    frames: Vec<Frame>,
    current_frame: usize,
    frame_duration: Duration,
    loop_count: Option<u32>,
}

struct Frame {
    texture: GpuTexture,
    duration: Duration,
    disposal: DisposalMethod,
}
```

动画播放通过 GPUI 的定时器机制:
- 根据帧间隔调度下一帧
- 支持暂停/继续/重置
- 循环播放或指定次数

### 3.2 缓存子系统

#### 3.2.1 两级缓存架构

```
┌────────────────────────────────────────────┐
│          Memory Cache (L1)                 │
│  - 当前图片纹理 (必须)                      │
│  - 相邻图片预加载 (±1)                      │
│  - LRU 淘汰策略                            │
│  - 预算: 512MB - 1GB                       │
└────────────────┬───────────────────────────┘
                 │ Miss
┌────────────────▼───────────────────────────┐
│          Disk Cache (L2)                   │
│  - 缩略图持久化缓存                         │
│  - 降采样图片缓存                           │
│  - 位置: ~/.cache/biuviewer/               │
│  - 清理策略: LRU + 时间过期                 │
└────────────────┬───────────────────────────┘
                 │ Miss
┌────────────────▼───────────────────────────┐
│          File System (Source)              │
│  - 原始图片文件                             │
│  - 解码后上传到 Memory Cache               │
└────────────────────────────────────────────┘
```

#### 3.2.2 内存缓存策略

```rust
struct MemoryCache {
    entries: HashMap<PathBuf, CacheEntry>,
    lru: LruList,
    budget: usize,          // 字节预算
    current_usage: usize,   // 当前使用
}

struct CacheEntry {
    texture: Arc<GpuTexture>,
    size: usize,
    last_access: Instant,
    ref_count: usize,
}
```

**缓存策略**:
- 当前图片始终保持在缓存中 (pin)
- 预加载相邻 ±1 张图片
- 超出预算时按 LRU 淘汰
- 视口外纹理释放
- 后台线程定期清理

#### 3.2.3 磁盘缓存

```rust
struct DiskCache {
    cache_dir: PathBuf,  // ~/.cache/biuviewer/
    index: CacheIndex,    // SQLite or JSON
}

// 缓存 key 生成
fn cache_key(path: &Path, size: Option<(u32, u32)>) -> String {
    let hash = hash_file_path_and_mtime(path);
    match size {
        Some((w, h)) => format!("{hash}_{w}x{h}.webp"),
        None => format!("{hash}.webp"),
    }
}
```

**缓存内容**:
- 缩略图 (128x128, 256x256)
- 降采样预览图 (1920x1080)
- 使用 WebP 压缩存储
- 带 mtime 校验,文件变更自动失效

### 3.3 渲染子系统

#### 3.3.1 GPU 纹理管理

```rust
struct TextureManager {
    context: GpuContext,
    textures: HashMap<ImageId, GpuTexture>,
    max_texture_size: (u32, u32),
}

struct GpuTexture {
    handle: TextureHandle,    // GPUI 纹理句柄
    size: (u32, u32),
    format: TextureFormat,
    mipmap: bool,
}
```

**纹理创建流程**:
```
Decoded RGBA Data
       │
       ├─> Check Size
       │   (超大图降采样)
       │
       ├─> Upload to GPU
       │   (GPUI::Texture::new)
       │
       └─> Generate Mipmap (可选)
           (用于缩放时的高质量采样)
```

#### 3.3.2 渲染管线

```rust
impl ViewerWindow {
    fn render(&mut self, cx: &mut ViewContext) {
        let transform = self.calculate_transform();
        
        // 计算视口裁剪
        let visible_rect = self.calculate_visible_rect();
        
        // 绘制背景
        self.draw_background(cx);
        
        // 绘制图片
        if let Some(texture) = self.current_texture() {
            cx.draw_texture(
                texture,
                transform,
                visible_rect,
                self.interpolation_mode(),
            );
        }
        
        // 绘制 UI 覆盖层
        self.draw_overlays(cx);
    }
}
```

#### 3.3.3 变换矩阵

```rust
struct TransformState {
    scale: f32,           // 缩放比例
    offset: Point2D,      // 平移偏移
    rotation: Rotation,   // 旋转角度 (0°/90°/180°/270°)
    flip_h: bool,         // 水平翻转
    flip_v: bool,         // 垂直翻转
}

impl TransformState {
    fn to_matrix(&self) -> Matrix3x3 {
        Matrix3x3::identity()
            .translate(self.offset)
            .scale(self.scale, self.scale)
            .rotate(self.rotation.to_radians())
            .scale(
                if self.flip_h { -1.0 } else { 1.0 },
                if self.flip_v { -1.0 } else { 1.0 },
            )
    }
}
```

#### 3.3.4 缩放与平移

**缩放模式**:
- **Fit to Window**: 自动计算最佳缩放比例
  ```rust
  fn fit_to_window(&self, image_size: Size, window_size: Size) -> f32 {
      let scale_x = window_size.width / image_size.width;
      let scale_y = window_size.height / image_size.height;
      scale_x.min(scale_y)
  }
  ```
- **Actual Size**: 1:1 显示,`scale = 1.0`
- **Custom Scale**: 用户自定义缩放比例

**平移锚点**:
```rust
fn zoom_at_point(&mut self, delta: f32, cursor_pos: Point2D) {
    // 缩放前的图像坐标
    let image_pos_before = self.screen_to_image(cursor_pos);
    
    // 应用缩放
    self.scale *= delta;
    self.scale = self.scale.clamp(0.1, 32.0);
    
    // 缩放后的图像坐标
    let image_pos_after = self.screen_to_image(cursor_pos);
    
    // 调整偏移,保持光标下的图像点不变
    self.offset += image_pos_before - image_pos_after;
}
```

### 3.4 文件系统子系统

#### 3.4.1 文件加载

```rust
struct FileLoader {
    executor: TaskExecutor,
}

impl FileLoader {
    async fn load_file(&self, path: PathBuf) -> Result<Vec<u8>> {
        tokio::fs::read(&path).await
            .map_err(|e| Error::FileRead(path, e))
    }
    
    async fn load_folder(&self, path: PathBuf) -> Result<Vec<PathBuf>> {
        let mut entries = tokio::fs::read_dir(&path).await?;
        let mut images = Vec::new();
        
        while let Some(entry) = entries.next_entry().await? {
            if self.is_supported_image(&entry.path()) {
                images.push(entry.path());
            }
        }
        
        // 自然排序
        images.sort_by(|a, b| natord::compare(
            a.to_string_lossy().as_ref(),
            b.to_string_lossy().as_ref(),
        ));
        
        Ok(images)
    }
}
```

#### 3.4.2 文件监视

```rust
use notify::{Watcher, RecursiveMode, Event};

struct FileWatcher {
    watcher: RecommendedWatcher,
    tx: mpsc::Sender<WatchEvent>,
}

enum WatchEvent {
    Modified(PathBuf),
    Deleted(PathBuf),
    Created(PathBuf),
}

impl FileWatcher {
    fn watch_file(&mut self, path: &Path) -> Result<()> {
        self.watcher.watch(path, RecursiveMode::NonRecursive)?;
        Ok(())
    }
    
    fn handle_event(&self, event: Event) {
        match event.kind {
            EventKind::Modify(_) => {
                // 文件修改 -> 刷新图片
            }
            EventKind::Remove(_) => {
                // 文件删除 -> 跳转到下一张
            }
            _ => {}
        }
    }
}
```

### 3.5 状态管理子系统

#### 3.5.1 应用状态

```rust
struct AppState {
    // 图片列表
    image_list: ImageList,
    current_index: usize,
    
    // 视图状态
    transform: TransformState,
    viewport: ViewportState,
    fullscreen: bool,
    
    // UI 状态
    toolbar_visible: bool,
    thumbnail_panel_visible: bool,
    
    // 播放状态 (动画)
    playing: bool,
    
    // 用户配置
    config: Config,
}

impl AppState {
    fn next_image(&mut self) {
        if self.current_index < self.image_list.len() - 1 {
            self.current_index += 1;
            self.reset_transform();
        }
    }
    
    fn prev_image(&mut self) {
        if self.current_index > 0 {
            self.current_index -= 1;
            self.reset_transform();
        }
    }
}
```

#### 3.5.2 事件处理

```rust
enum ViewerEvent {
    // 文件操作
    OpenFile(PathBuf),
    OpenFolder(PathBuf),
    
    // 导航
    NextImage,
    PrevImage,
    JumpToIndex(usize),
    
    // 视图操作
    ZoomIn,
    ZoomOut,
    ZoomTo(f32),
    ZoomFit,
    ZoomActual,
    Rotate(Rotation),
    Flip(FlipAxis),
    
    // 系统操作
    ToggleFullscreen,
    CopyToClipboard,
    DeleteFile,
    
    // 内部事件
    ImageLoaded(ImageId, Arc<GpuTexture>),
    LoadError(ImageId, Error),
}

impl App {
    fn handle_event(&mut self, event: ViewerEvent, cx: &mut AppContext) {
        match event {
            ViewerEvent::NextImage => {
                self.state.next_image();
                self.load_current_image(cx);
                self.preload_adjacent_images(cx);
            }
            ViewerEvent::ZoomIn => {
                self.state.transform.scale *= 1.2;
                cx.notify();
            }
            // ... 其他事件处理
        }
    }
}
```

---

## 四、关键技术实现

### 4.1 异步加载与并发

#### 4.1.1 任务调度

```rust
struct TaskScheduler {
    decoder_pool: ThreadPool,    // 解码线程池
    io_runtime: Runtime,          // Tokio 运行时
    pending_tasks: TaskQueue,
}

impl TaskScheduler {
    fn schedule_decode(&self, path: PathBuf, priority: Priority) -> JoinHandle<DecodedImage> {
        let (tx, rx) = oneshot::channel();
        
        self.decoder_pool.spawn(move || {
            let result = decode_image(&path);
            tx.send(result);
        });
        
        rx
    }
    
    fn cancel_task(&self, handle: TaskHandle) {
        // 取消过期任务
        handle.abort();
    }
}
```

#### 4.1.2 预加载策略

```rust
impl PreloadManager {
    fn preload_adjacent(&self, current: usize, list: &ImageList) {
        let to_load = vec![
            current.checked_sub(1),  // 前一张
            Some(current + 1),        // 后一张
        ];
        
        for index in to_load.into_iter().flatten() {
            if let Some(path) = list.get(index) {
                if !self.cache.contains(path) {
                    self.scheduler.schedule_decode(
                        path.clone(),
                        Priority::Low,
                    );
                }
            }
        }
    }
}
```

### 4.2 EXIF 处理

```rust
use exif::{Reader, Tag};

fn read_exif_orientation(data: &[u8]) -> Result<Orientation> {
    let reader = Reader::new();
    let exif = reader.read_from_container(&mut Cursor::new(data))?;
    
    if let Some(field) = exif.get_field(Tag::Orientation, In::PRIMARY) {
        match field.value.get_uint(0) {
            Some(1) => Ok(Orientation::Normal),
            Some(3) => Ok(Orientation::Rotate180),
            Some(6) => Ok(Orientation::Rotate90CW),
            Some(8) => Ok(Orientation::Rotate270CW),
            _ => Ok(Orientation::Normal),
        }
    } else {
        Ok(Orientation::Normal)
    }
}

fn apply_orientation(image: &mut RgbaImage, orientation: Orientation) {
    match orientation {
        Orientation::Normal => {},
        Orientation::Rotate90CW => image.rotate90(),
        Orientation::Rotate180 => image.rotate180(),
        Orientation::Rotate270CW => image.rotate270(),
    }
}
```

### 4.3 大图优化

#### 4.3.1 降采样加载

```rust
fn load_large_image(path: &Path, max_size: u32) -> Result<RgbaImage> {
    let reader = image::io::Reader::open(path)?;
    let dimensions = reader.into_dimensions()?;
    
    if dimensions.0.max(dimensions.1) > max_size {
        // 需要降采样
        let scale = max_size as f32 / dimensions.0.max(dimensions.1) as f32;
        let target_size = (
            (dimensions.0 as f32 * scale) as u32,
            (dimensions.1 as f32 * scale) as u32,
        );
        
        // 快速降采样解码
        image::open(path)?
            .resize(target_size.0, target_size.1, FilterType::Triangle)
            .to_rgba8()
    } else {
        image::open(path)?.to_rgba8()
    }
}
```

#### 4.3.2 渐进式加载

```rust
struct ProgressiveLoader {
    path: PathBuf,
    preview: Option<Arc<GpuTexture>>,
    full: Option<Arc<GpuTexture>>,
}

impl ProgressiveLoader {
    async fn load(&mut self) {
        // 第一阶段: 快速加载缩略图
        if let Some(thumb) = self.load_from_cache(Size::Small).await {
            self.preview = Some(thumb);
            self.notify_preview_ready();
        }
        
        // 第二阶段: 加载降采样图
        let preview = self.load_downsampled(1920).await;
        self.preview = Some(preview);
        self.notify_preview_ready();
        
        // 第三阶段: 加载完整图片
        let full = self.load_full().await;
        self.full = Some(full);
        self.notify_full_ready();
    }
}
```

### 4.4 缩略图生成

```rust
struct ThumbnailGenerator {
    size: u32,
    quality: u8,
    cache: DiskCache,
}

impl ThumbnailGenerator {
    fn generate(&self, path: &Path) -> Result<Vec<u8>> {
        // 检查缓存
        if let Some(cached) = self.cache.get_thumbnail(path, self.size) {
            return Ok(cached);
        }
        
        // 生成缩略图
        let img = image::open(path)?;
        let thumb = img.thumbnail(self.size, self.size);
        
        // 编码为 WebP
        let mut buf = Vec::new();
        let encoder = webp::Encoder::from_rgba(
            thumb.as_bytes(),
            thumb.width(),
            thumb.height(),
        );
        buf = encoder.encode(self.quality as f32);
        
        // 写入缓存
        self.cache.put_thumbnail(path, self.size, &buf)?;
        
        Ok(buf)
    }
}
```

### 4.5 性能优化技巧

#### 4.5.1 零拷贝传输

```rust
// 使用 Arc 共享纹理,避免拷贝
type SharedTexture = Arc<GpuTexture>;

struct ImageData {
    pixels: Arc<[u8]>,  // 共享像素数据
    width: u32,
    height: u32,
}
```

#### 4.5.2 懒加载

```rust
struct LazyImage {
    path: PathBuf,
    state: ImageState,
}

enum ImageState {
    Unloaded,
    Loading(JoinHandle<DecodedImage>),
    Loaded(Arc<GpuTexture>),
    Error(Error),
}

impl LazyImage {
    fn ensure_loaded(&mut self) -> Result<()> {
        match &self.state {
            ImageState::Unloaded => {
                let handle = spawn_decode(&self.path);
                self.state = ImageState::Loading(handle);
            }
            _ => {}
        }
        Ok(())
    }
}
```

#### 4.5.3 内存池

```rust
struct TexturePool {
    free_textures: Vec<GpuTexture>,
    size: (u32, u32),
}

impl TexturePool {
    fn acquire(&mut self) -> GpuTexture {
        self.free_textures.pop()
            .unwrap_or_else(|| self.allocate_new())
    }
    
    fn release(&mut self, texture: GpuTexture) {
        if self.free_textures.len() < MAX_POOL_SIZE {
            self.free_textures.push(texture);
        } else {
            drop(texture);  // 销毁多余纹理
        }
    }
}
```

---

## 五、平台适配层

### 5.1 平台抽象接口

```rust
trait PlatformApi {
    // 剪贴板
    fn clipboard_set_image(&self, data: &[u8], format: ImageFormat);
    fn clipboard_set_text(&self, text: &str);
    
    // 文件对话框
    fn open_file_dialog(&self, filters: &[FileFilter]) -> Option<PathBuf>;
    fn open_folder_dialog(&self) -> Option<PathBuf>;
    
    // 系统集成
    fn reveal_in_file_manager(&self, path: &Path);
    fn get_system_theme(&self) -> Theme;
    fn watch_theme_changes(&self) -> Receiver<Theme>;
    
    // 窗口
    fn set_fullscreen(&self, window: WindowHandle, fullscreen: bool);
}
```

### 5.2 平台特定实现

#### 5.2.1 macOS

```rust
#[cfg(target_os = "macos")]
mod macos {
    use cocoa::{base::id, foundation::NSString};
    use objc::{msg_send, sel, sel_impl};
    
    pub struct MacOSPlatform;
    
    impl PlatformApi for MacOSPlatform {
        fn reveal_in_file_manager(&self, path: &Path) {
            unsafe {
                let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];
                let path_str = NSString::alloc(nil)
                    .init_str(&path.display().to_string());
                let _: () = msg_send![workspace, selectFile:path_str
                    inFileViewerRootedAtPath:nil];
            }
        }
    }
}
```

#### 5.2.2 Windows

```rust
#[cfg(target_os = "windows")]
mod windows {
    use winapi::um::shellapi::ShellExecuteW;
    
    pub struct WindowsPlatform;
    
    impl PlatformApi for WindowsPlatform {
        fn reveal_in_file_manager(&self, path: &Path) {
            let args = format!("/select,{}", path.display());
            Command::new("explorer")
                .args(&[&args])
                .spawn()
                .ok();
        }
    }
}
```

#### 5.2.3 Linux

```rust
#[cfg(target_os = "linux")]
mod linux {
    pub struct LinuxPlatform;
    
    impl PlatformApi for LinuxPlatform {
        fn reveal_in_file_manager(&self, path: &Path) {
            // 尝试多个文件管理器
            let managers = ["nautilus", "dolphin", "thunar", "nemo"];
            
            for manager in &managers {
                if Command::new(manager)
                    .arg("--select")
                    .arg(path)
                    .spawn()
                    .is_ok()
                {
                    return;
                }
            }
            
            // 回退: 打开父目录
            if let Some(parent) = path.parent() {
                Command::new("xdg-open")
                    .arg(parent)
                    .spawn()
                    .ok();
            }
        }
    }
}
```

---

## 六、配置与持久化

### 6.1 配置文件结构

```rust
#[derive(Serialize, Deserialize)]
struct Config {
    // 窗口配置
    window: WindowConfig,
    
    // 查看器配置
    viewer: ViewerConfig,
    
    // 性能配置
    performance: PerformanceConfig,
    
    // UI 配置
    ui: UiConfig,
    
    // 最近文件
    recent: RecentFiles,
}

#[derive(Serialize, Deserialize)]
struct WindowConfig {
    width: u32,
    height: u32,
    x: i32,
    y: i32,
    maximized: bool,
}

#[derive(Serialize, Deserialize)]
struct ViewerConfig {
    default_zoom_mode: ZoomMode,
    background_color: Color,
    interpolation: InterpolationMode,
    loop_navigation: bool,  // 到末尾后循环到开头
}

#[derive(Serialize, Deserialize)]
struct PerformanceConfig {
    memory_cache_mb: usize,
    disk_cache_mb: usize,
    preload_adjacent: bool,
    max_texture_size: u32,
}

#[derive(Serialize, Deserialize)]
struct UiConfig {
    theme: Theme,
    language: String,
    toolbar_position: Position,
    thumbnail_size: u32,
}
```

### 6.2 配置文件位置

```rust
fn config_dir() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        dirs::home_dir().unwrap()
            .join("Library/Application Support/BiuViewer")
    }
    
    #[cfg(target_os = "windows")]
    {
        dirs::config_dir().unwrap()
            .join("BiuViewer")
    }
    
    #[cfg(target_os = "linux")]
    {
        dirs::config_dir().unwrap()
            .join("biuviewer")
    }
}

fn cache_dir() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        dirs::home_dir().unwrap()
            .join("Library/Caches/BiuViewer")
    }
    
    #[cfg(target_os = "windows")]
    {
        dirs::cache_dir().unwrap()
            .join("BiuViewer")
    }
    
    #[cfg(target_os = "linux")]
    {
        dirs::cache_dir().unwrap()
            .join("biuviewer")
    }
}
```

### 6.3 日志系统

```rust
use tracing::{info, warn, error, debug};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn init_logging() {
    let log_file = config_dir().join("biuviewer.log");
    
    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::fs::File::create(log_file).unwrap())
        .with_ansi(false);
    
    let stdout_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stdout);
    
    tracing_subscriber::registry()
        .with(file_layer)
        .with(stdout_layer)
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    
    info!("BiuViewer started");
}
```

---

## 七、构建与打包

### 7.0 开源项目打包策略

作为开源项目,BiuViewer 的打包和分发遵循以下原则:

- **无需签名/公证**: MVP 阶段不进行代码签名和公证,通过文档引导用户处理安全警告
- **GitHub CI/CD**: 使用 GitHub Actions 进行跨平台自动化构建和发布
- **校验和**: 所有发布产物提供 SHA256 校验和,确保完整性
- **简化优先**: 优先提供最简单的可执行包,避免复杂的安装程序
- **社区友好**: 提供清晰的安装说明和常见问题解答

### 7.1 项目结构

```
biu-viewer/
├── Cargo.toml               # 工作区根清单
├── Cargo.lock
├── .github/
│   └── workflows/
│       ├── release.yml      # 发布构建 CI/CD
│       └── ci.yml           # 日常 CI 测试
├── crates/
│   ├── biuviewer/           # 主应用
│   ├── biuviewer-core/      # 核心逻辑
│   ├── biuviewer-ui/        # UI 组件
│   ├── biuviewer-decoder/   # 解码器
│   └── biuviewer-platform/  # 平台适配
├── resources/
│   ├── icons/               # 应用图标
│   │   ├── icon.icns        # macOS 图标
│   │   ├── icon.ico         # Windows 图标
│   │   └── icon.png         # Linux 图标
│   └── i18n/                # 国际化资源
├── scripts/
│   ├── bundle-macos.sh      # macOS 打包脚本
│   ├── bundle-windows.sh    # Windows 打包脚本
│   ├── bundle-linux.sh      # Linux 打包脚本
│   └── install.sh           # Linux 安装脚本
├── docs/
│   ├── INSTALL.md           # 安装说明
│   └── FAQ.md               # 常见问题
└── README.md
```

### 7.2 Cargo.toml 配置

```toml
[workspace]
members = [
    "crates/biuviewer",
    "crates/biuviewer-core",
    "crates/biuviewer-ui",
    "crates/biuviewer-decoder",
    "crates/biuviewer-platform",
]

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true

[profile.release-with-debug]
inherits = "release"
strip = false
debug = true
```

### 7.3 GitHub Actions CI/CD

#### 7.3.1 发布构建配置

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

env:
  CARGO_TERM_COLOR: always

jobs:
  build-macos:
    runs-on: macos-latest
    steps:
      - name: Checkout code
        uses: actions/checkout@v4
      
      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: aarch64-apple-darwin,x86_64-apple-darwin
      
      - name: Setup Rust cache
        uses: Swatinem/rust-cache@v2
      
      - name: Build for macOS (Universal Binary)
        run: |
          cargo build --release --target x86_64-apple-darwin
          cargo build --release --target aarch64-apple-darwin
          # 创建通用二进制
          mkdir -p target/release
          lipo -create \
            target/x86_64-apple-darwin/release/biuviewer \
            target/aarch64-apple-darwin/release/biuviewer \
            -output target/release/biuviewer
      
      - name: Bundle macOS app
        run: ./scripts/bundle-macos.sh
      
      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: macos-bundle
          path: target/bundle/BiuViewer-v*-macOS-universal.zip

  build-windows:
    runs-on: windows-latest
    steps:
      - name: Checkout code
        uses: actions/checkout@v4
      
      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: x86_64-pc-windows-msvc
      
      - name: Setup Rust cache
        uses: Swatinem/rust-cache@v2
      
      - name: Build for Windows
        run: cargo build --release --target x86_64-pc-windows-msvc
      
      - name: Bundle Windows app
        run: ./scripts/bundle-windows.sh
        shell: bash
      
      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: windows-bundle
          path: target/bundle/BiuViewer-v*-windows-x64.zip

  build-linux:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout code
        uses: actions/checkout@v4
      
      - name: Install dependencies
        # GPUI 在 Linux 上依赖 X11/Wayland、Vulkan 与字体相关开发库
        run: |
          sudo apt-get update
          sudo apt-get install -y libxkbcommon-dev libxkbcommon-x11-dev \
            libwayland-dev libvulkan-dev libx11-xcb-dev libxcb1-dev \
            libfontconfig-dev libfreetype-dev libssl-dev
      
      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: x86_64-unknown-linux-gnu
      
      - name: Setup Rust cache
        uses: Swatinem/rust-cache@v2
      
      - name: Build for Linux
        run: cargo build --release --target x86_64-unknown-linux-gnu
      
      - name: Bundle Linux app
        run: ./scripts/bundle-linux.sh
      
      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: linux-bundle
          path: target/bundle/BiuViewer-v*-linux-x86_64.tar.gz

  create-release:
    needs: [build-macos, build-windows, build-linux]
    runs-on: ubuntu-latest
    permissions:
      contents: write
    steps:
      - name: Checkout code
        uses: actions/checkout@v4
      
      - name: Download all artifacts
        uses: actions/download-artifact@v4
        with:
          path: artifacts
      
      - name: Prepare release files
        run: |
          mkdir -p release
          cp artifacts/macos-bundle/* release/
          cp artifacts/windows-bundle/* release/
          cp artifacts/linux-bundle/* release/
          
          # 统一在 Linux 上生成标准格式校验和,保证 sha256sum -c 可直接校验
          cd release
          sha256sum BiuViewer-* > checksums.txt
      
      - name: Generate release notes
        id: release_notes
        run: |
          VERSION=${GITHUB_REF#refs/tags/}
          cat > release_notes.md << EOF
          ## 博览 BiuViewer ${VERSION}
          
          ### 下载
          
          | 平台 | 文件 | 说明 |
          |:---|:---|:---|
          | macOS | BiuViewer-${VERSION}-macOS-universal.zip | 通用二进制(Intel + Apple Silicon) |
          | Windows | BiuViewer-${VERSION}-windows-x64.zip | 64位便携版 |
          | Linux | BiuViewer-${VERSION}-linux-x86_64.tar.gz | 64位通用包 |
          
          ### 安装说明
          
          #### macOS
          1. 解压 \`BiuViewer-${VERSION}-macOS-universal.zip\`
          2. 将 \`BiuViewer.app\` 拖入 Applications 文件夹
          3. 首次打开时,右键点击应用选择"打开",或执行:
             \`\`\`bash
             xattr -cr /Applications/BiuViewer.app
             \`\`\`
          
          #### Windows
          1. 解压 \`BiuViewer-${VERSION}-windows-x64.zip\`
          2. 运行 \`biuviewer.exe\`
          3. 如遇 SmartScreen 警告,点击"更多信息" → "仍要运行"
          
          #### Linux
          1. 解压 \`BiuViewer-${VERSION}-linux-x86_64.tar.gz\`
          2. 进入目录: \`cd biuviewer\`
          3. 运行安装脚本: \`./install.sh\` (可选)
          4. 或直接运行: \`./bin/biuviewer\`
          
          ### 校验和
          
          请使用 \`checksums.txt\` 验证下载文件的完整性:
          \`\`\`bash
          # macOS/Linux (--ignore-missing 仅校验已下载的文件)
          shasum -a 256 -c checksums.txt --ignore-missing
          
          # Windows (PowerShell)
          Get-FileHash BiuViewer-${VERSION}-windows-x64.zip -Algorithm SHA256
          \`\`\`
          
          ### 更新日志
          
          - 首次发布
          
          ---
          
          **注意**: 本项目为开源软件,不提供代码签名。首次运行时系统可能提示安全警告,这是正常现象。
          EOF
          
          echo "notes_file=release_notes.md" >> $GITHUB_OUTPUT
      
      - name: Create GitHub Release
        uses: softprops/action-gh-release@v1
        with:
          body_path: release_notes.md
          files: |
            release/BiuViewer-*
            release/checksums.txt
          draft: false
          prerelease: false
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

### 7.4 打包脚本详解

#### 7.4.1 macOS 打包脚本

```bash
#!/bin/bash
# scripts/bundle-macos.sh

set -e

VERSION=$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].version')
APP_NAME="BiuViewer"
BUNDLE_DIR="target/bundle"
APP_DIR="${BUNDLE_DIR}/${APP_NAME}.app"

echo "==> Building macOS app bundle for version ${VERSION}"

# 创建 .app 目录结构
mkdir -p "${APP_DIR}/Contents/MacOS"
mkdir -p "${APP_DIR}/Contents/Resources"

# 复制可执行文件
cp target/release/biuviewer "${APP_DIR}/Contents/MacOS/${APP_NAME}"
chmod +x "${APP_DIR}/Contents/MacOS/${APP_NAME}"

# 复制图标
if [ -f "resources/icons/icon.icns" ]; then
    cp resources/icons/icon.icns "${APP_DIR}/Contents/Resources/"
fi

# 创建 Info.plist
cat > "${APP_DIR}/Contents/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
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
            <key>LSItemContentTypes</key>
            <array>
                <string>public.image</string>
                <string>public.png</string>
                <string>public.jpeg</string>
                <string>com.compuserve.gif</string>
                <string>public.webp</string>
            </array>
        </dict>
    </array>
</dict>
</plist>
EOF

# 创建 PkgInfo
echo "APPL????" > "${APP_DIR}/Contents/PkgInfo"

# 打包成 zip (文件名格式: BiuViewer-v1.0.0-macOS-universal.zip)
cd "${BUNDLE_DIR}"
zip -r "BiuViewer-v${VERSION}-macOS-universal.zip" "${APP_NAME}.app"
rm -rf "${APP_NAME}.app"

echo "==> macOS bundle created: ${BUNDLE_DIR}/BiuViewer-v${VERSION}-macOS-universal.zip"
```

#### 7.4.2 Windows 打包脚本

```bash
#!/bin/bash
# scripts/bundle-windows.sh

set -e

VERSION=$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].version')
APP_NAME="BiuViewer"
BUNDLE_DIR="target/bundle"
STAGE_DIR="${BUNDLE_DIR}/stage"

echo "==> Building Windows package for version ${VERSION}"

# 创建临时目录
mkdir -p "${STAGE_DIR}"

# 复制可执行文件
cp target/x86_64-pc-windows-msvc/release/biuviewer.exe "${STAGE_DIR}/"

# 复制图标(可选)
if [ -f "resources/icons/icon.ico" ]; then
    cp resources/icons/icon.ico "${STAGE_DIR}/"
fi

# 创建 README.txt
cat > "${STAGE_DIR}/README.txt" << EOF
BiuViewer ${VERSION}
===================

Thank you for downloading BiuViewer!

Getting Started:
1. Run biuviewer.exe
2. If you see a Windows SmartScreen warning, click "More info" then "Run anyway"

For more information, visit: https://github.com/your-org/biu-viewer

EOF

# 创建快捷启动脚本(可选)
cat > "${STAGE_DIR}/biuviewer.bat" << 'EOF'
@echo off
start "" "%~dp0biuviewer.exe" %*
EOF

# 打包成 zip (文件名格式: BiuViewer-v1.0.0-windows-x64.zip) - GitHub Windows runner 预装 7-Zip
cd "${BUNDLE_DIR}"
7z a -tzip "BiuViewer-v${VERSION}-windows-x64.zip" ./stage/*

# 清理临时目录
rm -rf stage

echo "==> Windows bundle created: ${BUNDLE_DIR}/BiuViewer-v${VERSION}-windows-x64.zip"
```

#### 7.4.3 Linux 打包脚本

```bash
#!/bin/bash
# scripts/bundle-linux.sh

set -e

VERSION=$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].version')
APP_NAME="biuviewer"
BUNDLE_DIR="target/bundle"
STAGE_DIR="${BUNDLE_DIR}/${APP_NAME}"

echo "==> Building Linux package for version ${VERSION}"

# 创建目录结构
mkdir -p "${STAGE_DIR}/bin"
mkdir -p "${STAGE_DIR}/share/applications"
mkdir -p "${STAGE_DIR}/share/icons/hicolor/256x256/apps"
mkdir -p "${STAGE_DIR}/share/doc/${APP_NAME}"

# 复制可执行文件
cp target/x86_64-unknown-linux-gnu/release/biuviewer "${STAGE_DIR}/bin/"
chmod +x "${STAGE_DIR}/bin/biuviewer"

# 复制图标
if [ -f "resources/icons/icon.png" ]; then
    cp resources/icons/icon.png "${STAGE_DIR}/share/icons/hicolor/256x256/apps/${APP_NAME}.png"
fi

# 创建 .desktop 文件
cat > "${STAGE_DIR}/share/applications/${APP_NAME}.desktop" << EOF
[Desktop Entry]
Version=1.0
Type=Application
Name=BiuViewer
GenericName=Image Viewer
Comment=Lightweight cross-platform image viewer
Exec=biuviewer %f
Icon=biuviewer
Terminal=false
Categories=Graphics;Viewer;
MimeType=image/png;image/jpeg;image/gif;image/bmp;image/webp;image/tiff;
StartupNotify=true
EOF

# 创建安装脚本
cat > "${STAGE_DIR}/install.sh" << 'EOF'
#!/bin/bash

set -e

PREFIX="${PREFIX:-$HOME/.local}"

echo "Installing BiuViewer to ${PREFIX}"

# 安装二进制
install -Dm755 bin/biuviewer "${PREFIX}/bin/biuviewer"

# 安装桌面文件
install -Dm644 share/applications/biuviewer.desktop \
    "${PREFIX}/share/applications/biuviewer.desktop"

# 安装图标
if [ -f "share/icons/hicolor/256x256/apps/biuviewer.png" ]; then
    install -Dm644 share/icons/hicolor/256x256/apps/biuviewer.png \
        "${PREFIX}/share/icons/hicolor/256x256/apps/biuviewer.png"
fi

# 更新桌面数据库
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "${PREFIX}/share/applications" || true
fi

# 更新图标缓存
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache "${PREFIX}/share/icons/hicolor" || true
fi

echo ""
echo "Installation complete!"
echo ""
echo "Run 'biuviewer' to start the application"
echo "or find it in your application menu."
EOF

chmod +x "${STAGE_DIR}/install.sh"

# 创建 README
cat > "${STAGE_DIR}/README.md" << EOF
# BiuViewer ${VERSION}

Lightweight cross-platform image viewer.

## Installation

### Option 1: Install to ~/.local (Recommended)

\`\`\`bash
./install.sh
\`\`\`

### Option 2: Install system-wide

\`\`\`bash
sudo PREFIX=/usr ./install.sh
\`\`\`

### Option 3: Run directly

\`\`\`bash
./bin/biuviewer
\`\`\`

## Uninstallation

\`\`\`bash
rm -f ~/.local/bin/biuviewer
rm -f ~/.local/share/applications/biuviewer.desktop
rm -f ~/.local/share/icons/hicolor/256x256/apps/biuviewer.png
\`\`\`

For more information, visit: https://github.com/your-org/biu-viewer
EOF

# 打包成 tar.gz (文件名格式: BiuViewer-v1.0.0-linux-x86_64.tar.gz)
cd "${BUNDLE_DIR}"
tar czf "BiuViewer-v${VERSION}-linux-x86_64.tar.gz" "${APP_NAME}/"
rm -rf "${APP_NAME}/"

echo "==> Linux bundle created: ${BUNDLE_DIR}/BiuViewer-v${VERSION}-linux-x86_64.tar.gz"
```

#### 7.4.4 通用工具函数

```bash
#!/bin/bash
# scripts/utils.sh

# 获取版本号
get_version() {
    cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].version'
}

# 检查命令是否存在
check_command() {
    command -v "$1" >/dev/null 2>&1
}

# 确保目录存在
ensure_dir() {
    mkdir -p "$1"
}

# 清理构建产物
clean_bundle() {
    rm -rf target/bundle
}
```

---

### 7.5 平台特定注意事项

#### 7.5.1 macOS

**无签名应用处理**:
- 用户首次打开时会遇到 Gatekeeper 警告
- 解决方法 1: 右键点击 → 打开 → 仍要打开
- 解决方法 2: 终端执行 `xattr -cr /Applications/BiuViewer.app`
- 在 README 中明确说明这是开源软件,未进行代码签名

**通用二进制(Universal Binary)**:
- 支持 Intel (x86_64) 和 Apple Silicon (aarch64)
- 使用 `lipo` 命令合并两个架构的二进制文件
- 确保更好的兼容性和性能

**文件关联**:
- 通过 `Info.plist` 中的 `CFBundleDocumentTypes` 声明支持的图片格式
- 用户可在 Finder 中设置默认打开方式

#### 7.5.2 Windows

**SmartScreen 警告**:
- 未签名的应用会触发 Windows Defender SmartScreen
- README 中说明: "更多信息" → "仍要运行"
- 随着下载量增加,Windows 会逐渐建立信誉

**便携版优势**:
- 无需管理员权限
- 可放置在任意位置
- 不写入注册表
- 适合 U 盘使用

**文件关联(可选)**:
- 创建注册表脚本供用户选择性执行
- 或通过"打开方式"手动关联

#### 7.5.3 Linux

**依赖处理**:
- 静态链接尽可能多的依赖
- 动态链接系统库(如 glibc、GTK)
- README 中列出必要的系统依赖

**多发行版兼容**:
- `.tar.gz` 适用于所有发行版
- `.deb` 适用于 Debian/Ubuntu 系
- 后续可考虑 AUR (Arch Linux)、Flatpak、AppImage

**桌面集成**:
- 提供 `.desktop` 文件
- 安装图标到 `~/.local/share/icons`
- 更新桌面数据库和图标缓存

### 7.6 发布检查清单

每次发布前确保:

**代码质量**:
- [ ] 所有测试通过 (`cargo test`)
- [ ] 无 Clippy 警告 (`cargo clippy`)
- [ ] 无安全漏洞 (`cargo audit`)
- [ ] 代码格式化 (`cargo fmt`)

**版本管理**:
- [ ] 更新 `Cargo.toml` 中的版本号
- [ ] 更新 `CHANGELOG.md`
- [ ] Git tag 版本 (`git tag -a v0.1.0 -m "Release v0.1.0"`)

**构建测试**:
- [ ] macOS 构建成功且可运行
- [ ] Windows 构建成功且可运行
- [ ] Linux 构建成功且可运行
- [ ] 所有平台的校验和正确

**文档更新**:
- [ ] README.md 更新安装说明
- [ ] INSTALL.md 包含详细步骤
- [ ] FAQ.md 更新常见问题
- [ ] Release Notes 完整

**发布后**:
- [ ] 验证 GitHub Release 页面正常
- [ ] 验证所有下载链接可用
- [ ] 校验和文件正确
- [ ] 在各平台测试下载和安装

### 7.7 后续打包增强(P1/P2)

#### P1 功能:
- **Windows 安装程序**: 按 PRD 使用 Inno Setup 生成 `.exe` 安装包(备选 WiX 生成 `.msi`),负责文件关联与卸载
- **Linux .deb 包**: 使用 `cargo-deb` 生成 Debian 包

#### P2 功能:
- **更新检查**: 应用内检测新版本(仅提示,不自动下载;PRD 中网络功能延后,需单独评审)
- **Linux AppImage**: 单文件可执行,无需安装
- **Linux Flatpak**: 沙盒化分发
- **AUR 包**: Arch Linux 用户仓库
- **Homebrew Cask**: macOS 包管理器支持
- **Scoop/Chocolatey**: Windows 包管理器支持

---

## 八、性能指标与监控

### 8.1 性能监控点

```rust
use std::time::Instant;

struct PerformanceMetrics {
    decode_time: Duration,
    upload_time: Duration,
    render_time: Duration,
    frame_time: Duration,
}

impl App {
    fn measure_performance(&self) {
        let start = Instant::now();
        
        // 解码
        let decode_start = Instant::now();
        let decoded = decode_image(&path);
        let decode_time = decode_start.elapsed();
        
        // 上传
        let upload_start = Instant::now();
        let texture = upload_to_gpu(decoded);
        let upload_time = upload_start.elapsed();
        
        // 渲染
        let render_start = Instant::now();
        self.render();
        let render_time = render_start.elapsed();
        
        tracing::info!(
            decode_ms = decode_time.as_millis(),
            upload_ms = upload_time.as_millis(),
            render_ms = render_time.as_millis(),
            "Performance metrics"
        );
    }
}
```

### 8.2 性能目标

| 指标 | 目标值 | 测量方法 |
|:---|:---|:---|
| 启动时间 | < 1s | 从进程启动到窗口可交互 |
| 图片加载时间 | < 1s (10MB) | 从请求到纹理显示 |
| 翻页响应时间 | < 100ms | 按键到下一张显示 |
| 缩放响应时间 | < 16ms (60fps) | 滚轮到视图更新 |
| 内存占用 | < 200MB (基线) | 空载或单图显示 |
| 内存峰值 | < 1GB | 连续浏览 100 张 10MB 图片 |

### 8.3 性能优化检查清单

- [ ] 解码在后台线程执行
- [ ] 纹理上传批量执行
- [ ] 相邻图片预加载
- [ ] LRU 缓存淘汰
- [ ] 大图自动降采样
- [ ] 视口外纹理释放
- [ ] 零拷贝数据传递
- [ ] GPU 加速渲染
- [ ] 取消过期加载任务
- [ ] 缩略图磁盘缓存

---

## 九、测试策略

### 9.1 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_matrix() {
        let transform = TransformState {
            scale: 2.0,
            offset: Point2D::new(10.0, 20.0),
            rotation: Rotation::Deg90,
            flip_h: false,
            flip_v: false,
        };
        
        let matrix = transform.to_matrix();
        let point = Point2D::new(100.0, 100.0);
        let transformed = matrix.transform_point(point);
        
        assert_eq!(transformed, Point2D::new(-180.0, 220.0));
    }
    
    #[test]
    fn test_lru_cache() {
        let mut cache = MemoryCache::new(1024);
        
        cache.insert("a", vec![0u8; 512]);
        cache.insert("b", vec![0u8; 512]);
        cache.insert("c", vec![0u8; 512]); // 触发淘汰
        
        assert!(cache.contains("b"));
        assert!(cache.contains("c"));
        assert!(!cache.contains("a")); // a 被淘汰
    }
}
```

### 9.2 集成测试

```rust
#[test]
fn test_image_loading_flow() {
    let app = App::new();
    let path = PathBuf::from("tests/fixtures/test.png");
    
    app.open_file(path.clone());
    
    // 等待加载完成
    std::thread::sleep(Duration::from_secs(1));
    
    let state = app.state();
    assert_eq!(state.current_image().unwrap().path, path);
    assert!(state.current_texture().is_some());
}
```

### 9.3 性能测试

```rust
#[bench]
fn bench_decode_jpeg(b: &mut Bencher) {
    let data = std::fs::read("tests/fixtures/large.jpg").unwrap();
    b.iter(|| {
        let _ = image::load_from_memory(&data);
    });
}

#[bench]
fn bench_texture_upload(b: &mut Bencher) {
    let img = RgbaImage::new(1920, 1080);
    let context = GpuContext::new();
    b.iter(|| {
        let _ = context.create_texture(&img);
    });
}
```

### 9.4 UI 测试

```rust
#[test]
fn test_keyboard_navigation() {
    let app = App::new();
    app.load_folder("tests/fixtures/photos");
    
    assert_eq!(app.state().current_index, 0);
    
    app.send_key(Key::Right);
    assert_eq!(app.state().current_index, 1);
    
    app.send_key(Key::Left);
    assert_eq!(app.state().current_index, 0);
}
```

---

## 十、安全性考虑

### 10.1 文件安全

```rust
fn validate_file_path(path: &Path) -> Result<()> {
    // 防止路径遍历攻击
    let canonical = path.canonicalize()
        .map_err(|_| Error::InvalidPath)?;
    
    // 检查文件大小
    let metadata = std::fs::metadata(&canonical)?;
    if metadata.len() > MAX_FILE_SIZE {
        return Err(Error::FileTooLarge);
    }
    
    Ok(())
}

fn sanitize_filename(name: &str) -> String {
    // 移除不安全字符
    name.chars()
        .filter(|c| c.is_alphanumeric() || *c == '.' || *c == '-' || *c == '_')
        .collect()
}
```

### 10.2 解码安全

```rust
fn safe_decode(data: &[u8]) -> Result<RgbaImage> {
    // 限制解码超时
    let handle = std::thread::spawn(move || {
        image::load_from_memory(data)
    });
    
    match handle.join_timeout(Duration::from_secs(10)) {
        Ok(Ok(img)) => Ok(img.to_rgba8()),
        Ok(Err(e)) => Err(Error::DecodeError(e)),
        Err(_) => Err(Error::DecodeTimeout),
    }
}
```

### 10.3 内存安全

- 使用 Rust 的所有权系统防止内存泄漏
- 所有不安全代码(FFI)封装在 `unsafe` 块中并仔细审查
- 定期运行 `cargo clippy` 和 `cargo audit`
- 使用 AddressSanitizer 检测内存问题

---

## 十一、可扩展性设计

### 11.1 插件系统(后期)

```rust
trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn on_load(&mut self, ctx: &mut PluginContext);
    fn on_image_loaded(&mut self, image: &Image);
}

struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginManager {
    fn load_plugin(&mut self, path: &Path) -> Result<()> {
        // 动态加载 .so/.dylib/.dll
        let lib = unsafe { Library::new(path)? };
        let create_plugin: Symbol<fn() -> Box<dyn Plugin>> =
            unsafe { lib.get(b"create_plugin")? };
        let plugin = create_plugin();
        self.plugins.push(plugin);
        Ok(())
    }
}
```

### 11.2 自定义解码器

```rust
// 用户可提供自定义解码器
struct CustomDecoder {
    extensions: Vec<String>,
}

impl ImageDecoder for CustomDecoder {
    fn supports(&self, format: ImageFormat) -> bool {
        self.extensions.contains(&format.extension())
    }
    
    fn decode(&self, data: &[u8]) -> Result<DecodedImage> {
        // 自定义解码逻辑
        todo!()
    }
}

// 注册
app.register_decoder(Box::new(CustomDecoder {
    extensions: vec!["jxl".to_string()],
}));
```

---

## 十二、总结

### 12.1 架构优势

1. **高性能**: GPU 加速渲染 + 异步 I/O + 智能缓存
2. **模块化**: 清晰的分层和模块划分,易于维护和扩展
3. **跨平台**: 统一代码库,平台差异封装在适配层
4. **类型安全**: Rust 的类型系统在编译期捕获大部分错误
5. **可扩展**: 插件系统和可插拔解码器支持功能扩展

### 12.2 技术亮点

- **零拷贝**: 使用 `Arc` 共享数据,避免不必要的内存拷贝
- **GPU 加速**: 利用 GPUI 的 GPU 渲染路径,流畅的缩放和平移
- **智能缓存**: 两级缓存 + LRU 淘汰 + 预加载策略
- **渐进式加载**: 缩略图 → 降采样 → 完整图片的渐进式体验
- **异步架构**: 所有 I/O 操作异步执行,UI 始终响应

### 12.3 后续优化方向

1. **瓦片加载**: 支持超大图片的按需加载
2. **SIMD 优化**: 利用 SIMD 指令加速图像处理
3. **多线程解码**: 支持多核并行解码
4. **增量渲染**: 只渲染可见区域,减少 GPU 负载
5. **WebGPU**: 探索使用 WebGPU 替代 OpenGL

---

**文档状态**: 待评审  
**最后更新**: 2026-11-04  
**维护者**: BiuViewer 开发团队
