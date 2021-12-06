use std::{
    io::{self, BufRead},
};

type FishCounts = Vec<usize>;

fn parse_fishies(mut reader: impl BufRead) -> Vec<usize> {
    let mut counts_str = String::new();
    reader.read_to_string(&mut counts_str).unwrap();
    counts_str.trim().split(",").map(|age_str| usize::from_str_radix(age_str.trim(), 10).unwrap()).collect()
}

fn main() {
    let fish = {
        let stdin = io::stdin();
        let stdin_lock = stdin.lock();
        parse_fishies(stdin_lock)
    };


    let mut fish_count_by_tts: [usize; 9] = [0; 9];

    for fish_tts in fish.into_iter() {
        fish_count_by_tts[fish_tts] += 1;
    }

    dbg!(fish_count_by_tts);

    for day in 0..256 {
        let mut new_fish_count_by_tts: [usize; 9] = [0; 9];
        let spawns = fish_count_by_tts[0];

        new_fish_count_by_tts[6] = spawns;
        new_fish_count_by_tts[8] = spawns;

        for i in 1..9 {
            new_fish_count_by_tts[i-1] += fish_count_by_tts[i];
        }

        fish_count_by_tts = new_fish_count_by_tts;
        dbg!(fish_count_by_tts);
    }

    let total_fish: usize = fish_count_by_tts.iter().copied().sum();

    println!("total fish: {}", total_fish);
}




#[cfg(test)]
mod test {
    use std::io::Cursor;

    use crate::{parse_fishies};

    fn get_test_input() -> Vec<usize> {
        let test_data_str = include_str!("../test_input.txt");

        let test_data_reader = Cursor::new(test_data_str.to_owned());

        parse_fishies(test_data_reader)
    }

    #[test]
    fn test_parse() {
        let fish = get_test_input();

        assert_eq!(fish, vec![3,4,3,1,2]);
    }
}
