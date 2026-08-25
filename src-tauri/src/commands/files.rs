use std::{fs, path::PathBuf};

use serde::Serialize;
use tauri::{AppHandle, State};

use crate::runtime::{resolve_log_path, RuntimeServices};

const COVER_PREVIEW_MAX_BYTES: u64 = 20 * 1024 * 1024;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImagePreviewResponse {
    bytes: Vec<u8>,
    mime_type: String,
}

#[tauri::command]
pub async fn open_path(services: State<'_, RuntimeServices>, path: String) -> Result<(), String> {
    let files = services.files();
    tauri::async_runtime::spawn_blocking(move || files.open_path(&path))
        .await
        .map_err(|error| format!("打开路径失败: {error}"))?
}

#[tauri::command]
pub async fn get_log_path(app: AppHandle) -> Result<String, String> {
    Ok(resolve_log_path(&app)?.to_string_lossy().to_string())
}

#[tauri::command]
#[tauri::command]
pub async fn read_image_preview(path: String) -> Result<ImagePreviewResponse, String> {
    let image_path = PathBuf::from(path);
    let metadata =
        fs::metadata(&image_path).map_err(|error| format!("读取封面文件信息失败: {error}"))?;
    if !metadata.is_file() {
        return Err("选择的封面路径不是文件。".to_string());
    }
    if metadata.len() > COVER_PREVIEW_MAX_BYTES {
        return Err("封面文件超过 20 MB，无法生成预览。".to_string());
    }
    let bytes = fs::read(&image_path).map_err(|error| format!("读取封面文件失败: {error}"))?;
    let mime_type = detect_preview_image_mime(&bytes)
        .ok_or_else(|| "封面预览仅支持有效的 JPG、PNG 或 WebP 图片。".to_string())?;
    Ok(ImagePreviewResponse {
        bytes,
        mime_type: mime_type.to_string(),
    })
}

#[tauri::command]
pub async fn collect_epub_files(
    services: State<'_, RuntimeServices>,
    directory_path: String,
) -> Result<Vec<String>, String> {
    let files = services.files();
    tauri::async_runtime::spawn_blocking(move || files.collect_epub_files(&directory_path))
        .await
        .map_err(|error| format!("扫描 EPUB 目录失败: {error}"))?
}

#[tauri::command]
pub async fn validate_output_directory(
    services: State<'_, RuntimeServices>,
    directory_path: String,
) -> Result<(), String> {
    let files = services.files();
    tauri::async_runtime::spawn_blocking(move || files.validate_output_directory(&directory_path))
        .await
        .map_err(|error| format!("验证输出目录失败: {error}"))?
}

#[tauri::command]
pub async fn resolve_input_sources(
    services: State<'_, RuntimeServices>,
    input_paths: Vec<String>,
) -> Result<Vec<String>, String> {
    let files = services.files();
    tauri::async_runtime::spawn_blocking(move || files.resolve_input_sources(&input_paths))
        .await
        .map_err(|error| format!("解析输入来源失败: {error}"))?
}

fn detect_preview_image_mime(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return Some("image/jpeg");
    }
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Some("image/png");
    }
    if bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        return Some("image/webp");
    }
    None
}

#[cfg(test)]
mod tests {
    use super::detect_preview_image_mime;

    #[test]
    fn detects_supported_preview_formats() {
        assert_eq!(
            detect_preview_image_mime(&[0xFF, 0xD8, 0xFF]),
            Some("image/jpeg")
        );
        assert_eq!(
            detect_preview_image_mime(b"\x89PNG\r\n\x1a\n"),
            Some("image/png")
        );
        assert_eq!(
            detect_preview_image_mime(b"RIFF0000WEBP"),
            Some("image/webp")
        );
        assert_eq!(detect_preview_image_mime(b"GIF89a"), None);
    }
}
