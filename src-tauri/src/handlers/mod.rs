// Python subprocess handler for Tauri commands
use std::process::{Command, Stdio};
use std::io::Write;
use serde_json::{json, Value};
use tauri::AppHandle;

pub struct PythonBridge {
    python_path: String,
}

impl PythonBridge {
    pub fn new() -> Self {
        let python_path = std::env::var("PYTHON_PATH")
            .unwrap_or_else(|_| "python3".to_string());
        Self { python_path }
    }

    pub async fn call_python(&self, method: &str, args: Value) -> Result<Value, String> {
        let payload = json!({
            "method": method,
            "args": args
        });

        let mut child = Command::new(&self.python_path)
            .arg("-m")
            .arg("md_scanner.tauri_bridge")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn Python process: {}", e))?;

        {
            let mut stdin = child.stdin.take()
                .ok_or_else(|| "Failed to open stdin".to_string())?;
            
            stdin.write_all(payload.to_string().as_bytes())
                .map_err(|e| format!("Failed to write to stdin: {}", e))?;
        }

        let output = child.wait_with_output()
            .map_err(|e| format!("Failed to wait for Python process: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Python error: {}", stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        serde_json::from_str(&stdout)
            .map_err(|e| format!("Failed to parse Python response: {}", e))
    }
}

pub mod channel {
    pub fn create_progress_emitter(app: &tauri::AppHandle, event_name: &str) 
        -> impl Fn(usize, usize) + Send + Sync 
    {
        use tauri::Emitter;
        let app = app.clone();
        let event_name = event_name.to_string();
        
        move |current: usize, total: usize| {
            let percent = if total > 0 {
                (current as f32 / total as f32) * 100.0
            } else {
                0.0
            };
            
            let payload = serde_json::json!({
                "current": current,
                "total": total,
                "percent": percent
            });
            
            let _ = app.emit(&event_name, payload);
        }
    }
}
