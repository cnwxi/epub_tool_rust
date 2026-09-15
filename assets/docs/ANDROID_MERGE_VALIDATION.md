# Android 功能合并验证

2026-09-14，本地分支 `merge-android-into-desktop`；修改尚未提交或推送。

## 集成范围

- 唯一 Vue 入口、protobuf 协议、任务队列和进程内 Rust 引擎。
- Android 系统 URI 导入/封面暂存（每次独立目录，保留文件名和任务后缀）、统一或逐本封面选择、结果导出、外部打开事件队列接入共享页面。
- `PlatformFiles` 隔离文件操作；Android 禁用字体、目录扫描与路径打开，桌面保留原有功能。
- Android 仅打包 OpenCC；桌面保留 OCR 模型。core 继续固定独立仓库的 `v26.8.25` tag。
- 根发布 workflow 汇总桌面矩阵与 Android 四 ABI APK；未触发云端构建或发布。

## 已验证

- 桌面和移动模式的 Vue/TypeScript 构建通过。
- 桌面 `cargo check`、应用和 xtask 的 fmt/Clippy 通过。
- 应用 Rust 测试 18 项通过，包含跨平台能力、外部打开队列、旧 Android protobuf/JSON 请求和 URI 暂存回归；xtask 的 3 项 Android 参数测试通过。
- `epub_tool_core --no-default-features`：21 项单元测试和 3 项集成回归通过。核心目录 HEAD 为 `a27052b`，与应用锁定的 Git 依赖一致。
- 使用真实 ONNX session 的 OCR 模型校验通过。
- `npm run tauri:build -- --debug --bundles app` 成功生成 macOS 调试 `.app`，`codesign --verify --deep --strict` 通过；这是 ad-hoc 签名，未公证。包内 OpenCC 与 OCR 资源经逐字节对比，与源码资源一致。
- `npm run protocol:check` 通过，生成绑定无漂移。
- Android aarch64 依赖树无 `font`、Stylo、`ort` 或 `ort-sys`。
- 合并配置后，Windows/macOS/Linux 的窗口配置与资源映射保持原有值；Android 使用移动构建钩子且无 OCR 资源。
- workflow YAML、四 ABI 矩阵与发布依赖检查通过；`git diff --check` 通过。

浏览器曾确认共享页面正常启动、移动概览隐藏字体入口。最后补入统一封面按钮后的再次页面加载因自动审批超时未完成；最新代码的桌面和移动前端构建均通过。

## 命令约定

在 `epub_tool_rust` 根目录安装 `npm ci` 和 `npm --prefix frontend ci` 后：

- 桌面开发：`npm run tauri:dev`；生产打包：`npm run tauri:build`。
- Android 初始化：`npm run tauri:android:init`；开发：`npm run tauri:android:dev`，可追加 `-- "设备名" --no-watch`。
- Android 构建：`npm run tauri:android:build -- aarch64 --split-per-abi --apk --ci`；省略 ABI 时构建全部目标。
- `android dev` 的位置参数是设备名，`android build` 的位置参数由 xtask 转换为 `--target ABI`。`--help` 无需 SDK 即可显示。

## 尚未验证

Android 配置显式设置 `versionName` 为 `26.9.15`（与 Cargo.toml 同步）；此前缺少该字段时 Tauri 默认生成 `1.0`。

本机未安装 Android SDK，`tauri android init --ci --skip-targets-install` 明确报错，因此未验证 Android Rust target 编译、Gradle、APK 打包或真机操作。依赖树与浏览器检查不能替代 Android 构建。

在配置 SDK/NDK/JDK/Rust targets 的环境执行：

```bash
npm run tauri -- android init --ci --skip-targets-install
cargo run --locked --manifest-path xtask/Cargo.toml -- mobile-build android aarch64 --split-per-abi --apk --ci
```

真机需覆盖系统文件选择、外部打开、批量任务、统一/逐本封面、简繁转换、取消导出和成功导出。Android 其他 ABI、Windows/Linux 安装包及 macOS 发布版包需由统一 workflow 和对应设备继续验证。长期 Android 签名凭据需要在统一应用仓库配置。
