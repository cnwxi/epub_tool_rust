# 构建、打包与发布

## 构建组成

应用由 Vue 前端、Tauri 壳层、统一 Rust EPUB 核心和运行资源组成：

- Windows、macOS、Linux、Android 都将同一 Rust 核心链接进应用进程；
- 桌面携带 `PP-OCRv6_small_rec_onnx` 与 OpenCC 词典；Android 仅携带 OpenCC；
- Protobuf 只用于 Tauri IPC，业务核心使用类型化 `TaskSpec`、`TaskOptions`、`TaskEvent`、`TaskResult`。

## 桌面构建

```bash
npm ci
npm --prefix frontend ci
npm run build:bundle-assets
npm run tauri:build
```

桌面 `beforeBuildCommand` 会执行 `build:bundle-assets`：

1. 构建 Vue 前端；
2. 以真实 Rust ONNX Runtime session 校验 OCR 模型；
3. 由 Tauri 生成包含 Rust 核心的目标平台 bundle；macOS 会先准备官方 ONNX Runtime xcframework 并静态链接其通用切片，因此 Intel 与 Apple Silicon 都不依赖 `ort-sys` 的预编译下载。

发布 workflow 的桌面矩阵：

| 平台 | 架构 | Bundle |
| --- | --- | --- |
| Linux | x64、arm64 | deb、rpm |
| Windows | x64、arm64 | NSIS |
| macOS | x64、arm64 | app、DMG |

当前 macOS 配置使用 ad-hoc identity，Windows 安装包也未配置生产证书。CI 能验证构建和打包；正式代码签名、公证和信誉链需要仓库外凭据。

## macOS ONNX Runtime

macOS 使用官方 ONNX Runtime `1.24.3` xcframework 的 `macos-arm64_x86_64` 切片。`xtask` 会校验归档 SHA-256（`b7eedc45932bac758ffd057cac0feb3f682269e47750b159e4c865145cbf0a8e`），提取 macOS framework，并生成供 `ort-sys` 使用的静态库目录。缓存位于已忽略的 `src-tauri/.desktop-runtime/`，离线时可用 `EPUB_TOOL_ORT_MACOS_ARCHIVE` 指向同一归档。

## 质量门槛

提交前应执行以下检查（发布 workflow 的打包成功不替代这些检查）：

```bash
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo fmt --manifest-path xtask/Cargo.toml -- --check
cargo test --locked --manifest-path src-tauri/Cargo.toml
cargo test --locked --manifest-path xtask/Cargo.toml
cargo clippy --locked --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo clippy --locked --manifest-path xtask/Cargo.toml --all-targets -- -D warnings
npm run protocol:check
npm run build
npm run build:verify-ocr-model
```

安装包还应在目标系统上做启动、任务执行、输出、日志和真实 EPUB 回归。

## 版本与 Release

版本唯一来源是 `src-tauri/Cargo.toml` 的 `package.version`，Vite、Tauri 与 Release workflow 均读取该值。版本采用“年.月.日”形式，同日修订可加 `-1`、`-2` 后缀。

GitHub Release 发布 Windows、macOS 和 Linux 桌面安装包。发布前在 `assets/docs/CHANGELOG.md` 添加对应版本记录。

## Android 与统一发布

```bash
npm run tauri -- android init --ci --skip-targets-install
cargo run --locked --manifest-path xtask/Cargo.toml -- mobile-build android aarch64 --split-per-abi --apk --ci
```

`xtask` 初始化生成工程并同步 launcher 图标，`tauri.android.conf.json` 选择移动资源构建。基础配置包含 OpenCC，各桌面 overlay 增加 OCR 模型。Tauri 的 `macos-private-api` feature 与配置保持一致，仅在 macOS 上生效。

根目录 `.github/workflows/build.yml` 同时构建桌面矩阵和 Android 四 ABI：`aarch64`、`armv7`、`x86_64`、`i686`。发布 job 等待所有平台成功后汇总同一 release 的资产。

Android 签名沿用 `ANDROID_KEYSTORE_BASE64`、`ANDROID_KEYSTORE_PASSWORD`、`ANDROID_KEY_ALIAS`、`ANDROID_KEY_PASSWORD` 仓库 secrets；需在统一应用仓库配置。缺失时沿用临时 CI 签名，不能用于跨版本覆盖升级。APK 位于 `src-tauri/gen/android/app/build/outputs/apk/`；初始化目录和构建缓存均不提交。
