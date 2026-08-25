use sha2::{Digest, Sha256};
use std::{
    env,
    fs::{self, File},
    io::{self, BufReader, Read},
    path::{Path, PathBuf},
    process::{Command, ExitCode},
};
use zip::ZipArchive;

const ORT_VERSION: &str = "1.24.3";
const MACOS_URL: &str = "https://download.onnxruntime.ai/pod-archive-onnxruntime-c-1.24.3.zip";
const MACOS_SHA256: &str = "b7eedc45932bac758ffd057cac0feb3f682269e47750b159e4c865145cbf0a8e";
const DEFAULT_OCR_MODEL: &str = "PP-OCRv6_small_rec";

fn main() -> ExitCode {
    match run(env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run(arguments: Vec<String>) -> Result<(), String> {
    let Some(command) = arguments.first().map(String::as_str) else {
        return Err(usage());
    };
    match command {
        "verify-ocr-model" => verify_ocr_model(arguments.get(1).map(String::as_str)),
        "desktop-build" => desktop_build(&arguments[1..]),
        "update-homebrew-cask" => update_homebrew_cask(&arguments[1..]),
        _ => Err(usage()),
    }
}

fn usage() -> String {
    "Usage:\n  cargo run --locked --manifest-path xtask/Cargo.toml -- verify-ocr-model [model-name]\n  cargo run --locked --manifest-path xtask/Cargo.toml -- desktop-build [Tauri build options]\n  cargo run --locked --manifest-path xtask/Cargo.toml -- update-homebrew-cask <formula> <version> <arm64-sha256> <x64-sha256> <url>".to_string()
}

fn repo_root() -> Result<PathBuf, String> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| "无法定位仓库根目录".to_string())
}

fn verify_ocr_model(model_name: Option<&str>) -> Result<(), String> {
    let model_name = model_name
        .map(str::to_string)
        .or_else(|| env::var("EPUB_TOOL_OCR_MODEL_NAME").ok())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_OCR_MODEL.to_string());
    if !matches!(
        model_name.as_str(),
        "PP-OCRv6_small_rec" | "PP-OCRv6_medium_rec"
    ) {
        return Err(format!("不支持的 OCR 模型: {model_name}"));
    }
    let model_dir = repo_root()?
        .join("src-tauri/bundle-resources/ocr-models")
        .join(format!("{model_name}_onnx"));
    let mut command = Command::new("cargo");
    command.current_dir(repo_root()?).args([
        "run",
        "--locked",
        "--manifest-path",
        "src-tauri/Cargo.toml",
        "--bin",
        "verify-ocr-model",
        "--",
    ]);
    if cfg!(target_os = "macos") {
        command.env("ORT_LIB_PATH", prepare_macos_ort()?);
    }
    let status = command
        .arg(&model_dir)
        .status()
        .map_err(|error| format!("启动 Rust OCR 模型校验失败: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("Rust OCR 模型校验失败: {status}"))
    }
}

fn desktop_build(arguments: &[String]) -> Result<(), String> {
    let mut command = npm_command();
    command
        .current_dir(repo_root()?)
        .args(["run", "tauri", "--", "build"])
        .args(arguments);
    if cfg!(target_os = "macos") {
        command.env("ORT_LIB_PATH", prepare_macos_ort()?);
    }
    let status = command
        .status()
        .map_err(|error| format!("启动桌面 Tauri 构建失败: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("桌面 Tauri 构建失败: {status}"))
    }
}

fn npm_command() -> Command {
    if cfg!(windows) {
        Command::new("npm.cmd")
    } else {
        Command::new("npm")
    }
}

fn update_homebrew_cask(arguments: &[String]) -> Result<(), String> {
    let [formula, version, arm_sha256, intel_sha256, url] = arguments else {
        return Err(usage());
    };
    validate_sha256(arm_sha256)?;
    validate_sha256(intel_sha256)?;
    let path = Path::new(formula);
    let source = fs::read_to_string(path)
        .map_err(|error| format!("读取 Homebrew Cask 失败 {}: {error}", path.display()))?;
    let updated = updated_homebrew_cask(&source, version, arm_sha256, intel_sha256, url)?;
    fs::write(path, updated)
        .map_err(|error| format!("写入 Homebrew Cask 失败 {}: {error}", path.display()))
}

fn validate_sha256(value: &str) -> Result<(), String> {
    if value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        Ok(())
    } else {
        Err(format!("无效 SHA-256: {value}"))
    }
}

fn updated_homebrew_cask(
    source: &str,
    version: &str,
    arm_sha256: &str,
    intel_sha256: &str,
    url: &str,
) -> Result<String, String> {
    let trailing_newline = source.ends_with('\n');
    let mut lines: Vec<String> = source.lines().map(str::to_string).collect();
    let version_index = find_cask_line(&lines, "version ")?;
    lines[version_index] = format!("  version \"{version}\"");

    let arch_line = "  arch arm: \"arm64\", intel: \"x64\"".to_string();
    if let Some(index) = lines
        .iter()
        .position(|line| line.trim_start().starts_with("arch "))
    {
        lines[index] = arch_line;
    } else {
        lines.insert(version_index + 1, arch_line);
    }

    let sha_index = find_cask_line(&lines, "sha256 ")?;
    let url_index = find_cask_line(&lines, "url ")?;
    if url_index <= sha_index {
        return Err("Homebrew Cask 的 sha256 必须位于 url 之前".to_string());
    }
    lines.splice(
        sha_index..url_index,
        [
            format!("  sha256 arm: \"{arm_sha256}\","),
            format!("         intel: \"{intel_sha256}\""),
        ],
    );
    let url_index = find_cask_line(&lines, "url ")?;
    lines[url_index] = format!("  url \"{url}\"");

    let mut output = lines.join("\n");
    if trailing_newline {
        output.push('\n');
    }
    Ok(output)
}

fn find_cask_line(lines: &[String], prefix: &str) -> Result<usize, String> {
    lines
        .iter()
        .position(|line| line.trim_start().starts_with(prefix))
        .ok_or_else(|| format!("Homebrew Cask 缺少 {prefix}字段"))
}

fn prepare_macos_ort_framework() -> Result<PathBuf, String> {
    let root = repo_root()?;
    let cache = root.join("src-tauri/.desktop-runtime");
    let archive = cache
        .join("archives")
        .join(format!("onnxruntime-c-{ORT_VERSION}.zip"));
    let archive = verified_archive(
        "EPUB_TOOL_ORT_MACOS_ARCHIVE",
        MACOS_URL,
        MACOS_SHA256,
        &archive,
    )?;
    let destination = cache.join(format!("onnxruntime-c-{ORT_VERSION}"));
    let framework = destination.join("onnxruntime.xcframework");
    for prefix in [
        "onnxruntime.xcframework/Info.plist",
        "onnxruntime.xcframework/macos-arm64_x86_64/",
    ] {
        extract_prefix(&archive, prefix, &destination)?;
    }
    let binary = framework
        .join("macos-arm64_x86_64")
        .join("onnxruntime.framework")
        .join("Versions")
        .join("A")
        .join("onnxruntime");
    if !binary.is_file() {
        return Err(format!(
            "macOS ONNX Runtime 切片不完整: {}",
            binary.display()
        ));
    }
    Ok(framework)
}

fn macos_onnx_runtime_library(framework: &Path) -> Result<PathBuf, String> {
    let library = framework
        .join("macos-arm64_x86_64")
        .join("onnxruntime.framework")
        .join("Versions")
        .join("A")
        .join("onnxruntime");
    if library.is_file() {
        Ok(library)
    } else {
        Err(format!(
            "macOS ONNX Runtime 切片不完整: {}",
            library.display()
        ))
    }
}

fn prepare_macos_ort() -> Result<PathBuf, String> {
    let framework = prepare_macos_ort_framework()?;
    let library = macos_onnx_runtime_library(&framework)?;
    let destination = framework
        .parent()
        .ok_or_else(|| {
            format!(
                "macOS ONNX Runtime 框架路径无父目录: {}",
                framework.display()
            )
        })?
        .join("macos-static");
    copy_if_changed(&library, &destination.join("libonnxruntime.a"))?;
    println!(
        "macOS ONNX Runtime prepared: ORT_LIB_PATH={}",
        destination.display()
    );
    Ok(destination)
}

fn download_verified(url: &str, expected_sha256: &str, destination: &Path) -> Result<(), String> {
    if destination.is_file() && sha256(destination)? == expected_sha256 {
        return Ok(());
    }
    let parent = destination
        .parent()
        .ok_or_else(|| format!("归档路径无父目录: {}", destination.display()))?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("创建归档目录失败 {}: {error}", parent.display()))?;
    let temporary = destination.with_extension("download");
    let status = Command::new("curl")
        .args(["-fL", "--retry", "3", "--output"])
        .arg(&temporary)
        .arg(url)
        .status()
        .map_err(|error| format!("启动 curl 失败: {error}"))?;
    if !status.success() {
        return Err(format!("下载 ONNX Runtime 失败: {url}"));
    }
    let actual = sha256(&temporary)?;
    if actual != expected_sha256 {
        return Err(format!(
            "ONNX Runtime SHA-256 不匹配: {actual} != {expected_sha256}"
        ));
    }
    fs::rename(&temporary, destination).map_err(|error| {
        format!(
            "保存 ONNX Runtime 归档失败 {}: {error}",
            destination.display()
        )
    })
}

fn verified_archive(
    environment_name: &str,
    url: &str,
    expected_sha256: &str,
    cache_path: &Path,
) -> Result<PathBuf, String> {
    if let Ok(value) = env::var(environment_name) {
        if !value.trim().is_empty() {
            let path = PathBuf::from(value);
            let actual = sha256(&path)?;
            if actual != expected_sha256 {
                return Err(format!(
                    "{environment_name} 指定归档的 SHA-256 不匹配: {actual} != {expected_sha256}"
                ));
            }
            return Ok(path);
        }
    }
    download_verified(url, expected_sha256, cache_path)?;
    Ok(cache_path.to_path_buf())
}

fn sha256(path: &Path) -> Result<String, String> {
    let file =
        File::open(path).map_err(|error| format!("读取文件失败 {}: {error}", path.display()))?;
    let mut reader = BufReader::new(file);
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = reader
            .read(&mut buffer)
            .map_err(|error| format!("计算 SHA-256 失败 {}: {error}", path.display()))?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
    }
    Ok(format!("{:x}", digest.finalize()))
}

fn extract_prefix(archive: &Path, prefix: &str, destination: &Path) -> Result<(), String> {
    let file = File::open(archive)
        .map_err(|error| format!("打开归档失败 {}: {error}", archive.display()))?;
    let mut zip = ZipArchive::new(file)
        .map_err(|error| format!("读取归档失败 {}: {error}", archive.display()))?;
    for index in 0..zip.len() {
        let mut entry = zip
            .by_index(index)
            .map_err(|error| format!("读取归档条目失败: {error}"))?;
        if !entry.name().starts_with(prefix) {
            continue;
        }
        let relative = entry
            .enclosed_name()
            .ok_or_else(|| format!("归档包含不安全路径: {}", entry.name()))?;
        let output = destination.join(relative);
        if entry.is_dir() {
            fs::create_dir_all(&output)
                .map_err(|error| format!("创建目录失败 {}: {error}", output.display()))?;
        } else {
            write_zip_entry(&mut entry, &output)?;
        }
    }
    Ok(())
}

fn write_zip_entry(entry: &mut zip::read::ZipFile<'_>, destination: &Path) -> Result<(), String> {
    let parent = destination
        .parent()
        .ok_or_else(|| format!("解压路径无父目录: {}", destination.display()))?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("创建解压目录失败 {}: {error}", parent.display()))?;
    #[cfg(unix)]
    if entry.is_symlink() {
        use std::os::unix::fs::symlink;

        let mut target = String::new();
        entry
            .read_to_string(&mut target)
            .map_err(|error| format!("读取符号链接失败 {}: {error}", destination.display()))?;
        if destination.exists() || destination.is_symlink() {
            fs::remove_file(destination).map_err(|error| {
                format!("移除旧符号链接失败 {}: {error}", destination.display())
            })?;
        }
        symlink(target, destination)
            .map_err(|error| format!("创建符号链接失败 {}: {error}", destination.display()))?;
        return Ok(());
    }
    let mut output = File::create(destination)
        .map_err(|error| format!("创建解压文件失败 {}: {error}", destination.display()))?;
    io::copy(entry, &mut output)
        .map_err(|error| format!("解压文件失败 {}: {error}", destination.display()))?;
    #[cfg(unix)]
    if let Some(mode) = entry.unix_mode() {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(destination, fs::Permissions::from_mode(mode))
            .map_err(|error| format!("设置解压文件权限失败 {}: {error}", destination.display()))?;
    }
    Ok(())
}

fn copy_if_changed(source: &Path, destination: &Path) -> Result<(), String> {
    if destination.is_file()
        && fs::metadata(source).ok().map(|value| value.len())
            == fs::metadata(destination).ok().map(|value| value.len())
    {
        return Ok(());
    }
    let parent = destination
        .parent()
        .ok_or_else(|| format!("目标路径无父目录: {}", destination.display()))?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("创建目标目录失败 {}: {error}", parent.display()))?;
    fs::copy(source, destination)
        .map_err(|error| format!("复制原生库失败 {}: {error}", destination.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::updated_homebrew_cask;

    #[test]
    fn updates_homebrew_cask_without_external_script_runtime() {
        let source = r#"cask \"epub-tool-newui\" do
  version \"1.0.0\"
  sha256 arm: \"old-arm\",
         intel: \"old-intel\"
  url \"https://old.invalid\"
end
"#;
        let arm = "a".repeat(64);
        let intel = "b".repeat(64);
        let updated = updated_homebrew_cask(
            source,
            "2.0.0",
            &arm,
            &intel,
            "https://example.invalid/#{version}/#{arch}.dmg",
        )
        .unwrap();
        assert!(updated.contains("  version \"2.0.0\""));
        assert!(updated.contains("  arch arm: \"arm64\", intel: \"x64\""));
        assert!(updated.contains(&format!("  sha256 arm: \"{arm}\",")));
        assert!(updated.contains(&format!("         intel: \"{intel}\"")));
        assert!(updated.contains("  url \"https://example.invalid/#{version}/#{arch}.dmg\""));
    }
}
