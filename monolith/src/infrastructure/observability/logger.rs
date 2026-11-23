use chrono::Utc;
use serde_json::{Map, Value};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

/// Logger JSON estruturado que emite eventos em linha única para `stdout` e opcionalmente
/// para um arquivo de log (ex.: `storage/logs/laravel.log` para compatibilidade com scripts).
#[derive(Debug)]
pub struct StructuredLogger {
    emit_stdout: bool,
    file_path: Option<PathBuf>,
    file_lock: Mutex<()>,
}

impl StructuredLogger {
    pub fn new(file_path: Option<PathBuf>, emit_stdout: bool) -> Self {
        if let Some(ref path) = file_path
            && let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            let _ = fs::create_dir_all(parent);
        }

        Self {
            emit_stdout,
            file_path,
            file_lock: Mutex::new(()),
        }
    }

    pub fn noop() -> Self {
        Self {
            emit_stdout: false,
            file_path: None,
            file_lock: Mutex::new(()),
        }
    }

    pub fn info(&self, event: &str, message: &str, request_id: Option<&str>, fields: Map<String, Value>) {
        self.log("INFO", event, message, request_id, fields);
    }

    pub fn warning(&self, event: &str, message: &str, request_id: Option<&str>, fields: Map<String, Value>) {
        self.log("WARNING", event, message, request_id, fields);
    }

    pub fn error(&self, event: &str, message: &str, request_id: Option<&str>, fields: Map<String, Value>) {
        self.log("ERROR", event, message, request_id, fields);
    }

    pub fn log(
        &self,
        level: &str,
        event: &str,
        message: &str,
        request_id: Option<&str>,
        fields: Map<String, Value>,
    ) {
        if !self.emit_stdout && self.file_path.is_none() {
            return;
        }

        let mut payload = Map::new();
        payload.insert("timestamp".to_string(), Value::String(Utc::now().to_rfc3339()));
        payload.insert("level".to_string(), Value::String(level.to_string()));
        payload.insert("message".to_string(), Value::String(message.to_string()));
        payload.insert(
            "request_id".to_string(),
            request_id
                .map(|id| Value::String(id.to_string()))
                .unwrap_or(Value::Null),
        );
        payload.insert("event".to_string(), Value::String(event.to_string()));

        for (k, v) in fields {
            payload.insert(k, v);
        }

        if let Ok(line) = serde_json::to_string(&Value::Object(payload)) {
            if self.emit_stdout {
                println!("{}", line);
            }
            if let Some(ref path) = self.file_path {
                let _guard = self.file_lock.lock();
                if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
                    let _ = writeln!(file, "{}", line);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_structured_logger_writes_valid_json_line_to_file() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let tmp_path = std::env::temp_dir().join(format!("infocar_log_test_{}.log", unique));

        let logger = StructuredLogger::new(Some(tmp_path.clone()), false);
        let mut fields = Map::new();
        fields.insert("provider".to_string(), Value::String("rest".to_string()));
        fields.insert("plate".to_string(), Value::String("ABC****".to_string()));

        logger.info("provider.request", "provider.request", Some("req-123"), fields);

        let content = fs::read_to_string(&tmp_path).unwrap();
        let parsed: Value = serde_json::from_str(content.trim()).unwrap();

        assert_eq!(parsed["level"], "INFO");
        assert_eq!(parsed["event"], "provider.request");
        assert_eq!(parsed["request_id"], "req-123");
        assert_eq!(parsed["provider"], "rest");
        assert_eq!(parsed["plate"], "ABC****");

        let _ = fs::remove_file(&tmp_path);
    }
}
