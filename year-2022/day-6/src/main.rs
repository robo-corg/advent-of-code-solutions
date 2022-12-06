use std::io::{self, BufRead};

type Input = String;

fn parse_input(mut reader: impl BufRead) -> Input {
    reader.lines().map(|l| l.unwrap()).next().unwrap()
}

fn scan_for_start(packet: &str, sz: usize) -> usize {
    for i in 0..packet.len() - sz {
        let start = &packet[i..i + sz];

        let mut found = true;

        for n in 0usize..sz {
            for m in n + 1usize..sz {
                if start[n..n + 1] == start[m..m + 1] {
                    found = false;
                }
            }
        }

        if found {
            return i + sz;
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

        assert_eq!(scan_for_start(&test_data), 0);
    }
}
