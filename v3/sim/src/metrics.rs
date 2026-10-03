//! CSV helpers for training logs.

pub fn csv_header(cols: &[&str]) -> String {
    format!("{}\n", cols.join(","))
}

pub fn csv_row(vals: &[f32]) -> String {
    let v: Vec<String> = vals.iter().map(|x| format!("{}", x)).collect();
    format!("{}\n", v.join(","))
}
