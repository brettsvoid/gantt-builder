mod components;
mod models;
mod server;

use dioxus::prelude::*;

use components::chart::Chart;
use components::task_list::TaskList;
use components::toolbar::Toolbar;
use models::Task;
use server::api;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let mut tasks = use_signal(Vec::<Task>::new);
    let mut loading = use_signal(|| true);

    use_effect(move || {
        spawn(async move {
            match api::get_tasks().await {
                Ok(t) => {
                    tasks.set(t);
                    loading.set(false);
                }
                Err(_) => {
                    loading.set(false);
                }
            }
        });
    });

    let on_tasks_changed = move |new_tasks: Vec<Task>| {
        tasks.set(new_tasks);
    };

    rsx! {
        document::Stylesheet { href: asset!("/assets/main.css") }
        div { class: "app",
            h1 { class: "app-title", "Gantt Builder" }
            Toolbar { on_tasks_changed }
            if loading() {
                div { class: "loading", "Loading tasks..." }
            } else {
                div { class: "main-layout",
                    TaskList {
                        tasks: tasks(),
                        on_tasks_changed,
                    }
                    Chart {
                        tasks: tasks(),
                        on_tasks_changed,
                    }
                }
            }
        }
    }
}
