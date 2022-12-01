use std::fmt;
use std::io::{self, BufRead};

type Vec3 = nalgebra::Vector3<i32>;
use building_blocks::core::prelude::*;
use building_blocks::storage::{prelude::*, ChunkHashMap, ChunkMap2x1};

//type Map = ChunkHashMap<[i32; 2], u8, ChunkMapBuilder2x1<u8>>;
type Map = Array2x1<u8>;
type Input = Map;

const EMPTY: u8 = 0;
const EAST_CUCUMBER: u8 = 1;
const SOUTH_CUCUMBER: u8 = 2;

fn parse_input(mut reader: impl BufRead) -> Input {

    let mut rows = Vec::new();

    for (y, line) in reader.lines().enumerate() {
        let line = line.unwrap();

        let row: Vec<u8> = line.chars().map(|ch| {
            let cucumber = match ch {
                '.' => EMPTY,
                '>' => EAST_CUCUMBER,
                'v' => SOUTH_CUCUMBER,
                other => panic!("Invalid cucumber cell")
            };

            cucumber
        }).collect();

        rows.push(row);
    }
    let extent = Extent2i::from_min_and_shape(Point2i::fill(0), PointN([rows[0].len() as i32, rows.len() as i32]));

    let map = Array2x1::fill_with(extent, |p| {
        let val = rows[p.y() as usize][p.x() as usize];
        val
    });
    //let mut map: Array2x1<u8> = Array2x1::fill(extent, 0);

    map
}

struct DisplayMap<'a>(&'a Map);

impl <'a> fmt::Display for DisplayMap<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let shape = self.0.extent();

        for y in shape.minimum.y()..(shape.minimum.y()+shape.shape.y()) {
            for x in shape.minimum.x()..(shape.minimum.x()+shape.shape.x()) {
                match self.0.get(PointN([x, y])) {
                    EMPTY => write!(f, ".")?,
                    EAST_CUCUMBER => write!(f, ">")?,
                    SOUTH_CUCUMBER => write!(f, "v")?,
                    other => write!(f, "?")?
                }
            }

            writeln!(f, "")?;
        }

        Ok(())
    }
}

fn step_herd(map: &Map, herd: u8) -> (Map, usize) {
    let extent = *map.extent();

    let width = extent.shape.x();
    let height = extent.shape.y();

    let mut next_map = Array2x1::fill(extent, EMPTY);

    let mut moves = 0;

    match herd {
        EAST_CUCUMBER => {
            map.for_each(&extent, |p: Point2i, val| {
                if val == EAST_CUCUMBER {
                    let right_cell_x = (p.x() + 1) % width;
                    let right_cell_p = PointN([right_cell_x, p.y()]);
                    let right_cell_cur = map.get(right_cell_p);
                    let right_cell_next = next_map.get(right_cell_p);


                    if right_cell_cur == EMPTY && right_cell_next == EMPTY {
                        *next_map.get_mut(right_cell_p) = EAST_CUCUMBER;
                        moves += 1;
                    }
                    else {
                        *next_map.get_mut(p) = EAST_CUCUMBER;
                    }
                }
                else if val == SOUTH_CUCUMBER {
                    *next_map.get_mut(p) = SOUTH_CUCUMBER;
                }
            });
        }
        SOUTH_CUCUMBER => {
            map.for_each(&extent, |p: Point2i, val| {
                if val == SOUTH_CUCUMBER {
                    let down_cell_y = (p.y() + 1) % height;
                    let down_cell_p = PointN([p.x(), down_cell_y]);
                    let down_cell_cur = map.get(down_cell_p);
                    let down_cell_next = next_map.get(down_cell_p);


                    if down_cell_cur == EMPTY && down_cell_next == EMPTY {
                        *next_map.get_mut(down_cell_p) = SOUTH_CUCUMBER;
                        moves += 1;
                    }
                    else {
                        *next_map.get_mut(p) = SOUTH_CUCUMBER;
                    }
                }
                else if val == EAST_CUCUMBER {
                    *next_map.get_mut(p) = EAST_CUCUMBER;
                }
            });
        }
        other => {
            panic!("Invalid cucumber type: {}", other);
        }
    }

    (next_map, moves)
}

fn step(map: &Map) -> (Map, usize) {
    let (step1, step1_moves) = step_herd(map, EAST_CUCUMBER);
    let (step2, step2_moves) = step_herd(&step1, SOUTH_CUCUMBER);

    (step2, step1_moves + step2_moves)
}

fn main() {
    let map = {
        let stdin = io::stdin();
        let stdin_lock = stdin.lock();
        parse_input(stdin_lock)
    };

    //dbg!(&input);
    println!("{}", DisplayMap(&map));

    let mut cur_map = map;

    for step_n in 1.. {
        let (next_map, moves) = step(&cur_map);

        println!("Step: {} Moves: {}", step_n, moves);
        println!("{}", DisplayMap(&next_map));

        cur_map = next_map;

        if moves == 0 {
            break;
        }
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
