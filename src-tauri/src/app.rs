use crate::{
    commands::{self, PersistedStore},
    runtime::RuntimeServices,
};
#[cfg(target_os = "android")]
use tauri::Emitter;
use tauri::Manager;

#[cfg(target_os = "macos")]
use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial};

#[cfg(target_os = "windows")]
use window_vibrancy::{apply_blur, apply_mica};

#[cfg(not(target_os = "android"))]
fn setup_window_effects(app: &tauri::App) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "未找到主窗口 main".to_string())?;

    #[cfg(target_os = "macos")]
    {
        apply_vibrancy(&window, NSVisualEffectMaterial::HudWindow, None, None)
            .map_err(|error| format!("应用 macOS 毛玻璃效果失败: {error}"))?;
    }

    #[cfg(target_os = "windows")]
    {
        window
            .set_decorations(true)
            .map_err(|error| format!("恢复 Windows 原生窗口装饰失败: {error}"))?;
        apply_mica(&window, None)
            .or_else(|_| apply_blur(&window, Some((245, 239, 231, 180))))
            .map_err(|error| format!("应用 Windows 毛玻璃效果失败: {error}"))?;
    }

    #[cfg(target_os = "linux")]
    {
        let _ = window;
    }

    Ok(())
}

pub fn run() {
    let builder = tauri::Builder::default().plugin(tauri_plugin_dialog::init());
    #[cfg(target_os = "android")]
    let builder = builder.plugin(tauri_plugin_fs::init());
    let app = builder
        .setup(|app| {
            app.manage(PersistedStore::load(app.handle()));
            app.manage(RuntimeServices::new(app.handle())?);
            #[cfg(target_os = "android")]
            app.manage(commands::files::OpenedSources::new());
            #[cfg(not(target_os = "android"))]
            setup_window_effects(app)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::files::collect_epub_files,
            commands::files::get_log_path,
            commands::engine::get_platform_capabilities,
            commands::state::get_persisted_store_path,
            commands::engine::get_engine_status,
            commands::tasks::list_font_targets_batch,
            commands::state::load_persisted_state,
            commands::files::open_path,
            commands::files::read_image_preview,
            commands::files::resolve_input_sources,
            commands::tasks::run_epub_task,
            commands::state::save_persisted_state,
            commands::files::validate_output_directory,
            #[cfg(target_os = "android")]
            commands::files::export_output,
            #[cfg(target_os = "android")]
            commands::files::stage_source_for_task,
            #[cfg(target_os = "android")]
            commands::files::take_opened_sources,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|_app_handle, _event| {
        #[cfg(target_os = "android")]
        if let tauri::RunEvent::Opened { urls } = _event {
            let sources = urls
                .into_iter()
                .map(|url| url.to_string())
                .collect::<Vec<_>>();
            _app_handle
                .state::<commands::files::OpenedSources>()
                .extend(sources.iter().cloned());
            let _ = _app_handle.emit("opened", sources);
        }
    });
}
