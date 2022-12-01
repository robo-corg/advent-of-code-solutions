use std::io::{self, BufRead};

fn main() -> Result<(), anyhow::Error> {
    let lines: Vec<String> = {
        let stdin = io::stdin();
        let stdin_lock = stdin.lock();
        stdin_lock.lines().collect::<Result<Vec<_>, _>>()?
    };

    let depths: Vec<i64> = lines
        .into_iter()
        .map(|s| i64::from_str_radix(s.trim(), 10))
        .collect::<Result<Vec<_>, _>>()?;

    //dbg!(&depths);

    let windowed_depths: Vec<i64> = depths.windows(3).map(|w| w.iter().sum()).collect();

    //dbg!(&windowed_depths);

    let (increases, _) =
        windowed_depths
            .iter()
            .fold((0usize, None), |(count, maybe_last), next| {
                let next_count = match maybe_last {
                    Some(last) if next > last => count.saturating_add(1),
                    _ => count,
                };

                (next_count, Some(next))
            });

    println!("{}", increases);

    Ok(())
}
