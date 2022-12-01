use std::{
    collections::HashSet,
    convert::Infallible,
    io::{self, BufRead},
    str::FromStr,
};

use nalgebra::DimMax;
use ndarray::Array2;

type Point = nalgebra::Point2<i32>;
type Vec2 = nalgebra::Vector2<i32>;
type Map = Array2<i32>;

fn parse_point(s: &str) -> Point {
    let (x, y) = s.split_once(",").unwrap();

    Point::new(
        i32::from_str_radix(x, 10).unwrap(),
        i32::from_str_radix(y, 10).unwrap(),
    )
}

#[derive(Debug)]
struct Line(Point, Point);

impl Line {
    fn get_diff(&self) -> Vec2 {
        self.1 - self.0
    }

    fn horiz_or_vert(&self) -> bool {
        let d = self.get_diff();

        let x0 = d[0] == 0;
        let y0 = d[1] == 0;

        (x0 || y0) && !(x0 && y0)
    }

    fn diagional(&self) -> bool {
        let d = self.get_diff().abs();

        d[0] == d[1]
    }
}

impl FromStr for Line {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (a_s, b_s) = s.trim().split_once("->").unwrap();

        Ok(Line(parse_point(a_s.trim()), parse_point(b_s.trim())))
    }
}

fn parse_lines(mut reader: impl BufRead) -> Vec<Line> {
    reader
        .lines()
        .map(|line| Line::from_str(line.unwrap().trim()).unwrap())
        .collect()
}

fn main() {
    let lines = {
        let stdin = io::stdin();
        let stdin_lock = stdin.lock();
        parse_lines(stdin_lock)
    };

    //dbg!(&lines);

    //let valid_line: Vec<_> = lines.iter().filter(|l| l.horiz_or_vert()).collect();
    let valid_line: Vec<_> = lines
        .iter()
        .filter(|l| l.horiz_or_vert() || l.diagional())
        .collect();

    dbg!(&valid_line);

    let mut map = Map::zeros((1000, 1000));

    let mut multiple_overlaps = HashSet::new();

    for line in valid_line.iter() {
        let mut p = line.0;
        let diff = line.get_diff();

        let d = diff.map(|e| i32::clamp(e, -1, 1));

        dbg!(d);

        loop {
            let coord = (p[1] as usize, p[0] as usize);

            map[coord] += 1;

            if map[coord] > 1 {
                multiple_overlaps.insert(p);
            }

            if p == line.1 {
                break;
            }

            p += d;
        }
    }

    //dbg!(&map);
    println!("{:?}", &map);
    println!("multiple overlaps count: {}", multiple_overlaps.len());
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use ndarray::array;

    use crate::{parse_lines, Line};

    fn get_test_input() -> Vec<Line> {
        let test_data_str = include_str!("../test_input.txt");

        let test_data_reader = Cursor::new(test_data_str.to_owned());

        parse_lines(test_data_reader)
    }

    #[test]
    fn test_parse() {
        let lines = get_test_input();

        assert_eq!(lines.len(), 10);
    }
}
