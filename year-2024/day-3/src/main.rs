use std::io::{self, Read};

use regex::Regex;

fn part1(buf: &str) -> i32 {
    let re = Regex::new(r"mul\((\d{1,3}),(\d{1,3})\)").unwrap();

    let mut result = 0;

    for (_, [lhs_s, rhs_s]) in re.captures_iter(buf).map(|c| c.extract()) {
        let lhs = i32::from_str_radix(lhs_s, 10).unwrap();
        let rhs = i32::from_str_radix(rhs_s, 10).unwrap();

        result += lhs * rhs;
    }

    result
}

fn part2(buf: &str) -> i32 {
    let re = Regex::new(r"(?:(don't)\(\))|(?:(do)\(\))|(?:(mul)\((\d{1,3}),(\d{1,3})\))").unwrap();

    let mut result = 0;
    let mut enabled = true;

    for c in re.captures_iter(buf) {

        dbg!(&c);

        let op = c.get(1).or_else(|| c.get(2)).or_else(|| c.get(3)).unwrap().as_str();


        match op {
            "do" => { enabled = true; },
            "don't" => { enabled = false },
            "mul" => {
                if enabled {
                    let lhs_s = c.get(4).unwrap().as_str();
                    let rhs_s = c.get(5).unwrap().as_str();

                    let lhs = i32::from_str_radix(lhs_s, 10).unwrap();
                    let rhs = i32::from_str_radix(rhs_s, 10).unwrap();

                    result += lhs * rhs;
                }
            },
            s => panic!("Unexpected op: {}", s)
        }
    }

    result
}

fn main() -> anyhow::Result<()> {
    let mut stdin = io::stdin();

    let mut buf = String::new();
    stdin.read_to_string(&mut buf)?;

    buf = buf.replace("\n", "");

    println!("Result (part1): {}", part1(&buf));

    println!("Result (part2): {}", part2(&buf));

    Ok(())
}
