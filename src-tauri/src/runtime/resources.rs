use std::path::PathBuf;

use tauri::{AppHandle, Manager};

use super::paths::workspace_root;

#[derive(Debug, Clone)]
pub struct RuntimeResources {
    pub opencc_dir: Option<PathBuf>,
    pub ocr_model_dir: Option<PathBuf>,
}

pub fn prepare(app: &AppHandle) -> Result<RuntimeResources, String> {
    Ok(RuntimeResources {
        opencc_dir: resolve_opencc_dir(app),
        ocr_model_dir: resolve_ocr_model_dir(app),
    })
}

fn resolve_opencc_dir(app: &AppHandle) -> Option<PathBuf> {
    workspace_root()
        .map(|root| {
            root.join("src-tauri")
                .join("bundle-resources")
                .join("opencc")
        })
        .or_else(|| {
            app.path()
                .resource_dir()
                .ok()
                .map(|directory| directory.join("opencc"))
        })
        .filter(|directory| directory.is_dir())
}

fn resolve_ocr_model_dir(app: &AppHandle) -> Option<PathBuf> {
    if let Ok(path) = std::env::var("EPUB_TOOL_OCR_ONNX_MODEL_DIR") {
        if !path.is_empty() {
            return Some(PathBuf::from(path));
        }
    }
    let model_name = std::env::var("EPUB_TOOL_OCR_MODEL_NAME")
        .ok()
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "PP-OCRv6_small_rec".to_string())
        + "_onnx";
    workspace_root()
        .map(|root| {
            root.join("src-tauri")
                .join("bundle-resources")
                .join("ocr-models")
                .join(&model_name)
        })
        .or_else(|| {
            app.path()
                .resource_dir()
                .ok()
                .map(|directory| directory.join("ocr-models").join(model_name))
        })
        .filter(|directory| directory.is_dir())
}
