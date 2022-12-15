use std::io::{self, BufRead};
use std::iter;
use std::str::FromStr;
use ndarray::{s, Array1, Array2};
use std::collections::{BinaryHeap, HashMap, HashSet};

type Map = Array2<i32>;
type Point = nalgebra::Point2<i32>;
type Vec2 = nalgebra::Vector2<i32>;

type Input = Vec<LineStrip>;

#[derive(Debug)]
struct LineStrip(Vec<Point>);

fn signum_vec(v: Vec2) -> Vec2 {
    Vec2::new(i32::signum(v.x), i32::signum(v.y))
}


impl LineStrip {
    fn points<'a>(&'a self) -> impl Iterator<Item=Point> + 'a {
        self.0.iter().zip(self.0.iter().skip(1)).flat_map(|(s, e)| {
            let delta = e - s;
            let d = signum_vec(delta);

            let mut cur_pos = *s;
            let mut done = false;

            iter::from_fn(move || {
                if done {
                    return None;
                }

                let ret_pos = cur_pos;

                if ret_pos == *e {
                    done = true;
                }

                cur_pos += d;
                Some(ret_pos)
            })
        })
    }
}

impl FromStr for LineStrip {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let points: Vec<Point> = s.split(" -> ").map(|p_str| {
            let (x_str, y_str) = p_str.split_once(',').unwrap();
            Point::new(
                i32::from_str_radix(x_str, 10).unwrap(),
                i32::from_str_radix(y_str, 10).unwrap(),
            )
        }).collect();

        Ok(LineStrip(points))
    }
}

fn parse_input(mut reader: impl BufRead) -> Input {
    let mut lines = reader.lines().map(|l| l.unwrap());

    let line_strips = lines.map(|line| {
        LineStrip::from_str(&line).unwrap()
    }).collect();

    line_strips
}

fn fill_lines(line_strips: &[LineStrip]) -> Map {
    let mut map = Map::zeros((700, 700));

    for line_strip in line_strips.iter() {
        for p in line_strip.points() {
            map[(p.y as usize, p.x as usize)] = 1;
        }
    }

    map
}

fn print_map(map: &Map) {
    for row in map.rows() {
        for val in row.iter() {
            match val {
                0 => print!(" "),
                1 => print!("#"),
                2 => print!("o"),
                _ => panic!("invalid map cell")
            }
        }

        println!("");
    }
}

struct MapWithFloor {
    floor: Option<i32>,
    cells: Map
}

impl MapWithFloor {
    fn map_get(&self, p: Point) -> i32 {
        let shape = self.cells.shape();

        if p.y >= (shape[1] as i32) || p.x >= (shape[0] as i32) {
            return 0;
        }

        if Some(p.y) == self.floor {
            return 1;
        }

        self.cells[(p.y as usize, p.x as usize)]
    }

    fn map_set(&mut self, p: Point, val: i32) {
        self.cells[(p.y as usize, p.x as usize)] = val;
    }

    fn no_floor(map: Map) -> MapWithFloor {
        MapWithFloor { floor: None, cells:map }
    }

    fn with_floor(map: Map) -> MapWithFloor {
        let lowest_y = map.indexed_iter().filter_map(|((y, x), val)| if *val != 0 { Some(y) } else { None }).max().unwrap();

        //dbg!(lowest_y);

        MapWithFloor { floor: Some((lowest_y + 2) as i32), cells:map }
    }
}


fn drop_sand(map: &mut MapWithFloor, start_pos: Point) -> bool {
    let mut cur_pos = start_pos;

    let shape = map.cells.shape();

    if map.map_get(start_pos) != 0 {
        return false;
    }

    loop {
        if cur_pos.y >= (shape[1] as i32) || cur_pos.x >= (shape[0] as i32) {
            return false;
        }

        if map.map_get(cur_pos + Vec2::new(0, 1)) == 0 {
            cur_pos += Vec2::new(0, 1);
            continue;
        }

        if map.map_get(cur_pos + Vec2::new(-1, 1)) == 0 {
            cur_pos += Vec2::new(-1, 1);
            continue;
        }

        if map.map_get(cur_pos + Vec2::new(1, 1)) == 0 {
            cur_pos += Vec2::new(1, 1);
            continue;
        }

        if cur_pos.y >= (shape[1] as i32) || cur_pos.x >= (shape[0] as i32) {
            return false;
        }


        map.map_set(cur_pos, 2);

        return true;
    }
}

fn fill_sand(map: &mut MapWithFloor) -> usize {
    let mut count = 0;

    while drop_sand(map, Point::new(500, 0)) {
        count += 1;
    }

    count
}

fn main() {
    let input = {
        let stdin = io::stdin();
        let stdin_lock = stdin.lock();
        parse_input(stdin_lock)
    };

    dbg!(&input);

    let mut map = fill_lines(&input);
    //dbg!(&map);

    let mut map_no_floor = MapWithFloor::no_floor(map.clone());

    dbg!(fill_sand(&mut map_no_floor));

    let mut map_with_floor = MapWithFloor::with_floor(map.clone());

    dbg!(fill_sand(&mut map_with_floor));
    //print_map(&map);
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
