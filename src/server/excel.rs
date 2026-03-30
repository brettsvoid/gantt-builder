use calamine::{open_workbook_auto_from_rs, Data, Reader};
use chrono::NaiveDate;
use rust_xlsxwriter::Workbook;
use std::io::Cursor;

use crate::models::Task;

/// Resolve dependency names to task IDs after all tasks are created.
fn resolve_dependencies(tasks: &mut [Task], dep_strings: &[String]) {
    let name_to_id: std::collections::HashMap<String, String> = tasks
        .iter()
        .map(|t| (t.name.clone(), t.id.clone()))
        .collect();

    for (task, dep_str) in tasks.iter_mut().zip(dep_strings.iter()) {
        if !dep_str.is_empty() {
            task.dependencies = dep_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter_map(|name| name_to_id.get(&name).cloned())
                .collect();
        }
    }
}

// -- CSV --

pub fn import_csv(bytes: &[u8]) -> Result<Vec<Task>, String> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(bytes);

    let mut tasks = Vec::new();
    let mut dep_strings = Vec::new();
    let mut next_id = 1u32;

    for result in reader.records() {
        let record = result.map_err(|e| format!("CSV parse error: {e}"))?;

        let name = record.get(0).unwrap_or("").trim().to_string();
        if name.is_empty() {
            continue;
        }

        let start_date = record
            .get(1)
            .and_then(|s| NaiveDate::parse_from_str(s.trim(), "%Y-%m-%d").ok())
            .ok_or(format!("Invalid start date for task '{name}'"))?;

        let end_date = record
            .get(2)
            .and_then(|s| NaiveDate::parse_from_str(s.trim(), "%Y-%m-%d").ok())
            .ok_or(format!("Invalid end date for task '{name}'"))?;

        let progress = record
            .get(3)
            .and_then(|s| s.trim().parse::<f32>().ok())
            .unwrap_or(0.0)
            .clamp(0.0, 1.0);

        let dep_str = record.get(4).unwrap_or("").trim().to_string();
        dep_strings.push(dep_str);

        let color = record
            .get(5)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "#4a90d9".into());

        tasks.push(Task {
            id: next_id.to_string(),
            name,
            start_date,
            end_date,
            progress,
            dependencies: vec![],
            color,
            sort_order: (next_id - 1) as i32,
        });
        next_id += 1;
    }

    resolve_dependencies(&mut tasks, &dep_strings);
    Ok(tasks)
}

// -- Excel (xlsx/xls) --

fn excel_date_to_naive(serial: f64) -> Option<NaiveDate> {
    let epoch = NaiveDate::from_ymd_opt(1899, 12, 30)?;
    epoch.checked_add_signed(chrono::Duration::days(serial as i64))
}

fn parse_date_cell(cell: &Data) -> Option<NaiveDate> {
    match cell {
        Data::DateTime(dt) => excel_date_to_naive(dt.as_f64()),
        Data::Float(f) => excel_date_to_naive(*f),
        Data::String(s) => NaiveDate::parse_from_str(s, "%Y-%m-%d").ok(),
        _ => None,
    }
}

pub fn import_xlsx(bytes: &[u8]) -> Result<Vec<Task>, String> {
    let cursor = Cursor::new(bytes);
    let mut workbook =
        open_workbook_auto_from_rs(cursor).map_err(|e| format!("Cannot open workbook: {e}"))?;

    let sheet_name = workbook
        .sheet_names()
        .first()
        .cloned()
        .ok_or("No sheets found")?;

    let range = workbook
        .worksheet_range(&sheet_name)
        .map_err(|e| format!("Cannot read sheet: {e}"))?;

    let mut tasks = Vec::new();
    let mut dep_strings = Vec::new();
    let mut next_id = 1u32;

    for row in range.rows().skip(1) {
        let name = match row.first() {
            Some(Data::String(s)) if !s.is_empty() => s.clone(),
            _ => continue,
        };

        let start_date = row
            .get(1)
            .and_then(parse_date_cell)
            .ok_or(format!("Invalid start date for task '{name}'"))?;

        let end_date = row
            .get(2)
            .and_then(parse_date_cell)
            .ok_or(format!("Invalid end date for task '{name}'"))?;

        let progress = match row.get(3) {
            Some(Data::Float(f)) => *f as f32,
            Some(Data::Int(i)) => *i as f32,
            _ => 0.0,
        }
        .clamp(0.0, 1.0);

        let dep_str = match row.get(4) {
            Some(Data::String(s)) => s.clone(),
            _ => String::new(),
        };
        dep_strings.push(dep_str);

        let color = match row.get(5) {
            Some(Data::String(s)) if !s.is_empty() => s.clone(),
            _ => "#4a90d9".into(),
        };

        tasks.push(Task {
            id: next_id.to_string(),
            name,
            start_date,
            end_date,
            progress,
            dependencies: vec![],
            color,
            sort_order: (next_id - 1) as i32,
        });
        next_id += 1;
    }

    resolve_dependencies(&mut tasks, &dep_strings);
    Ok(tasks)
}

pub fn export_xlsx(tasks: &[Task]) -> Result<Vec<u8>, String> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    let headers = [
        "Name",
        "Start Date",
        "End Date",
        "Progress",
        "Dependencies",
        "Color",
    ];
    for (col, header) in headers.iter().enumerate() {
        worksheet
            .write_string(0, col as u16, *header)
            .map_err(|e| e.to_string())?;
    }

    let id_to_name: std::collections::HashMap<&str, &str> = tasks
        .iter()
        .map(|t| (t.id.as_str(), t.name.as_str()))
        .collect();

    for (i, task) in tasks.iter().enumerate() {
        let row = (i + 1) as u32;
        worksheet
            .write_string(row, 0, &task.name)
            .map_err(|e| e.to_string())?;
        worksheet
            .write_string(row, 1, task.start_date.format("%Y-%m-%d").to_string())
            .map_err(|e| e.to_string())?;
        worksheet
            .write_string(row, 2, task.end_date.format("%Y-%m-%d").to_string())
            .map_err(|e| e.to_string())?;
        worksheet
            .write_number(row, 3, task.progress as f64)
            .map_err(|e| e.to_string())?;

        let dep_names: Vec<&str> = task
            .dependencies
            .iter()
            .filter_map(|id| id_to_name.get(id.as_str()).copied())
            .collect();
        worksheet
            .write_string(row, 4, dep_names.join(", "))
            .map_err(|e| e.to_string())?;
        worksheet
            .write_string(row, 5, &task.color)
            .map_err(|e| e.to_string())?;
    }

    let buf = workbook.save_to_buffer().map_err(|e| e.to_string())?;
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_csv() -> &'static [u8] {
        b"Name,Start Date,End Date,Progress,Dependencies,Color\n\
          Alpha,2026-04-01,2026-04-07,0.5,,#aabbcc\n\
          Beta,2026-04-05,2026-04-14,0.0,Alpha,#112233\n"
    }

    fn sample_tasks() -> Vec<Task> {
        vec![
            Task {
                id: "1".into(),
                name: "Alpha".into(),
                start_date: NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(),
                end_date: NaiveDate::from_ymd_opt(2026, 4, 7).unwrap(),
                progress: 0.5,
                dependencies: vec![],
                color: "#aabbcc".into(),
                sort_order: 0,
            },
            Task {
                id: "2".into(),
                name: "Beta".into(),
                start_date: NaiveDate::from_ymd_opt(2026, 4, 5).unwrap(),
                end_date: NaiveDate::from_ymd_opt(2026, 4, 14).unwrap(),
                progress: 0.0,
                dependencies: vec!["1".into()],
                color: "#112233".into(),
                sort_order: 1,
            },
        ]
    }

    // -- CSV tests --

    #[test]
    fn csv_import_basic() {
        let tasks = import_csv(sample_csv()).unwrap();
        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].name, "Alpha");
        assert_eq!(tasks[1].name, "Beta");
        assert_eq!(
            tasks[0].start_date,
            NaiveDate::from_ymd_opt(2026, 4, 1).unwrap()
        );
        assert_eq!(tasks[0].progress, 0.5);
        assert_eq!(tasks[0].color, "#aabbcc");
    }

    #[test]
    fn csv_import_resolves_dependencies() {
        let tasks = import_csv(sample_csv()).unwrap();
        assert!(tasks[0].dependencies.is_empty());
        assert_eq!(tasks[1].dependencies, vec!["1"]);
    }

    #[test]
    fn csv_import_multi_dependencies() {
        let csv = b"Name,Start Date,End Date,Progress,Dependencies,Color\n\
                     A,2026-01-01,2026-01-05,0.0,,\n\
                     B,2026-01-02,2026-01-06,0.0,,\n\
                     C,2026-01-03,2026-01-07,0.0,\"A,B\",\n";
        let tasks = import_csv(csv).unwrap();
        assert_eq!(tasks[2].dependencies.len(), 2);
        assert!(tasks[2].dependencies.contains(&"1".to_string()));
        assert!(tasks[2].dependencies.contains(&"2".to_string()));
    }

    #[test]
    fn csv_import_skips_empty_rows() {
        let csv = b"Name,Start Date,End Date\n\
                     A,2026-01-01,2026-01-05\n\
                     ,,,\n\
                     B,2026-01-02,2026-01-06\n";
        let tasks = import_csv(csv).unwrap();
        assert_eq!(tasks.len(), 2);
    }

    #[test]
    fn csv_import_defaults_missing_fields() {
        let csv = b"Name,Start Date,End Date\n\
                     A,2026-01-01,2026-01-05\n";
        let tasks = import_csv(csv).unwrap();
        assert_eq!(tasks[0].progress, 0.0);
        assert_eq!(tasks[0].color, "#4a90d9");
        assert!(tasks[0].dependencies.is_empty());
    }

    #[test]
    fn csv_import_bad_date_returns_error() {
        let csv = b"Name,Start Date,End Date\n\
                     A,not-a-date,2026-01-05\n";
        assert!(import_csv(csv).is_err());
    }

    #[test]
    fn csv_import_clamps_progress() {
        let csv = b"Name,Start Date,End Date,Progress\n\
                     A,2026-01-01,2026-01-05,5.0\n\
                     B,2026-01-01,2026-01-05,-1.0\n";
        let tasks = import_csv(csv).unwrap();
        assert_eq!(tasks[0].progress, 1.0);
        assert_eq!(tasks[1].progress, 0.0);
    }

    #[test]
    fn csv_import_unresolvable_dep_is_dropped() {
        let csv = b"Name,Start Date,End Date,Progress,Dependencies\n\
                     A,2026-01-01,2026-01-05,0.0,NonExistent\n";
        let tasks = import_csv(csv).unwrap();
        assert!(tasks[0].dependencies.is_empty());
    }

    // -- Excel round-trip tests --

    #[test]
    fn xlsx_round_trip() {
        let original = sample_tasks();
        let bytes = export_xlsx(&original).unwrap();
        let imported = import_xlsx(&bytes).unwrap();

        assert_eq!(imported.len(), original.len());
        for (orig, imp) in original.iter().zip(imported.iter()) {
            assert_eq!(orig.name, imp.name);
            assert_eq!(orig.start_date, imp.start_date);
            assert_eq!(orig.end_date, imp.end_date);
            assert_eq!(orig.color, imp.color);
            assert!((orig.progress - imp.progress).abs() < 0.01);
        }
    }

    #[test]
    fn xlsx_round_trip_preserves_dependencies() {
        let original = sample_tasks();
        let bytes = export_xlsx(&original).unwrap();
        let imported = import_xlsx(&bytes).unwrap();

        assert!(imported[0].dependencies.is_empty());
        assert_eq!(imported[1].dependencies, vec!["1"]);
    }

    #[test]
    fn xlsx_export_empty_tasks() {
        let bytes = export_xlsx(&[]).unwrap();
        let imported = import_xlsx(&bytes).unwrap();
        assert!(imported.is_empty());
    }

    // -- Sample file test --

    #[test]
    fn sample_csv_is_valid() {
        let bytes = include_bytes!("../../examples/sample.csv") as &[u8];
        let tasks = import_csv(bytes).unwrap();
        assert!(!tasks.is_empty());
        // Verify dependency chain is resolved
        let planning = tasks.iter().find(|t| t.name == "Design Phase").unwrap();
        assert!(!planning.dependencies.is_empty());
    }
}
