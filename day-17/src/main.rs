use nalgebra::DimMax;
use rayon::prelude::*;
use std::io::{self, BufRead};

type Point = nalgebra::Point2<i64>;
type Vec2 = nalgebra::Vector2<i64>;

type Input = Vec<i64>;

fn parse_input(mut reader: impl BufRead) -> Input {
    unimplemented!()
}

fn simulate_probe(pos: Vec2, vel: Vec2) -> (Vec2, Vec2) {
    let next_pos = pos + vel;
    let mut next_vel = vel;

    let x_drag = vel[0].signum() * -1;
    next_vel[0] += x_drag;

    next_vel[1] -= 1;

    (next_pos, next_vel)
}


fn check_hit(target_min: Vec2, target_max: Vec2, x: i64, y: i64) -> Option<(Vec2, Vec2, i64)>{
    let mut pos = Vec2::new(0, 0);
    let initial_vel = Vec2::new(x, y);
    let mut vel = initial_vel;
    let mut max_y = 0i64;

    let mut hit = None;

    loop {
        //println!("pos={} vel={}", pos, vel);
        if pos[0] <= target_max[0] && pos[0] >= target_min[0] && pos[1] <= target_max[1] && pos[1] >= target_min[1] {
            hit = Some((initial_vel, pos, max_y));
            print!(".");
            break;
        }

        if pos[1] < target_min[1] && vel[1] <= 0 {
            break;
        }

        if pos[0] < target_min[0] && vel[0] <= 0 {
            break;
        }

        if pos[0] > target_max[0] && vel[0] >= 0 {
            break;
        }

        // if step == 10000 {
        //     println!("LONG SIM: initial vel: {} pos: {} vel: {}", initial_vel, pos, vel);
        // }

        let (new_pos, new_vel) = simulate_probe(pos, vel);

        pos = new_pos;
        vel = new_vel;

        max_y = i64::max(pos[1], max_y);
    }

    hit
}

fn main() {
    let target_min = Vec2::new(81, -150);
    let target_max = Vec2::new(129, -108);

    let mut hits: Vec<(Vec2, Vec2, i64)> = (0..1000)
        .into_par_iter()
        .flat_map(|x| {
            //         if x < 0 && target_min[0] > 0 {
            //             continue;
            //         }

            (-1000i64..100000).into_par_iter().filter_map(move |y| {
                check_hit(target_min, target_max, x, y)
            })
        })
        .collect();

    println!("done");

    println!("{} hits found", hits.len());

    hits.sort_by_key(|hit| hit.2);
    hits.reverse();

    for hit in &hits[..10] {
        println!(
            "initial vel: {} hit pos: {} max height: {}",
            hit.0, hit.1, hit.2
        );
    }
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use crate::{parse_input, Input};

    fn get_test_input() -> Input {
        let test_data_str = include_str!("../test_input.txt");

        let test_data_reader = Cursor::new(test_data_str.to_owned());

        parse_input(test_data_reader)
    }

    #[test]
    fn test_parse() {
        let test_data = get_test_input();
    }
}
