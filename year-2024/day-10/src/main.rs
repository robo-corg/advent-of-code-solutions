use std::collections::{HashMap, HashSet};
use std::io::{self, BufRead};
use anyhow::Result;
use ndarray::{Array1, Array2};

type Pos = nalgebra::Point2<i32>;
type Vec2 = nalgebra::Vector2<i32>;

type Input = Map;

#[derive(Debug)]
struct Map {
    heights: Array2<u8>
}

impl Map {
    fn pos_with_height<'a>(&'a self, height: u8) -> impl Iterator<Item=Pos> + 'a {
        self.heights.indexed_iter().filter_map(move |(p, p_height)| {
            if *p_height == height {
                Some(Pos::new(p.0 as i32, p.1 as i32))
            }
            else {
                None
            }
        })
    }

    fn size(&self) -> Vec2 {
        let s = self.heights.shape();
        Vec2::new(s[0] as i32, s[1] as i32)
    }

    fn in_bounds(&self, pos: Pos) -> bool {
        let s = self.size();
        pos[0] >= 0 && pos[1] >= 0 && pos[0] < s[0] && pos[1] < s[1]
    }

    fn neighbors<'a>(&'a self, pos: Pos) -> impl Iterator<Item=Pos> + 'a {
        (0..4).map(|dir| {
            match dir {
                0 => Vec2::new(0, -1),
                1 => Vec2::new(1, 0),
                2 => Vec2::new(0, 1),
                3 => Vec2::new(-1, 0),
                _ => panic!()
            }
        }).map(move |d| {
            pos + d
        }).filter(|n| self.in_bounds(*n))
    }

    fn get(&self, pos: Pos) -> u8 {
        self.heights[(pos[0] as usize, pos[1] as usize)]
    }

    fn search<V, S>(&self, start: Pos, mut v: V, succ: S)
        where V: FnMut(Option<Pos>, Pos, u8),
            S: Fn(Pos, u8, Pos, u8) -> bool
    {
        let mut seen = HashSet::new();
        let mut fringe = Vec::new();
        let mut fringe_set = HashSet::new();

        fringe.push((None, start));
        fringe_set.insert((None, start));

        while let Some((prev_pos, cur_pos)) = fringe.pop() {
            fringe_set.remove(&(prev_pos, cur_pos));
            seen.insert((prev_pos, cur_pos));

            let val = self.get(cur_pos);

            v(prev_pos, cur_pos, val);

            for neigh in self.neighbors(cur_pos) {
                let neigh_val = self.get(neigh);

                let f = (Some(cur_pos), neigh);

                if !seen.contains(&f) && !fringe_set.contains(&f) && succ(cur_pos, val, neigh, neigh_val) {
                    fringe.push(f);
                    fringe_set.insert(f);
                }
            }
        }
    }
}

fn parse_input(mut reader: impl BufRead) -> Result<Input> {
    let map_vec: Vec<Array1<_>> = reader
        .lines()
        .map(|line| {
            Array1::from_vec(
                line.unwrap()
                    .chars()
                    .map(|ch| {
                        let s = ch.to_string();
                        u8::from_str_radix(&s, 10).unwrap()
                    })
                    .collect(),
            )
        })
        .collect();

    dbg!(&map_vec);

    let width = map_vec[0].len();

    let mut map = Array2::from_shape_fn((0, width), |_| 0);

    for row in map_vec.into_iter() {
        map.push_row(row.view()).unwrap();
    }


    Ok(Map {
        heights: map
    })
}

fn count_back_link_paths(p: Pos, back_links: &HashMap<Pos, HashSet<Pos>>) -> i32 {
    if let Some(priors) = back_links.get(&p) {
        if priors.is_empty() {
            return 1;
        }

        let mut count = 0;

        for prior in priors {
            count += count_back_link_paths(*prior, back_links);
        }

        return count;
    }

    1
}

fn main() -> Result<()> {
    let map = {
        let stdin = io::stdin();
        let stdin_lock = stdin.lock();
        parse_input(stdin_lock)?
    };

    dbg!(&map);

    let peaks: Vec<Pos> = map.pos_with_height(9).collect();
    let starts: Vec<Pos> = map.pos_with_height(0).collect();

    let mut score = 0;
    let mut score_part2 = 0;

    for start in starts.iter() {
        let mut peaks = HashSet::new();

        let mut back_links: HashMap<Pos, HashSet<Pos>> = HashMap::new();

        dbg!("search start", &start);

        map.search(*start, |prior_pos, pos, val| {
            dbg!(pos, val);

            if let Some(pp) = prior_pos {
                back_links.entry(pos).or_default().insert(pp);
            }

            if val == 9 {
                peaks.insert(pos);
            }
        }, |pp, prior_height, p, height| {
            //dbg!(&pp, &p);
            let ph = prior_height as i16;
            let h = height as i16;

            //dbg!(ph, h);

            (h - ph) == 1
        });

        for peak in peaks.iter() {
            score_part2 += count_back_link_paths(*peak, &back_links);
        }

        score += peaks.len();
    }

    println!("part1: {}", score);
    println!("part2: {}", score_part2);

    Ok(())
}
