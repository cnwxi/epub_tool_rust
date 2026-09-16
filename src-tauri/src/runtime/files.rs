use std::sync::Arc;
use tauri::AppHandle;

#[cfg(any(target_os = "android", test))]
#[path = "staging.rs"]
mod staging;

pub trait PlatformFiles: Send + Sync {
    fn resolve_input_sources(&self, paths: &[String]) -> Result<Vec<String>, String>;
    fn collect_epub_files(&self, _directory: &str) -> Result<Vec<String>, String> {
        Err("当前平台不支持目录扫描".into())
    }
    fn validate_output_directory(&self, _directory: &str) -> Result<(), String> {
        Err("当前平台不支持输出目录".into())
    }
    fn open_path(&self, _path: &str) -> Result<(), String> {
        Err("当前平台不支持打开路径".into())
    }
    #[cfg(target_os = "android")]
    fn stage_source(&self, source: &str, extension: &str) -> Result<String, String>;
    #[cfg(target_os = "android")]
    fn export_output(&self, source: &str, destination: &str) -> Result<(), String>;
}
#[cfg(target_os = "android")]
#[path = "android_files.rs"]
mod platform;
#[cfg(not(target_os = "android"))]
#[path = "desktop_files.rs"]
mod platform;

pub fn create(app: AppHandle) -> Arc<dyn PlatformFiles> {
    platform::create(app)
}
