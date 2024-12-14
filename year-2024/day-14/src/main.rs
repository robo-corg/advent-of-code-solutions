use anyhow::Result;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::io::{self, BufRead};

type Pos = nalgebra::Point2<i64>;
type Vec2 = nalgebra::Vector2<i64>;

#[derive(Debug, Clone)]
struct Robot {
    pos: Pos,
    vel: Vec2,
}

type Input = Vec<Robot>;

fn parse_robot(s: &str) -> Robot {
    let re = Regex::new(r"p=(-?\d+),(-?\d+) v=(-?\d+),(-?\d+)").unwrap();

    let (_, [px_s, py_s, vx_s, vy_s]) = re.captures(s).unwrap().extract();

    Robot {
        pos: Pos::new(
            i64::from_str_radix(px_s, 10).unwrap(),
            i64::from_str_radix(py_s, 10).unwrap(),
        ),
        vel: Vec2::new(
            i64::from_str_radix(vx_s, 10).unwrap(),
            i64::from_str_radix(vy_s, 10).unwrap(),
        ),
    }
}

fn parse_input(mut reader: impl BufRead) -> Result<Input> {
    let robots = reader.lines().map(|l| parse_robot(&l.unwrap())).collect();

    Ok(robots)
}

fn sim_robots(robots: &Vec<Robot>, steps: i64, map_size: Vec2) -> Vec<Robot> {
    robots
        .iter()
        .map(|robot| {
            let p = robot.pos + robot.vel * steps;

            let pos = Pos::new(p[0].rem_euclid(map_size[0]), p[1].rem_euclid(map_size[1]));

            assert!(pos[0] >= 0);
            assert!(pos[1] >= 0);

            Robot { pos, ..*robot }
        })
        .collect()
}

fn print_robots(robots: &Vec<Robot>, map_size: Vec2) {
    let mut robots_at: HashMap<Pos, i64> = HashMap::new();

    for robot in robots.iter() {
        *robots_at.entry(robot.pos).or_default() += 1;
    }

    for y in 0..map_size[1] {
        for x in 0..map_size[0] {
            if let Some(count) = robots_at.get(&Pos::new(x, y)) {
                print!("{}", count);
            } else {
                print!(".");
            }
        }

        println!();
    }
}

fn count_tris(robots: &Vec<Robot>, map_size: Vec2) -> usize {
    let mut robots_at: HashSet<Pos> = HashSet::with_capacity(robots.len());

    let mut tri_count = 0;

    for robot in robots.iter() {
        robots_at.insert(robot.pos);
    }

    for y in 0..map_size[1] {
        for x in 0..map_size[0] {
            let p = Pos::new(x, y);

            if !robots_at.contains(&p) {
                continue;
            }

            let d1 = robots_at.contains(&(p + Vec2::new(1, 1)));
            let d2 = robots_at.contains(&(p + Vec2::new(-1, -1)));
            let d3 = robots_at.contains(&(p + Vec2::new(0, -1)));

            if d1 && d2 && d3 {
                tri_count += 1;
            }
        }
    }

    tri_count
}

fn find_robot_tree(robots: &Vec<Robot>, max_steps: i64, map_size: Vec2) {
    let mut robots = robots.clone();

    let mid = map_size / 2;

    for s in 0..max_steps {
        for robot in robots.iter_mut() {
            let p = robot.pos + robot.vel;

            let pos = Pos::new(p[0].rem_euclid(map_size[0]), p[1].rem_euclid(map_size[1]));

            assert!(pos[0] >= 0);
            assert!(pos[1] >= 0);

            robot.pos = pos;
        }

        if count_tris(&robots, map_size) >= 6 {
            println!("step: {}", s);
            print_robots(&robots, map_size);
        }
    }
}

fn main() -> Result<()> {
    let map = {
        let stdin = io::stdin();
        let stdin_lock = stdin.lock();
        parse_input(stdin_lock)?
    };

    dbg!(&map);

    // 101 103
    //let map_size = Vec2::new(11, 7);
    let map_size = Vec2::new(101, 103);

    let future_bots = sim_robots(&map, 100, map_size);

    let mid = map_size / 2;

    dbg!(&mid);

    let mut quad_counts = [0; 4];

    for future_bot in future_bots.iter() {
        if future_bot.pos[0] == mid[0] || future_bot.pos[1] == mid[1] {
            continue;
        }

        let x_q = if future_bot.pos[0] > mid[0] { 1 } else { 0 };
        let y_q = if future_bot.pos[1] > mid[1] { 1 } else { 0 };

        //dbg!(x_q, y_q);

        quad_counts[x_q | (y_q << 1)] += 1;
    }

    print_robots(&future_bots, map_size);

    let part1_saftey_fac: i64 = quad_counts.iter().product();

    println!("part1: {}", part1_saftey_fac);

    find_robot_tree(&map, 1000000, map_size);

    Ok(())
}
