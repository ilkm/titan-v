use std::fs::OpenOptions;
use std::io::Write;

use serde_json::json;

pub(crate) fn agent_debug_log(
    hypothesis_id: &str,
    location: &str,
    message: &str,
    data: serde_json::Value,
) {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or_default();
    let payload = json!({
        "sessionId":"1f0423",
        "runId":"run2",
        "hypothesisId":hypothesis_id,
        "location":location,
        "message":message,
        "data":data,
        "timestamp":timestamp,
    });
    if let Ok(mut f) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("debug-1f0423.log")
    {
        let _ = writeln!(f, "{}", payload);
    }
}
