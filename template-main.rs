use std::collections::{HashMap, HashSet};
use std::io::{self, BufRead};
use anyhow::Result;

type Pos = nalgebra::Point2<i32>;
type Vec2 = nalgebra::Vector2<i32>;

type Input = ();

fn parse_input(mut reader: impl BufRead) -> Result<Input> {
    Ok(())
}

fn main() -> Result<()> {
    let map = {
        let stdin = io::stdin();
        let stdin_lock = stdin.lock();
        parse_input(stdin_lock)?
    };

    dbg!(&map);

    Ok(())
}
