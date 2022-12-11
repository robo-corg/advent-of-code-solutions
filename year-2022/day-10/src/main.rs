use std::io::{self, BufRead};
use std::iter;
use std::str::FromStr;

#[derive(Debug, Copy, Clone, PartialEq)]
enum Instruction {
    AddX(i32),
    Noop,
}

impl FromStr for Instruction {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "noop" {
            Ok(Instruction::Noop)
        } else {
            let (inst_s, amount_s) = s.split_once(' ').unwrap();
            if inst_s != "addx" {
                return Err(anyhow::anyhow!("Invalid instruction {}", inst_s));
            }

            return Ok(Instruction::AddX(i32::from_str_radix(amount_s, 10)?));
        }
    }
}

type Input = Vec<Instruction>;

fn parse_input(mut reader: impl BufRead) -> Input {
    reader
        .lines()
        .map(|l| l.unwrap())
        .map(|line| Instruction::from_str(&line).unwrap())
        .collect()
}

struct VM {
    x: i32,
}

impl VM {
    fn new() -> Self {
        VM { x: 1 }
    }

    fn exec<'a>(&'a mut self, instructions: &'a [Instruction]) -> impl Iterator<Item = i32> + 'a {
        let mut maybe_pending_inst = None;
        let mut next_inst = instructions.iter().copied();

        iter::from_fn(move || {
            match maybe_pending_inst {
                Some(Instruction::AddX(pending_amount)) => {
                    self.x += pending_amount;
                    maybe_pending_inst = None;
                    return Some(self.x);
                }
                Some(Instruction::Noop) => panic!("noop should not be delayed"),
                None => {}
            }

            match next_inst.next() {
                Some(Instruction::Noop) => Some(self.x),
                Some(delayed_inst) => {
                    maybe_pending_inst = Some(delayed_inst);
                    Some(self.x)
                }
                None => None,
            }
        })
    }

    fn exec2(&mut self, instructions: &[Instruction]) -> impl Iterator<Item = i32> {
        let mut values = Vec::new();

        for inst in instructions {
            match inst {
                Instruction::AddX(amount) => {
                    values.push(self.x);
                    self.x += amount;
                    values.push(self.x);
                },
                Instruction::Noop => {
                    values.push(self.x);
                },
            }
            dbg!(values.len(), inst, self.x);
        }

        values.into_iter()
    }
}

fn draw(sprite_positions: &[i32]) {
    for (cycle, p) in sprite_positions.iter().copied().enumerate() {
        let crt_pos = (cycle as i32) % 40;

        if crt_pos == 0 {
            println!("");
        }

        if crt_pos >= (p - 1) && crt_pos <= (p + 1) {
            print!("#");
        }
        else {
            print!(" ");
        }
    }
}

fn main() {
    let input = {
        let stdin = io::stdin();
        let stdin_lock = stdin.lock();
        parse_input(stdin_lock)
    };

    dbg!(&input);

    let mut vm = VM::new();

    let signal: Vec<i32> = vm
         .exec2(&input)
         .collect();

    dbg!(&signal);

    dbg!(signal[18]);
    dbg!(signal[58]);
    dbg!(signal[98]);
    dbg!(signal[138]);
    dbg!(signal[178]);
    dbg!(signal[218]);

    // let signal_samples: Vec<i32> = signal.iter().copied()
    //     .enumerate()
    //     .filter_map(|(cycle_minus_one, x)| {
    //         let cycle = (cycle_minus_one + 1) as i32;
    //         if ((cycle - 20) % 40) == 0 {
    //             Some((cycle as i32) * x)
    //         } else {
    //             None
    //         }
    //     })
    //     .take(6)
    //     .collect();

    //dbg!(&signal_samples);

    //let sum: i32 = signal_samples.iter().copied().sum();

    dbg!(signal[18]*20 + signal[58]*60 + signal[98]*100 + signal[138]*140 + signal[178] * 180 + signal[218] * 220);

    let mut padded_signal = vec![1];
    padded_signal.extend(&signal);

    draw(&padded_signal);
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use crate::{parse_input, Input, Instruction};

    fn get_test_input() -> Input {
        let test_data_str = include_str!("../test_input.txt");

        let test_data_reader = Cursor::new(test_data_str.to_owned());

        parse_input(test_data_reader)
    }

    #[test]
    fn test_parse() {
        let test_data = get_test_input();
        assert_eq!(test_data[0], Instruction::AddX(15));
    }
}
