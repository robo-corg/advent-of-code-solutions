use std::io::{self, BufRead};
use std::mem;

type Input = Vec<Vec<i64>>;

fn parse_input(mut reader: impl BufRead) -> Input {
    let mut elves = Vec::new();

    let mut cur_elf = Vec::new();

    for line in reader.lines().map(|l| l.unwrap()) {
        if line.len() == 0 {
            elves.push(mem::replace(&mut cur_elf, Vec::new()));
        } else {
            cur_elf.push(i64::from_str_radix(&line, 10).unwrap());
        }
    }

    elves.push(mem::replace(&mut cur_elf, Vec::new()));

    elves
}

fn main() {
    let input = {
        let stdin = io::stdin();
        let stdin_lock = stdin.lock();
        parse_input(stdin_lock)
    };

    let mut elf_sizes: Vec<i64> = input.iter().map(|elf| elf.iter().sum::<i64>()).collect();

    let big_elf = elf_sizes
        .iter()
        .copied()
        .enumerate()
        .max_by_key(|(i, s)| *s);

    // Part 1
    println!("Largest elf: {:?}", big_elf);

    // Part 2
    elf_sizes.sort();
    elf_sizes.reverse();
    println!(
        "Top 3 elves: {:?}",
        elf_sizes[0] + elf_sizes[1] + elf_sizes[2]
    );
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
