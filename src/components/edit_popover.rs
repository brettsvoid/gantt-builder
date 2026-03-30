use dioxus::prelude::*;

use crate::models::Task;
use crate::server::api;

#[component]
pub fn EditPopover(
    task: Task,
    position_x: f64,
    position_y: f64,
    on_close: EventHandler<()>,
    on_tasks_changed: EventHandler<Vec<Task>>,
) -> Element {
    let mut name = use_signal(|| task.name.clone());
    let mut start = use_signal(|| task.start_date.format("%Y-%m-%d").to_string());
    let mut end = use_signal(|| task.end_date.format("%Y-%m-%d").to_string());
    let mut progress = use_signal(|| (task.progress * 100.0) as i32);
    let mut color = use_signal(|| task.color.clone());
    let task_id = task.id.clone();

    let handle_save = {
        let task_id = task_id.clone();
        let task = task.clone();
        move |_| {
            let task_id = task_id.clone();
            let task = task.clone();
            let on_changed = on_tasks_changed;
            let on_close = on_close;
            let name_val = name();
            let start_val = start();
            let end_val = end();
            let progress_val = progress();
            let color_val = color();

            spawn(async move {
                let start_date = chrono::NaiveDate::parse_from_str(&start_val, "%Y-%m-%d")
                    .unwrap_or(task.start_date);
                let end_date = chrono::NaiveDate::parse_from_str(&end_val, "%Y-%m-%d")
                    .unwrap_or(task.end_date);

                let updated = Task {
                    id: task_id,
                    name: name_val,
                    start_date,
                    end_date,
                    progress: progress_val as f32 / 100.0,
                    dependencies: task.dependencies.clone(),
                    color: color_val,
                    sort_order: task.sort_order,
                };

                if api::update_task(updated).await.is_ok() {
                    if let Ok(tasks) = api::get_tasks().await {
                        on_changed.call(tasks);
                    }
                }
                on_close.call(());
            });
        }
    };

    let style = format!("left: {}px; top: {}px;", position_x, position_y);

    rsx! {
        div {
            class: "popover-backdrop",
            onclick: move |_| on_close.call(()),
        }
        div {
            class: "edit-popover",
            style: "{style}",
            onclick: move |evt| evt.stop_propagation(),
            h3 { "Edit Task" }
            div { class: "popover-field",
                label { "Name" }
                input {
                    value: "{name}",
                    oninput: move |evt| name.set(evt.value()),
                }
            }
            div { class: "popover-field",
                label { "Start Date" }
                input {
                    r#type: "date",
                    value: "{start}",
                    oninput: move |evt| start.set(evt.value()),
                }
            }
            div { class: "popover-field",
                label { "End Date" }
                input {
                    r#type: "date",
                    value: "{end}",
                    oninput: move |evt| end.set(evt.value()),
                }
            }
            div { class: "popover-field",
                label { "Progress: {progress}%" }
                input {
                    r#type: "range",
                    min: "0",
                    max: "100",
                    value: "{progress}",
                    oninput: move |evt| {
                        if let Ok(v) = evt.value().parse::<i32>() {
                            progress.set(v);
                        }
                    },
                }
            }
            div { class: "popover-field",
                label { "Color" }
                input {
                    r#type: "color",
                    value: "{color}",
                    oninput: move |evt| color.set(evt.value()),
                }
            }
            div { class: "popover-actions",
                button {
                    class: "btn-save",
                    onclick: handle_save,
                    "Save"
                }
                button {
                    class: "btn-cancel",
                    onclick: move |_| on_close.call(()),
                    "Cancel"
                }
            }
        }
    }
}
