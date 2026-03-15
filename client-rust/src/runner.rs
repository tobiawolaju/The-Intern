use crate::command::CommandItem;
use crate::logger::Logger;
use crate::windows::LocalBody;
use std::time::SystemTime;

#[derive(Debug, serde::Serialize)]
pub struct LogEvent {
    pub ts_ms: u128,
    pub duration_ms: u128,
    pub index: u32,
    pub tag: String,
    pub instruction: String,
    pub status: String,
    pub detail: String,
    pub screenshot_path: Option<String>,
    pub screenshot_data_url: Option<String>,
}

pub fn run_command(
    local_body: &mut LocalBody,
    logger: &mut Logger,
    cmd: &CommandItem,
) -> Result<LogEvent, String> {
    let start = now_ms();
    let result = local_body.execute(cmd);
    let end = now_ms();

    let event = LogEvent {
        ts_ms: end,
        duration_ms: end.saturating_sub(start),
        index: result.index,
        tag: result.tag,
        instruction: result.instruction,
        status: result.status,
        detail: result.detail,
        screenshot_path: result.screenshot_path,
        screenshot_data_url: result.screenshot_data_url,
    };

    logger.log_event(&event)?;
    Ok(event)
}

pub fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}
