# 本地开发

应用由 Vue 前端、Tauri 壳层、统一 Rust EPUB 核心和 Rust `xtask` 维护工具组成。日常开发、测试、构建与打包只需要 Node.js、Rust 及目标平台工具链。

## 前置依赖

| 依赖 | 用途 | 验证命令 |
| --- | --- | --- |
| Node.js（版本见 `.nvmrc`） | 前端和 Tauri CLI | `node --version` |
| npm | 安装依赖和运行脚本 | `npm --version` |
| Rust stable / Cargo | 业务核心、Tauri、xtask | `rustc --version`、`cargo --version` |

### macOS

桌面构建至少需要 Apple Command Line Tools：

```bash
xcode-select --install
```


### Windows

使用原生 PowerShell 或 Windows Terminal：

1. 安装 Visual Studio Build Tools 的 **Desktop development with C++**。
2. 安装 WebView2 Runtime。
3. 通过 Rustup 安装 `stable-msvc`。

### Linux

Debian/Ubuntu 示例：

```bash
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file \
  libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev patchelf
```

其他发行版按 Tauri 2 对应平台前置依赖安装。

## 安装依赖

```bash
npm ci
npm --prefix frontend ci
```

## 启动与调试

完整桌面开发环境：

```bash
npm run tauri:dev
```

启动顺序是：校验内置 ONNX OCR 模型、启动 Vite、启动 Tauri。桌面任务路径一致：

```text
Vue -> Tauri IPC -> spawn_blocking -> in-process EngineRuntime -> rust_backend
```

仅调试前端时：

```bash
npm run dev
```

此模式没有 Tauri Runtime，不能执行 EPUB 任务。

## 验证

```bash
# 格式
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo fmt --manifest-path xtask/Cargo.toml -- --check

# 单元和集成测试
cargo test --locked --manifest-path src-tauri/Cargo.toml
cargo test --locked --manifest-path xtask/Cargo.toml

# 静态检查
cargo clippy --locked --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo clippy --locked --manifest-path xtask/Cargo.toml --all-targets -- -D warnings

# wire contract 与前端
npm run protocol:check
npm run build

# 使用真实 Rust ONNX Runtime session 校验模型输入、输出和字典维度
npm run build:verify-ocr-model
```

`src-tauri/tests/core_regression.rs` 使用运行时生成的稳定 EPUB fixture 覆盖输出后缀、跳过行为、加密/解密往返、简繁转换和任务事件/结果。

## 桌面构建

```bash
npm run build:bundle-assets
npm run tauri:build
```

`build:bundle-assets` 构建前端并验证 OCR 模型。安装包携带前端、进程内 Rust 核心、OCR 模型和 OpenCC 词典。

## Cargo 排查

若 Tauri 提示找不到 Cargo，在同一终端确认：

```bash
cargo --version
```

使用 Rustup 时重新加载其环境，或重新打开终端/IDE。桌面打包失败时需确认对应平台的 Tauri 系统依赖已安装；宿主测试不能代替目标系统上的启动和任务回归。
