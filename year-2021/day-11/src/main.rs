use std::{
    collections::{HashSet, VecDeque},
    io::{self, BufRead}, mem,
};

use nalgebra::DimMax;
use ndarray::{s, Array1, Array2};

type Point = nalgebra::Point2<i32>;
type Vec2 = nalgebra::Vector2<i32>;
type Map = Array2<i32>;

type Input = Map;

fn parse_input(mut reader: impl BufRead) -> Input {
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

    dbg!(&map_vec);

    let width = map_vec[0].len();

    let mut map = Map::zeros((0, width));

    for row in map_vec.into_iter() {
        map.push_row(row.view()).unwrap();
    }

    map
}

fn get_neighbors(p: Vec2, row_count: usize, col_count: usize) -> impl Iterator<Item=Vec2> {
    let row_count = row_count as i32;
    let col_count = col_count as i32;

    vec![
        p + Vec2::new(-1, 0),
        p + Vec2::new(1, 0),
        p + Vec2::new(0, -1),
        p + Vec2::new(0, 1),
        p + Vec2::new(-1, -1),
        p + Vec2::new(1, 1),
        p + Vec2::new(-1, 1),
        p + Vec2::new(1, -1),
    ].into_iter().filter(move |p| {
        p[0] >= 0 && p[1] >= 0 && p[0] < row_count && p[1] < col_count
    })
}

fn v((x, y): (usize, usize)) -> Vec2 {
    Vec2::new(x as i32, y as i32)
}

fn p(v :Vec2) -> (usize, usize) {
    (v[0] as usize, v[1] as usize)
}

fn main() {
    let input = {
        let stdin = io::stdin();
        let stdin_lock = stdin.lock();
        parse_input(stdin_lock)
    };

    dbg!(&input);

    // for window in input.windows((3, 3)).into_iter() {
    //     dbg!(window);
    // }
    let shape = input.shape();

    let row_count = shape[0];
    let col_count = shape[1];

    let mut octopuses = input;
    let mut flash_count = 0;

    for step in 0..1000000 {
        //let flashed = Array2::zeros((row_count, col_count));
        let mut flashes = HashSet::new();

        octopuses += 1;

        let mut new_flashes: Vec<Vec2> = octopuses
            .indexed_iter()
            .filter(|(pos, val)| **val > 9 && !flashes.contains(&v(*pos)))
            .map(|(p, _)| v(p))
            .collect();

        for p in new_flashes.iter() {
            flashes.insert(*p);
        }

        let mut next_new_flashes: Vec<Vec2> = Vec::new();

        while !new_flashes.is_empty() {
            assert!(next_new_flashes.is_empty());

            for pos in new_flashes.drain(..) {
                flash_count += 1;
                flashes.insert(pos);
                //octopuses[p(pos)] = 0;

                for neighbor in get_neighbors(pos, row_count, col_count) {
                    if !flashes.contains(&neighbor) {
                        octopuses[p(neighbor)] += 1;

                        if octopuses[p(neighbor)] > 9 {
                            flashes.insert(neighbor);
                            next_new_flashes.push(neighbor);
                        }
                    }
                }
            }

            mem::swap(&mut next_new_flashes, &mut new_flashes);
        }

        for flashed_pos in flashes.iter() {
            assert_eq!(octopuses[p(*flashed_pos)], 10);
            octopuses[p(*flashed_pos)] = 0;
        }

        //dbg!(step, &octopuses);

        if step == 99 {
            println!("flash count at 100 steps (part 1): {}", flash_count);
        }

        if flashes.len() == col_count * row_count {
            println!("first step all flash (part 2): {}", step + 1);
            break;
        }
    }

    //dbg!(flash_count);
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
