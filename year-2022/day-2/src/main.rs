use std::io::{self, BufRead};
use std::str::FromStr;
use anyhow::anyhow;

#[derive(Debug, PartialEq, Copy, Clone)]
enum RPS {
    Rock,
    Paper,
    Scissors
}

impl FromStr for RPS {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "A" | "X" => Ok(RPS::Rock),
            "B" | "Y" => Ok(RPS::Paper),
            "C" | "Z" => Ok(RPS::Scissors),
            _ => Err(anyhow!("Invalid rock paper scissor value"))
        }
    }
}

type Input = Vec<(RPS, RPS)>;

fn parse_input(mut reader: impl BufRead) -> Input {
    reader.lines().map(|l| l.unwrap()).map(|line| {
        let (left, right) = line.split_once(' ').unwrap();

        (RPS::from_str(left).unwrap(), RPS::from_str(right).unwrap())
    }).collect()
}

fn score_strat(strat: (RPS, RPS)) -> u32 {
    let outcome_score = match strat {
        (RPS::Rock, RPS::Rock) => 3,
        (RPS::Rock, RPS::Paper) => 6,
        (RPS::Rock, RPS::Scissors) => 0,
        (RPS::Paper, RPS::Rock) => 0,
        (RPS::Paper, RPS::Paper) => 3,
        (RPS::Paper, RPS::Scissors) => 6,
        (RPS::Scissors, RPS::Rock) => 6,
        (RPS::Scissors, RPS::Paper) => 0,
        (RPS::Scissors, RPS::Scissors) => 3,
    };

    let play_score: u32 = match strat.1 {
        RPS::Rock => 1,
        RPS::Paper => 2,
        RPS::Scissors => 3,
    };

    return play_score + outcome_score;
}

fn get_guide_score(strats: &[(RPS, RPS)]) -> u32 {
    strats.iter().copied().map(score_strat).sum()
}

fn decrypt_strat(strat: (RPS, RPS)) -> (RPS, RPS) {
    let m = match strat {
        (RPS::Rock, RPS::Rock) => RPS::Scissors,
        (RPS::Rock, RPS::Paper) => RPS::Rock,
        (RPS::Rock, RPS::Scissors) => RPS::Paper,
        (RPS::Paper, RPS::Rock) => RPS::Rock,
        (RPS::Paper, RPS::Paper) => RPS::Paper,
        (RPS::Paper, RPS::Scissors) => RPS::Scissors,
        (RPS::Scissors, RPS::Rock) => RPS::Paper,
        (RPS::Scissors, RPS::Paper) => RPS::Scissors,
        (RPS::Scissors, RPS::Scissors) => RPS::Rock,
    };

    (strat.0, m)
}

fn decrypt_guide(strats: &[(RPS, RPS)]) -> Vec<(RPS, RPS)> {
    strats.iter().copied().map(decrypt_strat).collect()
}

fn main() {
    let input = {
        let stdin = io::stdin();
        let stdin_lock = stdin.lock();
        parse_input(stdin_lock)
    };

    dbg!(&input);

    dbg!(get_guide_score(&input));

    let decrypted = decrypt_guide(&input);

    dbg!(get_guide_score(&decrypted));
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use crate::{parse_input, Input, RPS, get_guide_score, decrypt_strat, decrypt_guide};

    fn get_test_input() -> Input {
        let test_data_str = include_str!("../test_input.txt");

        let test_data_reader = Cursor::new(test_data_str.to_owned());

        parse_input(test_data_reader)
    }

    #[test]
    fn test_parse() {
        let test_data = get_test_input();

        assert_eq!(test_data, vec![
            (RPS::Rock, RPS::Paper),
            (RPS::Paper, RPS::Rock),
            (RPS::Scissors, RPS::Scissors)
        ]);

        assert_eq!(get_guide_score(test_data.as_slice()), 15);

        let decrypted = decrypt_guide(&test_data);

        dbg!(&decrypted);

        assert_eq!(get_guide_score(decrypted.as_slice()), 12);
    }
}
