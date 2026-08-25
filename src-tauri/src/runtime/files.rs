use std::{fs, path::Path, sync::Arc};

use std::process::Command;

use tauri::AppHandle;

use super::paths::resolve_path;

pub trait PlatformFiles: Send + Sync {
    fn collect_epub_files(&self, directory_path: &str) -> Result<Vec<String>, String>;
    fn validate_output_directory(&self, directory_path: &str) -> Result<(), String>;
    fn resolve_input_sources(&self, input_paths: &[String]) -> Result<Vec<String>, String>;
    fn open_path(&self, path: &str) -> Result<(), String>;
}

pub fn create(app: AppHandle) -> Arc<dyn PlatformFiles> {
    Arc::new(DesktopFiles { app })
}

fn collect_epubs_recursive(directory: &Path, result: &mut Vec<String>) -> Result<(), String> {
    let entries = fs::read_dir(directory)
        .map_err(|error| format!("读取目录失败 {}: {error}", directory.display()))?;
    for entry in entries {
        let entry = entry.map_err(|error| format!("读取目录项失败: {error}"))?;
        let path = entry.path();
        if path.is_dir() {
            collect_epubs_recursive(&path, result)?;
        } else if path
            .extension()
            .and_then(|value| value.to_str())
            .is_some_and(|value| value.eq_ignore_ascii_case("epub"))
        {
            result.push(path.to_string_lossy().to_string());
        }
    }
    Ok(())
}

struct DesktopFiles {
    app: AppHandle,
}

impl PlatformFiles for DesktopFiles {
    fn collect_epub_files(&self, directory_path: &str) -> Result<Vec<String>, String> {
        let directory = resolve_path(&self.app, directory_path)?;
        if !directory.is_dir() {
            return Err(format!("不是有效目录: {}", directory.display()));
        }
        let mut files = Vec::new();
        collect_epubs_recursive(&directory, &mut files)?;
        files.sort();
        Ok(files)
    }

    fn validate_output_directory(&self, directory_path: &str) -> Result<(), String> {
        let directory = resolve_path(&self.app, directory_path)?;
        if directory.is_dir() {
            Ok(())
        } else {
            Err(format!("不是有效目录: {}", directory.display()))
        }
    }

    fn resolve_input_sources(&self, input_paths: &[String]) -> Result<Vec<String>, String> {
        let mut files = Vec::new();
        for input_path in input_paths {
            let path = resolve_path(&self.app, input_path)?;
            if path.is_dir() {
                collect_epubs_recursive(&path, &mut files)?;
            } else if path.is_file()
                && path
                    .extension()
                    .and_then(|value| value.to_str())
                    .is_some_and(|value| value.eq_ignore_ascii_case("epub"))
            {
                files.push(path.to_string_lossy().to_string());
            }
        }
        files.sort();
        files.dedup();
        Ok(files)
    }

    fn open_path(&self, path: &str) -> Result<(), String> {
        let external = path.to_ascii_lowercase().starts_with("https://")
            || path.to_ascii_lowercase().starts_with("http://");
        let target = if external {
            path.to_string()
        } else {
            resolve_path(&self.app, path)?.to_string_lossy().to_string()
        };
        let mut command = if cfg!(target_os = "macos") {
            let mut command = Command::new("open");
            command.arg(&target);
            command
        } else if cfg!(target_os = "windows") {
            let mut command = Command::new("cmd");
            command.args(["/C", "start", "", &target]);
            command
        } else {
            let mut command = Command::new("xdg-open");
            command.arg(&target);
            command
        };
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x0800_0000);
        }
        let status = command
            .status()
            .map_err(|error| format!("打开路径失败: {error}"))?;
        if status.success() {
            Ok(())
        } else {
            Err(format!("系统命令返回失败状态: {status}"))
        }
    }
}
