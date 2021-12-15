use std::{
    collections::BinaryHeap,
    io::{self, BufRead},
};

use ndarray::{Array1, Array2};
use rustc_hash::{FxHashMap, FxHashSet};

type Vec2 = nalgebra::Vector2<i32>;
type Map = Array2<i32>;

type Input = Map;

type Cost = i32;

fn parse_input(reader: impl BufRead) -> Input {
    let map_vec: Vec<Array1<_>> = reader
        .lines()
        .map(|line| {
            Array1::from_vec(
                line.unwrap()
                    .chars()
                    .map(|ch| ch.to_digit(10).unwrap() as i32)
                    .collect(),
            )
        })
        .collect();

    //dbg!(&map_vec);

    let width = map_vec[0].len();

    let mut map = Map::zeros((0, width));

    for row in map_vec.into_iter() {
        map.push_row(row.view()).unwrap();
    }

    map
}

fn get_neighbors(p: Vec2, world_size: Vec2) -> impl Iterator<Item = Vec2> {
    // let row_count = row_count as i32;
    // let col_count = col_count as i32;

    vec![
        p + Vec2::new(-1, 0),
        p + Vec2::new(1, 0),
        p + Vec2::new(0, -1),
        p + Vec2::new(0, 1),
    ]
    .into_iter()
    .filter(move |p| p[0] >= 0 && p[1] >= 0 && p[0] < world_size[0] && p[1] < world_size[1])
}

fn wrap_risk(unwrapped: Cost) -> Cost {
    if unwrapped > 9 {
        ((unwrapped - 1) % 9) + 1
    } else {
        unwrapped
    }
}

fn get_risk(map: &Map, pos: Vec2) -> Cost {
    let shape = map.shape();

    let size_x = shape[1] as Cost;
    let size_y = shape[0] as Cost;

    let world_x = pos[1] as Cost;
    let world_y = pos[0] as Cost;

    let tile_x = world_x % size_x;
    let tile_y = world_y % size_y;
    let extra_risk_x = world_x / size_x;
    let extra_risk_y = world_y / size_y;

    let extra_risk_pre_wrap = extra_risk_x + extra_risk_y;

    // let extra_risk = if extra_risk_pre_wrap > 9 {
    //     ((extra_risk_pre_wrap - 1) % 9) + 1
    // } else {
    //     extra_risk_pre_wrap
    // } as Cost;

    //dbg!(world_x, world_y, tile_x, tile_y, extra_risk_pre_wrap);

    wrap_risk(map[(tile_x as usize, tile_y as usize)] as Cost + extra_risk_pre_wrap)
}

fn dist(a: Vec2, b: Vec2) -> Cost {
    let d = b - a;

    Cost::abs(d[0] as Cost) + Cost::abs(d[1] as Cost)
}

fn h(a: Vec2, b: Vec2) -> Cost {
    dist(a, b)
}

#[derive(Debug, Eq)]
struct VisitItem {
    estimated_cost: Cost,
    pos: Vec2,
}

impl PartialEq for VisitItem {
    fn eq(&self, other: &Self) -> bool {
        self.estimated_cost == other.estimated_cost
    }
}

impl PartialOrd for VisitItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for VisitItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.estimated_cost.cmp(&self.estimated_cost)
    }
}

fn find_path_cost(map: &Map, goal: Vec2, world_size: Vec2) -> Cost {
    let start_pos = Vec2::new(0, 0);

    let mut costs = FxHashMap::default();
    let mut visit_queue = BinaryHeap::new();
    let mut open_set = FxHashSet::default();
    let mut previous_links = FxHashMap::default();

    visit_queue.push(VisitItem {
        estimated_cost: get_risk(&map, start_pos) + h(start_pos, goal),
        pos: start_pos,
    });
    costs.insert(start_pos, get_risk(&map, start_pos));

    let mut maybe_last = None;

    loop {
        let VisitItem {
            estimated_cost: current_estimated_cost,
            pos,
        } = visit_queue.pop().unwrap();

        let current_cost = costs.get(&pos).copied().unwrap();

        maybe_last = Some((current_estimated_cost, pos));

        if pos == goal {
            break;
        }

        open_set.remove(&pos);

        for neighbor in get_neighbors(pos, world_size) {
            let neighbor_cost = current_cost + get_risk(&map, neighbor);

            let cur_best = costs.get(&neighbor).copied();

            if cur_best.is_none() || neighbor_cost < cur_best.unwrap_or(Cost::MAX) {
                costs.insert(neighbor, neighbor_cost);
                previous_links.insert(neighbor, pos);

                if !open_set.contains(&neighbor) {
                    let estimated_cost = neighbor_cost + h(neighbor, goal);
                    visit_queue.push(VisitItem {
                        estimated_cost,
                        pos: neighbor,
                    });
                    open_set.insert(neighbor);
                }
            }
        }
    }

    let (last_cost, _last_pos) = maybe_last.unwrap();

    if false {
        let mut path = Vec::new();
        let mut path_costs = Vec::new();

        let mut pos = _last_pos;

        path.push(_last_pos);
        path_costs.push(get_risk(&map, _last_pos));

        loop {
            if pos == start_pos {
                break;
            }

            let prev_pos = previous_links.get(&pos).copied().unwrap();
            path.push(prev_pos);
            path_costs.push(get_risk(&map, prev_pos));
            pos = prev_pos;
        }

        path.reverse();
        path_costs.reverse();

        for p in path.iter() {
            let risk = get_risk(&map, *p);
            println!("{},{}: {}", p[0], p[1], risk);
        }
    }

    // Risk for start doesn't count unless rerenter it
    last_cost - get_risk(&map, start_pos)
}

fn main() {
    let input = {
        let stdin = io::stdin();
        let stdin_lock = stdin.lock();
        parse_input(stdin_lock)
    };

    let map = input;

    let shape = map.shape();

    let row_count = shape[0];
    let col_count = shape[1];

    let world_size_part_2 = Vec2::new((row_count * 5) as i32, (col_count * 5) as i32);

    let goal_part_1 = Vec2::new(row_count as i32 - 1, col_count as i32 - 1);
    let goal_part_2 = Vec2::new((row_count * 5) as i32 - 1, (col_count * 5) as i32 - 1);

    //println!("{}", find_path_cost(&map, goal_part_1));
    println!("{}", find_path_cost(&map, goal_part_2, world_size_part_2));

    // dbg!(&input);
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
