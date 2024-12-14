use std::collections::{HashMap, HashSet};
use std::io::{self, BufRead};
use anyhow::Result;
use nalgebra::Matrix2;
use regex::Regex;

type Pos = nalgebra::Point2<i64>;
type Vec2 = nalgebra::Vector2<i64>;

#[derive(Debug)]
struct ClawMachine {
    button_a: Vec2,
    button_b: Vec2,
    prize: Pos
}

type Input = Vec<ClawMachine>;

fn parse_button(s: &str) -> Vec2 {
    let re = Regex::new(r"X\+(\d+), Y\+(\d+)").unwrap();

    let (_, [x_s, y_s]) = re.captures(s).unwrap().extract();

    Vec2::new(
        i64::from_str_radix(x_s, 10).unwrap(),
        i64::from_str_radix(y_s, 10).unwrap()
    )
}

fn parse_prize(s: &str) -> Pos {
    dbg!(s);
    let re = Regex::new(r"X\=(\d+), Y\=(\d+)").unwrap();

    let (_, [x_s, y_s]) = re.captures(s).unwrap().extract();

    Pos::new(
        i64::from_str_radix(x_s, 10).unwrap(),
        i64::from_str_radix(y_s, 10).unwrap()
    )
}

fn parse_input(mut reader: impl BufRead) -> Result<Input> {
    let mut claw_machines = Vec::new();

    let mut lines = reader.lines();

    while let Some(line) = lines.next() {
        let button_a = parse_button(&line.unwrap());

        let button_b = parse_button(&lines.next().unwrap().unwrap());

        let prize = parse_prize(&lines.next().unwrap().unwrap());

        claw_machines.push(ClawMachine {
            button_a,
            button_b,
            prize,
        });

        let _ = lines.next();
    }

    Ok(claw_machines)
}

//

fn main() -> Result<()> {
    let map = {
        let stdin = io::stdin();
        let stdin_lock = stdin.lock();
        parse_input(stdin_lock)?
    };

    dbg!(&map);

    let token_cost = Vec2::new(3, 1);

    let mut part1_cost = 0;

    for claw_machine in map.iter() {
        let mat = Matrix2::from_columns(&[claw_machine.button_a, claw_machine.button_b]);

        let mat_f = mat.cast::<f64>();
        let prize_f = claw_machine.prize.cast::<f64>().coords;

        // let decomp = mat_f.clone().lu();
        // let x = decomp.solve(&prize_f).expect("Linear resolution failed.");

        let mat_invert = mat_f.try_inverse().unwrap();
        let x = mat_invert * prize_f;


        let sol = Vec2::new(
            f64::round(x[0]) as i64,
            f64::round(x[1]) as i64,
        );

        let should_be_prize = mat * sol;

        if should_be_prize == claw_machine.prize.coords {
            println!("{:?} {:?}", claw_machine.prize, sol.cast() - x);
            let cost = sol.dot(&token_cost);
            //println!("{:?} : {:?} cost: {}", sol, claw_machine.prize, cost);

            part1_cost += cost;
        }

        //dbg!(x);
    }



    println!("part1: {}", part1_cost);

    let mut part2_cost = 0;

    for claw_machine in map.iter() {
        let mat = Matrix2::from_columns(&[claw_machine.button_a, claw_machine.button_b]);

        let adjusted_prize = claw_machine.prize + Vec2::new(10000000000000, 10000000000000);

        let mat_f = mat.cast::<f64>();
        let prize_f = adjusted_prize.cast::<f64>().coords;


        //let mat_invert = mat_f.try_inverse().unwrap();

        //let x = mat_invert * prize_f;
        let decomp = mat_f.clone().lu();
        let x = decomp.solve(&prize_f).expect("Linear resolution failed.");

        let sol = Vec2::new(
            f64::round(x[0]) as i64,
            f64::round(x[1]) as i64,
        );

        //dbg!(sol)

        let should_be_prize = mat * sol;

        if should_be_prize == adjusted_prize.coords {
            let cost = sol.dot(&token_cost);
            //println!("{:?} : {:?} cost: {}", sol, claw_machine.prize, cost);

            part2_cost += cost;
        }

        //dbg!(x);
    }


    println!("part2: {}", part2_cost);

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{parse_button, parse_prize, Pos, Vec2};

    #[test]
    fn test_parse_button() {
        let button = parse_button("Button A: X+94, Y+34");
        assert_eq!(button, Vec2::new(94, 34));
    }

    #[test]
    fn test_parse_prize() {
        let button = parse_prize("Prize: X=18641, Y=10279");
        assert_eq!(button, Pos::new(18641, 10279));
    }
}
