use std::io::{self, BufRead};


enum Dir {
    Up,
    Down,
    Forward,
    Backward,
}

fn main() -> Result<(), anyhow::Error> {
    let lines: Vec<String> = {
        let stdin = io::stdin();
        let stdin_lock = stdin.lock();
        stdin_lock.lines().collect::<Result<Vec<_>, _>>()?
    };

    let commands = lines.map(|line| line.split(" "))


    println!("{}", increases);

    Ok(())
}
