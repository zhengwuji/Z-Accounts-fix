# Z-Accounts 源码

Z-Accounts 1.8.3（ZCode / BigModel / z.ai 账号工作台，Tauri 2 桌面应用）的完整可编译源码。

[![构建状态](https://github.com/zhengwuji/Z-Accounts-fix/actions/workflows/release.yml/badge.svg)](https://github.com/zhengwuji/Z-Accounts-fix/actions/workflows/release.yml)

## 最新更新内容

> 每次推送代码（README.md 除外）都会由 GitHub Actions 自动编译，并发布到
> [Releases](https://github.com/zhengwuji/Z-Accounts-fix/releases)，更新说明随 Release 附带。

**2026-09-29（本次）**

1. **修复 ZCode 安装路径识别失败**（影响多处功能）：当 ZCode 装在非系统盘（如 F 盘）而
   `%LOCALAPPDATA%`/`%USERPROFILE%` 仍指向 C 盘时，工具找不到 ZCode.exe，导致
   「用 ZCode 添加账号」弹窗的按钮错误地显示为「设置」、主界面「启动 ZCode」按钮被禁用、
   切换账号后也无法拉起客户端。现在按顺序尝试：设置路径 → 默认安装目录（LOCALAPPDATA 与
   用户目录）→ 注册表卸载项（解析 InstallLocation / DisplayIcon 推导安装目录）→
   正在运行的 ZCode 进程映像路径（原生 API，无控制台弹窗）。
2. **切换账号体验优化**：点击「切换」后，**成功即自动打开 ZCode 客户端**（不再受"切换后启动"
   设置影响；热切换时客户端本就开着，不会重复拉起）。切换结果有**明确提示**：
   成功 → 绿色提示「已切换到 xxx」+「已打开 ZCode 客户端」；
   失败（如 ZCode 路径无效、启动失败）→ 红色错误提示及具体原因；
   ZCode 正在运行且未开热切换时的确认弹窗文案也改为如实描述"先关闭、切换后自动重新打开"。
3. **修复托盘图标重复**：此前托盘区会出现两个图标（一个仅显示 "Z-Accounts"，一个带运行状态）。
   根因是 `tauri.conf.json` 的 `trayIcon` 配置托盘与代码托盘并存，已删除配置托盘，只保留
   功能完整的代码托盘，并为托盘刷新逻辑加了并发互斥保护。
4. **修复额度进度条**：进度条现在按各模型**剩余额度百分比**变长短（与原版一致的 4px 细条、
   渐变配色，用量 ≥70% 转蓝、≥90% 转红）；**没有额度数据的条目不再显示进度条**，仅显示 `--`。
   根因：Tauri 会为内联样式注入 CSP nonce 导致所有 `style="width:…"` 失效（进度条全部满格），
   以及两个样式表加载顺序装反（细条样式被粗条样式覆盖）。
5. **修复额度不显示**：兼容 ZCode 桌面端 `enc:v1` 凭据加密，额度查询走
   `zcode.z.ai/api/v1/zcode-plan/billing/balance`（带 ZCode 客户端源标头），与原版行为一致。
6. **修复 cmd 黑色控制台窗口不断弹出**：所有后台子进程调用（tasklist/taskkill/reg 等）已加
   `CREATE_NO_WINDOW` 标志。
7. **修复「设置」窗口打开后白屏无响应**（验证码窗口同理）：动态创建 webview 窗口时，wry 在
   tao 事件循环回调内用消息泵等待 WebView2 完成回调，事件循环被重入导致新窗口的 webview
   永远停在 about:blank（白屏）、调用方永不返回。修复：设置 / 验证码窗口改为**启动时预创建
   并隐藏**（与主窗口同一创建路径），打开时仅显示并刷新状态，关闭时隐藏而非销毁；
   同时统一了各窗口的 WebView2 浏览器参数（参数不一致会导致同数据目录下环境创建失败）。

## 使用说明

### 获取程序

- **下载现成版本（推荐）**：到 [Releases](https://github.com/zhengwuji/Z-Accounts-fix/releases)
  页面下载最新版附件中的 `Z-Accounts.exe`——绿色单文件、无需安装，双击即用
  （Windows 10/11 自带 WebView2 运行时；若系统缺失，程序启动时会引导安装）。
- **自行构建**：见下方 [构建](#构建) 一节。

### 基本用法

1. **首次使用**：启动后主界面点击 **「保存当前登录」**，把当前 ZCode 的登录状态存入账号库；
   或点击 **「添加账号」** 走 OAuth 登录（bigmodel / z.ai）新增账号。
2. **切换账号**：点击账号卡片右下角的 **「切换」** 按钮（或双击卡片）。切换**成功后会自动打开
   ZCode 客户端**并弹出绿色成功提示；失败会弹出红色错误提示说明原因。默认为重启式切换
   （自动关闭并重新拉起 ZCode）；也可在设置中启用热切换（ZCode 运行中直接替换凭据，带并发
   回写校验）。切换前会自动备份当前未保存的登录（Backup-时间戳）。
3. **查看额度**：仪表盘显示各账号 / 各模型的剩余额度百分比与进度条、当日 Token 估算、
   7 天用量趋势、最近模型请求状态（限流 / 网关拦截）。
4. **托盘**：左键点击托盘图标显示主窗口；右键菜单可「保存当前登录 / 启动 ZCode / 关闭 ZCode /
   退出」。关闭主窗口默认最小化到托盘（可在设置中更改）。
5. **套餐领取**：支持 preview / claim（阿里云验证码：无感 + 人工）、自动领取（10 分钟轮询）、
   全部领取。
6. **自动切号**：设置中开启后，每 90 秒检查一次，连续两次确认额度耗尽即自动切换并重启 ZCode。
7. **导入 / 导出**：账号可导出为 `.zsb` 口令加密捆绑包，在任意机器导入。
8. **设置**：中/英双语、深/浅主题、开机自启、ZCode 路径、代理（仅登录窗口走代理）、
   隐私模式（隐藏邮箱）等。

### 命令行（CLI）

`Z-Accounts.exe <子命令>`，支持：
`state|list|capture|rename|delete|update|switch|quota|claim-preview|kill|export|export-all|import|behavior|setpath|launch`。
密码通过 `ZSW_PASSWORD` 环境变量或 `--password` 参数传入。示例：

```bat
Z-Accounts.exe list
Z-Accounts.exe switch --account 2
Z-Accounts.exe quota
```

## 构建

前置要求：Node.js ≥ 18、npm、Rust stable-msvc（rustup）、MSVC BuildTools + Windows SDK。

```bat
cd 源码
npm install
npm run build          &rem 生成 dist/（前端产物）
cd src-tauri
cargo build --release  &rem 产出 target/release/Z-Accounts.exe（绿色单文件）
```

或直接运行根目录的 `build.bat`。

### 自动构建（GitHub Actions）

仓库已配置 [`.github/workflows/release.yml`](.github/workflows/release.yml)：
推送到 `main` 的任何代码变更会自动在 Windows 环境完成前端 + 后端编译，
并创建带版本号 tag 的 Release（附件为编译好的 `Z-Accounts.exe`，说明为中文更新内容）。
**修改 `README.md` 不会触发构建。** 也可在 Actions 页面手动触发（workflow_dispatch）。

调试运行：`npm run dev`（Vite 于 127.0.0.1:5173）+ `cargo run`（tauri.conf 已配置 devUrl）。

## 目录结构

```
源码/
├─ build.bat                 一键构建脚本
├─ package.json / vite.config.js   前端构建（Vite，3 个 HTML 入口）
├─ .github/workflows/        CI：推送自动编译并发布 Release
├─ src/                      前端源码（从原二进制提取还原）
│  ├─ index.html / settings.html / captcha.html
│  ├─ main.js                主界面（账号库 + 仪表盘渲染与全部交互逻辑）
│  ├─ i18n.js                中/英文案表 + Tauri API 封装
│  ├─ ui.js                  图标库 / toast / 确认弹窗 / 加密切换弹窗 / 提供方选择
│  ├─ settings.js            设置窗口
│  ├─ captcha.js             阿里云验证码窗口（无感验证 + 人工验证）
│  ├─ styles/                手写设计系统（深色琥珀主题）
│  └─ public/brand-icon.png
└─ src-tauri/
   ├─ tauri.conf.json        标识 com.zaccounts.app、CSP、主窗口配置（与原版一致）
   ├─ capabilities/default.json
   ├─ icons/                 由 brand-icon.png 生成
   └─ src/
      ├─ main.rs / lib.rs    入口、插件装配、托盘、后台线程（自动切号看门狗）
      ├─ commands.rs         全部 36 个 IPC 命令（与前端 invoke 一一对应）
      ├─ store.rs            账号库 accounts.json + settings.json
      ├─ zcode.rs            ZCode 集成：数据目录、凭据读写、设备指纹虚拟化、
      │                      进程管理、热切换校验、模型请求日志解析
      ├─ oauth.rs            登录流程（zcode://oauth/callback 拦截 + code 交换）
      ├─ quota.rs            额度查询（z.ai billing/balance + BigModel 侧）
      ├─ claim.rs            套餐领取（preview/claim/激活上报/业务码映射）
      ├─ crypto.rs           .zsb 加密捆绑包（PBKDF2-HMAC-SHA256 + AES-256-GCM）
      ├─ captcha_window.rs   验证码窗口
      ├─ tray.rs             托盘（状态联动刷新）
      ├─ cli.rs              CLI 子命令
      └─ i18n.rs             后端错误文案（中/英，键名与原版一致）
```

## 功能清单（与原版 1.8.3 对齐）

- 账号库：OAuth 添加（bigmodel / z.ai）、保存当前登录、重命名、删除、搜索、隐私模式（隐藏邮箱）
- 切换：重启式切换（自动关闭/拉起 ZCode）、热切换（运行中直接替换，带并发回写校验，3 次重试）
- 切换前自动备份未保存登录（Backup-时间戳）；切换时同步 config.json / credentials.json / 设备指纹（deviceMid + ARMS uid）/ providerFamilyDomain 对齐
- 额度仪表盘：账号额度查询、当日 Token 估算、7 天用量趋势（本地记录）、最近模型请求状态（解析 ZCode CLI 日志的 gateway_blocked / rate_limited）
- 套餐领取：preview / claim（阿里云验证码：无感 + 人工）、激活上报、自动领取（10 分钟轮询）、全部领取
- 自动切号：90 秒检查，连续两次确认额度耗尽后切换并重启 ZCode
- 导入 / 导出：`.zsb` 口令加密捆绑包（可在任意机器导入）
- CLI：`Z-Accounts.exe state|list|capture|rename|delete|update|switch|quota|claim-preview|kill|export|export-all|import|behavior|setpath|launch`（密码走 `ZSW_PASSWORD` 环境变量或 `--password`）
- 托盘（状态联动）、关闭到托盘、开机自启、单实例、中/英双语、深/浅主题、代理设置（仅登录窗口走代理）

## 数据位置

- 账号库 / 设置：`%APPDATA%\com.zaccounts.app\`（accounts.json、settings.json、oauth.log）
- ZCode 数据：`%USERPROFILE%\.zcode\v2\`（credentials.json、config.json、setting.json、telemetry-state.json），ARMS 指纹：`%APPDATA%\ZCode\rum-electron-store\`
- 环境变量覆盖：`ZCODE_SWITCH_HOME`（本工具数据目录）、`ZCODE_DATA_BASE_DIR`（ZCode 数据目录，与 ZCode 官方行为一致）

## 重建保真度说明

以下为与原版的已知差异（均为无法从二进制 100% 恢复的实现细节）：

> **额度与凭据说明（已完全打通）**：账号快照中的凭据值使用 ZCode 桌面端同款
> `enc:v1` 加密（`SHA256("zcode-credential-fallback:win32:<主目录>:<用户名>")` 为密钥，
> AES-256-GCM，`enc:v1:iv.tag.ct` URL-safe base64 三段）——本重建已实现该加解密，
> 加密账号可正常查询额度、切换；额度走 `zcode.z.ai/api/v1/zcode-plan/billing/balance`
> （`?app_version=` + ZCode 客户端源标头集，Bearer 为 `zcodejwttoken`），响应含
> `data.plans[]`（套餐与授权）与 `data.balances[]`（逐模型用量）——均已按真实接口实现。

1. **Rust 后端为行为等价重建**，非逐行反编译。IPC 协议（命令名、参数、事件、数据结构字段）与二进制字符串严格一致；内部算法（缓存周期、重试节奏等）按前端代码中的常量对齐（如额度刷新抖动 ±20%、5 分钟周期）。
2. **`.zsb` 格式**：字段名（format/kdf/cipher/salt/nonce/tag/data）与错误文案严格一致，KDF 为 pbkdf2-hmac-sha256（310,000 次）+ AES-256-GCM。与原版互导未实测（原版已下架）。
3. 窗口尺寸（主窗 1240×820 等）为合理近似（数值常量无法从二进制可靠恢复）。
4. GitHub 链接（设置页"问题反馈"）保留原地址 `github.com/Kang-code-sudo/Z-Accounts`（仓库已删除，点击 404 属预期）。

## 修复记录

### 进度条渲染

额度进度条曾出现"全部 100% 满格、且为 17px 粗条"的问题，与原版（4px 细条、按剩余百分比变长短）不符。根因有两个，均已修复：

1. **Tauri CSP nonce 注入屏蔽了所有内联 style**：`index.html` 中存在内联 `<style>`（启动画面）时，
   Tauri 会自动为其注入随机 nonce 并把 `'nonce-…'` 追加到 CSP 的 `style-src`；按 CSP3 规范，
   一旦出现 nonce，`'unsafe-inline'` 即被忽略 → 模板里的 `style="width:53%"` 全部失效，
   进度条填充恒为 100% 宽。修复：`tauri.conf.json` 的 `app.security.dangerousDisableAssetCspModification: true`
   （CSP 本身已含 `'unsafe-inline'`，禁用注入后脚本/样式/内联宽度均正常）。
2. **两个样式表的加载顺序装反**：原版为 `i18n-*.css`（本重建中的 `styles/fonts.css`）在前、
   `ui-*.css`（`styles/ui.css`）在后，后者覆盖前者；重建初期顺序颠倒，导致 19px 深色粗条样式
   覆盖了原版 4px 细条样式。已按原版顺序还原（三个 HTML 入口均已修正）。

另按需求调整：额度百分比未知（无总量/余量数据）的条目不再渲染进度条，仅显示 `--`。

### 托盘图标重复

曾出现**两个托盘图标**：一个 tooltip 为 `Z-Accounts`（无状态），另一个为 `Z-Accounts · <运行状态>`。
根因是托盘来源重复：`tauri.conf.json` 里配置了 `app.trayIcon`（Tauri 框架自动创建一个静态托盘，
tooltip 固定为 `Z-Accounts`），而 `src-tauri/src/tray.rs` 又在 setup 里用代码创建并维护一个动态托盘
（tooltip/菜单随 ZCode 运行状态刷新）。两套并存且互不知晓。

修复：删除 `tauri.conf.json` 的 `trayIcon` 配置块，只保留代码托盘（图标取 `default_window_icon()`，
tooltip、菜单、左键显示主窗口等行为全部由 `tray.rs` 管理）。另为 `tray::rebuild` 加了进程级
`try_lock` 互斥（setup、5 秒轮询线程、菜单回调三方并发时保证"查重→创建"原子；拿不到锁直接跳过，
避免轮询线程经 `run_on_main_thread` 建托盘、主线程又等锁造成的死锁）。验证：启动后跨两个轮询周期，
进程内 `tray_icon_app` 隐藏窗口数稳定为 1。

### 设置 / 验证码窗口打开后白屏

「设置」窗口（以及套餐领取的验证码窗口）曾打开后**整窗空白、无响应**，主窗口不受影响。

根因：这两个窗口此前是**运行时动态创建**的——从 IPC 命令里调用 `WebviewWindowBuilder::build()`。
在 Windows 上，wry 创建 WebView2 时通过 `GetMessage` 消息泵同步等待异步完成回调
（`webview2_com::wait_with_pump`），而动态建窗发生在 tao 事件循环的回调内部——泵消息会让
事件循环重入，破坏 wry 的后续初始化流程：新窗口的原生 HWND 与空白 webview（about:blank）
都创建成功，但**导航到目标页面从未执行**（白屏），且调用方的 promise 永不返回。主窗口在
事件循环启动之前创建，不走这条路径，所以一直正常。

修复（`src-tauri/src/lib.rs`、`commands.rs`、`captcha_window.rs`）：

1. **启动时预创建**：在 setup（事件循环尚未运行，与主窗口同一创建路径）中把 settings 与
   captcha 窗口建好并 `visible(false)` 隐藏；打开命令只做 show / focus，并对设置窗口补发
   `state-changed` 事件强制刷新状态，对验证码窗口执行 `reload` 让验证流程每次从零开始。
2. **关闭即隐藏**：这两个窗口的 `CloseRequested` 改为 hide + prevent_close，窗口永不销毁，
   彻底避免再次动态创建。
3. **统一浏览器参数**：各窗口的 `additional_browser_args` 必须完全一致（原验证码窗口少了
   `--no-proxy-server`），否则同一 WebView2 用户数据目录下第二次环境创建会直接失败。

验证（CDP 实测）：启动后三个页面 target 全部加载（`/`、`settings.html`、`captcha.html`）；
`open_settings` 的 promise 立即返回 OK、窗口可见、设置页 DOM 正常渲染（#app 5535 字符、
splash 隐藏）；关闭后窗口隐藏、再次打开复用同一实例。

## 许可

仅供学习交流与个人本地使用。请仅在你自己的设备与账号上使用；不对账号安全、凭据有效性或服务可用性作任何保证。
