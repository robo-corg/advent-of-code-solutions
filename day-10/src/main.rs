use std::{io::{self, BufRead}, ops::Index};

type Input = Vec<Vec<Token>>;

fn parse_input(mut reader: impl BufRead) -> Input {
    reader.lines().map(|l| l.unwrap().chars().map(Token::parse).collect()).collect()
}

const OPENERS: [char; 4] = ['(', '[', '{', '<'];
const CLOSERS: [char; 4] = [')', ']', '}', '>'];
const SCORE: [u32; 4] = [
    3,
    57,
    1197,
    25137
];

#[derive(Copy, Clone)]
enum Token {
    Open(u8),
    Close(u8)
}

impl Token {
    fn parse(ch: char) -> Self {
        if let Some(opener_index) = OPENERS.iter().position(|v| *v == ch) {
            Token::Open(opener_index as u8)
        }
        else if let Some(closer_index) = CLOSERS.iter().position(|v| *v == ch) {
            Token::Close(closer_index as u8)
        }
        else {
            panic!("Unknown character `{}` encountered", ch);
        }
    }

    fn closer(&self) -> Option<u8> {
        if let Token::Close(index) = self {
            Some(*index)
        }
        else {
            None
        }
    }

    fn part_1_score(&self) -> u32 {
        match self {
            Token::Open(_) => 0,
            Token::Close(closer) => {
                SCORE[*closer as usize]
            }
        }
    }

    fn part_2_score(&self) -> u64 {
        match self {
            Token::Open(_) => 0,
            Token::Close(closer) => *closer as u64 + 1
        }
    }
}

fn main() {
    let input = {
        let stdin = io::stdin();
        let stdin_lock = stdin.lock();
        parse_input(stdin_lock)
    };

    let score: u32 = input.iter().filter_map(|line| {
        let mut expected = Vec::new();

        for token in line.iter().copied() {
            if let Token::Open(opener_index) = token {
                expected.push(opener_index);
            }
            else if let Some(expected_index) = expected.pop() {
                if token.closer().unwrap() != expected_index {
                    return Some(token.part_1_score());
                }
            }
        }

        None
    }).sum();

    println!("score part 1: {}", score);

    let mut scores_part_2: Vec<u64> = input.iter().filter_map(|line| {
        let mut expected = Vec::new();

        let mut score = 0;

        for token in line.iter().copied() {
            if let Token::Open(opener_index) = token {
                expected.push(opener_index);
            }
            else if let Some(expected_index) = expected.pop() {
                if token.closer().unwrap() != expected_index {
                    return None;
                }
            }
        }

        while let Some(expected_index) = expected.pop() {
            let completion_token = Token::Close(expected_index);
            let completion_value = completion_token.part_2_score();

            score *= 5;
            score += completion_value;
        }

        if score > 0 { Some(score) } else { None }
    }).collect();

    scores_part_2.sort();

    dbg!(&scores_part_2);

    let score_part_2 = scores_part_2[scores_part_2.len()/2];

    println!("score part 2: {}", score_part_2);

}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use crate::{parse_input, Input};

    fn get_test_input() -> Input {
        let test_data_str = include_str!("../test_input.txt");

        let test_data_reader = Cursor::new(test_data_str.to_owned());

        parse_input(test_data_reader)
    }

    #[test]
    fn test_parse() {
        let test_data = get_test_input();
    }
}
