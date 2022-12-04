use std::collections::HashSet;
use std::io::{self, BufRead};

type Input = Vec<(String, String)>;

fn parse_input(mut reader: impl BufRead) -> Input {
    reader.lines().map(|l| l.unwrap()).map(|line| {
        let half = line.len()/2;
        (line[0..half].to_string(), line[half..].to_string())
    }).collect()
}

fn score(ch: char) -> u32 {
    if ch.is_alphabetic() && ch.is_ascii_lowercase() {
        return (ch as u32) - ('a' as u32) + 1;
    }
    else {
        return (ch as u32) - ('A' as u32) + 27;
    }
}

fn main() {
    let input = {
        let stdin = io::stdin();
        let stdin_lock = stdin.lock();
        parse_input(stdin_lock)
    };

    dbg!(&input);

    dbg!(score_all(&input));

    let badge_scores: u32 = input.chunks(3).map(find_common_badge).map(score).sum();

    dbg!(badge_scores);
}

fn find_common(pair: &(String, String)) -> char {
    for a in pair.0.chars() {
        for b in pair.1.chars() {
            if a == b {
                dbg!(a);
                return a;
            }
        }
    }

    panic!("No common");
}

fn find_common_badge(elves: &[(String, String)]) -> char {
    let sets: Vec<HashSet<char>> = elves.iter().map(|p|
        p.0.chars().chain(p.1.chars()).collect()
    ).collect();

    let int1: HashSet<char> = sets[0].intersection(&sets[1]).copied().collect();

    let common = int1.intersection(&sets[2]);

    common.copied().next().unwrap()
}

fn score_all(packages: &[(String, String)]) -> u32 {
    packages.iter().map(|p| dbg!(score(find_common(p)))).sum()
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use crate::{parse_input, Input, find_common, score, score_all, find_common_badge};

    fn get_test_input() -> Input {
        let test_data_str = include_str!("../test_input.txt");

        let test_data_reader = Cursor::new(test_data_str.to_owned());

        parse_input(test_data_reader)
    }

    #[test]
    fn test_parse() {
        let test_data = get_test_input();

        assert_eq!(test_data[0], ("vJrwpWtwJgWr".to_owned(), "hcsFMMfFFhFp".to_owned()));

        assert_eq!(find_common(&test_data[0]), 'p');

        assert_eq!(score('b'), 2);

        assert_eq!(score_all(&test_data), 157);

        assert_eq!(find_common_badge(&test_data[..3]), 'r');
    }
}
