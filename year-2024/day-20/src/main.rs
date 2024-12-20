use std::collections::{BinaryHeap, HashMap, HashSet};
use std::io::{self, BufRead};
use anyhow::{bail, Result};
use ndarray::Array2;

type Pos = nalgebra::Point2<i32>;
type Vec2 = nalgebra::Vector2<i32>;


//type Map = Array2<bool>;
type Input = (Map, Pos, Pos);

fn parse_char_grid<T, Conv>(reader: impl BufRead, conv: Conv) -> Result<(Pos, Pos, ndarray::Array2<T>)>
where
    Conv: Fn(char) -> T,
{
    let mut elems = Vec::new();

    let mut width = 0;
    let mut height = 0;
    let mut robo_pos = None;
    let mut exit_pos = None;

    for line in reader.lines() {
        let line = line.unwrap();

        if line.is_empty() {
            break;
        }

        if let Some(robo_x) = line.find('S') {
            robo_pos = Some(Pos::new(robo_x as i32, height as i32 ));
        }


        if let Some(exit_x) = line.find('E') {
            exit_pos = Some(Pos::new(exit_x as i32, height as i32 ));
        }

        let new_width = line.len();

        if width != 0 && new_width != width {
            bail!("Width of rows does not match");
        }

        width = new_width;

        elems.extend(line.chars().map(|ch| conv(ch)));

        height += 1;
    }

    let map = Array2::from_shape_vec((width, height), elems)?;

   // map.t();

    Ok((robo_pos.expect("Start pos"), exit_pos.expect("End pos"), map.reversed_axes()))
}

fn parse_input(mut reader: impl BufRead) -> Result<Input> {
    let (pos, exit, map) = parse_char_grid(reader.by_ref(), |ch| {
        ch == '#'
    })?;

    Ok((
        Map(map),
        pos,
        exit
    ))
}

fn a2_get<T: Copy>(map: &Array2<T>, pos: Pos) -> T {
    map[(pos[0] as usize, pos[1] as usize)]
}


fn a2_set<T>(map: &mut Array2<T>, pos: Pos, value: T) {
    map[(pos[0] as usize, pos[1] as usize)] = value;
}

#[derive(Debug)]
struct Map(Array2<bool>);

impl Map {
    fn size(&self) -> Vec2 {
        let shape = self.0.shape();

        Vec2::new(shape[0] as i32, shape[1] as i32)
    }

    fn is_pos_empty(&self, pos: Pos) -> bool {
        let sz = self.size();

        if pos.x < 0 || pos.y < 0 || pos.x >= sz.x || pos.y >= sz.y {
            return false;
        }

        !a2_get(&self.0, pos)
    }
}

fn cost(a: Pos, b: Pos) -> i32 {
    (a - b).abs().sum()
}

#[derive(Debug)]
struct SearchItem(i32, (Pos, bool));

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


fn dijkstra(map: &Map, start_pos: Pos, end_pos: Pos, used_cheat_positions: &HashSet<Pos>, allow_cheats: bool) -> Option<(usize, Option<Pos>)> {
    let mut dist= HashMap::new();
    let mut prev = HashMap::new();
    let mut fringe_q = BinaryHeap::new();
    let mut seen = HashSet::new();

    dist.insert((start_pos, false), 0);
    fringe_q.push(SearchItem(0, (start_pos, false)));

    while let Some(SearchItem(_, cur)) = fringe_q.pop() {
        seen.insert(cur);

        assert!(allow_cheats || map.is_pos_empty(cur.0));


        // if cur == (end_pos, allow_cheats) {
        //     dbg!("found end");
        //     break;
        // }

        let neighbors = [
            Vec2::new(1, 0),
            Vec2::new(0, 1),
            Vec2::new(0, -1),
            Vec2::new(-1, 0),
        ].map(|d| (cur.0 + d, cur.1));

       // dbg!(&neighbors);

        let cheat_neighs= neighbors.iter().filter_map(|neigh| {
            if allow_cheats && !neigh.1 && !used_cheat_positions.contains(&neigh.0) {
                Some((neigh.0.clone(), true))
            }
            else {
                None
            }
        });

        let cur_cost = dist.get(&cur).copied().unwrap_or(i32::MAX);

        for neigh in neighbors.iter().cloned().chain(cheat_neighs) {
            if seen.contains(&neigh) {
                continue;
            }

            let is_wall = !map.is_pos_empty(neigh.0);
            let cheated = !cur.1 && neigh.1;

            if !cheated && is_wall {
                continue;
            }

            //dbg!(&neigh);

            let existing_neigh_cost = dist.get(&neigh).copied().unwrap_or(i32::MAX);


            let neigh_cost = cur_cost + cost(cur.0, neigh.0);

            if neigh_cost < existing_neigh_cost {
                dist.insert(neigh, neigh_cost);
                let prev_entry = prev.insert(neigh, cur);

                fringe_q.push(SearchItem(neigh_cost, neigh));
            }
        }
    }

    let mut path = vec![(end_pos, allow_cheats)];
    let mut cheat_pos = None;

    while let Some(next) = prev.get(path.last().unwrap()) {
        let last = path.last().unwrap();

        // cheat used
        if last.1 && !next.1 {
            cheat_pos = Some(last.0.clone());
            assert!(map.is_pos_empty(next.0));
        }

        path.push(next.clone());
    }

    path.reverse();

    if path[0].0 == start_pos {
        Some((path.len() - 1, cheat_pos))
    }
    else {
        None
    }
}


fn part1(map: &Map, start_pos: Pos, end_pos: Pos) -> usize {
    let mut used_cheat_positions = HashSet::new();

    let (base_cost, _) = dijkstra(map, start_pos, end_pos, &used_cheat_positions, false).unwrap();

    dbg!(base_cost);

    let mut cheat_count = 0;

    while let Some((cheat_cost, new_cheat_pos)) = dijkstra(map, start_pos, end_pos, &used_cheat_positions, true) {
        dbg!(cheat_cost);

        let new_cheat_pos = new_cheat_pos.unwrap();

        assert!(!used_cheat_positions.contains(&new_cheat_pos));
        used_cheat_positions.insert(new_cheat_pos);

        let savings = base_cost.saturating_sub(cheat_cost);

        if savings >= 100 {
            cheat_count += 1;
        }
        else {
            break;
        }
    }

    cheat_count
}

fn main() -> Result<()> {
    let (map, start_pos, end_pos) = {
        let stdin = io::stdin();
        let stdin_lock = stdin.lock();
        parse_input(stdin_lock)?
    };

    dbg!(&map);

    println!("Part1: {}", part1(&map, start_pos, end_pos));

    Ok(())
}
