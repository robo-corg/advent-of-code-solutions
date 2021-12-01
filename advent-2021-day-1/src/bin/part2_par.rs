use std::io::{self, BufRead};

use rayon::{slice::ParallelSlice, iter::ParallelIterator};


fn main() -> Result<(), anyhow::Error> {
    let lines: Vec<String> = {
        let stdin = io::stdin();
        let stdin_lock = stdin.lock();
        stdin_lock.lines().collect::<Result<Vec<_>, _>>()?
    };

    let depths: Vec<i64> = lines.into_iter().map(|s| {
        i64::from_str_radix(s.trim(), 10)
    }).collect::<Result<Vec<_>, _>>()?;

    //dbg!(&depths);

    let windowed_depths: Vec<i64> = depths.par_windows(3).map(|w| {
        w.iter().sum()
    }).collect();


    let (increases, _) = windowed_depths.iter().fold((0usize, None), |(count, maybe_last), next| {
        let next_count =
            match maybe_last {
                Some(last) if next > last => count.saturating_add(1),
                _ => count
            };

        (
            next_count,
            Some(next)
        )
    });


    // let (increases, _) = depths.par_windows(3).map(|w| {
    //     w.iter().sum()
    // }).fold_with((0usize, None), |(count, maybe_last): (usize, Option<i64>), next| {
    //     let next_count =
    //         match maybe_last {
    //             Some(last) if next > last => count.saturating_add(1),
    //             _ => count
    //         };

    //     (
    //         next_count,
    //         Some(next)
    //     )
    // }).reduce(|| (0usize, None), |a, b| {
    //     (
    //         a.0 + b.0 + match (a.1, b.1) {
    //             (Some(left), Some(right)) if right > left => 1,
    //             _ => 0
    //         },
    //         b.1
    //     )
    // });


    println!("{}", increases);

    Ok(())
}
