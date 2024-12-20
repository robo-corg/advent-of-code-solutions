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

type State = (Pos, bool);

#[derive(Debug, Clone)]
struct DijkstraState {
    dist: HashMap<State, i32>,
    prev: HashMap<State, State>,
    seen: HashSet<State>
}

impl DijkstraState {
    fn new() -> Self {
        DijkstraState {
            dist: HashMap::new(),
            prev: HashMap::new(),
            seen: HashSet::new(),
        }
    }

    fn dijkstra(&mut self, map: &Map, start_pos: Pos, end_pos: Pos, used_cheat_positions: &HashSet<(Pos, Pos)>, cheat_len:i32) -> Option<(usize, Option<(Pos, Pos)>)> {
        let mut fringe_q = BinaryHeap::new();

        let allow_cheats = cheat_len > 0;

        self.dist.insert((start_pos, false), 0);
        fringe_q.push(SearchItem(0, (start_pos, false)));

        //dbg!(used_cheat_positions);

        while let Some(SearchItem(_, cur)) = fringe_q.pop() {
            self.seen.insert(cur);

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
            ].map(|d| ((cur.0 + d, cur.1), 1));

        // dbg!(&neighbors);

            // let cheat_neighs= neighbors.iter().filter_map(|neigh| {
            //     if allow_cheats && !neigh.1 && !used_cheat_positions.contains(&neigh.0) {
            //         Some((neigh.0.clone(), true))
            //     }
            //     else {
            //         None
            //     }
            // });

            let cheat_neighs = (-cheat_len..cheat_len+1).flat_map(|y_off| {
                (-cheat_len..cheat_len+1).filter_map(move |x_off| {
                    let off = Vec2::new(x_off, y_off);
                    let pos = cur.0 + off;

                    if !map.is_pos_empty(pos) {
                        return None;
                    }

                    let d= off.abs().sum();

                    //dbg!(&off, d, cheat_len);

                    if d > cheat_len || off == Vec2::new(0, 0) {
                        return None;
                    }

                    let neigh = (
                        pos,
                        true
                    );

                    if !cur.1 && !used_cheat_positions.contains(&(cur.0, neigh.0)) {
                        Some((neigh, d))
                    }
                    else {
                        None
                    }
                })
            });

            let cur_cost = self.dist.get(&cur).copied().unwrap_or(i32::MAX);

            for (neigh, neigh_cost_delta) in neighbors.iter().cloned().chain(cheat_neighs) {
                if self.seen.contains(&neigh) {
                    continue;
                }

                assert!(neigh_cost_delta > 0);

                let is_wall = !map.is_pos_empty(neigh.0);
                let cheated = !cur.1 && neigh.1;

                // if cheated {
                //     dbg!(cur, neigh, neigh_cost_delta);
                // }

                if !cheated && is_wall {
                    //assert!(!used_cheat_positions.contains(&neigh.0));
                    continue;
                }

                //dbg!(&neigh);

                let existing_neigh_cost = self.dist.get(&neigh).copied().unwrap_or(i32::MAX);


                let neigh_cost = cur_cost + neigh_cost_delta;

                if neigh_cost < existing_neigh_cost {
                    self.dist.insert(neigh, neigh_cost);
                    self.prev.insert(neigh, cur);

                    fringe_q.push(SearchItem(neigh_cost, neigh));
                }
            }
        }

        if !self.seen.contains(&(end_pos, allow_cheats)) {
            return None;
        }

        let mut path = vec![(end_pos, allow_cheats)];
        let mut cheat_pos = None;
        let cost = self.dist[&(end_pos, allow_cheats)];

        while let Some(next) = self.prev.get(path.last().unwrap()) {
            let last = path.last().unwrap();

            // cheat used
            if last.1 && !next.1 {
                cheat_pos = Some((next.0.clone(), last.0.clone()));
                //assert!(!used_cheat_positions.contains(&last.0));
                assert!(map.is_pos_empty(next.0));
                //assert!(!map.is_pos_empty(last.0));
            }

            path.push(next.clone());
        }

        path.reverse();

        if path[0].0 == start_pos {
            Some((cost as usize, cheat_pos))
        }
        else {
            None
        }
    }
}


fn solve(map: &Map, start_pos: Pos, end_pos: Pos, cheat_len: i32) -> usize {
    let mut used_cheat_positions = HashSet::new();

    let mut base_dijkstra= DijkstraState::new();

    let (base_cost, _) = base_dijkstra.dijkstra(map, start_pos, end_pos, &used_cheat_positions, 0).unwrap();
    base_dijkstra.seen = HashSet::new();
    base_dijkstra.prev = HashMap::new();

    //dbg!(&base_dijkstra.dist);

    dbg!(base_cost);

    let mut cheat_count = 0;

    while let Some((cheat_cost, new_cheat_pos)) = DijkstraState::new().dijkstra(map, start_pos, end_pos, &used_cheat_positions, cheat_len) {
    //while let Some((cheat_cost, new_cheat_pos)) = base_dijkstra.clone().dijkstra(map, start_pos, end_pos, &used_cheat_positions, 1) {

        let new_cheat_pos = new_cheat_pos.unwrap();
        //dbg!(cheat_cost, new_cheat_pos);

        assert!(!used_cheat_positions.contains(&new_cheat_pos));
        used_cheat_positions.insert(new_cheat_pos);

        let savings = base_cost.saturating_sub(cheat_cost);
        // dbg!(savings, new_cheat_pos);
        dbg!(savings);

        if savings >= 100 {
            cheat_count += 1;
        }
        else {
            break;
        }
    }

    cheat_count
}

fn part1(map: &Map, start_pos: Pos, end_pos: Pos) -> usize {
    solve(map, start_pos, end_pos, 1)
}

fn part2(map: &Map, start_pos: Pos, end_pos: Pos) -> usize {
    solve(map, start_pos, end_pos, 20)
}

fn main() -> Result<()> {
    let (map, start_pos, end_pos) = {
        let stdin = io::stdin();
        let stdin_lock = stdin.lock();
        parse_input(stdin_lock)?
    };

    dbg!(&map);

    //println!("Part1: {}", part1(&map, start_pos, end_pos));

    println!("Part2: {}", part2(&map, start_pos, end_pos));

    Ok(())
}
