use std::{fs, io::Write, path::PathBuf, sync::Arc};

use tauri::{AppHandle, Manager};

use super::{staging::stage_reader, PlatformFiles};

pub fn create(app: AppHandle) -> Arc<dyn PlatformFiles> {
    Arc::new(AndroidFiles { app })
}

struct AndroidFiles {
    app: AppHandle,
}

impl AndroidFiles {
    fn staging_directory(&self) -> Result<PathBuf, String> {
        let directory = self
            .app
            .path()
            .app_cache_dir()
            .map_err(|error| format!("无法定位 Android 临时目录: {error}"))?
            .join("epub-tool-inputs");
        fs::create_dir_all(&directory).map_err(|error| {
            format!("创建 Android 临时目录失败 {}: {error}", directory.display())
        })?;
        Ok(directory)
    }
}

impl PlatformFiles for AndroidFiles {
    fn resolve_input_sources(&self, input_paths: &[String]) -> Result<Vec<String>, String> {
        input_paths
            .iter()
            .map(|path| self.stage_source(path, "epub"))
            .collect()
    }

    fn stage_source(&self, source_path: &str, extension: &str) -> Result<String, String> {
        use std::str::FromStr;
        use tauri_plugin_fs::{FilePath, FsExt, OpenOptions};

        let source =
            FilePath::from_str(source_path).map_err(|_| "无效的 Android 文件 URI。".to_string())?;
        let mut source_options = OpenOptions::new();
        source_options.read(true);
        let mut source = self
            .app
            .fs()
            .open(source, source_options)
            .map_err(|error| format!("读取所选文件失败: {error}"))?;
        let directory = self.staging_directory()?;
        let destination = stage_reader(&directory, source_path, extension, &mut source)?;
        Ok(destination.to_string_lossy().to_string())
    }

    fn export_output(&self, source_path: &str, destination_path: &str) -> Result<(), String> {
        use std::str::FromStr;
        use tauri_plugin_fs::{FilePath, FsExt, OpenOptions};

        let mut source = fs::File::open(source_path)
            .map_err(|error| format!("读取处理结果失败 {source_path}: {error}"))?;
        let destination = FilePath::from_str(destination_path)
            .map_err(|_| "无效的 Android 导出文件 URI。".to_string())?;
        let mut destination_options = OpenOptions::new();
        destination_options.write(true).truncate(true).create(true);
        let mut output = self
            .app
            .fs()
            .open(destination, destination_options)
            .map_err(|error| format!("创建导出文件失败: {error}"))?;
        std::io::copy(&mut source, &mut output)
            .map_err(|error| format!("导出处理结果失败: {error}"))?;
        output
            .flush()
            .map_err(|error| format!("刷新导出文件失败: {error}"))
    }
}
