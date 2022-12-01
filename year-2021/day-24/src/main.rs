use anyhow::{anyhow, bail, Context, Error};
use rayon::prelude::*;

use std::{
    fmt,
    io::{self, BufRead},
    iter::repeat,
    mem,
    ops::Deref,
    str::FromStr,
    sync::Arc,
};

type Input = Vec<Inst>;

fn parse_input(mut reader: impl BufRead) -> Input {
    reader
        .lines()
        .map(|line| Inst::from_str(&line.unwrap()))
        .collect()
}

#[derive(Debug, Copy, Clone)]
struct Var(u8);

impl FromStr for Var {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Var(match s {
            "w" => 0,
            "x" => 1,
            "y" => 2,
            "z" => 3,
            other => bail!("Invalid var `{}`", other),
        }))
    }
}

#[derive(Debug, Copy, Clone)]
enum InstInput {
    Var(Var),
    Int(i64),
}

impl FromStr for InstInput {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Var::from_str(s).map(|v| InstInput::Var(v)).or_else(|_| {
            Ok(InstInput::Int(
                i64::from_str_radix(s, 10).context("Could not parse instruction input")?,
            ))
        })
    }
}

#[derive(Debug, Copy, Clone)]
enum Op {
    Add,
    Mul,
    Div,
    Mod,
    Eql,
}

impl FromStr for Op {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "add" => Op::Add,
            "mul" => Op::Mul,
            "div" => Op::Div,
            "mod" => Op::Mod,
            "eql" => Op::Eql,
            other => bail!("Invalid op name `{}`", other),
        })
    }
}

#[derive(Debug, Copy, Clone)]
enum Inst {
    Inp(Var),
    Op(Op, Var, InstInput),
}

impl Inst {
    fn from_str(s: &str) -> Self {
        let mut items = s.split(" ");

        let op = items.next().unwrap();

        if op == "inp" {
            return Inst::Inp(Var::from_str(items.next().unwrap()).unwrap());
        }

        let op = Op::from_str(op).unwrap();
        let lhs = Var::from_str(items.next().unwrap()).unwrap();
        let rhs = InstInput::from_str(items.next().unwrap()).unwrap();

        Inst::Op(op, lhs, rhs)
    }
}

struct Machine {
    registers: [i64; 4],
}

impl Machine {
    fn new() -> Self {
        Machine {
            registers: Default::default(),
        }
    }

    fn load(&self, r: Var) -> i64 {
        self.registers[r.0 as usize]
    }

    fn store(&mut self, r: Var, value: i64) {
        self.registers[r.0 as usize] = value;
    }

    fn load_or_get_lit(&self, r: InstInput) -> i64 {
        match r {
            InstInput::Var(var) => self.load(var),
            InstInput::Int(lit) => lit,
        }
    }

    fn execute(&mut self, program: &[Inst], inputs: &[i64]) {
        let mut inputs = inputs.iter();

        for inst in program {
            match inst {
                Inst::Inp(a) => self.store(*a, *inputs.next().unwrap()),
                Inst::Op(op, lhs, rhs) => {
                    let lhs_value = self.load(*lhs);
                    let rhs_value = self.load_or_get_lit(*rhs);

                    match op {
                        Op::Add => {
                            self.store(*lhs, lhs_value + rhs_value);
                        }
                        Op::Mul => {
                            self.store(*lhs, lhs_value * rhs_value);
                        }
                        Op::Div => {
                            self.store(*lhs, lhs_value / rhs_value);
                        }
                        Op::Mod => {
                            self.store(*lhs, lhs_value % rhs_value);
                        }
                        Op::Eql => {
                            self.store(*lhs, (lhs_value == rhs_value) as i64);
                        }
                    }
                }
            }
        }
    }
}

fn validate_model_number(program: &Vec<Inst>, model_number: &[i64]) -> bool {
    assert_eq!(model_number.len(), 14);

    let mut machine = Machine::new();

    machine.execute(&program, &model_number);

    machine.registers[3] == 0
}

// inp w
// mul x 0
// add x z
// mod x 26
// div z 1
// add x 12
// eql x w
// eql x 0
// mul y 0
// add y 25
// mul y x
// add y 1
// mul z y
// mul y 0
// add y w
// add y 7
// mul y x
// add z y

fn input_1(input: i64) {
    // inp w
    let w0 = input;
    // mul x 0
    // add x z
    // mod x 26
    // div z 1
    // add x 12
    let x0 = 12;
    // eql x w
    let x1 = (x0 == w0) as i64;
    // eql x 0
    let x2 = (x1 == 0) as i64;
    // mul y 0
    // add y 25
    let y0 = 25;
    // mul y x
    let y1 = y0 * x2;
    // add y 1
    let y2 = y1 + 1;
    // mul z y
    // mul y 0
    // add y w
    let y3 = w0;
    // add y 7
    let y4 = y3 + 7;
    // mul y x
    let y5 = y4 * x2;
    // add z y
    let z0 = y5;
}

#[derive(Clone, Debug)]
enum AstNode {
    Op(Op, Arc<AstNode>, Arc<AstNode>),
    Input(i32),
    Literal(i64),
}

impl fmt::Display for AstNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AstNode::Input(index) => write!(f, "input({})", index)?,
            AstNode::Op(Op::Add, lhs, rhs) => write!(f, "({} + {})", lhs, rhs)?,
            AstNode::Op(Op::Div, lhs, rhs) => write!(f, "({} / {})", lhs, rhs)?,
            AstNode::Op(Op::Eql, lhs, rhs) => write!(f, "({} == {})", lhs, rhs)?,
            AstNode::Op(Op::Mod, lhs, rhs) => write!(f, "({} % {})", lhs, rhs)?,
            AstNode::Op(Op::Mul, lhs, rhs) => write!(f, "({} * {})", lhs, rhs)?,
            AstNode::Literal(lit) => write!(f, "{}", lit)?,
        }

        Ok(())
    }
}

impl AstNode {
    fn lit(&self) -> Option<i64> {
        match self {
            AstNode::Literal(lit) => Some(*lit),
            _ => None,
        }
    }

    fn subs_dfs<F>(&self, query: &F) -> Option<AstNode>
    where
        F: Fn(&AstNode) -> Option<AstNode>,
    {
        match self {
            AstNode::Op(op, lhs, rhs) => {
                let maybe_lhs = lhs.subs_dfs(query).map(Arc::new);
                let maybe_rhs = rhs.subs_dfs(query).map(Arc::new);

                let children_dirty = maybe_lhs.is_some() || maybe_rhs.is_some();

                let lhs = maybe_lhs.unwrap_or(lhs.clone());
                let rhs = maybe_rhs.unwrap_or(rhs.clone());

                let new_node = AstNode::Op(*op, lhs, rhs);

                let maybe_new_node_subs = query(&new_node); //new_node.subs_dfs(query);

                if let Some(new_node_subs) = maybe_new_node_subs {
                    return Some(new_node_subs);
                }

                if children_dirty {
                    return Some(new_node);
                }
            }
            other => {
                return query(other);
            }
        }

        None
    }

    // fn constant_eval(&self) -> Option<AstNode> {
    //     match self {
    //         AstNode::Op(op, lhs, rhs) => {
    //             let lhs = lhs.constant_eval().map(Arc::new).unwrap_or(lhs.clone());
    //             let rhs = rhs.constant_eval().map(Arc::new).unwrap_or(rhs.clone());

    //             match (op, lhs.lit(), rhs.lit()) {
    //                 (Op::Div, Some(0), _) => Some(AstNode::Literal(0)),
    //                 (Op::Div, _, Some(1)) => Some(lhs.deref().clone()),
    //                 (Op::Mul, Some(0), _) => Some(AstNode::Literal(0)),
    //                 (Op::Mul, _, Some(0)) => Some(AstNode::Literal(0)),
    //                 (Op::Add, Some(0), _) => Some(rhs.deref().clone()),
    //                 (Op::Add, _, Some(0)) => Some(lhs.deref().clone()),
    //                 (Op::Mod, Some(0), _) => Some(AstNode::Literal(0)),
    //                 _ => None
    //             }
    //         },
    //         _ => None
    //     }
    // }

    fn constant_eval(&self) -> Option<AstNode> {
        self.subs_dfs(&|node| match node {
            AstNode::Op(op, lhs, rhs) => match (op, lhs.lit(), rhs.lit()) {
                (Op::Div, Some(0), _) => Some(AstNode::Literal(0)),
                (Op::Div, _, Some(1)) => Some(lhs.deref().clone()),
                (Op::Mul, Some(0), _) => Some(AstNode::Literal(0)),
                (Op::Mul, _, Some(0)) => Some(AstNode::Literal(0)),
                (Op::Add, Some(0), _) => Some(rhs.deref().clone()),
                (Op::Add, _, Some(0)) => Some(lhs.deref().clone()),
                (Op::Mod, Some(0), _) => Some(AstNode::Literal(0)),
                (Op::Add, Some(a), Some(b)) => Some(AstNode::Literal(a + b)),
                (Op::Mul, Some(a), Some(b)) => Some(AstNode::Literal(a * b)),
                (Op::Mod, Some(a), Some(b)) => Some(AstNode::Literal(a % b)),
                (Op::Div, Some(a), Some(b)) => Some(AstNode::Literal(a / b)),
                (Op::Eql, Some(a), Some(b)) => Some(AstNode::Literal((a == b) as i64)),
                _ => None,
            },
            _ => None,
        })
    }

    fn subs_input(&self, index: i32, replacement: i64) -> AstNode {
        let maybe_input_subs = self.subs_dfs(&|node| match node {
            AstNode::Input(i) if *i == index => Some(AstNode::Literal(replacement)),
            _ => None,
        });

        let with_input_subs = maybe_input_subs.unwrap_or(self.clone());

        let maybe_constant_eval = with_input_subs.constant_eval();
        let constant_eval = maybe_constant_eval.unwrap_or(with_input_subs);

        constant_eval
    }

    fn decompile(program: &[Inst]) -> AstNode {
        let mut registers = [
            AstNode::Literal(0),
            AstNode::Literal(0),
            AstNode::Literal(0),
            AstNode::Literal(0),
        ];

        let mut cur_input = 0;

        for inst in program {
            dbg!(cur_input);
            //println!("{}", &registers[3]);
            match inst {
                Inst::Inp(v) => {
                    registers[v.0 as usize] = AstNode::Input(cur_input);
                    cur_input += 1;
                }
                Inst::Op(op, lhs, rhs) => {
                    let store_register = lhs.0;
                    let rhs_val = match rhs {
                        InstInput::Var(r) => registers[r.0 as usize].clone(),
                        InstInput::Int(v) => AstNode::Literal(*v),
                    };
                    let lhs_val =
                        mem::replace(&mut registers[store_register as usize], AstNode::Literal(0));

                    let new_node = AstNode::Op(*op, Arc::new(lhs_val), Arc::new(rhs_val));

                    registers[store_register as usize] = new_node;
                }
            }

            for register in registers.iter_mut() {
                if let Some(folded) = register.constant_eval() {
                    *register = folded;
                }
            }
        }

        mem::replace(&mut registers[3], AstNode::Literal(0))
    }
}

macro_rules! inp {
    ($input:expr, $register:expr) => {
        $register = $input.next().unwrap();
    };
}

macro_rules! mul {
    ($a:expr, $b:expr) => {
        $a = ($a * $b);
    };
}

macro_rules! add {
    ($a:expr, $b:expr) => {
        $a = ($a + $b);
    };
}

macro_rules! mod_ {
    ($a:expr, $b:expr) => {
        $a = ($a % $b);
    };
}

macro_rules! div {
    ($a:expr, $b:expr) => {
        $a = ($a / $b);
    };
}

macro_rules! eql {
    ($a:expr, $b:expr) => {
        $a = ($a == $b) as i64;
    };
}

fn rust_program(mut input: impl Iterator<Item = i64>) -> i64 {
    let mut w = 0;

    let mut y = 0;
    let mut z = 0;
    {
        inp!(input, w);
        let mut x = 0;

        add!(x, z);
        mod_!(x, 26);
        div!(z, 1);
        add!(x, 12);
        eql!(x, w);
        eql!(x, 0);
        mul!(y, 0);
        add!(y, 25);
        mul!(y, x);
        add!(y, 1);
        mul!(z, y);
        mul!(y, 0);
        add!(y, w);
        add!(y, 7);
        mul!(y, x);
        add!(z, y);
    }

    let mut x = 0;

    inp!(input, w);
    mul!(x, 0);
    add!(x, z);
    mod_!(x, 26);
    div!(z, 1);
    add!(x, 12);
    eql!(x, w);
    eql!(x, 0);
    mul!(y, 0);
    add!(y, 25);
    mul!(y, x);
    add!(y, 1);
    mul!(z, y);
    mul!(y, 0);
    add!(y, w);
    add!(y, 8);
    mul!(y, x);
    add!(z, y);

    inp!(input, w);
    mul!(x, 0);
    add!(x, z);
    mod_!(x, 26);
    div!(z, 1);
    add!(x, 13);
    eql!(x, w);
    eql!(x, 0);
    mul!(y, 0);
    add!(y, 25);
    mul!(y, x);
    add!(y, 1);
    mul!(z, y);
    mul!(y, 0);
    add!(y, w);
    add!(y, 2);
    mul!(y, x);
    add!(z, y);

    inp!(input, w);
    mul!(x, 0);
    add!(x, z);
    mod_!(x, 26);
    div!(z, 1);
    add!(x, 12);
    eql!(x, w);
    eql!(x, 0);
    mul!(y, 0);
    add!(y, 25);
    mul!(y, x);
    add!(y, 1);
    mul!(z, y);
    mul!(y, 0);
    add!(y, w);
    add!(y, 11);
    mul!(y, x);
    add!(z, y);
    inp!(input, w);
    mul!(x, 0);
    add!(x, z);
    mod_!(x, 26);
    div!(z, 26);
    add!(x, -3);
    eql!(x, w);
    eql!(x, 0);
    mul!(y, 0);
    add!(y, 25);
    mul!(y, x);
    add!(y, 1);
    mul!(z, y);
    mul!(y, 0);
    add!(y, w);
    add!(y, 6);
    mul!(y, x);
    add!(z, y);
    inp!(input, w);
    mul!(x, 0);
    add!(x, z);
    mod_!(x, 26);
    div!(z, 1);
    add!(x, 10);
    eql!(x, w);
    eql!(x, 0);
    mul!(y, 0);
    add!(y, 25);
    mul!(y, x);
    add!(y, 1);
    mul!(z, y);
    mul!(y, 0);
    add!(y, w);
    add!(y, 12);
    mul!(y, x);
    add!(z, y);
    inp!(input, w);
    mul!(x, 0);
    add!(x, z);
    mod_!(x, 26);
    div!(z, 1);
    add!(x, 14);
    eql!(x, w);
    eql!(x, 0);
    mul!(y, 0);
    add!(y, 25);
    mul!(y, x);
    add!(y, 1);
    mul!(z, y);
    mul!(y, 0);
    add!(y, w);
    add!(y, 14);
    mul!(y, x);
    add!(z, y);
    inp!(input, w);
    mul!(x, 0);
    add!(x, z);
    mod_!(x, 26);
    div!(z, 26);
    add!(x, -16);
    eql!(x, w);
    eql!(x, 0);
    mul!(y, 0);
    add!(y, 25);
    mul!(y, x);
    add!(y, 1);
    mul!(z, y);
    mul!(y, 0);
    add!(y, w);
    add!(y, 13);
    mul!(y, x);
    add!(z, y);
    inp!(input, w);
    mul!(x, 0);
    add!(x, z);
    mod_!(x, 26);
    div!(z, 1);
    add!(x, 12);
    eql!(x, w);
    eql!(x, 0);
    mul!(y, 0);
    add!(y, 25);
    mul!(y, x);
    add!(y, 1);
    mul!(z, y);
    mul!(y, 0);
    add!(y, w);
    add!(y, 15);
    mul!(y, x);
    add!(z, y);
    inp!(input, w);
    mul!(x, 0);
    add!(x, z);
    mod_!(x, 26);
    div!(z, 26);
    add!(x, -8);
    eql!(x, w);
    eql!(x, 0);
    mul!(y, 0);
    add!(y, 25);
    mul!(y, x);
    add!(y, 1);
    mul!(z, y);
    mul!(y, 0);
    add!(y, w);
    add!(y, 10);
    mul!(y, x);
    add!(z, y);
    inp!(input, w);
    mul!(x, 0);
    add!(x, z);
    mod_!(x, 26);
    div!(z, 26);
    add!(x, -12);
    eql!(x, w);
    eql!(x, 0);
    mul!(y, 0);
    add!(y, 25);
    mul!(y, x);
    add!(y, 1);
    mul!(z, y);
    mul!(y, 0);
    add!(y, w);
    add!(y, 6);
    mul!(y, x);
    add!(z, y);
    inp!(input, w);
    mul!(x, 0);
    add!(x, z);
    mod_!(x, 26);
    div!(z, 26);
    add!(x, -7);
    eql!(x, w);
    eql!(x, 0);
    mul!(y, 0);
    add!(y, 25);
    mul!(y, x);
    add!(y, 1);
    mul!(z, y);
    mul!(y, 0);
    add!(y, w);
    add!(y, 10);
    mul!(y, x);
    add!(z, y);
    inp!(input, w);
    mul!(x, 0);
    add!(x, z);
    mod_!(x, 26);
    div!(z, 26);
    add!(x, -6);
    eql!(x, w);
    eql!(x, 0);
    mul!(y, 0);
    add!(y, 25);
    mul!(y, x);
    add!(y, 1);
    mul!(z, y);
    mul!(y, 0);
    add!(y, w);
    add!(y, 8);
    mul!(y, x);
    add!(z, y);
    inp!(input, w);
    mul!(x, 0);
    add!(x, z);
    mod_!(x, 26);
    div!(z, 26);
    add!(x, -11);
    eql!(x, w);
    eql!(x, 0);
    mul!(y, 0);
    add!(y, 25);
    mul!(y, x);
    add!(y, 1);
    mul!(z, y);
    mul!(y, 0);
    add!(y, w);
    add!(y, 5);
    mul!(y, x);
    add!(z, y);

    z
}

const x_add: [i64; 14] = [12, 12, 13, 12, -3, 10, 14, -16, 12, -8, -12, -7, -6, -11];

const add_y: [i64; 14] = [7, 8, 2, 11, 6, 12, 14, 13, 15, 10, 6, 10, 8, 5];

const div_z: [i64; 14] = [
    1,
    1,
    1,
    1,
    26,
    1,
    1,
    26,
    1,
    26,
    26,
    26,
    26,
    26,
];


fn rust_program_condensed(inputs_arr: &[i64]) -> i64 {
    let mut z = 0;


    for (n, w) in inputs_arr.iter().copied().enumerate()
    {
        let mut x = z;

        //dbg!(z);

        mod_!(x, 26);


        // most passes
        div!(z, div_z[n]);

        add!(x, x_add[n]);

        //dbg!(x);

        if x != w {
            z *= 26;
            z += w + add_y[n];
        }

        z %= 26i64.pow(7);
    }

    z
}

fn rust_program_condensed_mod26(inputs_arr: &[i64]) -> i64 {
    let mut z = 0;


    for (n, w) in inputs_arr.iter().copied().enumerate()
    {
        let mut x = z;

        //dbg!(z);

        mod_!(x, 26);


        // most passes
        div!(z, div_z[n]);

        add!(x, x_add[n]);

        //dbg!(x);

        if x != w {
            z *= 26;
            z += w + add_y[n];
        }

        z %= 26*26;
    }

    z
}

// struct ModEq {
//     modulo: Option<i64>,
//     equals: i64
// }

// impl ModEq {

// }

// fn invert_step(mut z: i64, n: usize) {
//     // x == w

//     (1..10).map(|w| {
//         ModEq(26, Some((w - x_add[n])))
//     });

// }

fn model_no_to_inputs(mut model_number: i64) -> Option<Vec<i64>> {
    let mut output = Vec::with_capacity(14);

    for _ in 0..14 {
        let digit = model_number % 10;

        if digit == 0 {
            return None;
        }

        output.push(digit);
        model_number /= 10;
    }

    output.reverse();

    //let model_str = model_number.to_string();


    // if model_str.as_str().contains("0") {
    //     return None;
    // }

    // let model_input: Vec<i64> = model_str
    //     .chars()
    //     .map(|ch| ch.to_digit(10).unwrap() as i64)
    //     .collect();

    Some(output)
}

const max_model_number: i64 = 99999999999999;

fn find_highest_single_thread() {
    let pct = max_model_number / 10000;

    for model_number in 0..max_model_number {
        if model_number % pct == 0 {
            println!("scanned to: {}", model_number);
        }

        let model_number = max_model_number - model_number;

        if let Some(model_input) = model_no_to_inputs(model_number) {
            let w = rust_program_condensed(&model_input);

            let valid = w == 0;

            if valid {
                println!("{}", model_number);
                return;
            }
        }
    }
}

fn find_highest_multi_thread() {
    let pct = max_model_number / 5000;
    let min_model_number = 11111111111111;

    //75819131181373

    let largest_known_serial = 75819131181374;

    let (first_valid, z) = (min_model_number..largest_known_serial).into_par_iter().filter_map(|model_number| {
        if model_number % pct == 0 {
            println!("scanned to: {}", model_number);
        }

        //let model_number = max_model_number - model_number;

        if let Some(model_input) = model_no_to_inputs(model_number) {
            let z = rust_program_condensed(&model_input);

            if z == 0 {
                println!("Found valid candidate: {}", model_number);
            }

            Some((model_number, z))
            // let valid = w == 0;

            // if valid {
            //     //println!("{}", model_number);
            //     Some(model_number)
            // }
        }
        else {
            None
        }
    }).find_first(|(_model_number, z)| *z == 0).unwrap();

    println!("first valid: {}", first_valid);
}

fn main() {
    // let program = {
    //     let stdin = io::stdin();
    //     let stdin_lock = stdin.lock();
    //     parse_input(stdin_lock)
    // };

    //dbg!(&program);

    // dbg!("decompiling...");

    // let ast = AstNode::decompile(&program);

    // dbg!("done!");

    // //println!("w=");
    // //println!("{}", ast);

    // let mut ast = ast;

    // for n in 0..12 {
    //     ast = ast.subs_input(n, 9);
    // }

    // println!("w=");
    // println!("{}", ast);

    find_highest_multi_thread();



    // let inputs: Vec<i32> = repeat(9).take(14).collect();

    // let mut machine = Machine::new();

    // machine.execute(&program, &inputs);

    // dbg!(machine.registers);
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use crate::{parse_input, Input, model_no_to_inputs, rust_program_condensed, Machine, rust_program};

    fn get_test_input() -> Input {
        let test_data_str = include_str!("../test_input.txt");

        let test_data_reader = Cursor::new(test_data_str.to_owned());

        parse_input(test_data_reader)
    }

    #[test]
    fn test_parse() {
        let test_data = get_test_input();
    }

    #[test]
    fn test_condensed_rust_program_99999999999999() {
        let inputs = model_no_to_inputs(99999999999999).unwrap();

        let output = rust_program(inputs.iter().copied());
        let condensed_output = rust_program_condensed(&inputs);

        assert_eq!(output, condensed_output);

        let test_data = get_test_input();

        let mut machine = Machine::new();

        machine.execute(&test_data, &inputs);
        let machien_output = machine.registers[3];

        assert_eq!(machien_output, condensed_output);
    }

    #[test]
    fn test_condensed_rust_program_11111111111111() {
        let inputs = model_no_to_inputs(11111111111111).unwrap();


        let condensed_output = rust_program_condensed(&inputs);

        let test_data = get_test_input();

        let mut machine = Machine::new();

        machine.execute(&test_data, &inputs);
        let machien_output = machine.registers[3];

        assert_eq!(machien_output, condensed_output);
    }


    #[test]
    fn test_condensed_rust_program_97919997299495() {
        let inputs = model_no_to_inputs(97919997299495).unwrap();


        let condensed_output = rust_program_condensed(&inputs);

        let test_data = get_test_input();

        let mut machine = Machine::new();

        machine.execute(&test_data, &inputs);
        let machien_output = machine.registers[3];

        assert_eq!(machien_output, condensed_output);
    }

    #[test]
    fn test_condensed_rust_program_51619131181131() {
        let inputs = model_no_to_inputs(51619131181131).unwrap();


        let condensed_output = rust_program_condensed(&inputs);

        let test_data = get_test_input();

        let mut machine = Machine::new();

        machine.execute(&test_data, &inputs);
        let machien_output = machine.registers[3];

        assert_eq!(machien_output, condensed_output);
    }

    #[test]
    fn test_eql() {
        //Inst::
    }
}
