use std::io;

use day_7::{parse_input, brute_force_find_best, find_with_local_minima};


fn main() {
    let input = {
        let stdin = io::stdin();
        let stdin_lock = stdin.lock();
        parse_input(stdin_lock)
    };

    let crab_positions = input;

    let maybe_brute_force_best = brute_force_find_best(&crab_positions);

    if let Some(brute_foce_best) = maybe_brute_force_best {
        println!(
            "brute force best postion: {} cost: {}",
            brute_foce_best.0, brute_foce_best.1
        );
    }

    let maybe_minima_best = find_with_local_minima(&crab_positions);

    if let Some(maybe_minima_best) = maybe_minima_best {
        println!(
            "minima best position: {} cost: {}",
            maybe_minima_best.0, maybe_minima_best.1
        );
    }
}
