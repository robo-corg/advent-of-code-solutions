use ndarray::{s, Array1, Array2};
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::io::{self, BufRead};

type Map = Array2<i32>;
type Point = nalgebra::Point2<i32>;
type Vec2 = nalgebra::Vector2<i32>;
type Input = (Map, Point, Point);
type Cost = i32;

fn get_neighbors(p: Point, world_size: Vec2) -> impl Iterator<Item = Point> {
    vec![
        p + Vec2::new(-1, 0),
        p + Vec2::new(1, 0),
        p + Vec2::new(0, -1),
        p + Vec2::new(0, 1),
    ]
    .into_iter()
    .filter(move |p| p[0] >= 0 && p[1] >= 0 && p[0] < world_size[0] && p[1] < world_size[1])
}

#[derive(Debug, Eq)]
struct VisitItem {
    estimated_cost: Cost,
    pos: Point,
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

fn get_cost(map: &Map, pos: Point) -> i32 {
    1
}

fn dist(a: Point, b: Point) -> Cost {
    let d = b - a;

    Cost::abs(d[0] as Cost) + Cost::abs(d[1] as Cost)
}

fn h(a: Point, b: Point) -> Cost {
    dist(a, b)
}

fn find_path(map: &Map, start_pos: Point, goal: Point) -> Option<i32> {
    let mut open_set = HashSet::new();
    let mut visit_queue = BinaryHeap::new();
    let mut costs: HashMap<Point, i32> = HashMap::default();

    let shape = map.shape();

    let row_count = shape[0];
    let col_count = shape[1];

    let world_size: Vec2 = Vec2::new(col_count as i32, row_count as i32);

    visit_queue.push(VisitItem {
        estimated_cost: get_cost(&map, start_pos) + h(start_pos, goal),
        pos: start_pos,
    });
    costs.insert(start_pos, get_cost(&map, start_pos));

    while let Some(visit_item) = visit_queue.pop() {
        let VisitItem {
            estimated_cost: current_estimated_cost,
            pos,
        } = visit_item;

        let current_cost = costs.get(&pos).copied().unwrap();

        if pos == goal {
            return Some(current_cost - 1);
        }

        open_set.remove(&pos);

        let cur_height = map[(pos.y as usize, pos.x as usize)];

        for neighbor in get_neighbors(pos, world_size) {
            let neighbor_height = map[(neighbor.y as usize, neighbor.x as usize)];

            if !(neighbor_height - cur_height <= 1) {
                continue;
            }

            let neighbor_cost = current_cost + get_cost(&map, neighbor);

            let cur_best = costs.get(&neighbor).copied();

            if cur_best.is_none() || neighbor_cost < cur_best.unwrap_or(Cost::MAX) {
                costs.insert(neighbor, neighbor_cost);
                //previous_links.insert(neighbor, pos);

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

    None
}

fn parse_input(mut reader: impl BufRead) -> Input {
    let map_vec: Vec<Array1<_>> = reader
        .lines()
        .map(|line| {
            Array1::from_vec(
                line.unwrap()
                    .chars()
                    .map(|ch| (ch as i32) - ('a' as i32))
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

    let start_marker = ('S' as i32) - ('a' as i32);
    let end_marker = ('E' as i32) - ('a' as i32);

    let mut start = None;
    let mut end = None;

    for (pos, val) in map.indexed_iter_mut() {
        if *val == start_marker {
            *val = 0;
            start = Some(Point::new(pos.1 as i32, pos.0 as i32));
        } else if *val == end_marker {
            *val = 25;
            end = Some(Point::new(pos.1 as i32, pos.0 as i32));
        }
    }

    (map, start.unwrap(), end.unwrap())
}

fn all_lowest_elevation<'a>(map: &'a Map) -> impl Iterator<Item = Point> + 'a {
    map.indexed_iter().filter_map(|((y, x), height)| {
        if *height == 0 {
            Some(Point::new(x as i32, y as i32))
        } else {
            None
        }
    })
}

fn main() {
    let (map, start, end) = {
        let stdin = io::stdin();
        let stdin_lock = stdin.lock();
        parse_input(stdin_lock)
    };

    dbg!(&map);

    dbg!(&start, &end);

    dbg!(find_path(&map, start, end));

    let best_part_2 = all_lowest_elevation(&map)
        .filter_map(|lowest| find_path(&map, lowest, end))
        .min();
    dbg!(best_part_2);
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
