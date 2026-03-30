use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use dioxus::prelude::*;

use crate::models::Task;
use crate::server::api;

#[component]
pub fn Toolbar(on_tasks_changed: EventHandler<Vec<Task>>) -> Element {
    let mut import_error = use_signal(|| None::<String>);

    let handle_import = move |evt: FormEvent| {
        let on_changed = on_tasks_changed;
        spawn(async move {
            let files = evt.files();
            if let Some(file) = files.first() {
                let file_name = file.name();
                match file.read_bytes().await {
                    Ok(bytes) => match api::import_tasks(bytes.to_vec(), file_name).await {
                        Ok(tasks) => {
                            import_error.set(None);
                            on_changed.call(tasks);
                        }
                        Err(e) => {
                            import_error.set(Some(format!("{e}")));
                        }
                    },
                    Err(e) => {
                        import_error.set(Some(format!("Failed to read file: {e}")));
                    }
                }
            }
        });
    };

    let handle_export = move |_| {
        spawn(async move {
            match api::export_tasks("xlsx".into()).await {
                Ok((bytes, filename)) => {
                    let encoded = STANDARD.encode(&bytes);
                    let mime = mime_for_filename(&filename);
                    let js = format!(
                        r#"
                        const b = Uint8Array.from(atob("{}"), c => c.charCodeAt(0));
                        const blob = new Blob([b], {{type: "{}"}});
                        const url = URL.createObjectURL(blob);
                        const a = document.createElement("a");
                        a.href = url;
                        a.download = "{}";
                        a.click();
                        URL.revokeObjectURL(url);
                        "#,
                        encoded, mime, filename
                    );
                    document::eval(&js);
                }
                Err(e) => {
                    import_error.set(Some(format!("Export failed: {e}")));
                }
            }
        });
    };

    let handle_add = move |_| {
        let on_changed = on_tasks_changed;
        spawn(async move {
            let today = chrono::Local::now().date_naive();
            let new_task = Task {
                id: uuid::Uuid::new_v4().to_string(),
                name: "New Task".into(),
                start_date: today,
                end_date: today + chrono::Duration::days(7),
                progress: 0.0,
                dependencies: vec![],
                color: "#4a90d9".into(),
                sort_order: 999,
            };
            if api::create_task(new_task).await.is_ok() {
                if let Ok(tasks) = api::get_tasks().await {
                    on_changed.call(tasks);
                }
            }
        });
    };

    rsx! {
        div { class: "toolbar",
            button {
                class: "toolbar-btn",
                onclick: handle_add,
                "+ Add Task"
            }
            label {
                class: "toolbar-btn import-btn",
                "Import"
                input {
                    r#type: "file",
                    accept: ".xlsx,.xls,.csv",
                    style: "display:none",
                    onchange: handle_import,
                }
            }
            button {
                class: "toolbar-btn",
                onclick: handle_export,
                "Export"
            }
            if let Some(err) = import_error() {
                span { class: "toolbar-error", "{err}" }
            }
        }
    }
}

fn mime_for_filename(name: &str) -> &'static str {
    match name.rsplit('.').next().unwrap_or("") {
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "xls" => "application/vnd.ms-excel",
        _ => "application/octet-stream",
    }
}
