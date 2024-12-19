use anyhow::Result;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::io::{self, BufRead};

type Input = (Vec<String>, Vec<String>);

fn parse_input(mut reader: impl BufRead) -> Result<Input> {
    let mut lines = reader.lines();

    let available_s = lines.next().unwrap()?;

    let available = available_s.split(", ").map(str::to_owned).collect();

    let blank = lines.next().unwrap().unwrap();
    assert!(blank.is_empty());

    let displays = lines.map(|l| l.unwrap()).collect();

    Ok((available, displays))
}

fn is_display_possible(d: &str, towels: &[String]) -> bool {
    if d.is_empty() {
        return true;
    }

    for towel in towels {
        if d.starts_with(towel) && is_display_possible(&d[towel.len()..], towels) {
            return true;
        }
    }

    false
}

fn count_possible_arrangements_with_simple_cache_inner(
    d: &str,
    towels: &[String],
    cache: &mut HashMap<String, usize>,
) -> usize {
    if d.is_empty() {
        return 1;
    }

    if let Some(hit) = cache.get(d).copied() {
        return hit;
    }

    let mut count = 0;

    for towel in towels {
        if d.starts_with(towel) {
            count += count_possible_arrangements_with_simple_cache_inner(
                &d[towel.len()..],
                towels,
                cache,
            );
        }
    }

    cache.insert(d.to_string(), count);

    count
}

fn count_possible_arrangements_with_simple_cache(d: &str, towels: &[String]) -> usize {
    let mut cache = HashMap::new();
    count_possible_arrangements_with_simple_cache_inner(d, towels, &mut cache)
}

fn main() -> Result<()> {
    let (towels, displays) = {
        let stdin = io::stdin();
        let stdin_lock = stdin.lock();
        parse_input(stdin_lock)?
    };

    dbg!(&towels, &displays);

    let part1 = displays
        .iter()
        .filter(|d| is_display_possible(d, &towels))
        .count();

    println!("part1: {}", part1);

    let part2: usize = displays
        .par_iter()
        .map(|d| count_possible_arrangements_with_simple_cache(d, &towels))
        .sum();

    println!("part2: {}", part2);

    Ok(())
}
