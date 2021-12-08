use std::io::{self, BufRead};

type Input = Vec<i32>;

fn parse_input(mut reader: impl BufRead) -> Input {
    let mut counts_str = String::new();
    reader.read_to_string(&mut counts_str).unwrap();
    counts_str
        .trim()
        .split(",")
        .map(|s| i32::from_str_radix(s.trim(), 10).unwrap())
        .collect()
}

// fn crab_align_cost(crabs: &[i32], pos: i32) -> i32 {
//     crabs.iter().map(|c| i32::abs(c - pos)).sum()
// }

fn fuel_cost(d: i32) -> i32 {
    //(1..(d+1)).sum()

    d * (1 + d) / 2
}


fn crab_align_cost(crabs: &[i32], pos: i32) -> i32 {
    crabs.iter().map(|c| fuel_cost(i32::abs(c - pos))).sum()
}

fn find_nearest_crab(crabs: &[i32], pos: i32) -> (usize, usize) {
    assert!(crabs.len() > 0);

    if crabs[0] >= pos {
        return (0, 1);
    }

    if crabs[crabs.len()-1] <= pos {
        return (crabs.len()-1, crabs.len());
    }

    let half = crabs.len()/2;

    if crabs[half] < pos {
        let (s, e) = find_nearest_crab(&crabs[half..], pos);
        return (s+half, e+half);
    }
    else {
        return find_nearest_crab(&crabs[..half], pos);
    }
}

fn brute_force_find_best(crab_positions: &[i32]) -> Option<(i32, i32)> {
    crab_positions.iter().copied().max().and_then(|search_space|
        (0..search_space+1).map(|n|
            (n, crab_align_cost(&crab_positions, n))
        ).min_by_key(|x| x.1)
    )
}

/// Given a sorted partitioned list of crabs find local minima of the cost function
///
/// (x-x0)*(1+x-x0)
fn cost_derivative_between(left_crabs: &[i32], right_crabs: &[i32]) -> f32 {
    let left_sum: i32 = left_crabs.iter().copied().map(|pos| pos).sum();
    let left_count = left_crabs.len() as f32;

    let right_sum: i32 = right_crabs.iter().copied().map(|pos| pos).sum();
    let right_count = right_crabs.len() as f32;

    let left_d = left_sum as f32 - (left_count as f32)/2.0;
    let right_d = right_sum as f32 + (right_count as f32)/2.0;


    (left_d + right_d) / (left_count + right_count)
}

fn best_position_between(left_crabs: &[i32], right_crabs: &[i32]) -> Option<i32> {
    let maybe_left_pos = left_crabs.last().copied();
    let maybe_right_pos = right_crabs.first().copied();

    let guess= cost_derivative_between(left_crabs, right_crabs);

    if let Some(left_pos) = maybe_left_pos {
        if guess < (left_pos as f32) {
            return None;
        }
    }

    if let Some(right_pos) = maybe_right_pos {
        if guess > (right_pos as f32) {
            return None;
        }
    }


    let guess_int = f32::round(guess) as i32;

    dbg!(maybe_left_pos, guess, guess_int);

    //dbg!(cost_derivative_between(left_crabs, right_crabs, 5));

    Some(guess_int)
}

fn find_with_local_minima(crab_positions: &[i32]) {
    let mut sorted_crab_positions = crab_positions.to_vec();
    sorted_crab_positions.sort();

    let best_pivots: Vec<(usize, i32)> = (0..(sorted_crab_positions.len()+1)).filter_map(|i| {
        let left_crabs = &sorted_crab_positions[..i];
        let right_crabs = &sorted_crab_positions[i..];

        best_position_between(left_crabs, right_crabs).map(|p| (i, p))
    }).collect();
}

fn main() {
    let input = {
        let stdin = io::stdin();
        let stdin_lock = stdin.lock();
        parse_input(stdin_lock)
    };

    let mut crab_positions = input;

    crab_positions.sort();

    //dbg!(&crab_positions);

    let crab_sum: i32 = crab_positions.iter().copied().sum();
    let center_of_crab = crab_sum / (crab_positions.len() as i32);

    dbg!(center_of_crab);

    let best_pos = Some(center_of_crab);

    let best = brute_force_find_best(&crab_positions);


    // let sum_part: i32 = crab_positions.iter().map(|p_0| 2*p_0 - 1).sum();
    // dbg!(sum_part / (2*crab_positions.len() as i32));

    // let costs: Vec<i32> = (0..search_space).map(|n|
    //     crab_align_cost(&crab_positions, n)
    // ).collect();

    // for c in costs {
    //     println!("{}", c);
    // }

    let best_pivots: Vec<(usize, i32)> = (0..(crab_positions.len()+1)).filter_map(|i| {
        let left_crabs = &crab_positions[..i];
        let right_crabs = &crab_positions[i..];

        best_position_between(left_crabs, right_crabs).map(|p| (i, p))
    }).collect();

    dbg!(best_pivots);
    dbg!(crab_align_cost(&crab_positions, 4));
    dbg!(best);

    // let nearest = find_nearest_crab(&crab_positions, 5);
    // dbg!(nearest, crab_positions[nearest.0 as usize], crab_positions[nearest.1 as usize]);
}

#[cfg(test)]
mod test {
    use proptest::prelude::*;
    use std::iter::repeat;
    use std::io::Cursor;

    use crate::{parse_input, Input, brute_force_find_best};

    fn get_test_input() -> Input {
        let test_data_str = include_str!("../test_input.txt");

        let test_data_reader = Cursor::new(test_data_str.to_owned());

        parse_input(test_data_reader)
    }

    #[test]
    fn test_parse() {
        let test_data = get_test_input();
        assert_eq!(test_data, vec![16,1,2,0,4,2,7,1,2,14]);
    }

    prop_compose! {
        fn crabs_strategy(max_crab_count: usize)
            (crab_count in 1..max_crab_count)
            (crabs in prop::collection::vec(0..1000, 0..crab_count))
        -> Vec<i32> {
            crabs
        }
    }

    proptest! {
        #[test]
        fn test_brute_force_find_best(crabs in crabs_strategy(1000)) {
            let maybe_best = brute_force_find_best(&crabs);

            if crabs.len() > 0 {
                let best = maybe_best.unwrap();

                let min_crab_pos = crabs.iter().copied().min().unwrap();
                //let max_crab_pos = crabs.iter().copied().max().unwrap();

                assert!(best.0 >= 0);
                assert!(best.1 >= 0);

                assert!(best.0 >= min_crab_pos);
                //assert!(best.0 <= max_crab_pos);
            }
            else {
                assert_eq!(maybe_best, None);
            }
        }
    }
}
