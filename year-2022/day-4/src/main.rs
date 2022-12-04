use std::io::{self, BufRead};
use std::str::FromStr;

#[derive(Debug, PartialEq)]
struct Range(i32, i32);

impl FromStr for Range {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (st, end) = s.split_once('-').unwrap();

        Ok(Range(
            i32::from_str_radix(st, 10).unwrap(),
            i32::from_str_radix(end, 10).unwrap(),
        ))
    }
}

impl Range {
    fn partial_contains(&self, other: &Range) -> bool {
        (self.0 <= other.0 && other.0 <= self.1) || (self.0 <= other.1 && other.1 <= self.1)
    }

    fn fully_contains(&self, other: &Range) -> bool {
        self.0 <= other.0 && self.1 >= other.1
    }

    fn either_contains(&self, other: &Range) -> bool {
        self.fully_contains(other) || other.fully_contains(self)
    }
}

type Input = Vec<(Range, Range)>;

fn parse_input(mut reader: impl BufRead) -> Input {
    reader
        .lines()
        .map(|l| l.unwrap())
        .map(|line| {
            let (lhs_str, rhs_str) = line.split_once(',').unwrap();

            (
                Range::from_str(lhs_str).unwrap(),
                Range::from_str(rhs_str).unwrap(),
            )
        })
        .collect()
}

fn main() {
    let input = {
        let stdin = io::stdin();
        let stdin_lock = stdin.lock();
        parse_input(stdin_lock)
    };

    dbg!(&input);

    let contians_count = input
        .iter()
        .filter(|pair| pair.0.either_contains(&pair.1))
        .count();

    dbg!(contians_count);

    let partial_contians_count = input
        .iter()
        .filter(|pair| pair.0.partial_contains(&pair.1) || pair.1.partial_contains(&pair.0))
        .count();

    dbg!(partial_contians_count);
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use crate::{parse_input, Input, Range};

    fn get_test_input() -> Input {
        let test_data_str = include_str!("../test_input.txt");

        let test_data_reader = Cursor::new(test_data_str.to_owned());

        parse_input(test_data_reader)
    }

    #[test]
    fn test_parse() {
        let test_data = get_test_input();

        assert_eq!(
            test_data,
            vec![
                (Range(2, 4), Range(6, 8)),
                (Range(2, 3), Range(4, 5)),
                (Range(5, 7), Range(7, 9)),
                (Range(2, 8), Range(3, 7)),
                (Range(6, 6), Range(4, 6)),
                (Range(2, 6), Range(4, 8))
            ]
        );

        assert!(Range(2, 8).fully_contains(&Range(3, 7)));

        assert!(Range(5, 7).partial_contains(&Range(7, 9)));
        assert!(Range(7, 9).partial_contains(&Range(5, 7)));
    }
}
