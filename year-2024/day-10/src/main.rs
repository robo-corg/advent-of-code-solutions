use anyhow::{bail, Result};
use ndarray::{Array1, Array2};
use std::collections::{HashMap, HashSet};
use std::io::{self, BufRead};

type Pos = nalgebra::Point2<i32>;
type Vec2 = nalgebra::Vector2<i32>;

type Input = Map;

#[derive(Debug)]
struct Map {
    heights: Array2<u8>,
}

impl Map {
    fn pos_with_height<'a>(&'a self, height: u8) -> impl Iterator<Item = Pos> + 'a {
        self.heights
            .indexed_iter()
            .filter_map(move |(p, p_height)| {
                if *p_height == height {
                    Some(Pos::new(p.0 as i32, p.1 as i32))
                } else {
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

    fn neighbors<'a>(&'a self, pos: Pos) -> impl Iterator<Item = Pos> + 'a {
        (0..4)
            .map(|dir| match dir {
                0 => Vec2::new(0, -1),
                1 => Vec2::new(1, 0),
                2 => Vec2::new(0, 1),
                3 => Vec2::new(-1, 0),
                _ => panic!(),
            })
            .map(move |d| pos + d)
            .filter(|n| self.in_bounds(*n))
    }

    fn get(&self, pos: Pos) -> u8 {
        self.heights[(pos[0] as usize, pos[1] as usize)]
    }

    fn dfs<V, S>(&self, start: Pos, mut v: V, succ: S)
    where
        V: FnMut(Option<Pos>, Pos, u8),
        S: Fn(Pos, u8, Pos, u8) -> bool,
    {
        let mut stack = Vec::new();
        stack.push((None, start));

        while let Some((last, cur_pos)) = stack.pop() {
            let val = self.get(cur_pos);

            v(last, cur_pos, val);

            for neigh in self.neighbors(cur_pos) {
                let neigh_val = self.get(neigh);
                if succ(cur_pos, val, neigh, neigh_val) {
                    stack.push((Some(cur_pos), neigh));
                }
            }
        }
    }
}

fn parse_char_grid<T, Conv>(mut reader: impl BufRead, conv: Conv) -> Result<ndarray::Array2<T>>
where
    Conv: Fn(char) -> T,
{
    let mut elems = Vec::new();

    let mut width = 0;
    let mut height = 0;

    for line in reader.lines() {
        let line = line.unwrap();

        let new_width = line.len();

        if width != 0 && new_width != width {
            bail!("Width of rows does not match");
        }

        width = new_width;

        elems.extend(line.chars().map(|ch| conv(ch)));

        height += 1;
    }

    let map = Array2::from_shape_vec((width, height), elems)?;

    Ok(map)
}

fn parse_input(reader: impl BufRead) -> Result<Input> {
    let map: Array2<u8> = parse_char_grid(reader, |ch| ch.to_digit(10).unwrap() as u8)?;

    Ok(Map { heights: map })
}

fn main() -> Result<()> {
    let map = {
        let stdin = io::stdin();
        let stdin_lock = stdin.lock();
        parse_input(stdin_lock)?
    };

    dbg!(&map);

    let starts: Vec<Pos> = map.pos_with_height(0).collect();

    let mut score = 0;
    let mut score_part2 = 0;

    for start in starts.iter() {
        let mut peaks = HashSet::new();

        map.dfs(
            *start,
            |_, pos, val| {
                if val == 9 {
                    peaks.insert(pos);
                    score_part2 += 1;
                }
            },
            |_, prior_height, _, height| height.saturating_sub(prior_height) == 1,
        );

        score += peaks.len();
    }

    println!("part1: {}", score);
    println!("part2: {}", score_part2);

    Ok(())
}
