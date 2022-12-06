use std::collections::VecDeque;
use std::io::{self, Read};

type Input = String;

fn parse_input(mut reader: impl Read) -> Input {
    let mut buffer = String::new();
    reader.read_to_string(&mut buffer).unwrap();
    buffer
}

fn scan_for_start(packet: &str, sz: usize) -> usize {
    // queue is most recently seen first so we can easily truncate off
    // the start of message seq
    let mut start_seq = VecDeque::with_capacity(sz);

    for (cur_pos, ch) in packet.chars().enumerate() {
        let maybe_duplicate_index = start_seq.iter().copied().position(|seq_ch| seq_ch == ch);

        // Remove the oldest part of the sequence up to and including the duplicating character
        if let Some(duplicate_index) = maybe_duplicate_index {
            start_seq.truncate(duplicate_index);
        }

        start_seq.push_front(ch);

        if start_seq.len() == sz {
            return cur_pos + 1;
        }
    }

    panic!("Did not find start");
}

fn main() {
    let input = {
        let stdin = io::stdin();
        let stdin_lock = stdin.lock();
        parse_input(stdin_lock)
    };

    dbg!(&input);

    dbg!(scan_for_start(&input, 4));

    dbg!(scan_for_start(&input, 14));
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
