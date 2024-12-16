use std::collections::{BinaryHeap, HashMap, HashSet};
use std::io::{self, BufRead};
use anyhow::{bail, Result};
use ndarray::Array2;

type Pos = nalgebra::Point2<i32>;
type Vec2 = nalgebra::Vector2<i32>;


#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
struct Reindeer {
    pos: Pos,
    facing: Vec2
}

type Map = Array2<bool>;
type Input = (Map, Pos, Reindeer);

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


fn map_get(map: &Map, pos: Pos) -> bool {
    map[(pos[0] as usize, pos[1] as usize)]
}


fn map_set(map: &mut Map, pos: Pos, value: bool) {
    map[(pos[0] as usize, pos[1] as usize)] = value;
}

fn rotate_90(v: Vec2) -> Vec2 {
    match (v.x, v.y) {
        (1, 0) => Vec2::new(0, 1),
        (0, 1) => Vec2::new(-1, 0),
        (-1, 0) => Vec2::new(0, -1),
        (0, -1) => Vec2::new(1, 0),
        bad => panic!("Invalid dir {:?}", bad)
    }
}

fn rotate_90_ccw(v: Vec2) -> Vec2 {
    match (v.x, v.y) {
        (0, 1) => Vec2::new(1, 0),
        (-1, 0) => Vec2::new(0, 1),
        (0, -1) => Vec2::new(-1, 0),
        (1, 0) => Vec2::new(0, -1),
        bad => panic!("Invalid dir {:?}", bad)
    }
}

fn cost(a: Reindeer, b: Reindeer) -> i64 {
    if a.pos != b.pos {
        return (b.pos - a.pos).abs().sum() as i64;
    }
    else if a.facing != b.facing {
        //assert_eq!(rotate_90(a.facing), b.facing);
        return 1000;
    }

    eprintln!("Warning: cost returning 0");

    0
}

fn h(a: Pos, b: Pos) -> i64 {
    (b - a).abs().sum() as i64
}

#[derive(Debug)]
struct SearchItem(i64, Reindeer);

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

fn find_path(map: &Map, start_rendeer: Reindeer, end_pos: Pos) -> (i64, Vec<Reindeer>) {
    let mut fringe_set = HashSet::new();
    let mut fringe_q = BinaryHeap::new();

    //let mut seen_set = HashSet::new();

    let mut known_costs: HashMap<Reindeer, i64> = HashMap::new();
    let mut estimated_costs: HashMap<Reindeer, i64> = HashMap::new();
    let mut backpath: HashMap<Reindeer, Reindeer> = HashMap::new();

    fringe_set.insert(start_rendeer.clone());
    fringe_q.push(SearchItem(i64::MAX, start_rendeer));

    known_costs.insert(start_rendeer, 0);

    while let Some(SearchItem(_, cur)) = fringe_q.pop() {
        let cur_cost = known_costs[&cur];

        if cur.pos == end_pos {
            let mut backlist = Vec::new();
            let mut cur_back = cur;

            loop {
                backlist.push(cur_back);

                if let Some(nextback) = backpath.get(&cur_back) {
                    cur_back = *nextback;
                }
                else {
                    break;
                }
            }

            backlist.reverse();

            assert_eq!(backlist[0], start_rendeer);
            assert_eq!(backlist.last().unwrap().pos, end_pos);

            return (cur_cost, backlist);
        }

        fringe_set.remove(&cur);

        let neighbors = [
            Reindeer {
                pos: cur.pos + cur.facing,
                ..cur
            },
            Reindeer {
                facing: rotate_90(cur.facing),
                ..cur
            },
            Reindeer {
                facing: rotate_90_ccw(cur.facing),
                ..cur
            }
        ];

        for neigh in neighbors {
            let is_wall = map_get(map, neigh.pos);

            if is_wall {
                continue;
            }

            let neigh_cost = cur_cost + cost(cur, neigh);

            let existing_cost = known_costs.get(&neigh).copied().unwrap_or(i64::MAX);

            if neigh_cost < existing_cost {
                // udpate backwards path
                backpath.insert(neigh, cur);

                known_costs.insert(neigh, neigh_cost);

                let est_cost = neigh_cost + h(neigh.pos, end_pos);
                estimated_costs.insert(neigh, est_cost);

                if !fringe_set.contains(&neigh) {
                    fringe_set.insert(neigh);
                    fringe_q.push(SearchItem(est_cost, neigh));
                }
            }
        }
    }

    panic!("No path found!");
}



fn find_best_paths(map: &Map, start_rendeer: Reindeer, end_pos: Pos) -> (i64, Vec<Vec<Reindeer>>) {
    let mut fringe_set = HashSet::new();
    let mut fringe_q = BinaryHeap::new();

    //let mut seen_set = HashSet::new();

    let mut known_costs: HashMap<Reindeer, i64> = HashMap::new();
    let mut estimated_costs: HashMap<Reindeer, i64> = HashMap::new();
    let mut backpath: HashMap<Reindeer, Reindeer> = HashMap::new();

    fringe_set.insert(vec![start_rendeer.clone()]);
    fringe_q.push(SearchItem(i64::MAX, start_rendeer));

    known_costs.insert(start_rendeer, 0);

    let mut best_paths = Vec::new();
    let mut best_path_cost = None;

    while let Some(SearchItem(_, cur)) = fringe_q.pop() {
        let cur_cost = known_costs[&cur];

        if cur.pos == end_pos {
            dbg!(cur_cost);
            if best_path_cost.is_none() || Some(cur_cost) == best_path_cost {
                let mut backlist = Vec::new();
                let mut cur_back = cur;

                loop {
                    backlist.push(cur_back);

                    if let Some(nextback) = backpath.get(&cur_back) {
                        cur_back = *nextback;
                    }
                    else {
                        break;
                    }
                }

                backlist.reverse();

                assert_eq!(backlist[0], start_rendeer);
                assert_eq!(backlist.last().unwrap().pos, end_pos);

                best_paths.push(backlist);
                best_path_cost = Some(cur_cost);
            }
        }

        fringe_set.remove(&cur);

        let neighbors = [
            Reindeer {
                pos: cur.pos + cur.facing,
                ..cur
            },
            Reindeer {
                facing: rotate_90(cur.facing),
                ..cur
            },
            Reindeer {
                facing: rotate_90_ccw(cur.facing),
                ..cur
            }
        ];

        for neigh in neighbors {
            let is_wall = map_get(map, neigh.pos);

            if is_wall {
                continue;
            }

            let neigh_cost = cur_cost + cost(cur, neigh);

            let existing_cost = known_costs.get(&neigh).copied().unwrap_or(i64::MAX);

            if neigh_cost < existing_cost {
                // udpate backwards path
                backpath.insert(neigh, cur);

                known_costs.insert(neigh, neigh_cost);

                let est_cost = neigh_cost + h(neigh.pos, end_pos);
                estimated_costs.insert(neigh, est_cost);

                if !fringe_set.contains(&neigh) {
                    fringe_set.insert(neigh);
                    fringe_q.push(SearchItem(est_cost, neigh));
                }
            }
        }
    }

    (best_path_cost.unwrap(), best_paths)
}


fn parse_input(mut reader: impl BufRead) -> Result<Input> {
    let (pos, exit, map) = parse_char_grid(reader.by_ref(), |ch| {
        ch == '#'
    })?;

    Ok((
        map,
        exit,
        Reindeer {
            pos,
            facing: Vec2::new(1, 0)
        }
    ))
}

fn draw_map_with_sol(map: &Map, path: &Vec<Reindeer>) {
    let mut dir_at_pos = HashMap::new();

    for r in path {
        dir_at_pos.insert(r.pos, r.facing);
    }

    let map_size = map.shape();

    for y in 0..map_size[1] {
        for x in 0..map_size[0] {
            let pos = Pos::new(x as i32, y as i32);

            if map_get(map, pos) {
                print!("#");
            }
            else if let Some(dir) = dir_at_pos.get(&pos) {
                let ch = match (dir.x, dir.y) {
                    (1, 0) => '>',
                    (0, 1) => 'v',
                    (-1, 0) => '<',
                    (0, -1) => '^',
                    bad => panic!("Bad dir")
                };

                print!("{}", ch);
            }
            else {
                print!(".");
            }
        }

        println!();
    }
}

fn tile_count( paths: &Vec<Vec<Reindeer>>) -> usize {
    let mut dir_at_pos = HashMap::new();


    for path in paths {
        for r in path {
            dir_at_pos.insert(r.pos, r.facing);
        }
    }

    dir_at_pos.len()
}

fn main() -> Result<()> {
    let (map, end_pos, start) = {
        let stdin = io::stdin();
        let stdin_lock = stdin.lock();
        parse_input(stdin_lock)?
    };

    dbg!(&map);

    let (final_cost, paths) = find_best_paths(&map, start, end_pos);

    let path = paths[0].clone();
    draw_map_with_sol(&map, &path);
    dbg!(paths.len());

    println!("part1: {}", final_cost);

    println!("part2: {}", tile_count(&paths));

    Ok(())
}
