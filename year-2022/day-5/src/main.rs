use anyhow;
use std::io::{self, BufRead};
use std::str::FromStr;

#[derive(Debug, PartialEq, Clone)]
struct Stacks(Vec<Vec<char>>);

impl Stacks {
    fn parse<'a>(lines: impl Iterator<Item = &'a str>) -> Self {
        let boxes: Vec<Vec<char>> = lines.map(|line| {
            let mut row = Vec::new();

            for n in 0usize.. {
                let ind = n*4+1;
                if let Some(ch) = line.get(ind..ind+1) {
                    row.push(ch.chars().next().unwrap());
                }
                else {
                    break;
                }
            }

            row
        }).collect();

        let mut stacks = Vec::new();

        stacks.resize(boxes.last().unwrap().len(), Vec::new());


        for cur_boxes in boxes.iter().rev().skip(1) {
            for (col, cur_box) in cur_boxes.iter().enumerate() {
                if *cur_box != ' ' {
                    stacks[col].push(*cur_box);
                }
            }
        }

        Stacks(stacks)
    }

    fn execute(&mut self, command: &Command) {
        for _ in 0..command.quantity {
            if let Some(b) = self.0[command.from-1].pop() {
                self.0[command.to-1].push(b);
            }
        }
    }

    fn execute_all(&mut self, commands: &[Command]) {
        for cmd in commands.iter() {
            self.execute(cmd);
        }
    }

    fn execute_9001(&mut self, command: &Command) {
        let mut crates = Vec::new();
        for _ in 0..command.quantity {
            if let Some(b) = self.0[command.from-1].pop() {
                crates.push(b);
            }
        }

        crates.reverse();

        self.0[command.to-1].extend(crates);
    }

    fn execute_9001_all(&mut self, commands: &[Command]) {
        for cmd in commands.iter() {
            self.execute_9001(cmd);
        }
    }

    fn top(&self) -> Vec<Option<char>> {
        self.0.iter().map(|s| s.last().copied()).collect()
    }
}

#[derive(Debug, PartialEq)]
struct Command {
    quantity: usize,
    from: usize,
    to: usize,
}

impl FromStr for Command {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut command_parts = s.split(' ');

        let _action = command_parts.next().unwrap();
        let quantity_str = command_parts.next().unwrap();
        let _from_ident = command_parts.next().unwrap();
        let from_str = command_parts.next().unwrap();
        let _to_ident = command_parts.next().unwrap();
        let to_str = command_parts.next().unwrap();

        Ok(Command {
            quantity: usize::from_str_radix(quantity_str, 10)?,
            from: usize::from_str_radix(from_str, 10)?,
            to: usize::from_str_radix(to_str, 10)?,
        })
    }
}

type Input = (Stacks, Vec<Command>);

fn parse_input(mut reader: impl BufRead) -> Input {
    let mut lines = reader.lines().map(|l| l.unwrap());

    let stack_lines: Vec<String> = lines.by_ref().take_while(|l| l.len() > 0).collect();

    let commands: Vec<Command> = lines
        .map(|line| Command::from_str(&line).unwrap())
        .collect();

    (
        Stacks::parse(stack_lines.iter().map(|l| l.as_str())),
        commands,
    )
}

fn main() {
    let (mut stacks, commands) = {
        let stdin = io::stdin();
        let stdin_lock = stdin.lock();
        parse_input(stdin_lock)
    };

    let mut part1_stacks = stacks.clone();
    part1_stacks.execute_all(&commands);

    dbg!(part1_stacks.top());


    stacks.execute_9001_all(&commands);

    dbg!(&stacks);

    dbg!(stacks.top());
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use crate::{parse_input, Input, Command, Stacks};

    fn get_test_input() -> Input {
        let test_data_str = include_str!("../test_input.txt");

        let test_data_reader = Cursor::new(test_data_str.to_owned());

        parse_input(test_data_reader)
    }

    #[test]
    fn test_parse() {
        let (mut test_stacks, test_commands) = get_test_input();

        assert_eq!(test_stacks, Stacks(vec![vec!['Z', 'N'], vec!['M', 'C', 'D'], vec!['P']]));

        assert_eq!(
            test_commands,
            vec![
                Command {
                    quantity: 1,
                    from: 2,
                    to: 1
                },
                Command {
                    quantity: 3,
                    from: 1,
                    to: 3
                },
                Command {
                    quantity: 2,
                    from: 2,
                    to: 1
                },
                Command {
                    quantity: 1,
                    from: 1,
                    to: 2
                }
            ]
        );

        test_stacks.execute_all(&test_commands);
    }
}
