use dioxus::prelude::*;

use crate::models::Task;
use crate::server::api;

#[component]
pub fn TaskList(tasks: Vec<Task>, on_tasks_changed: EventHandler<Vec<Task>>) -> Element {
    rsx! {
        div { class: "task-list",
            div { class: "task-list-header", "Tasks" }
            for task in tasks.iter() {
                TaskRow {
                    key: "{task.id}",
                    task: task.clone(),
                    on_tasks_changed: on_tasks_changed,
                }
            }
        }
    }
}

#[component]
fn TaskRow(task: Task, on_tasks_changed: EventHandler<Vec<Task>>) -> Element {
    let mut editing = use_signal(|| false);
    let mut edit_value = use_signal(|| task.name.clone());
    let task_id = task.id.clone();

    let save = {
        let task = task.clone();
        let on_changed = on_tasks_changed;
        move |_: FocusEvent| {
            let new_name = edit_value();
            let mut updated = task.clone();
            updated.name = new_name;
            let on_changed = on_changed;
            spawn(async move {
                if api::update_task(updated).await.is_ok() {
                    if let Ok(tasks) = api::get_tasks().await {
                        on_changed.call(tasks);
                    }
                }
            });
            editing.set(false);
        }
    };

    rsx! {
        div {
            class: "task-row",
            div {
                class: "task-color-dot",
                style: "background-color: {task.color};",
            }
            if editing() {
                input {
                    class: "task-name-input",
                    value: "{edit_value}",
                    autofocus: true,
                    oninput: move |evt| edit_value.set(evt.value()),
                    onkeypress: {
                        let task = task.clone();
                        let on_changed = on_tasks_changed;
                        move |evt: KeyboardEvent| {
                            if evt.key() == Key::Enter {
                                let new_name = edit_value();
                                let mut updated = task.clone();
                                updated.name = new_name;
                                let on_changed = on_changed;
                                spawn(async move {
                                    if api::update_task(updated).await.is_ok() {
                                        if let Ok(tasks) = api::get_tasks().await {
                                            on_changed.call(tasks);
                                        }
                                    }
                                });
                                editing.set(false);
                            }
                        }
                    },
                    onblur: save,
                }
            } else {
                span {
                    class: "task-name",
                    ondoubleclick: move |_| {
                        edit_value.set(task.name.clone());
                        editing.set(true);
                    },
                    "{task.name}"
                }
            }
            button {
                class: "task-delete-btn",
                title: "Delete task",
                onclick: {
                    let on_changed = on_tasks_changed;
                    let task_id = task_id.clone();
                    move |_| {
                        let on_changed = on_changed;
                        let task_id = task_id.clone();
                        spawn(async move {
                            if api::delete_task(task_id).await.is_ok() {
                                if let Ok(tasks) = api::get_tasks().await {
                                    on_changed.call(tasks);
                                }
                            }
                        });
                    }
                },
                "\u{00d7}"
            }
        }
    }
}
