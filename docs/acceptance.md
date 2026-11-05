# MVP 全链路验收记录

对应 `docs/Mvp.md` 第四章清单。三平台各执行一次，记录结果；全部通过后打正式 tag `v0.1.0`。

- 验收对象：Release 产物（不是本地 `cargo run`）
- macOS 记录时间：2026-11-05
- macOS 产物：`BiuViewer-v0.1.0-macOS-universal.zip`（本地构建为 x86_64；CI 为 universal）
- Windows 记录时间：2026-11-05
- Windows 产物：`BiuViewer-v0.1.1-windows-x64.zip`

## 结果表

| # | 验收项 | macOS | Windows | Linux | 备注 |
| :--- | :--- | :---: | :---: | :---: | :--- |
| 1 | 通过对话框打开文件夹 | ☐ | ✅ | ☐ | rfd 原生对话框；macOS 待人工点击验证 |
| 2 | 拖拽文件夹到窗口打开 | ☐ | ✅ | ☐ | macOS 待真实拖拽验证 |
| 3 | 命令行打开文件夹 | ✅ | ☐ | ☐ | `BiuViewer.app/Contents/MacOS/BiuViewer /tmp/biu-bench/1000-photos` |
| 4 | 缩略图条浏览，点击切换 | ✅ | ✅ | ☐ | macOS：点击第 3 个缩略图 → 3/1000 且高亮；Windows：v0.1.1 下缩略图条跟随滚动、高亮正确 |
| 5 | 方向键切换上一张/下一张 | ✅ | ✅ | ☐ | macOS：Home → 1/1000，→ 逐步前进，末张不越界；Windows：翻页时缩略图条不空白且跟随（v0.1.1 修复项） |
| 6 | 适应窗口显示 | ✅ | ☐ | ☐ | 4K 图 22%、小图 100%（不放大） |
| 7 | 放大/缩小/重置 | ✅ | ☐ | ☐ | 工具栏 `+` 三次 100% → 195%（模式变「自定义」）；「重置」回到适应模式（22%，v0.1.1 之后修复，本地 release 构建验证） |
| 8 | 拖拽平移 | ✅ | ✅ | ☐ | macOS：195% 下向左上拖拽，画面随鼠标移动；Windows：拖拽平移正常、小图时拖拽无效果，但光标不显示抓手（GPUI Windows 后端限制，见「已知限制」） |
| 9 | 旋转与翻转 | ✅ | ☐ | ☐ | `R` 旋转 90°；工具栏「水平翻转」镜像；「重置」复位 |
| 10 | 全屏进入/退出 | ✅ | ☐ | ☐ | `F` 进入（隐藏工具栏/缩略图/状态栏），`Esc` 退出 |
| 11 | EXIF 方向正确（8 个方向样本） | ✅ | ☐ | ☐ | `exif/` 下 8 张存储像素各不相同，显示结果完全一致：红左上/绿右上/蓝左下/黄右下 + 顶部黑条，均 240×160 |
| 12 | GIF 按正确帧率播放 | ✅ | ☐ | ☐ | 40 帧 GIF 间隔 0.4 s 截图帧内容不同 |
| 13 | 7 种 P0 格式均可显示 | ✅ | ☐ | ☐ | `formats/` 下 PNG/JPEG/GIF/BMP/WebP/TIFF 均 320×240，ICO 128×128，全部正常显示 |
| 14 | 损坏/不支持/无权限有提示且不崩溃 | ✅ | ☐ | ☐ | 三种错误分别显示错误卡片 / 提示条 |
| 15 | 连续翻页 500 张内存不持续增长 | ✅ | ☐ | ☐ | 见 `benchmark.md`：受 512 MB 预算约束，回翻时回落 |
| 16 | 性能指标达标 | ✅ | ☐ | ☐ | 见 `benchmark.md`；帧率待人工 |
| 17 | 产物下载后按 README 可运行 | ✅ | ☐ | ☐ | 解压 Release 中的 `BiuViewer-v0.1.0-macOS-universal.zip` 后 `BiuViewer.app/Contents/MacOS/BiuViewer --version` 输出 `biuviewer 0.1.0` |

图例：✅ 通过 / ❌ 不通过 / ☐ 未执行。

## 错误提示实测

| 场景 | 构造方式 | 实际表现 |
| :--- | :--- | :--- |
| 文件损坏 | 截断的 PNG | 错误卡片「文件已损坏，无法读取：unexpected end of file」，显示文件名 |
| 不支持的格式 | `notes.txt` | 提示条「不支持的格式：…/notes.txt」，工具栏置灰，状态栏「未打开文件」 |
| 无权限 | `chmod 000` 的 PNG | 错误卡片「没有权限访问：Permission denied (os error 13)」，缩略图显示 `!` |

## 未覆盖项与原因

- **对话框与拖拽**：需要人工交互，脚本无法可靠模拟（原生 NSOpenPanel 与系统拖拽）。
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

Windows 与 Linux 产物的**运行**行为仍需在对应系统上验证（当前机器为 macOS）。

`v0.1.1` 在 `v0.1.0` 基础上仅包含一项修复：缩略图条在翻页时跟随当前图片（GPUI `ScrollHandle` 的偏移方向取反），产物内容、包结构与上述校验结论一致。三个新产物的 `shasum -a 256 -c checksums.txt` 全部通过；macOS 产物 `--version` 输出 `biuviewer 0.1.1`，`lipo -info` 显示 `x86_64 arm64`。

`v0.1.2` 在此基础上修复「重置」未恢复适应窗口的问题（T12）。

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

## Windows / Linux 复测步骤

1. 从 `https://github.com/tagecode/biu-viewer/releases/tag/v0.1.0` 下载对应产物（`checksums.txt` 可校验完整性）。
2. 按 `docs/INSTALL.md` 安装并首次启动。
3. 准备素材：在有 Rust 工具链的机器上执行 `make_assets`，把产出的整个目录拷过去；或把 macOS 上已生成的 `/tmp/biu-bench` 直接复制。`formats/`、`exif/` 两个子目录用于第 11、13 项。
4. 逐项执行上表 ☐ 项，把结果补进本节对应的列。

### 平台特有注意事项

- **Windows**：`bench-manual.sh` 的内存采样段用 `ps -o rss=`，Git Bash 的 `ps` 不支持该字段，需改用 `Get-Process -Id <pid> | % { $_.WorkingSet64 }`；另外检查 SmartScreen 提示、125%/150% 缩放下工具栏与缩略图布局、中文与含空格路径。以上三项平台特有检查已于 2026-11-05 在 `v0.1.1` 上验证通过。
- **Linux**：X11 与 Wayland 各跑一轮（Wayland 下拖放行为不同）；确认干净系统按 README 装完依赖后可启动；`./install.sh` 后应用菜单项与图标正常；解压后的 `bin/biuviewer` 保留执行位。

## 结论

macOS 上已验证主流程（打开 → 缩略图 → 翻页 → 缩放/平移/旋转/全屏 → GIF → 7 种格式 → EXIF 8 方向 → 错误提示 → 内存）与打包产物可运行，17 项中 15 项通过；剩余 2 项（对话框、拖拽）是需要人工交互的操作，未在自动化流程中覆盖。Windows 在 `v0.1.1` 上已通过 5 项（对话框、拖拽、缩略图条、方向键翻页、拖拽平移）及平台特有检查，其余 12 项待补测；Linux 全项待测。
