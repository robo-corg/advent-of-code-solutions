use anyhow::Result;
use regex::Regex;
use std::io::{self, BufRead};

type Input = Machine;

#[derive(Debug, Clone, Copy)]
enum Inst {
    Adv,
    Bxl,
    Bst,
    Jnz,
    Bxc,
    Out,
    Bdv,
    Cdv,
}

impl Inst {
    fn decode(op_code: i64) -> Self {
        match op_code {
            0 => Inst::Adv,
            1 => Inst::Bxl,
            2 => Inst::Bst,
            3 => Inst::Jnz,
            4 => Inst::Bxc,
            5 => Inst::Out,
            6 => Inst::Bdv,
            7 => Inst::Cdv,
            bad => panic!("Invalid op code: {:?}", bad),
        }
    }
}

#[derive(Debug, Clone)]
struct Machine {
    registers: [i64; 3],
    ip: i64,
    program: Vec<i64>,
}

impl Machine {
    fn fetch_inst(&self) -> (i64, i64) {
        let ip = self.ip as usize;
        (self.program[ip], self.program[ip + 1])
    }

    fn get_combo(&self, oper: i64) -> i64 {
        match oper {
            n @ (0..4) => n,
            r @ (4..7) => self.registers[(r - 4) as usize],
            bad => panic!("Invalid combo oper:{}", bad),
        }
    }

    fn step(&mut self) -> (bool, Option<i64>) {
        let (op_cod, oper) = self.fetch_inst();
        let inst = Inst::decode(op_cod);
        let mut maybe_out = None;

        match inst {
            Inst::Adv => {
                self.registers[0] = dv(self.registers[0], self.get_combo(oper));
            }
            Inst::Bxl => {
                self.registers[1] = xl(self.registers[1], oper);
            }
            Inst::Bst => {
                self.registers[1] = st(self.registers[1], self.get_combo(oper));
            }
            Inst::Jnz => {
                if self.registers[0] != 0 {
                    self.ip = oper - 2;
                }
            }
            Inst::Bxc => {
                self.registers[1] = xc(self.registers[1], self.registers[2]);
            }
            Inst::Out => {
                maybe_out = Some(self.get_combo(oper) % 8);
            }
            Inst::Bdv => {
                self.registers[1] = dv(self.registers[0], self.get_combo(oper));
            }
            Inst::Cdv => {
                self.registers[2] = dv(self.registers[0], self.get_combo(oper));
            }
        }

        self.ip += 2;

        (self.ip >= self.program.len() as i64, maybe_out)
    }

    fn run_to_halt(&mut self) -> Vec<i64> {
        let mut outs = Vec::new();

        loop {
            let (halt, maybe_out) = self.step();

            if let Some(out) = maybe_out {
                outs.push(out);
            }

            if halt {
                break;
            }
        }

        outs
    }

    fn reset(&mut self) {
        self.ip = 0;
        // for ele in self.registers.iter_mut() {
        //     *ele = 0;
        // }
    }

    fn remove_last(&self) -> Self {
        let mut new_self = self.clone();

        new_self.program.pop();
        new_self.program.pop();

        new_self
    }
}

/// The adv instruction (opcode 0) performs division. The numerator is the value in the A register. The denominator is found by raising 2 to the power of the instruction's combo operand. (So, an operand of 2 would divide A by 4 (2^2); an operand of 5 would divide A by 2^B.) The result of the division operation is truncated to an integer and then written to the A register.
fn dv(reg: i64, oper: i64) -> i64 {
    reg >> oper
}

/// The bxl instruction (opcode 1) calculates the bitwise XOR of register B and the instruction's literal operand, then stores the result in register B.
fn xl(reg: i64, oper: i64) -> i64 {
    reg ^ oper
}

/// The bst instruction (opcode 2) calculates the value of its combo operand modulo 8 (thereby keeping only its lowest 3 bits), then writes that value to the B register.
fn st(_reg: i64, oper: i64) -> i64 {
    oper & 0b111
}

fn xc(reg1: i64, reg2: i64) -> i64 {
    reg1 ^ reg2
}

fn parse_register(s: &str) -> i64 {
    let re = Regex::new(r"Register [A-Z]: (\d+)").unwrap();

    let (_, [reg]) = re.captures(s).unwrap().extract();

    i64::from_str_radix(reg, 10).unwrap()
}

fn parse_input(reader: impl BufRead) -> Result<Input> {
    let mut lines = reader.lines();

    let registers = [(); 3].map(|()| parse_register(&lines.next().unwrap().unwrap()));

    let _ = lines.next().unwrap().unwrap();

    let program_raw = lines.next().unwrap().unwrap();

    let program: Vec<i64> = program_raw["Program: ".len()..]
        .split(',')
        .map(|elem_s| i64::from_str_radix(elem_s, 10).unwrap())
        .collect();

    Ok(Machine {
        registers,
        ip: 0,
        program,
    })
}

fn main() -> Result<()> {
    let machine = {
        let stdin = io::stdin();
        let stdin_lock = stdin.lock();
        parse_input(stdin_lock)?
    };

    dbg!(&machine);

    let outs = machine.clone().run_to_halt();

    eprintln!("len={}", outs.len());

    let outs_s = itertools::join(&outs, ",");

    println!("part1: {}", outs_s);

    let a = find_inv_for_program(&machine.program, machine.clone()).unwrap();

    println!("part2: {}", a);

    Ok(())
}

// 2,4
// 1,2
// 7,5
// 4,7
// 1,3
// 5,5
// 0,3
// 3,0

// BST A
// BXL 2
// CDV B
// BXC
// BXL 3
// OUT B
// ADV 3
// JNZ

fn find_inv_for_program(inputs: &[i64], machine: Machine) -> Option<i64> {
    let mut inputs_rev = inputs.to_vec();
    inputs_rev.reverse();

    machine.remove_last();

    find_inv_for_program_inner(&inputs_rev, 0, machine)
}

fn find_inv_for_program_inner(
    inputs_rev: &[i64],
    existing_a: i64,
    mut machine: Machine,
) -> Option<i64> {
    if inputs_rev.len() == 0 {
        return Some(existing_a);
    }

    for new_a in 0..8 {
        let a = existing_a << 3 | new_a;

        machine.reset();
        machine.registers[0] = a;
        machine.registers[1] = 0;
        machine.registers[2] = 0;

        let outs = machine.run_to_halt();

        let out = outs[0];

        if inputs_rev[0] != out {
            continue;
        }

        if let Some(sol_rest) = find_inv_for_program_inner(&inputs_rev[1..], a, machine.clone()) {
            return Some(sol_rest);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use crate::{find_inv_for_program, prog, prog_inline, Machine};

    #[test]
    fn test_inv_program_sample() {
        let mut m = Machine {
            registers: [729, 0, 0],
            ip: 0,
            program: vec![0, 3, 5, 4, 3, 0],
        };

        let a = find_inv_for_program(&m.program, m.clone()).expect("Solution for a");

        m.registers[0] = a;

        let out = m.run_to_halt();

        assert_eq!(&m.program, &out);
    }

    #[test]
    fn test_inv_program_input() {
        let mut m = Machine {
            registers: [729, 0, 0],
            ip: 0,
            program: vec![2, 4, 1, 2, 7, 5, 4, 7, 1, 3, 5, 5, 0, 3, 3, 0],
        };

        let a = find_inv_for_program(&m.program, m.clone()).expect("Solution for a");

        m.registers[0] = a;

        let out = m.run_to_halt();

        assert_eq!(&m.program, &out);
    }

    #[test]
    fn test_prog_compat() {
        let mut m = Machine {
            registers: [729, 0, 0],
            ip: 0,
            program: vec![2, 4, 1, 2, 7, 5, 4, 7, 1, 3, 5, 5, 0, 3, 3, 0],
        };

        let outs = m.clone().run_to_halt();

        let prog_outs = prog_inline(729);

        assert_eq!(outs, prog_outs);
    }

    #[test]
    fn test_input() {
        let mut m = Machine {
            registers: [729, 0, 0],
            ip: 0,
            program: vec![0, 1, 5, 4, 3, 0],
        };

        let outs = m.run_to_halt();

        assert_eq!(outs, vec![4, 6, 3, 5, 6, 3, 5, 2, 1, 0]);
    }

    #[test]
    fn test_bst() {
        let mut m = Machine {
            registers: [0, 0, 9],
            ip: 0,
            program: vec![2, 6],
        };

        let s = m.step();

        assert_eq!(m.registers[1], 1);
    }

    #[test]
    fn test_prog1() {
        let mut m = Machine {
            registers: [10, 0, 0],
            ip: 0,
            program: vec![5, 0, 5, 1, 5, 4],
        };

        let outs = m.run_to_halt();

        assert_eq!(outs, vec![0, 1, 2]);
    }

    #[test]
    fn test_prog2() {
        let mut m = Machine {
            registers: [2024, 0, 0],
            ip: 0,
            program: vec![0, 1, 5, 4, 3, 0],
        };

        let outs = m.run_to_halt();

        assert_eq!(outs, vec![4, 2, 5, 6, 7, 7, 7, 7, 3, 1, 0]);
        assert_eq!(m.registers[0], 0);
    }

    #[test]
    fn test_bxl() {
        let mut m = Machine {
            registers: [0, 29, 0],
            ip: 0,
            program: vec![1, 7],
        };

        let s = m.step();

        assert_eq!(m.registers[1], 26);
    }

    #[test]
    fn test_bxc() {
        let mut m = Machine {
            registers: [0, 2024, 43690],
            ip: 0,
            program: vec![4, 0],
        };

        let s = m.step();

        assert_eq!(m.registers[1], 44354);
    }
}
