use std::collections::{HashMap, HashSet};
use std::io::{self, BufRead};
use anyhow::Result;

type Pos = nalgebra::Point2<i32>;
type Vec2 = nalgebra::Vector2<i32>;
type Antanee = char;

#[derive(Debug)]
struct Map {
    antennas: HashMap<Antanee, Vec<Pos>>,
    antanna_pos: HashMap<Pos, Antanee>,
    size: Vec2
}

impl Map {
    fn in_bounds(&self, pos: Pos) -> bool {
        pos[0] >= 0 &&
        pos[1] >= 0 &&

        pos[0] < self.size[0] &&
        pos[1] < self.size[1]
    }
}

type Input = Map;


fn parse_input(mut reader: impl BufRead) -> Result<Input> {
    let mut size = Vec2::new(0, 0);

    let mut antennas: HashMap<Antanee, Vec<Pos>> = HashMap::new();
    let mut antanna_pos: HashMap<Pos, Antanee> = HashMap::new();

    for (y, line) in reader.lines().enumerate() {
        for (x, ch) in line?.chars().enumerate() {
            let pos = Pos::new(x as i32, y as i32);

            size[0] = i32::max(size[0], pos[0] + 1);
            size[1] = i32::max(size[1], pos[1] + 1);

            if ch == '.' {
                continue
            }

            antennas.entry(ch).or_default().push(pos);
            antanna_pos.insert(pos, ch);
        }
    }

    Ok(Map {
        antennas,
        antanna_pos,
        size,
    })
}

fn find_antinodes(map: &Map, part2: bool) -> Vec<(Antanee, Pos)> {
    let mut antinodes = Vec::new();

    for (freq, locations) in map.antennas.iter() {
        for i in 0..locations.len() {
            for j in (i+1)..locations.len() {
                let d = locations[i] - locations[j];
                dbg!(freq, i, j, d);


                if part2 {
                    for n in 0.. {
                        let p1 = locations[i] + d*n;

                        if map.in_bounds(p1) {
                            antinodes.push((*freq, p1));
                        }
                        else {
                            break;
                        }
                    }

                    for n in 0.. {
                        let p1 = locations[j] - d*n;

                        if map.in_bounds(p1) {
                            antinodes.push((*freq, p1));
                        }
                        else {
                            break;
                        }
                    }
                }
                else {
                    let p1 = locations[i] + d;
                    let p2 = locations[j] - d;

                    if map.in_bounds(p1) {
                        antinodes.push((*freq, p1));
                    }

                    if map.in_bounds(p2) {
                        antinodes.push((*freq, p2));
                    }
                }
            }
        }
    }

    antinodes
}

fn main() -> Result<()> {
    let map = {
        let stdin = io::stdin();
        let stdin_lock = stdin.lock();
        parse_input(stdin_lock)?
    };

    dbg!(&map);

    let antinodes = find_antinodes(&map, false);

    dbg!(&antinodes);

    let unique_points: HashSet<Pos> = antinodes.iter().map(|(_, p)| *p).collect();

    for y in 0..map.size[1] {
        for x in 0..map.size[0] {
            let pos = Pos::new(x, y);

            if let Some(a) = map.antanna_pos.get(&pos) {
                print!("{}", a);
                continue;
            }

            if unique_points.contains(&pos) {
                print!("x");
            }
            else {
                print!(".");
            }
        }

        println!("");
    }

    println!("part1: {}", unique_points.len());

    let antinodes2 = find_antinodes(&map, true);

    dbg!(&antinodes2);

    let unique_points2: HashSet<Pos> = antinodes2.iter().map(|(_, p)| *p).collect();

    println!("part2: {}", unique_points2.len());

    Ok(())
}
