mod app;
mod commands;
mod engine_adapter;
mod engine_protocol;
mod runtime;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    app::run();
}
