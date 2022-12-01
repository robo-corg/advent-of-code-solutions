use std::{io::{self, BufRead}, collections::{HashSet, VecDeque}};

use nalgebra::DimMax;
use ndarray::{Array2, Array1, s};

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
                .collect()
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
    ].into_iter().filter(move |p| {
        p[0] >= 0 && p[1] >= 0 && p[0] < row_count && p[1] < col_count
    })
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

    let mut low_points = 0;
    let mut risk = 0;
    let mut basin_sizes = Vec::new();

    for row in 0..(row_count) {
        for col in 0..(col_count) {
            let current = input[(row, col)];

            let neighbors = [
                if row > 0 { input.get((row - 1, col)) } else { None },
                input.get((row + 1, col)),
                if col > 0 { input.get((row, col - 1)) } else { None },
                input.get((row as usize, col + 1))
            ];

            //dbg!(neighbors);

            let low_point = neighbors.iter().copied().filter_map(|n| n).all(|n| *n > current);

            //dbg!(col, row, current, low_point);

            if low_point {
                low_points += 1;
                risk += current + 1;

                let mut basin_points = HashSet::new();

                let cur_pos = Vec2::new(row as i32, col as i32);
                let mut to_visit = VecDeque::new();
                basin_points.insert(cur_pos);
                to_visit.push_back(cur_pos);

                while let Some(cur_pos) = to_visit.pop_front() {
                    for neighbor in get_neighbors(cur_pos, row_count, col_count) {
                        if basin_points.contains(&neighbor) {
                            continue;
                        }

                        let neighbor_val = input[(neighbor[0] as usize, neighbor[1] as usize)];

                        if neighbor_val < 9 {
                            basin_points.insert(neighbor);
                            to_visit.push_back(neighbor);
                        }
                    }
                }

                //dbg!(basin_points.len());
                basin_sizes.push(basin_points.len());
            }

            // let row_start = i32::max(0, row - 1);
            // let col_start = i32::
            // let window = input.slice(s![row_start..(row+2), (col-1)..(col+2)]);
            // dbg!(window);
        }
    }

    dbg!(low_points);
    dbg!(risk);

    basin_sizes.sort();

    dbg!(&basin_sizes);

    let top_3_basins: Vec<_> = basin_sizes.iter().rev().take(3).collect();

    dbg!(top_3_basins);
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

        let shape = test_data.shape();

        assert_eq!(shape[0], 5);
        assert_eq!(shape[1], 10);
    }
}
