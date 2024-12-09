use std::collections::HashMap;
use std::io::{self, BufRead};

fn main() -> anyhow::Result<()> {
    let stdin = io::stdin();

    let mut lhs_vec = Vec::new();
    let mut rhs_vec = Vec::new();

    for maybe_line in stdin.lock().lines() {
        let line = maybe_line?;

        if let Some((lhs_s, rhs_s)) = line.split_once("   ") {
            lhs_vec.push(i32::from_str_radix(lhs_s, 10).unwrap());
            rhs_vec.push(i32::from_str_radix(rhs_s, 10).unwrap());
        }
    }

    lhs_vec.sort();
    rhs_vec.sort();

    let total_distance: i32 = lhs_vec
        .iter()
        .copied()
        .zip(rhs_vec.iter().copied())
        .map(|(lhs, rhs)| i32::abs_diff(lhs, rhs) as i32)
        .sum();

    println!("total distance: {}", total_distance);

    let mut rhs_counts = HashMap::new();

    for item in rhs_vec.iter().copied() {
        let count = rhs_counts.entry(item).or_insert(0);
        *count += 1;
    }

    let similarity_score: i32 = lhs_vec.iter().copied().map(|lhs| {
        let count = rhs_counts.get(&lhs).copied().unwrap_or(0);
        count * lhs
    }).sum();

    println!("similarity score: {}", similarity_score);

    Ok(())
}
