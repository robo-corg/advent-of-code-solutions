use anyhow::Result;
use std::io::{self, BufRead};

#[derive(Default, PartialEq)]
enum Op {
    #[default]
    Add,
    Mul,
    Concat
}

impl Op {
    fn apply(&self, lhs: i64, rhs: i64) -> i64 {
        match self {
            Op::Add => lhs.saturating_add(rhs),
            Op::Mul => lhs.saturating_mul(rhs),
            Op::Concat => {
                let s = format!("{}{}", lhs, rhs);
                i64::from_str_radix(&s, 10).unwrap()
            }
        }
    }
}

#[derive(Debug)]
struct Equation {
    result: i64,
    nums: Vec<i64>,
}

impl Equation {
    fn solvable(&self, part_2: bool) -> bool {
        let mut sol = Vec::new();
        sol.resize_with(self.nums.len() -1 , Op::default);

        loop {
            if compute(&self.nums, &sol) == self.result {
                return true;
            }

            if !inc(&mut sol, part_2) {
                return false;
            }
        }
    }
}

fn inc(ops: &mut [Op], part_2: bool) -> bool {
    for i in 0..ops.len() {
        if ops[i] == Op::Concat {
            ops[i] = Op::Add;
        }
        else if ops[i] == Op::Mul {
            if part_2 {
                ops[i] = Op::Concat;
                return true;
            }
            else {
                ops[i] = Op::Add;
            }
        }
        else if ops[i] == Op::Add {
            ops[i] = Op::Mul;
            return true;
        }
    }

    false
}

fn compute(mut nums: &[i64], mut ops: &[Op]) -> i64 {
    let mut cur = nums[0];

    nums = &nums[1..];

    while !nums.is_empty() {
        cur = ops[0].apply(cur, nums[0]);


        nums = &nums[1..];
        ops = &ops[1..];
    }

    cur
}

fn parse_input(mut reader: impl BufRead) -> Result<Vec<Equation>> {
    let mut eqs = Vec::new();

    for maybe_line in reader.lines() {
        let line = maybe_line.unwrap();

        let (res_s, all_nums_s) = line.split_once(": ").unwrap();

        let nums = all_nums_s
            .split(' ')
            .map(|num_s| i64::from_str_radix(num_s, 10).unwrap())
            .collect();

        eqs.push(Equation {
            result: i64::from_str_radix(res_s, 10).unwrap(),
            nums,
        })
    }

    Ok(eqs)
}

fn main() -> Result<()> {
    let eqs = {
        let stdin = io::stdin();
        let stdin_lock = stdin.lock();
        parse_input(stdin_lock)?
    };

    dbg!(&eqs);

    let part_1: i64 = eqs.iter().filter(|e| e.solvable(false)).map(|e| e.result).sum();

    println!("Part 1: {}", part_1);


    let part_2: i64 = eqs.iter().filter(|e| e.solvable(true)).map(|e| e.result).sum();

    println!("Part 2: {}", part_2);

    Ok(())
}
