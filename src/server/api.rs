use dioxus::prelude::*;

use crate::models::Task;

#[cfg(not(target_arch = "wasm32"))]
mod state {
    use std::sync::{Arc, Mutex};

    use crate::models::Task;

    static TASKS: std::sync::OnceLock<Arc<Mutex<Vec<Task>>>> = std::sync::OnceLock::new();

    pub fn get_store() -> Arc<Mutex<Vec<Task>>> {
        TASKS
            .get_or_init(|| Arc::new(Mutex::new(Vec::new())))
            .clone()
    }
}

#[server]
pub async fn get_tasks() -> Result<Vec<Task>, ServerFnError> {
    let store = state::get_store();
    let tasks = store.lock().unwrap().clone();
    Ok(tasks)
}

#[server]
pub async fn create_task(task: Task) -> Result<Task, ServerFnError> {
    let store = state::get_store();
    let mut tasks = store.lock().unwrap();
    tasks.push(task.clone());
    Ok(task)
}

#[server]
pub async fn update_task(updated: Task) -> Result<(), ServerFnError> {
    let store = state::get_store();
    let mut tasks = store.lock().unwrap();
    if let Some(task) = tasks.iter_mut().find(|t| t.id == updated.id) {
        *task = updated;
        Ok(())
    } else {
        Err(ServerFnError::new("Task not found"))
    }
}

#[server]
pub async fn delete_task(id: String) -> Result<(), ServerFnError> {
    let store = state::get_store();
    let mut tasks = store.lock().unwrap();
    let len_before = tasks.len();
    tasks.retain(|t| t.id != id);
    if tasks.len() < len_before {
        Ok(())
    } else {
        Err(ServerFnError::new("Task not found"))
    }
}

#[server]
pub async fn import_tasks(
    file_bytes: Vec<u8>,
    file_name: String,
) -> Result<Vec<Task>, ServerFnError> {
    let imported = match detect_format(&file_name, &file_bytes) {
        FileFormat::Excel => super::excel::import_xlsx(&file_bytes)
            .map_err(|e| ServerFnError::new(format!("Import failed: {e}")))?,
        FileFormat::Csv => super::excel::import_csv(&file_bytes)
            .map_err(|e| ServerFnError::new(format!("Import failed: {e}")))?,
        FileFormat::Unknown(ext) => {
            return Err(ServerFnError::new(format!(
                "Unsupported file format: {ext}"
            )));
        }
    };
    let store = state::get_store();
    let mut tasks = store.lock().unwrap();
    *tasks = imported.clone();
    Ok(imported)
}

#[server]
pub async fn export_tasks(format: String) -> Result<(Vec<u8>, String), ServerFnError> {
    let store = state::get_store();
    let tasks = store.lock().unwrap().clone();
    match format.as_str() {
        "xlsx" => {
            let bytes = super::excel::export_xlsx(&tasks)
                .map_err(|e| ServerFnError::new(format!("Export failed: {e}")))?;
            Ok((bytes, "gantt_export.xlsx".into()))
        }
        other => Err(ServerFnError::new(format!(
            "Unsupported export format: {other}"
        ))),
    }
}

#[cfg(not(target_arch = "wasm32"))]
enum FileFormat {
    Excel,
    Csv,
    Unknown(String),
}

#[cfg(not(target_arch = "wasm32"))]
fn detect_format(file_name: &str, bytes: &[u8]) -> FileFormat {
    let ext = file_name.rsplit('.').next().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "xlsx" | "xls" => return FileFormat::Excel,
        "csv" | "tsv" => return FileFormat::Csv,
        _ => {}
    }
    // Check magic bytes: ZIP (xlsx) starts with PK (0x504B)
    if bytes.len() >= 4 && bytes[0] == 0x50 && bytes[1] == 0x4B {
        return FileFormat::Excel;
    }
    FileFormat::Unknown(ext)
}
