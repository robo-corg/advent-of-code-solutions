use anyhow::Result;
use rayon::prelude::*;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::io::{self, BufRead};
use std::iter::once;

type Pos = nalgebra::Point2<i64>;
type Vec2 = nalgebra::Vector2<i64>;

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
                //eprintln!("{}: ({:?}, {}, {}) <{:?}>", self.ip, inst, self.registers[0], self.get_combo(oper), self.registers);
                self.registers[0] = dv(self.registers[0], self.get_combo(oper));
            }
            Inst::Bxl => {
                //eprintln!("{}: ({:?}, {}, {}) <{:?}>", self.ip, inst, self.registers[1], oper, self.registers);
                self.registers[1] = xl(self.registers[1], oper);
            }
            Inst::Bst => {
                //eprintln!("{}: ({:?}, {}, {}) <{:?}>", self.ip, inst, self.registers[1], self.get_combo(oper), self.registers);
                self.registers[1] = st(self.registers[1], self.get_combo(oper));
            }
            Inst::Jnz => {
                //eprintln!("{}: ({:?}, {}) <{:?}>", self.ip, inst, self.registers[0], self.registers);
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
    (reg >> oper)
}

/// The bxl instruction (opcode 1) calculates the bitwise XOR of register B and the instruction's literal operand, then stores the result in register B.
fn xl(reg: i64, oper: i64) -> i64 {
    (reg ^ oper)
}

/// The bst instruction (opcode 2) calculates the value of its combo operand modulo 8 (thereby keeping only its lowest 3 bits), then writes that value to the B register.
fn st(_reg: i64, oper: i64) -> i64 {
    oper & 0b111
}

fn xc(reg1: i64, reg2: i64) -> i64 {
    (reg1 ^ reg2)
}

fn parse_register(s: &str) -> i64 {
    let re = Regex::new(r"Register [A-Z]: (\d+)").unwrap();

    let (_, [reg]) = re.captures(s).unwrap().extract();

    i64::from_str_radix(reg, 10).unwrap()
}

fn parse_input(mut reader: impl BufRead) -> Result<Input> {
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

fn find_quine(machine: &Machine) -> i64 {
    (35184372088832..i64::MAX)
        .into_par_iter()
        .find_map_first(|reg_a| {
            let mut m = machine.clone();

            m.registers[0] = reg_a;

            let mut expected_out = machine.program.iter();

            for s in 0..1000 {
                if s >= 999 {
                    eprintln!("{}, {}", s, reg_a);
                }

                let (halt, maybe_out) = m.step();

                if let Some(out) = maybe_out {
                    let expected = expected_out.next().copied();

                    if expected != Some(out) {
                        return None;
                    }
                }

                if halt {
                    break;
                }
            }

            if expected_out.next().is_none() {
                return Some(reg_a);
            }

            None
        })
        .unwrap()
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



    // let part_2_reg_a = find_quine(&machine);

    // println!("part2: {}", part_2_reg_a);
    let a_seq = find_inv_for_program(&machine.program, machine.clone()).unwrap();

    let mut a_part_2 = 0;

    for (n, a_item) in a_seq.iter().copied().enumerate() {
        a_part_2 |= a_item;
    }

    dbg!(a_seq);

    println!("part2: {}", a_part_2);

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

// part1: 7,2,4,7,0,3,7,1,3

fn prog(init_a: i64) -> Vec<i64> {
    let mut a = init_a;
    let mut b = 0;
    let mut c = 0;

    let mut out = Vec::new();

    loop {
        b = st(0, a);
        b = xl(b, 2);
        c = dv(a, b);
        b = xc(b, c);
        b = xl(b, 3);
        out.push(b % 8);
        a = dv(a, 3);

        if a == 0 {
            break;
        }
    }

    out
}

fn forward_b(a: i64) -> (i64, i64) {
    let mut b = a & 0b111; //st(0, a);
    b = b ^ 2; // xl(b, 2);
               //c = a >> b; //dv(a, b);
    //dbg!(b);
    let bits = b;
    b = b ^ (a >> b); //xc(b, c);
    b = b ^ 3; //xl(b, 3);

    (b, bits)
}

// fn inv(out_b: i64, out_b_next: Option<i64>) -> i64 {
//     for a in 0..65536 {
//         let a1 = a;
//         let a2 = a >> 3;

//         if let Some(out_b_next) = out_b_next {
//             if forward_b(a2) != out_b_next {
//                 continue;
//             }
//         }

//         if forward_b(a1) == out_b {
//             return a;
//         }
//     }

//     panic!("no a solution");
// }

//  too low: 192348148

fn find_inv_for_program(inputs: &[i64], mut machine: Machine) -> Option<Vec<i64>> {
    if inputs.is_empty() {
        return Some(Vec::new());
    }

    dbg!(inputs);

    'outer: for a in 0..0xFFFFFF {
        machine.reset();
        machine.registers[0] = a;

        let outs = machine.run_to_halt();


        if outs.len() == 0 {
            continue 'outer;
        }

        let mut next_inputs = inputs;

        let mut a_consume = 0;

        for out in outs.iter().copied() {
            if out != next_inputs[0] {
                continue 'outer;
            }

            next_inputs = &next_inputs[1..];
            a_consume += 1;
        }

        dbg!(a, a_consume, machine.registers[0]);

        let mut sol = Vec::new();

        sol.push(a);

        if let Some(next_sol) = find_inv_for_program(next_inputs, machine.clone()) {
            //sol.extend(next_sol);
            for s in next_sol {
                sol.push(s << (a_consume * 3));
            }
            return Some(sol);
        }
    }   None
}

// fn find_inv_for_program(p: &[i64]) -> Option<Vec<i64>> {
//     if p.is_empty() {
//         return Some(Vec::new());
//     }

//     dbg!(p);

//     'outer: for a in 0..32 {
//         let a1 = a;

//         //dbg!(a);

//         // if let Some(out_b_next) = out_b_next {
//         //     if forward_b(a2) != out_b_next {
//         //         continue;
//         //     }
//         // }

//         //dbg!(forward_b(a1));


//         let mut sol = vec![a1];


//         let mut p_next = p;

//         let mut a_remain = a;

//         let mut remain_bits = 0;

//         for n in 0..6 {
//             if (a & (0b111 << (n*3))) != 0 {
//                 remain_bits += 3;
//             }
//         }

//         dbg!(a);

//         loop {
//             a_remain = a_remain >> 3;
//             if p_next.len() < 1 {
//                 continue 'outer;
//             }

//             let masked_remain = a_remain & 0b111;

//             //dbg!(forward_b(a_remain));

//             let (b, bits_used) = forward_b(a_remain);

//             dbg!(a_remain, b, bits_used, remain_bits);

//             remain_bits = i64::max(0, remain_bits + bits_used - 3);

//             //dbg!(remain_bits);

//             if b == p[0] {
//                 sol.push(masked_remain);
//                 p_next = &p[1..];
//             }
//             else {
//                 continue 'outer;
//             }

//             if remain_bits == 0 {
//                 //dbg!(a_remain);
//                 break;
//             }
//         }

//         if let Some(next_sol) = find_inv_for_program(p_next) {
//             sol.extend(next_sol);
//             return Some(sol);
//         }
//     }

//     None
// }

fn prog_inline(init_a: i64) -> Vec<i64> {
    let mut a = init_a;
    let mut b = 0;
    let mut c = 0;

    let mut out = Vec::new();

    loop {
        // b = a & 0b111; //st(0, a);
        // b = b ^ 2; // xl(b, 2);
        //            //c = a >> b; //dv(a, b);
        // b = b ^ (a >> b); //xc(b, c);
        // b = b ^ 3; //xl(b, 3);
        b = forward_b(a).0;
        out.push(b % 8);
        a = a >> 3; //dv(a, 3);

        if a == 0 {
            break;
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use crate::{prog, prog_inline, Machine};

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
