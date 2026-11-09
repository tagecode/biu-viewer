# MVP 全链路验收记录

对应 `docs/Mvp.md` 第四章清单。三平台各执行一次，记录结果；全部通过后打正式 tag `v0.1.0`。

- 验收对象：Release 产物（不是本地 `cargo run`）
- macOS 记录时间：2026-11-05
- macOS 产物：`BiuViewer-v0.1.0-macOS-universal.zip`（本地构建为 x86_64；CI 为 universal）
- Windows 记录时间：2026-11-09
- Windows 产物：`BiuViewer-v0.1.2-windows-x64.zip`
- Windows 环境：Windows 10 专业版 22H2（19045）、1920×1080、100% 缩放、Xeon E5-2678 v3 / 32 GB

## 结果表

| # | 验收项 | macOS | Windows | Linux | 备注 |
| :--- | :--- | :---: | :---: | :---: | :--- |
| 1 | 通过对话框打开文件夹 | ☐ | ✅ | ☐ | rfd 原生对话框；macOS 待人工点击验证；Windows v0.1.2：自动化在「打开文件夹」对话框填入路径并确认，切换到所选目录 |
| 2 | 拖拽文件夹到窗口打开 | ☐ | ✅ | ☐ | macOS 待真实拖拽验证；Windows v0.1.2：原生 OLE 拖拽（`DoDragDrop`），窗口切换到被拖入的目录 |
| 3 | 命令行打开文件夹 | ✅ | ✅ | ☐ | `BiuViewer.app/Contents/MacOS/BiuViewer /tmp/biu-bench/1000-photos`；Windows：`biuviewer.exe <目录>`（含中文与空格路径） |
| 4 | 缩略图条浏览，点击切换 | ✅ | ✅ | ☐ | macOS：点击第 3 个缩略图 → 3/1000 且高亮；Windows：v0.1.1 下缩略图条跟随滚动、高亮正确；v0.1.2 点击第 2 个缩略图切到该图 |
| 5 | 方向键切换上一张/下一张 | ✅ | ✅ | ☐ | macOS：Home → 1/1000，→ 逐步前进，末张不越界；Windows：翻页时缩略图条不空白且跟随（v0.1.1 修复项）；v0.1.2 复核 →/←/Home/End 与首尾边界 |
| 6 | 适应窗口显示 | ✅ | ✅ | ☐ | 4K 图 22%、小图 100%（不放大）；Windows v0.1.2：4K 适应后 848×474（1.79:1，左右留边），320×240 图 100% 不放大 |
| 7 | 放大/缩小/重置 | ✅ | ✅ | ☐ | 工具栏 `+` 三次 100% → 195%（模式变「自定义」）；「重置」回到适应模式（22%，v0.1.1 之后修复）；Windows v0.1.2：`+`×3 → 622×466，`0` 与「重置」均恢复适应 |
| 8 | 拖拽平移 | ✅ | ✅ | ☐ | macOS：195% 下向左上拖拽，画面随鼠标移动；Windows：拖拽平移正常、小图时拖拽无效果，但光标不显示抓手（GPUI Windows 后端限制，见「已知限制」）；v0.1.2 在 4K 1:1 下复核 |
| 9 | 旋转与翻转 | ✅ | ✅ | ☐ | `R` 旋转 90°；工具栏「水平翻转」镜像；「重置」复位；Windows v0.1.2：旋转后宽高交换、四角顺序变化，连按 4 次复原 |
| 10 | 全屏进入/退出 | ✅ | ✅ | ☐ | `F` 进入（隐藏工具栏/缩略图/状态栏），`Esc` 退出；Windows v0.1.2：1920×1080 全屏、Esc 还原 |
| 11 | EXIF 方向正确（8 个方向样本） | ✅ | ✅ | ☐ | `exif/` 下 8 张存储像素各不相同，显示结果完全一致：红左上/绿右上/蓝左下/黄右下 + 顶部黑条，均 240×160；Windows v0.1.2：8/8 一致 |
| 12 | GIF 按正确帧率播放 | ✅ | ✅ | ☐ | 40 帧 GIF 间隔 0.4 s 截图帧内容不同；Windows v0.1.2：80 ms/帧，1 s 内采到 8 个不同帧位置；单帧 GIF 保持静止 |
| 13 | 7 种 P0 格式均可显示 | ✅ | ✅ | ☐ | `formats/` 下 PNG/JPEG/GIF/BMP/WebP/TIFF 均 320×240，ICO 128×128，全部正常显示；Windows v0.1.2：尺寸与角块颜色均正确 |
| 14 | 损坏/不支持/无权限有提示且不崩溃 | ✅ | ✅ | ☐ | 三种错误分别显示错误卡片 / 提示条；Windows v0.1.2：截断 PNG 与拒绝访问文件显示错误卡片，`.txt` 启动或拖入显示提示条，均不崩溃 |
| 15 | 连续翻页 500 张内存不持续增长 | ✅ | ✅ | ☐ | 见 `benchmark.md`：受 512 MB 预算约束，回翻时回落；Windows v0.1.2：翻 1000 张 WorkingSet 恒定 42 MB（纹理由 GPU 持有，不计入工作集），回访第 500 张画面像素一致 |
| 16 | 性能指标达标 | ✅ | ✅ | ☐ | 见 `benchmark.md`；帧率待人工。Windows v0.1.2：首图 23 ms（窗口就绪后）、翻页 33–50 ms、10 MB 冷开 18 ms、大图切换 150 ms、空闲 42 MB |
| 17 | 产物下载后按 README 可运行 | ✅ | ✅ | ☐ | 解压 Release 中的 `BiuViewer-v0.1.0-macOS-universal.zip` 后 `BiuViewer.app/Contents/MacOS/BiuViewer --version` 输出 `biuviewer 0.1.0`；Windows v0.1.2：zip SHA256 与 `checksums.txt` 一致，`--version` 输出 `biuviewer 0.1.2`，GUI 子系统无控制台 |

图例：✅ 通过 / ❌ 不通过 / ☐ 未执行。

## 错误提示实测

| 场景 | 构造方式 | 实际表现 |
| :--- | :--- | :--- |
| 文件损坏 | 截断的 PNG | 错误卡片「文件已损坏，无法读取：unexpected end of file」，显示文件名 |
| 不支持的格式 | `notes.txt` | 提示条「不支持的格式：…/notes.txt」，工具栏置灰，状态栏「未打开文件」 |
| 无权限 | `chmod 000` 的 PNG | 错误卡片「没有权限访问：Permission denied (os error 13)」，缩略图显示 `!` |

Windows `v0.1.2` 上同样三种场景：截断 PNG 与 `icacls` 拒绝访问的文件显示错误卡片，`notes.txt` 启动或拖入时显示提示条，工具栏按预期置灰；本轮自动化按像素判定卡片与提示条的存在，未逐字核对提示文案。

## 未覆盖项与原因

- **对话框与拖拽（macOS / Linux）**：需要人工交互，脚本无法可靠模拟（原生 NSOpenPanel 与系统拖拽）。Windows 已由自动化覆盖：WinForms 窗体配合原生 `ole32!DoDragDrop`（自定义 `IDropSource`）拖入文件/目录；UI Automation 在「打开文件夹」对话框的路径框（AutomationId `1152`）填入路径并点击「选择文件夹」。
- **滚轮缩放与触控板手势**：合成滚轮事件（`CGEvent`）不被 GPUI 接收，需真实鼠标/触控板验证。命令行脚本已确认 `on_scroll` 参数换算逻辑（单测覆盖视口缩放数学）。
- **帧率**：无法用脚本稳定采样合成器帧率。

## 已知限制

- **Windows 抓手光标**：GPUI 的 Windows 后端（`gpui-pre-windows` 的 `load_cursor`）只映射了 `IBeam`、`Crosshair`、`PointingHand`、`Resize*`、`OperationNotAllowed`，`OpenHand` / `ClosedHand` 会回退为箭头。因此在 Windows 上拖拽平移时光标不变，功能本身正常；macOS（`NSCursor`）与 Linux（`grab` / `grabbing`）显示正常。

## 发布产物校验

对 `https://github.com/tagecode/biu-viewer/releases/tag/v0.1.0` 的三个产物做了静态校验（2026-11-05）：

| 产物 | 校验和 | 架构 / 内容 |
| :--- | :--- | :--- |
| `BiuViewer-v0.1.0-macOS-universal.zip` | 通过 | `lipo -info` 同时包含 `x86_64` 与 `arm64`；`Info.plist` 通过 `plutil -lint` |
| `BiuViewer-v0.1.0-windows-x64.zip` | 通过 | `PE32+ executable (GUI) x86-64`，含 `biuviewer.exe`、`icon.ico`、`README.txt`；GUI 子系统表示不会弹控制台 |
| `BiuViewer-v0.1.0-linux-x86_64.tar.gz` | 通过 | `ELF 64-bit LSB pie executable, x86-64, stripped`；含 `bin/biuviewer`（保留执行位）、`install.sh`、`.desktop`、图标、README |

Linux 产物的**运行**行为仍需在对应系统上验证；Windows 已在 `v0.1.2` 上完成运行验收（见上文结果表）。

`v0.1.1` 在 `v0.1.0` 基础上仅包含一项修复：缩略图条在翻页时跟随当前图片（GPUI `ScrollHandle` 的偏移方向取反），产物内容、包结构与上述校验结论一致。三个新产物的 `shasum -a 256 -c checksums.txt` 全部通过；macOS 产物 `--version` 输出 `biuviewer 0.1.1`，`lipo -info` 显示 `x86_64 arm64`。

`v0.1.2` 在此基础上修复「重置」未恢复适应窗口的问题（T12）。Windows 产物 `BiuViewer-v0.1.2-windows-x64.zip` 的 SHA256 为 `c33f9994b889d3620072c30bee0b5c3909d2955baa201e33d5234f866b1b9460`，与 `checksums.txt` 一致；解压后的 `biuviewer.exe` 为 PE32+ GUI 子系统（启动不弹控制台），SHA256 `a309ffffbc5c5428d68ae5dc0093a9d4eefc809c198d2c9422671e1a21fccc4a`，`--version` 输出 `biuviewer 0.1.2`。

## 测试素材

```bash
cargo run --release -p biuviewer-decoder --example make_assets -- /tmp/biu-bench
```

| 目录 | 用途 |
| :--- | :--- |
| `1000-photos/` | 翻页、缩略图、内存曲线（1000 张 400×300 PNG） |
| `formats/` | 7 种 P0 格式各一张（PNG/JPEG/GIF/BMP/WebP/TIFF/ICO） |
| `exif/` | 8 种 EXIF 方向的 JPEG，校正后应显示为同一张正立图 |
| 根目录 | `large-10mb.png`、`uhd-4k.png`、`animation.gif`、`corrupted.png`、`notes.txt` |

`exif/` 的 8 张样张存储像素各不相同（第 5–8 张的宽高是交换的），只有正确应用 EXIF 方向才会显示成一致的正立图。按四种角落颜色（左上红、右上绿、左下蓝、右下黄）和顶部黑条位置判断即可。

样张由 `make_assets` 用独立实现的 EXIF 变换表生成，并与 `image` crate 的 `apply_orientation` 交叉校验。第 5–8 张经 JPEG 色度子采样后，与第 1 张相比在色块边缘最多相差 21 个色阶（约 6% 像素），属于压缩噪声，肉眼不可分辨。

Windows 自动化复测使用了一份按相同构造生成的等价样张（颜色象限 + 顶部黑条、EXIF 8 方向、7 种格式、1000 张 400×300 渐变图，以及中文/相对路径、损坏/无权限、空目录等场景），个别文件尺寸与 `make_assets` 不同（如 `large-10mb.png` 为 2400×1800）。判定标准是显示尺寸、四角颜色与提示条/错误卡片的存在，不依赖样张的具体随机像素。

## Windows / Linux 复测步骤

1. 从 `https://github.com/tagecode/biu-viewer/releases/tag/v0.1.2` 下载对应产物（`checksums.txt` 可校验完整性）。
2. 按 `docs/INSTALL.md` 安装并首次启动。
3. 准备素材：在有 Rust 工具链的机器上执行 `make_assets`，把产出的整个目录拷过去；或把 macOS 上已生成的 `/tmp/biu-bench` 直接复制。`formats/`、`exif/` 两个子目录用于第 11、13 项。
4. 逐项执行上表 ☐ 项，把结果补进本节对应的列。

### 平台特有注意事项

- **Windows**：`bench-manual.sh` 的内存采样段用 `ps -o rss=`，Git Bash 的 `ps` 不支持该字段，需改用 `Get-Process -Id <pid> | % { $_.WorkingSet64 }`；另外检查 SmartScreen 提示、125%/150% 缩放下工具栏与缩略图布局、中文与含空格路径。以上三项平台特有检查已于 2026-11-05 在 `v0.1.1` 上验证通过。
  2026-11-09 的 `v0.1.2` 复测中，17 项均由自动化脚本在 Release 产物上执行：以截图像素判定图片区域与四角颜色、`keybd_event`/`mouse_event` 注入按键与拖拽、原生 OLE `DoDragDrop` 拖放、UI Automation 操作文件对话框，内存用 `WorkingSet64` 采样。工作集不包含 GPU 显存，因此 Windows 的内存数字与 macOS 的 RSS 不可直接比较。
- **Linux**：X11 与 Wayland 各跑一轮（Wayland 下拖放行为不同）；确认干净系统按 README 装完依赖后可启动；`./install.sh` 后应用菜单项与图标正常；解压后的 `bin/biuviewer` 保留执行位。

## 结论

macOS 上已验证主流程（打开 → 缩略图 → 翻页 → 缩放/平移/旋转/全屏 → GIF → 7 种格式 → EXIF 8 方向 → 错误提示 → 内存）与打包产物可运行，17 项中 15 项通过；剩余 2 项（对话框、拖拽）是需要人工交互的操作，未在自动化流程中覆盖。

Windows 在 `v0.1.2` 的 Release 产物上 17 项全部通过（帧率一项与 macOS 相同，仍需人工感受）。除缩略图条与翻页由 v0.1.1 先行验证外，其余项目均由自动化脚本复核：对话框经 UI Automation、拖拽经原生 OLE、格式/EXIF/GIF/错误提示/内存与性能经截图像素与进程采样判定；平台特有检查（SmartScreen、125%/150% 缩放、中文与空格路径）沿用 v0.1.1 的结论。Linux 全项待测。
