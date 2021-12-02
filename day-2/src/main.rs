use std::io::{self, BufRead};
use anyhow::{Error, anyhow};


#[derive(Debug)]
enum Dir {
    Up,
    Down,
    Forward,
    Backward,
}

impl Dir {
    fn from_str(s: &str) -> Result<Self, Error> {
        match s {
            "forward" => Ok(Dir::Forward),
            "backward" => Ok(Dir::Backward),
            "up" => Ok(Dir::Up),
            "down" => Ok(Dir::Down),
            _ => Err(anyhow!("Invalid direction"))
        }
    }
}

fn main_old() -> Result<(), anyhow::Error> {
    let lines: Vec<String> = {
        let stdin = io::stdin();
        let stdin_lock = stdin.lock();
        stdin_lock.lines().collect::<Result<Vec<_>, _>>()?
    };

    let commands: Vec<_> = lines.iter().map(|line| {
        let (dir_s, amount_s) =  line.split_once(" ").ok_or(anyhow!("Invalid command (need dir and amount"))?;

        Ok((Dir::from_str(dir_s)?, i64::from_str_radix(amount_s.trim(), 10)?))
    }).collect::<Result<Vec<_>, Error>>()?;


    dbg!(&commands);

    let mut horiz = 0;
    let mut depth = 0;

    for (dir, amount) in commands {
        match dir {
            Dir::Forward => horiz += amount,
            Dir::Backward => horiz -= amount,
            Dir::Down => depth += amount,
            Dir::Up => depth -= amount
        }
    }

    dbg!(horiz, depth);

    println!("{}", horiz * depth);

    Ok(())
}


fn main() -> Result<(), anyhow::Error> {
    let lines: Vec<String> = {
        let stdin = io::stdin();
        let stdin_lock = stdin.lock();
        stdin_lock.lines().collect::<Result<Vec<_>, _>>()?
    };

    let commands: Vec<_> = lines.iter().map(|line| {
        let (dir_s, amount_s) =  line.split_once(" ").ok_or(anyhow!("Invalid command (need dir and amount"))?;

        Ok((Dir::from_str(dir_s)?, i64::from_str_radix(amount_s.trim(), 10)?))
    }).collect::<Result<Vec<_>, Error>>()?;


    dbg!(&commands);

    let mut horiz = 0;
    let mut depth = 0;
    let mut aim = 0;

    for (dir, amount) in commands {
        match dir {
            Dir::Forward => {
                horiz += amount;
                depth += aim * amount;
            },
            Dir::Backward => {
                horiz -= amount;
                depth -= aim * amount;
            },
            Dir::Down => aim += amount,
            Dir::Up => aim -= amount
        }
    }

    dbg!(horiz, depth);

    println!("{}", horiz * depth);

    Ok(())
}