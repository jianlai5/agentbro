use std::backtrace::Backtrace;
use std::fs::{create_dir_all, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

static STARTUP_LOG_PATH: OnceLock<PathBuf> = OnceLock::new();

pub fn init_startup_logging() {
    let path = log_file_path();
    let _ = STARTUP_LOG_PATH.set(path.clone());
    let _ = create_parent_dir(&path);
    append_line("INFO", "startup logging initialized");

    std::panic::set_hook(Box::new(|panic_info| {
        let location = panic_info
            .location()
            .map(|loc| format!("{}:{}:{}", loc.file(), loc.line(), loc.column()))
            .unwrap_or_else(|| "unknown".to_string());
        let payload = if let Some(msg) = panic_info.payload().downcast_ref::<&str>() {
            (*msg).to_string()
        } else if let Some(msg) = panic_info.payload().downcast_ref::<String>() {
            msg.clone()
        } else {
            "non-string panic payload".to_string()
        };
        let backtrace = Backtrace::force_capture();
        append_block(
            "PANIC",
            &format!(
                "panic at {location}\nmessage: {payload}\nbacktrace:\n{backtrace}"
            ),
        );
    }));
}

pub fn log_startup_info(message: &str) {
    append_line("INFO", message);
}

pub fn log_startup_error(message: &str) {
    append_line("ERROR", message);
}

fn log_file_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("agentbro")
        .join("logs")
        .join("startup-errors.log")
}

fn create_parent_dir(path: &PathBuf) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        create_dir_all(parent)?;
    }
    Ok(())
}

fn append_line(level: &str, message: &str) {
    let line = format!("[{}] [{}] {}", timestamp(), level, message);
    append_block_raw(&line);
}

fn append_block(level: &str, message: &str) {
    let block = format!("[{}] [{}] {}\n", timestamp(), level, message);
    append_block_raw(&block);
}

fn append_block_raw(message: &str) {
    let path = STARTUP_LOG_PATH
        .get()
        .cloned()
        .unwrap_or_else(log_file_path);
    let _ = create_parent_dir(&path);
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(file, "{message}");
    }
}

fn timestamp() -> String {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_secs().to_string(),
        Err(_) => "0".to_string(),
    }
}
