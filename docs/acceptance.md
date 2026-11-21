# MVP 全链路验收记录

对应 `docs/Mvp.md` 第四章清单。以下 17 项是三平台在 v0.1.x 上的历史记录；最新版本状态见「0.2.0 验收状态」。

- 验收对象：Release 产物（不是本地 `cargo run`）
- macOS 记录时间：2026-11-05
- macOS 产物：`BiuViewer-v0.1.0-macOS-universal.zip`（本地构建为 x86_64；CI 为 universal）
- Windows 记录时间：2026-11-09
- Windows 产物：`BiuViewer-v0.1.2-windows-x64.zip`
- Windows 环境：Windows 10 专业版 22H2（19045）、1920×1080、100% 缩放、Xeon E5-2678 v3 / 32 GB

> 下面的 17 项结果表是 v0.1.x 的历史记录，不代表 0.2.0 已通过三平台验收。0.2.0 新增了 HEIC/AVIF、GIF 暂停/逐帧、配置持久化等能力，其当前状态见下一节。

## 0.2.0 验收状态（2026-11-21，候选）

0.2.0 尚未打 tag；当前验收对象是本地 release 构建，不是正式发布包。

| 平台 | 0.2.0 状态 | 说明 |
| :--- | :--- | :--- |
| macOS | ✅ 实机 17/17 通过 | 2026-11-21 在 macOS 26.5.1 实机跑完 17 项并补测性能（翻页中位 10.2 ms、GIF 12.27 fps）；实机发现的「窗口尺寸每次重启 +32 pt」缺陷已修复，并为旧配置加了一次性迁移，详见「macOS 实机验收（0.2.0）」 |
| Windows | ⚠️ 冒烟通过，17 项未重跑 | `--version` 输出 `biuviewer 0.2.0`；4032×3024 HEIC 与 256×256 AVIF 均正常显示；110 个测试通过；未在 0.2.0 发布 zip 上重跑完整 17 项 |
| Linux | ☐ 未实机 | 0.2.0 未在真实桌面运行；v0.1.3 只通过第 17 项，第 1–16 项仍待测 |

### 0.2.0 新增能力

| 能力 | Windows | macOS | Linux | 说明 |
| :--- | :---: | :---: | :---: | :--- |
| HEIC/HEIF 打开 | ✅ 冒烟 | ✅ 实机 | ☐ | macOS：256×256 HEIC 正常显示，缩略图有真实像素；Windows：真实 4032×3024 样张正常显示，内嵌 `thmb` 缩略图 128×96 |
| AVIF 打开 | ✅ 冒烟 | ✅ 实机 | ☐ | macOS：256×256 AVIF 正常显示；Windows：256×256 AVIF 正常显示 |
| GIF 暂停/继续/逐帧 | ☐ | ✅ 实机 | ☐ | macOS：40 帧 GIF 空格暂停（两帧像素完全一致）、`,`/`.` 逐帧 22↔23/40；单帧 GIF 静止不闪；单测覆盖 |
| 配置持久化 | ☐ | ✅ 实机 | ☐ | macOS：`scale_mode`、`show_thumbnails`、窗口位置与尺寸均能恢复；实机验收发现的「每次重启 +32 pt」已修复并复验（见下） |
| 空文件夹提示时机 | ✅ 启动冒烟 | ✅ 实机 | ☐ | macOS：启动空目录显示欢迎页、无横条；手动选择同一目录后居中提示「该文件夹中没有可显示的图片」+ 路径 |

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
| 17 | 产物下载后按 README 可运行 | ✅ | ✅ | ✅ | 解压 Release 中的 `BiuViewer-v0.1.0-macOS-universal.zip` 后 `BiuViewer.app/Contents/MacOS/BiuViewer --version` 输出 `biuviewer 0.1.0`；Windows v0.1.2：zip SHA256 与 `checksums.txt` 一致，`--version` 输出 `biuviewer 0.1.2`，GUI 子系统无控制台；Linux v0.1.2 因缺少 `GLIBC_2.43` 启动失败（见「Linux 冒烟」），v0.1.3 复测通过：tar.gz SHA256 一致，Ubuntu 22.04 上 `--version` 输出 `biuviewer 0.1.3`，WSLg 下 GUI 持续运行 45 s（见「Linux 复测」） |

图例：✅ 通过 / ❌ 不通过 / ☐ 未执行。

## 错误提示实测

| 场景 | 构造方式 | 实际表现 |
| :--- | :--- | :--- |
| 文件损坏 | 截断的 PNG | 错误卡片「文件已损坏，无法读取：unexpected end of file」，显示文件名 |
| 不支持的格式 | `notes.txt` | 提示条「不支持的格式：…/notes.txt」，工具栏置灰，状态栏「未打开文件」 |
| 无权限 | `chmod 000` 的 PNG | 错误卡片「没有权限访问：Permission denied (os error 13)」，缩略图显示 `!` |

Windows `v0.1.2` 上同样三种场景：截断 PNG 与 `icacls` 拒绝访问的文件显示错误卡片，`notes.txt` 启动或拖入时显示提示条，工具栏按预期置灰；本轮自动化按像素判定卡片与提示条的存在，未逐字核对提示文案。

## Linux 冒烟（v0.1.2，2026-11-09）

在 WSL2 Ubuntu 22.04（glibc 2.35，x86_64）上按 README 流程对 Release 产物做冒烟，第 17 项即失败。

- **完整性**：`BiuViewer-v0.1.2-linux-x86_64.tar.gz` 的 SHA256 `b98d6b19fec321d9c24666c6cfcad0b5fa45326a963af9bc29dc9bd7b665d8e2` 与 `checksums.txt` 一致；包结构、`bin/biuviewer` 的 755 权限、图标与 `.desktop` 均正常。
- **失败现象**：安装 README 列出的依赖后，`bin/biuviewer --version` 在动态链接阶段退出：

  ```
  libm.so.6: version `GLIBC_2.43' not found (required by .../bin/biuviewer)
  libc.so.6: weak version `GLIBC_2.39' not found
  ```

  硬依赖来自基础数学函数 `acosf` / `asinf` / `coshf` / `sinhf` / `acoshf` / `atan2f@GLIBC_2.43`。`libxkbcommon-x11.so.0` 也缺失，但它在 tarball README 的依赖列表里，安装后仍被 glibc 挡住。
- **根因**：`release.yml` 的 Linux 构建使用 `runs-on: ubuntu-latest`。v0.1.2 构建日志显示运行器镜像为 `ubuntu26/20261101.186`（Ubuntu 26.04，glibc 2.43），产物因此把 glibc 下限抬到 2.43；而文档承诺的是 glibc 2.31+。Ubuntu 22.04（2.35）、24.04（2.39）、Fedora 43（2.42）都无法启动。GUI 未执行——加载器在进入 `main` 前就退出。
- **修复**：`build-linux` 固定为 `ubuntu-22.04`，并新增发布前检查：产物最高 glibc 需求不得高于 2.35，否则构建失败，同时在该环境执行 `--version`；`docs/INSTALL.md` 的要求同步改为 glibc 2.35+。修复后已重新发版并复测，见下一节；本表 Linux 列其余 16 项仍为待测。

## Linux 复测（v0.1.3，2026-11-10）

在同一 WSL2 Ubuntu 22.04（glibc 2.35）环境复测修复后的 Release 产物。

- **完整性**：`BiuViewer-v0.1.3-linux-x86_64.tar.gz` 的 SHA256 `ba84b12874dbb066a0191db964c2acc015e43f46b2e3db83a3323ad5da9b8bf8` 与 `checksums.txt` 一致。
- **glibc 下限**：`readelf -V` 统计的最高版本需求为 `GLIBC_2.35`（不再出现 2.43）；按 README 安装 `libxkbcommon-x11-0`、`libvulkan1` 后 `ldd` 无缺失项；`bin/biuviewer --version` 输出 `biuviewer 0.1.3`。
- **GUI（WSLg，Wayland）**：启动日志显示 wgpu 枚举到 `D3D12 (AMD Radeon RX 580 2048SP)`（GL）与 `llvmpipe`（Vulkan）两个适配器，选中前者并创建窗口；加载 `uhd-4k.png` 后进程持续运行 45 s，仅因 `timeout` 结束，无崩溃或错误输出。窗口未取得像素级截图——Windows 会话处于锁屏。
- **GUI（Xvfb，X11）**：在 `Xvfb :99` 下创建了标题为 `BiuViewer` 的 960×640 窗口，进程存活，适配器为 `llvmpipe`（Vulkan）；`scrot` 截取根窗口为全黑（无合成器、窗口为 32 位 ARGB），因此该截图不作为渲染通过的证据。
- **边界**：第 17 项在 Linux 上通过（产物完整、按 README 可启动）。WSLg 的 GPU 经 D3D12 转译且系统无中文字体，不能替代真实 Linux 桌面；第 1–16 项（对话框/拖拽、格式、EXIF、GIF、内存、性能、X11 与 Wayland 差异）仍需实机执行。

## WSL Linux 预验收（0.2.0，2026-11-21）

这不是实机验收，只用于在本地尽早发现 Linux 路径问题。

- **环境**：WSL2 Ubuntu 22.04.5 LTS（glibc 2.35），内核 `6.18.58.1-microsoft-standard-WSL2`；Xvfb `:99` + openbox + xcompmgr，GPU 为 `llvmpipe`（Vulkan）与 D3D12 转译的 GL 适配器。
- **产物**：本地 `cargo build --release --target x86_64-unknown-linux-gnu -p biuviewer`，再执行 `scripts/bundle-linux.sh`；安装到 `/tmp/biu-prefix`，`--version` 输出 `biuviewer 0.2.0`，最高 glibc 需求 `GLIBC_2.35`，`ldd` 无缺失项。
- **方法**：X11 下用 `xdotool` 注入按键/鼠标，用 `import -window` 截取窗口做像素/视觉判定；WSLg 仅做 Wayland 启动与视觉冒烟。Xvfb 的窗口是 32 位 ARGB，必须先启动 `xcompmgr`，否则截图全黑；Xvfb 默认没有 XInput 指针，需用 `xinput --create-master` 创建虚拟指针后鼠标才可用。
- **结果**：

| # | 结果 | 证据 / 说明 |
| :--- | :---: | :--- |
| 1 | ❌ 环境不可用 | rfd 走 XDG portal 报 `Failed to connect to address unix:path=/run/user/1000/bus`，无 zenity 回退；属 WSL 无桌面会话，不能据此判定应用缺陷，需真实桌面重测 |
| 2 | ☐ 未执行 | WSL 无 Linux 文件管理器，WSLg 也不桥接 Windows 资源管理器拖放 |
| 3 | ✅ | 命令行打开 `target/wsl/assets` 与单图均正常；`--version` 为 0.2.0 |
| 4 | ✅ | 点击第 3 个缩略图切到 `exif1.jpg`（3/14），高亮跟随 |
| 5 | ✅ | `→`/`←` 逐张切换，`Home`/`End` 到首尾 |
| 6 | ✅ | 320×240 为 100%；4000×3000 `large.jpg` 适应为 16%，小图不放大 |
| 7 | ✅ | `+` 连按 10 次到 148%，标签变「自定义」；`-` 可回退 |
| 8 | ✅ | 148% 下向左上拖拽，画面裁剪位置随鼠标变化 |
| 9 | ⚠️ 部分 | `R` 旋转后 4000×3000 变 3000×4000，连按 4 次复原；翻转未单独截图 |
| 10 | ❌ 未通过（Xvfb） | `F11` 与工具栏「全屏」均未改变 960×640 窗口几何，openbox 下 `_NET_WM_STATE` 无 `FULLSCREEN`；需在真实桌面复核，若复现则为 Linux 发布阻塞 |
| 11 | ✅ | `exif6.jpg` 存储 200×100、Orientation=6，显示为 100×200 竖图 |
| 12 | ✅ | `anim2.gif` 3 帧、5 ms/帧（被钳到 100 ms），截到红/绿帧；空格显示「已暂停」，`.` 步进到蓝帧 |
| 13 | ✅ | PNG/JPEG/GIF/BMP/WebP/TIFF/ICO + HEIC/AVIF 均显示；HEIC/AVIF 为 256×256 |
| 14 | ⚠️ 部分 | `truncated.jpg` 显示错误卡片，但详情仍为英文 `Format error decoding Jpeg: I/O errors Not enough bytes, expected 64 but found 6`；`notes.txt` 显示「不支持的格式」提示条；均不崩溃 |
| 15 | ⚠️ 超预算 | 500 张 320×240 图：扫描后 RSS `208740 KB`，末张 `214576 KB`，距 <200 MB 目标高约 4–7%；含 llvmpipe/D3D12 转译开销 |
| 16 | ⚠️ 部分 | 窗口出现时间 121–249 ms（3 次），仅代表窗口创建；12 MP 翻页、4K 解码、帧率未测 |
| 17 | ✅ | `BiuViewer-v0.2.0-linux-x86_64.tar.gz` 15 MB，`install.sh` 安装成功，`.desktop` MimeType 已含 heic/heif/avif |

- **结论**：10 项通过、4 项部分通过、2 项未通过/不可用、1 项未执行。Linux 的格式、EXIF、GIF、缩放/平移、缩略图、CLI 与包完整性路径基本可用；**全屏在 Xvfb 下未通过**，第 1、2 项仍需真实桌面，内存略超目标。以上不能替代 `docs/acceptance.md` 结果表中的 Linux 实机结论。

## macOS 实机验收（0.2.0，2026-11-21）

- **环境**：macOS 26.5.1（25F80），Intel Core i7-9750H，1792×1120 逻辑分辨率（@2x），真实登录桌面（Finder/Dock/WindowServer 正常）。
- **产物**：本地 `cargo build --release -p biuviewer` 后由 `scripts/bundle-macos.sh` 生成的 `BiuViewer-v0.2.0-macOS-universal.zip`，解压到 `/tmp` 后运行；`Contents/MacOS/BiuViewer --version` 输出 `biuviewer 0.2.0`，`plutil -lint Info.plist` 通过。本机 `lipo -info` 为 `x86_64`（本地构建未做 universal 合并，CI 产物另论）。17 项功能结果在修复前产物（7.2 MB，SHA256 `fb5cde79fd1e17e8820fbef9169e8489031a647f95e8fc5a08f7b0379710d77f`）上取得；修复窗口尺寸缺陷后重新打包（7,545,605 B，SHA256 `77df2e3ee01c89be694ecb601a0a1cf56695f2f0239dc45527860a1284b6a941`）复验配置持久化；性能补测与 legacy 迁移在加入耗时插桩后的最终产物（SHA256 `dcfb3e2b1fa8438af2808906444b3574e3e5736c90b0f56486913d394333f8ed`）上完成。
- **方法**：`CGEvent` 注入键鼠（系统已授权辅助功能），窗口截图后用 Vision OCR + 像素统计判定；启动时间用进程内 `CGWindowListCopyWindowInfo` 轮询，避免外部轮询开销；内存用 `ps -o rss=`。素材由 `make_assets` 生成，HEIC/AVIF 使用 `crates/biuviewer-decoder/tests/fixtures/` 下样张。帧级耗时用进程内插桩测量：设置 `BIUVIEWER_PERF_LOG=1` 时，`crates/biuviewer-ui/src/viewer_app.rs` 在每次渲染与翻页输入处向 stderr 打印 `frame`、`image-ready`、`input-to-frame` 时间戳（仅日志，不改行为）。
- **结果**：

| # | 结果 | 证据 / 说明 |
| :--- | :---: | :--- |
| 1 | ✅ | 工具栏「文件夹」打开 NSOpenPanel「打开文件夹」；`Cmd+Shift+G` 填入 `/tmp/biu-bench/formats` 后确认，窗口载入该目录 1/7 与对应缩略图 |
| 2 | ✅ | 从 Finder 拖入 `1000-photos` 目录后切到 1/1000；再拖入单图 `photo-0001.png` 后切到 2/1000；拖动中显示「松开以打开」遮罩与蓝色边框 |
| 3 | ✅ | `BiuViewer.app/Contents/MacOS/BiuViewer /tmp/biu-bench/1000-photos` → 1/1000；命令行传单图时打开其所在目录并定位该图 |
| 4 | ✅ | 点击第 4 个缩略图 → 4/1000 `photo-0003.png`，高亮跟随 |
| 5 | ✅ | `→`/`←` 逐张切换，`Home`/`End` 到首尾，首尾继续按时不越界 |
| 6 | ✅ | 3840×2160 适应为 22%（视口高度撑满）；400×300 保持 100%（小图不放大） |
| 7 | ✅ | 320×240 上 `+`×3：100% → 195% 且标签变「自定义」；`-`、`0`、工具栏「重置」均正确回到适应 |
| 8 | ✅ | 4K 放大到 78% 后拖拽，62.27% 像素变化、四角颜色随裁剪位置移动 |
| 9 | ✅ | `R` 后 320×240 → 240×320，连按 4 次复原（像素差 0.72，属噪声）；`H`/`V` 镜像四角顺序正确；「重置」复位 |
| 10 | ✅ | `F` 进入 1792×1120 全屏（工具栏/状态栏/缩略图隐藏），`Esc` 恢复 960×672 且保留 477% 缩放 |
| 11 | ✅ | 8 张 EXIF 方向样本显示结果一致（两两像素差 ≤0.34%，JPEG 噪声），均 240×160，红左上/绿右上/蓝左下/黄右下 + 顶部黑条 |
| 12 | ✅ | 40 帧 GIF 连续截图帧内容不同（约 35% 像素变化），状态栏显示「第 N/40 帧」；空格暂停（间隔 1 s 两帧完全一致）、`,`/`.` 在 22↔23 帧间步进；单帧 GIF 保持静止 |
| 13 | ✅ | `formats/` 下 PNG/JPEG/GIF/BMP/WebP/TIFF 均 320×240，ICO 128×128；HEIC/AVIF 256×256，全部正常 |
| 14 | ✅ | 截断 PNG 显示「文件已损坏，无法读取：unexpected end of file」卡片；`chmod 000` PNG 显示「没有权限访问：Permission denied (os error 13)」；`.txt` 启动显示「不支持的格式」提示条、工具栏置灰，均不崩溃 |
| 15 | ✅ | 1000 张翻页：打开后空闲 RSS 77.8 MB（< 200 MB），100/200/300/576/1000 张时为 141/211/281/474/642 MB，随 512 MB 缓存预算收敛不再无限增长；回到第 1 张 RSS 未回落但也不再增长 |
| 16 | ✅ | 1000 张启动到窗口 1.15–1.49 s、再过 0.2 s 出首图（< 3 s）；10 MB PNG 解码 20.7 ms（< 1 s）；翻页（缓存命中，n=29）中位 10.2 ms、p90 14.9 ms、最大 15.8 ms，10 MB PNG 冷解码翻页 31.5–40.0 ms（< 100 ms）；GIF 播放 12.27 fps（80 ms/帧，目标 12.5 fps，无掉帧）；空闲 RSS 77.8 MB |
| 17 | ✅ | zip 解压后 `--version` 输出 `biuviewer 0.2.0`，`Info.plist` 通过 `plutil -lint`；本机产物为 x86_64，universal 合并未在本地验证 |

- **0.2.0 新能力**：HEIC/AVIF 打开与缩略图 ✅；GIF 暂停/继续/逐帧 ✅；空文件夹时机 ✅（启动空目录显示欢迎页、无提示条，手动选择空目录后居中提示）；配置持久化 ✅（修复见下）。
- **发现并修复的缺陷：旧配置与窗口尺寸每次重启增长 32 pt**。复现：`config.txt` 写 `window_width=960`、`window_height=640` → 启动后窗口 960×672（含标题栏）→ 关闭时写回 672 → 再启动为 960×704 → 关闭写回 704 → 再启动 960×736，x/y 始终为 80/60。根因：`crates/biuviewer-ui/src/viewer_app.rs:1446-1448` 在 `on_window_should_close` 中保存 `window.window_bounds().get_bounds()`，GPUI 的 macOS 后端 `bounds()` 返回 `NSWindow.frame()`（含标题栏）；而 `crates/biuviewer/src/app.rs:64-67` 把该值作为 `WindowBounds::Windowed` 交回 GPUI，后者用 `initWithContentRect:` 按**内容区域**创建窗口，于是每轮多出一个标题栏高度。修复：保存时位置仍取 frame 原点，尺寸改用 `window.viewport_size()`（内容区域）。复验（新产物）：`height=640` 启动为 960×672 → 关闭后配置仍为 640 → 连续两次重启窗口均为 960×672；`scale_mode=actual`、`show_thumbnails=false` 同时恢复正确。旧配置兼容：新增 `CONFIG_VERSION = 2`（宽高记为内容尺寸，v1 记为含标题栏的 frame）与 `window_size_is_frame` 标记；启动时若读到无版本号的旧配置，按「frame − 实际内缩」一次性缩减内容区域。复验：写入旧格式（无 `config_version`、`height=672`）→ 窗口 960×672（不再变 704）→ 关闭后配置写为 `config_version=2`、`height=640` → 再启动仍为 960×672。
- **边界**：键鼠与拖拽为合成事件，虽经 `NSDragging` 完整链路验证，仍不等于真人操作；滚动缩放/触控板手势未覆盖；仅在 Intel 机器上运行，Apple Silicon 未测。

## 未覆盖项与原因

- **对话框与拖拽（Linux）**：仍需人工交互。macOS 已于 2026-11-21 实机覆盖：原生 NSOpenPanel 经 `Cmd+Shift+G` 填路径，拖拽经 `CGEvent` 合成 `NSDraggingSession` 完成（含「松开以打开」遮罩验证）；Windows 由原生 `ole32!DoDragDrop` 与 UI Automation 覆盖。Linux 无文件管理器与桌面会话，暂不可测。
- **滚轮缩放与触控板手势**：合成滚轮事件（`CGEvent`）不被 GPUI 接收，需真实鼠标/触控板验证。命令行脚本已确认 `on_scroll` 参数换算逻辑（单测覆盖视口缩放数学）。
- **帧率**：用应用进程内帧日志测得 GIF 播放 12.27 fps（80 ms/帧，目标 12.5，无掉帧）；未用外部工具采样显示器合成器帧率，也未覆盖高刷新率屏幕。

## 已知限制

- **Windows 抓手光标**：GPUI 的 Windows 后端（`gpui-pre-windows` 的 `load_cursor`）只映射了 `IBeam`、`Crosshair`、`PointingHand`、`Resize*`、`OperationNotAllowed`，`OpenHand` / `ClosedHand` 会回退为箭头。因此在 Windows 上拖拽平移时光标不变，功能本身正常；macOS（`NSCursor`）与 Linux（`grab` / `grabbing`）显示正常。

## 发布产物校验

对 `https://github.com/tagecode/biu-viewer/releases/tag/v0.1.0` 的三个产物做了静态校验（2026-11-05）：

| 产物 | 校验和 | 架构 / 内容 |
| :--- | :--- | :--- |
| `BiuViewer-v0.1.0-macOS-universal.zip` | 通过 | `lipo -info` 同时包含 `x86_64` 与 `arm64`；`Info.plist` 通过 `plutil -lint` |
| `BiuViewer-v0.1.0-windows-x64.zip` | 通过 | `PE32+ executable (GUI) x86-64`，含 `biuviewer.exe`、`icon.ico`、`README.txt`；GUI 子系统表示不会弹控制台 |
| `BiuViewer-v0.1.0-linux-x86_64.tar.gz` | 通过 | `ELF 64-bit LSB pie executable, x86-64, stripped`；含 `bin/biuviewer`（保留执行位）、`install.sh`、`.desktop`、图标、README |

Linux 产物已在 `v0.1.3` 上完成冒烟（见「Linux 复测」），完整验收仍待实机；Windows 已在 `v0.1.2` 上完成运行验收（见上文结果表）。

`v0.1.1` 在 `v0.1.0` 基础上仅包含一项修复：缩略图条在翻页时跟随当前图片（GPUI `ScrollHandle` 的偏移方向取反），产物内容、包结构与上述校验结论一致。三个新产物的 `shasum -a 256 -c checksums.txt` 全部通过；macOS 产物 `--version` 输出 `biuviewer 0.1.1`，`lipo -info` 显示 `x86_64 arm64`。

`v0.1.2` 在此基础上修复「重置」未恢复适应窗口的问题（T12）。Windows 产物 `BiuViewer-v0.1.2-windows-x64.zip` 的 SHA256 为 `c33f9994b889d3620072c30bee0b5c3909d2955baa201e33d5234f866b1b9460`，与 `checksums.txt` 一致；解压后的 `biuviewer.exe` 为 PE32+ GUI 子系统（启动不弹控制台），SHA256 `a309ffffbc5c5428d68ae5dc0093a9d4eefc809c198d2c9422671e1a21fccc4a`，`--version` 输出 `biuviewer 0.1.2`。Linux 产物 `BiuViewer-v0.1.2-linux-x86_64.tar.gz` 的 SHA256 为 `b98d6b19fec321d9c24666c6cfcad0b5fa45326a963af9bc29dc9bd7b665d8e2`，与 `checksums.txt` 一致，但该产物因 glibc 依赖问题无法在目标发行版上运行，见「Linux 冒烟」。

`v0.1.3` 修复 Linux 构建环境：`build-linux` 固定为 Ubuntu 22.04，并新增发布前 glibc 下限检查与 `--version` 冒烟。Linux 产物 `BiuViewer-v0.1.3-linux-x86_64.tar.gz` 的 SHA256 为 `ba84b12874dbb066a0191db964c2acc015e43f46b2e3db83a3323ad5da9b8bf8`，与 `checksums.txt` 一致；`readelf -V` 显示最高需求 `GLIBC_2.35`，在 Ubuntu 22.04 上 `--version` 输出 `biuviewer 0.1.3`（详见「Linux 复测」）。

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

1. 从 `https://github.com/tagecode/biu-viewer/releases/tag/v0.1.3` 下载对应产物（`checksums.txt` 可校验完整性）。
2. 按 `docs/INSTALL.md` 安装并首次启动。
3. 准备素材：在有 Rust 工具链的机器上执行 `make_assets`，把产出的整个目录拷过去；或把 macOS 上已生成的 `/tmp/biu-bench` 直接复制。`formats/`、`exif/` 两个子目录用于第 11、13 项。
4. 逐项执行上表 ☐ 项，把结果补进本节对应的列。

### 平台特有注意事项

- **Windows**：`bench-manual.sh` 的内存采样段用 `ps -o rss=`，Git Bash 的 `ps` 不支持该字段，需改用 `Get-Process -Id <pid> | % { $_.WorkingSet64 }`；另外检查 SmartScreen 提示、125%/150% 缩放下工具栏与缩略图布局、中文与含空格路径。以上三项平台特有检查已于 2026-11-05 在 `v0.1.1` 上验证通过。
  2026-11-09 的 `v0.1.2` 复测中，17 项均由自动化脚本在 Release 产物上执行：以截图像素判定图片区域与四角颜色、`keybd_event`/`mouse_event` 注入按键与拖拽、原生 OLE `DoDragDrop` 拖放、UI Automation 操作文件对话框，内存用 `WorkingSet64` 采样。工作集不包含 GPU 显存，因此 Windows 的内存数字与 macOS 的 RSS 不可直接比较。
- **Linux**：X11 与 Wayland 各跑一轮（Wayland 下拖放行为不同）；确认干净系统按 README 装完依赖后可启动；`./install.sh` 后应用菜单项与图标正常；解压后的 `bin/biuviewer` 保留执行位。

## 结论

macOS 上已验证主流程（打开 → 缩略图 → 翻页 → 缩放/平移/旋转/全屏 → GIF → 7 种格式 → EXIF 8 方向 → 错误提示 → 内存）与打包产物可运行，17 项中 15 项通过；剩余 2 项（对话框、拖拽）是需要人工交互的操作，未在自动化流程中覆盖。

Windows 在 `v0.1.2` 的 Release 产物上 17 项全部通过（帧率一项与 macOS 相同，仍需人工感受）。除缩略图条与翻页由 v0.1.1 先行验证外，其余项目均由自动化脚本复核：对话框经 UI Automation、拖拽经原生 OLE、格式/EXIF/GIF/错误提示/内存与性能经截图像素与进程采样判定；平台特有检查（SmartScreen、125%/150% 缩放、中文与空格路径）沿用 v0.1.1 的结论。Linux 于 2026-11-09 首次冒烟失败：v0.1.2 产物硬依赖 `GLIBC_2.43`，在 Ubuntu 22.04 上无法启动（根因与修复见「Linux 冒烟」）；修复构建环境后，v0.1.3 于 2026-11-10 复测通过第 17 项——Ubuntu 22.04 上产物完整、`--version` 正常、WSLg 下 GUI 启动并稳定运行。Linux 其余 16 项仍需在真实桌面（X11 与 Wayland 各一轮）执行，MVP 的「三平台验收」条件尚未完全成立。

0.2.0 的候选版本状态见文首「0.2.0 验收状态」：macOS 已完成实机验收（17/17，含性能补测），Windows 冒烟通过，Linux 仍未实机。macOS 上发现的「窗口尺寸每次重启增长 32 pt」已定位根因并修复，旧配置的一次性尺寸偏移也通过 `CONFIG_VERSION = 2` + 启动迁移解决，重新打包后均复验通过。0.2.0 剩余工作：完成 Linux 实机验收（X11 与 Wayland 各一轮）。
