use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Task {
    pub id: String,
    pub name: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub progress: f32,
    pub dependencies: Vec<String>,
    pub color: String,
    pub sort_order: i32,
}

impl Task {
    pub fn duration_days(&self) -> i64 {
        (self.end_date - self.start_date).num_days().max(1)
    }
}
