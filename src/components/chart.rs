use dioxus::prelude::*;

use crate::components::edit_popover::EditPopover;
use crate::components::task_bar::TaskBar;
use crate::components::timeline::{compute_date_range, px_per_day, TimelineHeader};
use crate::models::Task;

#[component]
pub fn Chart(tasks: Vec<Task>, on_tasks_changed: EventHandler<Vec<Task>>) -> Element {
    let mut editing_task = use_signal(|| None::<(Task, f64, f64)>);

    let (start, end) = compute_date_range(&tasks);
    let total_days = (end - start).num_days();
    let total_width = total_days as f64 * px_per_day();
    let total_height = tasks.len() as f64 * 44.0 + 20.0;

    // Generate row backgrounds
    let rows: Vec<(usize, f64)> = (0..tasks.len()).map(|i| (i, i as f64 * 44.0)).collect();

    // Dependency arrows
    let mut arrows = Vec::new();
    for task in &tasks {
        for dep_id in &task.dependencies {
            if let Some(dep) = tasks.iter().find(|t| &t.id == dep_id) {
                let dep_idx = tasks.iter().position(|t| t.id == dep.id).unwrap_or(0);
                let task_idx = tasks.iter().position(|t| t.id == task.id).unwrap_or(0);

                let from_x = crate::components::timeline::date_to_x(dep.end_date, start);
                let from_y = dep_idx as f64 * 44.0 + 22.0;
                let to_x = crate::components::timeline::date_to_x(task.start_date, start);
                let to_y = task_idx as f64 * 44.0 + 22.0;

                arrows.push((from_x, from_y, to_x, to_y));
            }
        }
    }

    rsx! {
        div { class: "chart-container",
            TimelineHeader { start, end }
            div {
                class: "chart-body",
                style: "width: {total_width}px; height: {total_height}px;",
                // Row backgrounds
                for (i, y) in rows {
                    div {
                        class: if i % 2 == 0 { "chart-row" } else { "chart-row chart-row-alt" },
                        style: "top: {y}px; width: {total_width}px;",
                    }
                }
                // Dependency arrows SVG
                svg {
                    class: "dependency-arrows",
                    width: "{total_width}",
                    height: "{total_height}",
                    defs {
                        marker {
                            id: "arrowhead",
                            marker_width: "8",
                            marker_height: "6",
                            ref_x: "8",
                            ref_y: "3",
                            orient: "auto",
                            path { d: "M 0 0 L 8 3 L 0 6 Z", fill: "#888" }
                        }
                    }
                    for (from_x, from_y, to_x, to_y) in arrows {
                        {
                            let mid_x = (from_x + to_x) / 2.0;
                            rsx! {
                                path {
                                    d: "M {from_x} {from_y} C {mid_x} {from_y} {mid_x} {to_y} {to_x} {to_y}",
                                    stroke: "#888",
                                    stroke_width: "1.5",
                                    fill: "none",
                                    marker_end: "url(#arrowhead)",
                                }
                            }
                        }
                    }
                }
                // Task bars
                for (i, task) in tasks.iter().enumerate() {
                    TaskBar {
                        key: "{task.id}",
                        task: task.clone(),
                        timeline_start: start,
                        row_index: i,
                        on_tasks_changed: on_tasks_changed,
                        on_edit: move |data: (Task, f64, f64)| {
                            editing_task.set(Some(data));
                        },
                    }
                }
                // Edit popover
                if let Some((task, px, py)) = editing_task() {
                    EditPopover {
                        task: task.clone(),
                        position_x: px,
                        position_y: py,
                        on_close: move |_| editing_task.set(None),
                        on_tasks_changed: on_tasks_changed,
                    }
                }
            }
        }
    }
}
