use std::io;

use anyhow::Result;
use day_11::{memoized_run_stones, parse_input};



fn main() -> Result<()> {
    let nums = {
        let stdin = io::stdin();
        let stdin_lock = stdin.lock();
        parse_input(stdin_lock)?
    };

    dbg!(&nums);

    let part1_total = memoized_run_stones(&nums, 25);

    println!("part1: {}", part1_total);

    let part2_total = memoized_run_stones(&nums, 75);

    println!("part2: {}", part2_total);

    Ok(())
}
