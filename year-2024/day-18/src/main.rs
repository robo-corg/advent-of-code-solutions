use std::collections::{BinaryHeap, HashMap, HashSet};
use std::io::{self, BufRead};
use std::usize;
use anyhow::Result;
use itertools::Itertools;
use ndarray::Array2;

type Pos = nalgebra::Point2<i32>;
type Vec2 = nalgebra::Vector2<i32>;
type HeightMap = Array2<i32>;

type PushDowns = Vec<Pos>;

type Input = PushDowns;

fn parse_input(mut reader: impl BufRead) -> Result<Input> {
    let push_downs: PushDowns = reader.lines().map_ok(|line| {
        let coord_s = line.split_once(',').unwrap();
        Pos::new(
            i32::from_str_radix(coord_s.0, 10).unwrap(),
            i32::from_str_radix(coord_s.1, 10).unwrap(),
        )
    }).collect::<Result<PushDowns, _>>()?;

    Ok(push_downs)
}

fn a2_get<T: Copy>(map: &Array2<T>, pos: Pos) -> T {
    map[(pos[0] as usize, pos[1] as usize)]
}


fn a2_set<T>(map: &mut Array2<T>, pos: Pos, value: T) {
    map[(pos[0] as usize, pos[1] as usize)] = value;
}


struct Map {
    pushdowns: HeightMap
}

impl Map {
    fn part1_from_pushdown_list(pushdown_list: PushDowns, count: Option<usize>, sx: usize, sy: usize) -> Self {
        let mut pushdowns_height = HeightMap::from_elem((sx, sy), 0);

        for (t, pos) in pushdown_list.into_iter().take(count.unwrap_or(usize::MAX)).enumerate() {
            let t = t as i32;
            a2_set(&mut pushdowns_height, pos, t as i32 + 1);
        }

        Map {
            pushdowns: pushdowns_height
        }
    }

    fn size(&self) -> Vec2 {
        let shape = self.pushdowns.shape();

        Vec2::new(shape[0] as i32, shape[1] as i32)
    }

    fn is_pos_empty(&self, pos: Pos) -> bool {
        let sz = self.size();

        if pos.x < 0 || pos.y < 0 || pos.x >= sz.x || pos.y >= sz.y {
            return false;
        }

        a2_get(&self.pushdowns, pos) == 0
    }
}

fn cost(a: Pos, b: Pos) -> i32 {
    (a - b).abs().sum()
}

#[derive(Debug)]
struct SearchItem(i32, Pos);

impl Ord for SearchItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0).reverse()
    }
}

impl PartialOrd for SearchItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for SearchItem {}

impl PartialEq for SearchItem {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}


fn dijkstra(map: &Map, start_pos: Pos, end_pos: Pos) -> Option<Vec<Pos>> {
    let mut dist: HashMap<<[_; 3] as IntoIterator>::Item, i32> = HashMap::new();
    let mut prev = HashMap::new();
    let mut fringe_q = BinaryHeap::new();
    let mut seen = HashSet::new();

    dist.insert(start_pos, 0);
    fringe_q.push(SearchItem(0, start_pos));

    while let Some(SearchItem(_, cur)) = fringe_q.pop() {
        seen.insert(cur);

        let neighbors = [
            Vec2::new(1, 0),
            Vec2::new(0, 1),
            Vec2::new(0, -1),
            Vec2::new(-1, 0),
        ].map(|d| cur + d);

       // dbg!(&neighbors);

        let cur_cost = dist.get(&cur).copied().unwrap_or(i32::MAX);

        for neigh in neighbors {
            if seen.contains(&neigh) {
                continue;
            }

            let is_wall = !map.is_pos_empty(neigh);

            if is_wall {
                continue;
            }

            //dbg!(&neigh);

            let existing_neigh_cost = dist.get(&neigh).copied().unwrap_or(i32::MAX);


            let neigh_cost = cur_cost + cost(cur, neigh);

            if neigh_cost < existing_neigh_cost {
                dist.insert(neigh, neigh_cost);
                let prev_entry = prev.insert(neigh, cur);

                fringe_q.push(SearchItem(neigh_cost, neigh));
            }
        }
    }

    let mut path = vec![end_pos];

    while let Some(next) = prev.get(path.last().unwrap()) {
        path.push(next.clone());
    }

    path.reverse();

    if path[0] == start_pos {
        Some(path)
    }
    else {
        None
    }
}

fn main() -> Result<()> {
    let pushdown_list = {
        let stdin = io::stdin();
        let stdin_lock = stdin.lock();
        parse_input(stdin_lock)?
    };

    //let map = Map::part1_from_pushdown_list(pushdown_list, 12, 7, 7);

    let s = 71; let count = 1024;
    //let s = 7; let count = 12;

    {
        let map = Map::part1_from_pushdown_list(pushdown_list.clone(), Some(count), s as usize, s as usize);

        //dbg!(&map);

        let path = dijkstra(&map, Pos::new(0, 0), Pos::new(s-1, s-1)).unwrap();

        //dbg!(&path);

        println!("part1: {}", path.len() - 1);
    }

    {
        //let found_blocker= None;
        let mut found = None;

        for c in 0..pushdown_list.len() {
            let map = Map::part1_from_pushdown_list(pushdown_list.clone(), Some(c), s as usize, s as usize);

            //dbg!(&map);

            let path = dijkstra(&map, Pos::new(0, 0), Pos::new(s-1, s-1));

            if path.is_none() {
                found = Some(pushdown_list[c-1]);
                break;
            }
        };

        println!("Part2: {:?}", found.unwrap());
    }


    Ok(())
}
