use std::{
    path::Path,
    time::{Duration, Instant},
};

use tauri::{ipc::Channel, AppHandle, State};

use crate::{
    engine_adapter,
    engine_protocol::v1::{
        engine_event, engine_request, engine_response, EngineEvent, EngineRequest, EngineResponse,
        FontScanProgress, FontScanResult, ProtocolVersion, TaskEvent as WireTaskEvent,
    },
    runtime::{resolve_log_path, ExecutionRequest, RuntimeServices},
};

const TASK_LOG_FORWARD_INTERVAL: Duration = Duration::from_millis(100);
const TASK_LOG_MESSAGE_LIMIT: usize = 4_096;

struct TaskEventForwarder {
    on_event: Channel<EngineEvent>,
    request_id: String,
    pending_log: Option<WireTaskEvent>,
    last_log_sent_at: Option<Instant>,
    receiver_unavailable: bool,
}

impl TaskEventForwarder {
    fn new(on_event: Channel<EngineEvent>, request_id: String) -> Self {
        Self {
            on_event,
            request_id,
            pending_log: None,
            last_log_sent_at: None,
            receiver_unavailable: false,
        }
    }

    fn forward(&mut self, mut task_event: WireTaskEvent) -> Result<(), String> {
        if task_event.event == "task.log" {
            truncate_task_log_message(&mut task_event.message);
            self.pending_log = Some(task_event);
            let should_flush = self
                .last_log_sent_at
                .is_none_or(|sent_at| sent_at.elapsed() >= TASK_LOG_FORWARD_INTERVAL);
            if should_flush {
                self.flush_pending_log()?;
            }
            return Ok(());
        }

        self.flush_pending_log()?;
        self.send(task_event)
    }

    fn flush_pending_log(&mut self) -> Result<(), String> {
        let Some(task_event) = self.pending_log.take() else {
            return Ok(());
        };

        self.send(task_event)?;
        self.last_log_sent_at = Some(Instant::now());
        Ok(())
    }

    fn send(&mut self, task_event: WireTaskEvent) -> Result<(), String> {
        if self.receiver_unavailable {
            return Ok(());
        }

        if let Err(error) = self.on_event.send(EngineEvent {
            protocol_version: ProtocolVersion::V1 as i32,
            request_id: self.request_id.clone(),
            payload: Some(engine_event::Payload::TaskEvent(task_event)),
        }) {
            self.receiver_unavailable = true;
            eprintln!("Rust 引擎事件接收端已断开，后续事件不再推送: {error}");
        }

        Ok(())
    }
}

fn truncate_task_log_message(message: &mut String) {
    let Some((byte_index, _)) = message.char_indices().nth(TASK_LOG_MESSAGE_LIMIT) else {
        return;
    };

    message.truncate(byte_index);
    message.push_str("…（日志过长，已截断）");
}

#[tauri::command]
pub async fn list_font_targets_batch(
    request: EngineRequest,
    on_event: Channel<EngineEvent>,
) -> Result<EngineResponse, String> {
    validate_engine_request(&request)?;
    let request_id = request.request_id.clone();
    let Some(engine_request::Operation::ScanFonts(scan_request)) = request.operation else {
        return Err("字体扫描命令只接受 scanFonts operation".to_string());
    };

    tauri::async_runtime::spawn_blocking(move || -> Result<EngineResponse, String> {
        let total_files = u32::try_from(scan_request.input_files.len())
            .map_err(|_| "字体扫描文件数超出 Protobuf uint32 范围".to_string())?;
        let mut results = Vec::with_capacity(scan_request.input_files.len());
        let mut receiver_unavailable = false;
        for (position, input_file) in scan_request.input_files.into_iter().enumerate() {
            let result = engine_adapter::font_target_result(
                input_file.clone(),
                epub_tool_core::list_font_targets(Path::new(&input_file))
                    .map_err(|error| error.to_string()),
            );
            if !receiver_unavailable {
                if let Err(error) = on_event.send(EngineEvent {
                    protocol_version: ProtocolVersion::V1 as i32,
                    request_id: request_id.clone(),
                    payload: Some(engine_event::Payload::FontScanProgress(FontScanProgress {
                        current_index: u32::try_from(position + 1)
                            .map_err(|_| "字体扫描索引超出 Protobuf uint32 范围".to_string())?,
                        total_files,
                        result: Some(result.clone()),
                    })),
                }) {
                    receiver_unavailable = true;
                    eprintln!("Rust 字体扫描事件接收端已断开，继续完成扫描: {error}");
                }
            }
            results.push(result);
        }
        Ok(EngineResponse {
            protocol_version: ProtocolVersion::V1 as i32,
            request_id,
            payload: Some(engine_response::Payload::FontScanResult(FontScanResult {
                results,
            })),
        })
    })
    .await
    .map_err(|error| format!("异步字体扫描失败: {error}"))?
}

#[tauri::command]
pub async fn run_epub_task(
    app: AppHandle,
    services: State<'_, RuntimeServices>,
    request: EngineRequest,
    on_event: Channel<EngineEvent>,
) -> Result<EngineResponse, String> {
    validate_engine_request(&request)?;
    let request_id = request.request_id.clone();
    let Some(engine_request::Operation::RunTask(run_request)) = request.operation else {
        return Err("任务命令只接受 runTask operation".to_string());
    };
    let execution = ExecutionRequest {
        task: engine_adapter::task_spec(&run_request)?,
        log_path: resolve_log_path(&app)?,
    };
    let engine = services.engine();
    tauri::async_runtime::spawn_blocking(move || -> Result<EngineResponse, String> {
        let mut event_forwarder = TaskEventForwarder::new(on_event, request_id.clone());
        let result = match engine.execute(execution, &mut |event| {
            event_forwarder.forward(engine_adapter::task_event(event)?)
        }) {
            Ok(result) => result,
            Err(error) => {
                let _ = event_forwarder.flush_pending_log();
                return Err(error);
            }
        };
        event_forwarder.flush_pending_log()?;
        Ok(EngineResponse {
            protocol_version: ProtocolVersion::V1 as i32,
            request_id,
            payload: Some(engine_adapter::task_result_response(result)?),
        })
    })
    .await
    .map_err(|error| format!("异步任务失败: {error}"))?
}

fn validate_engine_request(request: &EngineRequest) -> Result<(), String> {
    if request.protocol_version != ProtocolVersion::V1 as i32 {
        return Err("请求使用了不支持的 protocolVersion".to_string());
    }
    if request.request_id.trim().is_empty() {
        return Err("请求缺少 requestId".to_string());
    }
    if request.operation.is_none() {
        return Err("请求缺少 operation".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{truncate_task_log_message, TASK_LOG_MESSAGE_LIMIT};

    #[test]
    fn truncates_oversized_task_log_on_a_character_boundary() {
        let mut message = "字".repeat(TASK_LOG_MESSAGE_LIMIT + 1);

        truncate_task_log_message(&mut message);

        assert!(message.starts_with(&"字".repeat(TASK_LOG_MESSAGE_LIMIT)));
        assert!(message.ends_with("…（日志过长，已截断）"));
    }

    #[test]
    fn preserves_task_log_at_the_limit() {
        let mut message = "字".repeat(TASK_LOG_MESSAGE_LIMIT);

        truncate_task_log_message(&mut message);

        assert_eq!(message.chars().count(), TASK_LOG_MESSAGE_LIMIT);
    }
}
