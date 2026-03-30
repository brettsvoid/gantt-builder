use chrono::{Datelike, NaiveDate, Weekday};
use dioxus::prelude::*;

use crate::models::Task;

const PX_PER_DAY: f64 = 36.0;

pub fn compute_date_range(tasks: &[Task]) -> (NaiveDate, NaiveDate) {
    if tasks.is_empty() {
        let today = chrono::Local::now().date_naive();
        return (today, today + chrono::Duration::days(30));
    }
    let min_date = tasks.iter().map(|t| t.start_date).min().unwrap();
    let max_date = tasks.iter().map(|t| t.end_date).max().unwrap();
    (
        min_date - chrono::Duration::days(2),
        max_date + chrono::Duration::days(5),
    )
}

pub fn date_to_x(date: NaiveDate, timeline_start: NaiveDate) -> f64 {
    let diff = (date - timeline_start).num_days() as f64;
    diff * PX_PER_DAY
}

pub fn px_per_day() -> f64 {
    PX_PER_DAY
}

#[component]
pub fn TimelineHeader(start: NaiveDate, end: NaiveDate) -> Element {
    let total_days = (end - start).num_days();
    let total_width = total_days as f64 * PX_PER_DAY;

    let mut months = Vec::new();
    let mut current = start;
    while current <= end {
        let month_start = current;
        let month_end = if current.month() == 12 {
            NaiveDate::from_ymd_opt(current.year() + 1, 1, 1).unwrap()
        } else {
            NaiveDate::from_ymd_opt(current.year(), current.month() + 1, 1).unwrap()
        };
        let display_end = if month_end > end { end } else { month_end };
        let x = date_to_x(month_start, start);
        let width = date_to_x(display_end, start) - x;
        months.push((month_start, x, width));
        current = month_end;
    }

    let mut days = Vec::new();
    let mut d = start;
    while d <= end {
        let x = date_to_x(d, start);
        let is_weekend = d.weekday() == Weekday::Sat || d.weekday() == Weekday::Sun;
        days.push((d, x, is_weekend));
        d += chrono::Duration::days(1);
    }

    rsx! {
        div {
            class: "timeline-header",
            style: "width: {total_width}px;",
            div { class: "timeline-months",
                for (date, x, width) in months {
                    div {
                        class: "timeline-month",
                        style: "left: {x}px; width: {width}px;",
                        "{date.format(\"%b %Y\")}"
                    }
                }
            }
            div { class: "timeline-days",
                for (date, x, is_weekend) in days {
                    div {
                        class: if is_weekend { "timeline-day weekend" } else { "timeline-day" },
                        style: "left: {x}px; width: {PX_PER_DAY}px;",
                        "{date.format(\"%d\")}"
                    }
                }
            }
        }
    }
}
