use std::io::{self, BufRead};

pub type Input = Vec<i32>;

pub fn parse_input(mut reader: impl BufRead) -> Input {
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

pub fn crab_align_cost(crabs: &[i32], pos: i32) -> i32 {
    crabs.iter().map(|c| fuel_cost(i32::abs(c - pos))).sum()
}

/// TODO: Center of mass seems to be pretty accurate so we could probably do a binary to find
/// a good starting partition and work from there
fn find_nearest_crab(crabs: &[i32], pos: i32) -> (usize, usize) {
    assert!(crabs.len() > 0);

    if crabs[0] >= pos {
        return (0, 1);
    }

    if crabs[crabs.len() - 1] <= pos {
        return (crabs.len() - 1, crabs.len());
    }

    let half = crabs.len() / 2;

    if crabs[half] < pos {
        let (s, e) = find_nearest_crab(&crabs[half..], pos);
        return (s + half, e + half);
    } else {
        return find_nearest_crab(&crabs[..half], pos);
    }
}

pub fn brute_force_find_best(crab_positions: &[i32]) -> Option<(i32, i32)> {
    crab_positions
        .iter()
        .copied()
        .max()
        .and_then(|search_space| {
            (0..search_space + 1)
                .map(|n| (n, crab_align_cost(&crab_positions, n)))
                .min_by_key(|x| x.1)
        })
}

/// Given a sorted partitioned list of crabs find local minima of the cost function
///
/// Takes the basic arithmetic series sum that is our cost function with the abs() removed
/// for x > x0 (left partition: (x-x0)*(1+x-x0)
/// for x < x0 (right partition:(x-x0)*(1+x-x0)
///
/// Derivative of these is `x - x0 + 1/2` and `x - x0 + 1/2` respectively
///
/// The rate of change of our cost function is the rate of change of both of these summed over
/// all the crabs: sum(x - left_crabs[i] + 1/2) + sum(x - right_crabs[i] + 1/2)
///
/// With this we can solve for x to find the local minima which may not be valid
/// (though often times is super close) to the solution.
fn cost_derivative_between(left_crabs: &[i32], right_crabs: &[i32]) -> f32 {
    let left_sum: i32 = left_crabs.iter().copied().map(|pos| pos).sum();
    let left_count = left_crabs.len() as f32;

    let right_sum: i32 = right_crabs.iter().copied().map(|pos| pos).sum();
    let right_count = right_crabs.len() as f32;

    let left_d = left_sum as f32 - (left_count as f32) / 2.0;
    let right_d = right_sum as f32 + (right_count as f32) / 2.0;

    (left_d + right_d) / (left_count + right_count)
}

fn best_position_between(left_crabs: &[i32], right_crabs: &[i32]) -> Option<i32> {
    let maybe_left_pos = left_crabs.last().copied();
    let maybe_right_pos = right_crabs.first().copied();

    // TODO: For partitions with a small range its probably faster to just brute force check every value?
    let guess = cost_derivative_between(left_crabs, right_crabs);

    // If the minima picked is outside the defined region simply return None (no solution)
    // for this partitioning
    if let Some(left_pos) = maybe_left_pos {
        if guess < (left_pos as f32) - 0.5 {
            return None;
        }
    }

    if let Some(right_pos) = maybe_right_pos {
        if guess > (right_pos as f32) + 0.5 {
            return None;
        }
    }

    // Most minima don't cleanly lie on integers so round to the nearest (best) alternative
    let guess_int = f32::round(guess) as i32;

    Some(guess_int)
}

/// Examine the region between crabs where cost function is differentiable
///
/// We can rewrite `abs(p1 - p0)` into either `p1 - p0` or `p0 - p1` if we
/// know if p1 >= p0 or not. The set of positions where this does not change
/// is differentiable (unlike abs(p1 - p0))
///
/// This is O(n*log(n)) vs O(x*n) where x is the max crab position
pub fn find_with_local_minima(crab_positions: &[i32]) -> Option<(i32, i32)> {
    if crab_positions.len() == 0 {
        return None;
    }

    let mut sorted_crab_positions = crab_positions.to_vec();
    sorted_crab_positions.sort();

    // Examine every n+1 partitionings of the sorted crabs
    let best_pivots: Vec<(usize, i32)> = (0..(sorted_crab_positions.len() + 1))
        .filter_map(|i| {
            let left_crabs = &sorted_crab_positions[..i];
            let right_crabs = &sorted_crab_positions[i..];

            best_position_between(left_crabs, right_crabs).map(|p| (i, p))
        })
        .collect();

    //dbg!(&best_pivots);

    best_pivots
        .into_iter()
        .map(|(_, pos)| pos)
        .min()
        .map(|best_pos| (best_pos, crab_align_cost(&sorted_crab_positions, best_pos)))
}


#[cfg(test)]
mod test {
    use proptest::prelude::*;
    use std::io::Cursor;

    use crate::{brute_force_find_best, find_with_local_minima, parse_input, Input};

    fn get_test_input() -> Input {
        let test_data_str = include_str!("../test_input.txt");

        let test_data_reader = Cursor::new(test_data_str.to_owned());

        parse_input(test_data_reader)
    }

    #[test]
    fn test_parse() {
        let test_data = get_test_input();
        assert_eq!(test_data, vec![16, 1, 2, 0, 4, 2, 7, 1, 2, 14]);
    }

    #[test]
    fn test_brute_force_empty_crabs() {
        let empty_crabs = Vec::new();
        let actual = brute_force_find_best(&empty_crabs);
        assert_eq!(actual, None);
    }

    #[test]
    fn test_local_minima_empty_crabs() {
        let empty_crabs = Vec::new();
        let actual = find_with_local_minima(&empty_crabs);
        assert_eq!(actual, None);
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
            }
            else {
                assert_eq!(maybe_best, None);
            }
        }

        #[test]
        fn test_find_with_local_minima(crabs in crabs_strategy(1000)) {
            let maybe_best = brute_force_find_best(&crabs);
            let maybe_minima_best = find_with_local_minima(&crabs);

            assert_eq!(maybe_minima_best.map(|b|b.1), maybe_best.map(|b|b.1));
        }
    }
}
