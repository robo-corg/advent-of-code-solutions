use std::collections::VecDeque;
use std::io::{self, Read};

use day_6::*;

type Input = String;

fn parse_input(mut reader: impl Read) -> Input {
    let mut buffer = String::new();
    reader.read_to_string(&mut buffer).unwrap();
    buffer
}


fn main() {
    let input = {
        let stdin = io::stdin();
        let stdin_lock = stdin.lock();
        parse_input(stdin_lock)
    };

    dbg!(&input);

    dbg!(scan_for_start_3(&input, 4));

    dbg!(scan_for_start_3(&input, 14));
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use crate::{parse_input, scan_for_start, Input};

    fn get_test_input() -> Input {
        let test_data_str = include_str!("../test_input.txt");

        let test_data_reader = Cursor::new(test_data_str.to_owned());

        parse_input(test_data_reader)
    }

    #[test]
    fn test_parse() {
        let test_data = get_test_input();

        assert_eq!(scan_for_start(&test_data, 4), 7);
    }
}
