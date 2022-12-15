use nalgebra::SimdPartialOrd;
use ndarray::{s, Array1, Array2};
use regex::Regex;
use std::collections::HashSet;
use std::io::{self, BufRead};

type Map = Array2<i64>;
type Point = nalgebra::Point2<i64>;
type Vec2 = nalgebra::Vector2<i64>;

type Input = Vec<(Point, Point)>;

fn dist(a: &Point, b: &Point) -> i64 {
    (b - a).abs().sum()
}

fn parse_input(mut reader: impl BufRead) -> Input {
    // Sensor at x=2, y=18: closest beacon is at x=-2, y=15
    let re =
        Regex::new(r"Sensor at x=(-?\d+), y=(-?\d+): closest beacon is at x=(-?\d+), y=(-?\d+)")
            .unwrap();

    let mut lines = reader.lines().map(|l| l.unwrap());

    lines
        .map(|line| {
            let captures = re.captures(&line).unwrap();

            dbg!(&captures);

            (
                Point::new(
                    i64::from_str_radix(captures.get(1).unwrap().as_str(), 10).unwrap(),
                    i64::from_str_radix(captures.get(2).unwrap().as_str(), 10).unwrap(),
                ),
                Point::new(
                    i64::from_str_radix(captures.get(3).unwrap().as_str(), 10).unwrap(),
                    i64::from_str_radix(captures.get(4).unwrap().as_str(), 10).unwrap(),
                ),
            )
        })
        .collect()
}

fn get_extents(points: &[(Point, Point)]) -> (Point, Point) {
    points
        .iter()
        .copied()
        .reduce(|(mut min, mut max), (sensor, beacon)| {
            (
                Point::new(
                    min.x.min(sensor.x).min(beacon.x),
                    min.y.min(sensor.y).min(beacon.y),
                ),
                Point::new(
                    max.x.max(sensor.x).max(beacon.x),
                    max.y.max(sensor.y).max(beacon.y),
                ),
            )
        })
        .unwrap()
}

fn non_beacon_positions(sensors: &[(Point, Point)], y: i64) -> i64 {
    let mut intervals = Vec::new();

    for (sensor, beacon) in sensors.iter() {
        let d = dist(sensor, beacon);
        let dist_from_line = i64::abs(y - sensor.y);

        if dist_from_line <= d {
            let slack = d - dist_from_line;

            let interval = (sensor.x - slack, sensor.x + slack + 1);

            dbg!((d, slack, sensor, interval));

            intervals.push(interval);
            //non_beacon += 1 + 2*(d - dist_from_line);
        }
    }

    intervals.sort_by_key(|(s, e)| *s);

    dbg!(&intervals);

    let mut cur_start = intervals[0].0;
    let mut cur_end = intervals[0].1;

    let mut non_beacon = 0;

    let mut non_overlapping = Vec::new();

    for (s, e) in intervals.iter().copied() {
        if s > cur_end {
            non_overlapping.push((cur_start, cur_end));
            cur_start = s;
            cur_end = e;
        } else {
            cur_end = i64::max(e, cur_end);
        }
    }

    //dbg!(cur_start, cur_end);
    //non_beacon += cur_end - cur_start;
    non_overlapping.push((cur_start, cur_end));

    for (s, e) in non_overlapping.iter().copied() {
        let mut beacons_in_inveral = HashSet::new();

        for (_, beacon) in sensors.iter() {
            if beacon.y == y && beacon.x >= s && beacon.x < e {
                beacons_in_inveral.insert(beacon);
            }
        }

        non_beacon += e - s - (beacons_in_inveral.len() as i64);
    }

    non_beacon
}

fn find_beacon(sensors: &[(Point, Point)], max_search: i64) -> Point {
    for y in 0..max_search {
        let mut intervals = Vec::new();

        for (sensor, beacon) in sensors.iter() {
            let d = dist(sensor, beacon);
            let dist_from_line = i64::abs(y - sensor.y);

            if dist_from_line <= d {
                let slack = d - dist_from_line;

                let interval = (sensor.x - slack, sensor.x + slack + 1);

                intervals.push(interval);
                //non_beacon += 1 + 2*(d - dist_from_line);
            }
        }

        intervals.sort_by_key(|(s, e)| *s);

        let mut cur_start = intervals[0].0;
        let mut cur_end = intervals[0].1;

        let mut non_beacon = 0;

        let mut non_overlapping = Vec::new();

        for (s, e) in intervals.iter().copied() {
            if s > cur_end {
                non_overlapping.push((cur_start, cur_end));
                cur_start = s;
                cur_end = e;
            } else {
                cur_end = i64::max(e, cur_end);
            }
        }

        //dbg!(cur_start, cur_end);
        //non_beacon += cur_end - cur_start;
        non_overlapping.push((cur_start, cur_end));

        for (s, e) in non_overlapping.iter().copied() {
            let mut beacons_in_inveral = HashSet::new();

            for (_, beacon) in sensors.iter() {
                if beacon.y == y && beacon.x >= s && beacon.x < e {
                    beacons_in_inveral.insert(beacon);
                }
            }

            non_beacon += i64::min(e, max_search) - i64::max(s, 0); // - (beacons_in_inveral.len() as i64);
        }

        //dbg!(y, non_beacon);

        if non_beacon < max_search {
            let mut possible_x = 0;

            for (s, e) in non_overlapping.iter().copied() {
                if possible_x >= s && possible_x < e {
                    possible_x = e;
                } else {
                    return Point::new(possible_x, y);
                }
            }
        }
    }

    panic!("no solution found");
}

fn main() {
    let input = {
        let stdin = io::stdin();
        let stdin_lock = stdin.lock();
        parse_input(stdin_lock)
    };

    dbg!(&input);

    //dbg!(non_beacon_positions(&input, 10));
    //dbg!(non_beacon_positions(&input, 2000000));

    let part_2_pos = find_beacon(&input, 4000000);

    dbg!(part_2_pos);
    dbg!(part_2_pos.x * 4000000 + part_2_pos.y);
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use crate::{parse_input, Input, Point};

    fn get_test_input() -> Input {
        let test_data_str = include_str!("../test_input.txt");

        let test_data_reader = Cursor::new(test_data_str.to_owned());

        parse_input(test_data_reader)
    }

    #[test]
    fn test_parse() {
        let test_data = get_test_input();

        assert_eq!(test_data[0], (Point::new(2, 18), Point::new(-2, 15)));
    }
}
