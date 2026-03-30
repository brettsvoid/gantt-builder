use chrono::NaiveDate;
use dioxus::prelude::*;

use crate::components::timeline::{date_to_x, px_per_day};
use crate::models::Task;
use crate::server::api;

#[component]
pub fn TaskBar(
    task: Task,
    timeline_start: NaiveDate,
    row_index: usize,
    on_tasks_changed: EventHandler<Vec<Task>>,
    on_edit: EventHandler<(Task, f64, f64)>,
) -> Element {
    let x = date_to_x(task.start_date, timeline_start);
    let width = task.duration_days() as f64 * px_per_day();
    let y = row_index as f64 * 44.0;
    let progress_width = width * task.progress as f64;

    let mut dragging = use_signal(|| false);
    let mut drag_type = use_signal(|| DragType::None);
    let mut drag_start_x = use_signal(|| 0.0);
    let mut original_start = use_signal(|| task.start_date);
    let mut original_end = use_signal(|| task.end_date);

    let mut current_x = use_signal(|| x);
    let mut current_width = use_signal(|| width);

    if !dragging() {
        current_x.set(x);
        current_width.set(width);
    }

    let display_x = if dragging() { current_x() } else { x };
    let display_width = if dragging() { current_width() } else { width };

    let progress_style = format!(
        "width: {}px;",
        if dragging() {
            current_width() * task.progress as f64
        } else {
            progress_width
        }
    );

    let handle_pointerdown_move = {
        let task = task.clone();
        move |evt: PointerEvent| {
            evt.prevent_default();
            dragging.set(true);
            drag_type.set(DragType::Move);
            drag_start_x.set(evt.client_coordinates().x);
            original_start.set(task.start_date);
            original_end.set(task.end_date);
        }
    };

    let handle_pointerdown_resize_right = move |evt: PointerEvent| {
        evt.prevent_default();
        evt.stop_propagation();
        dragging.set(true);
        drag_type.set(DragType::ResizeRight);
        drag_start_x.set(evt.client_coordinates().x);
        original_end.set(task.end_date);
    };

    let handle_pointerdown_resize_left = {
        let task = task.clone();
        move |evt: PointerEvent| {
            evt.prevent_default();
            evt.stop_propagation();
            dragging.set(true);
            drag_type.set(DragType::ResizeLeft);
            drag_start_x.set(evt.client_coordinates().x);
            original_start.set(task.start_date);
        }
    };

    let task_for_pointermove = task.clone();
    let handle_pointermove = move |evt: PointerEvent| {
        if !dragging() {
            return;
        }
        let delta_px = evt.client_coordinates().x - drag_start_x();
        let delta_days = (delta_px / px_per_day()).round() as i64;

        match drag_type() {
            DragType::Move => {
                let new_x =
                    date_to_x(original_start(), timeline_start) + delta_days as f64 * px_per_day();
                current_x.set(new_x);
            }
            DragType::ResizeRight => {
                let orig_width = (original_end() - task_for_pointermove.start_date).num_days()
                    as f64
                    * px_per_day();
                let new_width = (orig_width + delta_days as f64 * px_per_day()).max(px_per_day());
                current_width.set(new_width);
            }
            DragType::ResizeLeft => {
                let orig_x = date_to_x(original_start(), timeline_start);
                let new_x = orig_x + delta_days as f64 * px_per_day();
                let orig_right = date_to_x(task_for_pointermove.end_date, timeline_start);
                let new_width = (orig_right - new_x).max(px_per_day());
                current_x.set(new_x);
                current_width.set(new_width);
            }
            DragType::None => {}
        }
    };

    let task_for_pointerup = task.clone();
    let handle_pointerup = move |evt: PointerEvent| {
        if !dragging() {
            return;
        }
        let delta_px = evt.client_coordinates().x - drag_start_x();
        let delta_days = (delta_px / px_per_day()).round() as i64;

        let mut updated = task_for_pointerup.clone();

        match drag_type() {
            DragType::Move => {
                updated.start_date = original_start() + chrono::Duration::days(delta_days);
                updated.end_date = original_end() + chrono::Duration::days(delta_days);
            }
            DragType::ResizeRight => {
                let new_end = original_end() + chrono::Duration::days(delta_days);
                if new_end > updated.start_date {
                    updated.end_date = new_end;
                }
            }
            DragType::ResizeLeft => {
                let new_start = original_start() + chrono::Duration::days(delta_days);
                if new_start < updated.end_date {
                    updated.start_date = new_start;
                }
            }
            DragType::None => {}
        }

        dragging.set(false);
        drag_type.set(DragType::None);

        let on_changed = on_tasks_changed;
        spawn(async move {
            if api::update_task(updated).await.is_ok() {
                if let Ok(tasks) = api::get_tasks().await {
                    on_changed.call(tasks);
                }
            }
        });
    };

    let task_for_dblclick = task.clone();

    rsx! {
        div {
            class: "task-bar-wrapper",
            style: "left: {display_x}px; top: {y}px; width: {display_width}px;",
            if dragging() {
                div {
                    class: "drag-overlay",
                    onpointermove: handle_pointermove,
                    onpointerup: handle_pointerup,
                }
            }
            div {
                class: "resize-handle resize-left",
                onpointerdown: handle_pointerdown_resize_left,
            }
            div {
                class: "task-bar",
                style: "background-color: {task.color};",
                onpointerdown: handle_pointerdown_move,
                ondoubleclick: move |evt| {
                    on_edit.call((task_for_dblclick.clone(), evt.client_coordinates().x, evt.client_coordinates().y));
                },
                div {
                    class: "task-progress",
                    style: "{progress_style}",
                }
                span { class: "task-bar-label", "{task.name}" }
            }
            div {
                class: "resize-handle resize-right",
                onpointerdown: handle_pointerdown_resize_right,
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum DragType {
    None,
    Move,
    ResizeLeft,
    ResizeRight,
}
