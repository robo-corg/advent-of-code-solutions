use std::collections::{HashMap, HashSet};
use std::io::{self, BufRead};
use anyhow::{bail, Result};

type Pos = nalgebra::Point2<i32>;
type Vec2 = nalgebra::Vector2<i32>;

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
enum StepState {
    Patrolling,
    OutOfBounds,
    InfiniteLoop
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
enum Facing {
    Up,
    Right,
    Down,
    Left
}

impl Default for Facing {
    fn default() -> Self {
        Facing::Up
    }
}

impl Facing {
    fn from_char(ch: char) -> Result<Self> {
        match ch {
            '^' => Ok(Facing::Up),
            '>' => Ok(Facing::Right),
            'v' => Ok(Facing::Down),
            '<' => Ok(Facing::Left),
            other => bail!("{:?} not a valid guard", other)
        }
    }

    fn next(&self) -> Facing {
        match self {
            Facing::Up => Facing::Right,
            Facing::Right => Facing::Down,
            Facing::Down => Facing::Left,
            Facing::Left => Facing::Up,
        }

        // match self {
        //     Facing::Up => Facing::Left,
        //     Facing::Right => Facing::Up,
        //     Facing::Down => Facing::Right,
        //     Facing::Left => Facing::Down,
        // }
    }
}

#[derive(Debug, Default, Clone, Hash, Eq, PartialEq)]
struct Guard {
    pos: Pos,
    facing: Facing
}

impl Guard {
    fn next_pos(&self) -> Pos {
        match self.facing {
            Facing::Up => self.pos + Vec2::new(0, -1),
            Facing::Right => self.pos + Vec2::new(1, 0),
            Facing::Down => self.pos + Vec2::new(0, 1),
            Facing::Left => self.pos + Vec2::new(-1, 0),
        }
    }
}


#[derive(Debug, Clone, Hash, Eq, PartialEq)]
struct Hist {
    tick: i32,
    guard: Guard
}

#[derive(Debug, Default, Clone)]
struct WorldState {
    tick: i32,
    size: Vec2,
    guard: Guard,
    guard_history: HashMap<Pos, Vec<Hist>>,
    guard_facings: HashSet<Guard>,
    pos_history: HashSet<Pos>,
    obsticals: HashSet<Pos>,
    bonks: HashSet<Pos>,
    guard_start: Guard,
}

impl WorldState {
    fn legal_pos(&self, pos: Pos) -> bool {
        !self.obsticals.contains(&pos)
    }

    fn in_bounds(&self, pos: Pos) -> bool {
        pos[0] >= 0 &&
        pos[1] >= 0 &&

        pos[0] < self.size[0] &&
        pos[1] < self.size[1]
    }

    fn step(&mut self) -> StepState {
        let mut next_guard = Guard {
            pos: self.guard.next_pos(),
            ..self.guard
        };

        if !self.legal_pos(next_guard.pos) {
            // println!("Bonk!");
            // self.draw();
            // println!();

            self.bonks.insert(self.guard.pos);

            next_guard = Guard {
                facing: self.guard.facing.next(),
                ..self.guard
            };
        }

        self.guard = next_guard;

        if self.guard_facings.contains(&self.guard) {
            return StepState::InfiniteLoop;
        }

        let in_bounds = self.in_bounds(self.guard.pos);

        if in_bounds {
            self.pos_history.insert(self.guard.pos);
            self.guard_history.entry(self.guard.pos).or_default().push(Hist {
                tick: self.tick,
                guard: self.guard.clone()
            });
            self.guard_facings.insert(self.guard.clone());
        }

        self.tick += 1;

        if in_bounds {
            StepState::Patrolling
        }
        else {
            StepState::OutOfBounds
        }
    }

    fn sanity_check(&self) {
        let overlaps = self.pos_history.intersection(&self.obsticals).count();
        assert_eq!(overlaps, 0);
    }

    fn draw(&self) {
        for y in 0..self.size[1] {
            for x in 0..self.size[0] {
                let pos = Pos::new(x as i32, y as i32);
                if self.guard.pos == pos {
                    print!("^");
                }
                else if self.pos_history.contains(&pos) {
                    print!("X");
                }
                else if self.obsticals.contains(&pos) {
                    print!("#");
                }
                else {
                    print!(".");
                }
            }

            println!("");
        }
    }

    fn draw_bonks(&self) {
        for y in 0..self.size[1] {
            for x in 0..self.size[0] {
                let pos = Pos::new(x as i32, y as i32);
                if self.guard.pos == pos {
                    print!("^");
                }
                else if self.bonks.contains(&pos) {
                    print!("B");
                }
                else if self.obsticals.contains(&pos) {
                    print!("#");
                }
                else {
                    print!(".");
                }
            }

            println!("");
        }
    }

    fn reset(&self) -> Self {
        WorldState {
            guard: self.guard_start.clone(),
            guard_start: self.guard_start.clone(),
            obsticals: self.obsticals.clone(),
            size: self.size,
            ..Default::default()
        }
    }

    fn find_infinite_loops<'a>(&'a self) -> impl Iterator<Item=Pos> + 'a {
        self.pos_history.iter().copied().filter(|pos| {
            let mut hypo_world = self.reset();

            hypo_world.obsticals.insert(*pos);

            hypo_world.complete() == StepState::InfiniteLoop
        })
    }

    fn complete(&mut self) -> StepState {
        loop {
            let step_state = self.step();

            if step_state == StepState::Patrolling {
                continue;
            }

            return step_state;
        }
    }
}

fn parse_input(mut reader: impl BufRead) -> Result<WorldState> {
    let mut maybe_guard = None;
    let mut obsticals = HashSet::new();

    let mut y = 0;

    let mut size = Vec2::new(0, 0);

    for (y, line) in reader.lines().enumerate() {
        for (x, ch) in line?.chars().enumerate() {
            let pos = Pos::new(x as i32, y as i32);

            size[0] = i32::max(size[0], pos[0] + 1);
            size[1] = i32::max(size[1], pos[1] + 1);

            match ch {
                '#' => {
                    obsticals.insert(pos);
                }
                '.' =>{}
                g => {
                    maybe_guard = Some(Guard {
                        pos,
                        facing: Facing::from_char(g)?,
                    })
                }
            }
        }
    }

    let mut pos_history = HashSet::new();
    pos_history.insert(maybe_guard.as_ref().unwrap().pos);

    let guard = maybe_guard.unwrap();

    Ok(WorldState {
        tick: 0,
        guard_start: guard.clone(),
        guard,
        guard_history: HashMap::new(),
        pos_history,
        bonks: HashSet::new(),
        guard_facings: HashSet::new(),
        obsticals,
        size
    })
}

fn main() -> Result<()> {
    let map = {
        let stdin = io::stdin();
        let stdin_lock = stdin.lock();
        parse_input(stdin_lock)?
    };

    map.draw();
    println!("");

    //dbg!(&map);

    let mut part1_map = map.clone();

    let part_1_res = part1_map.complete();
    assert_eq!(part_1_res, StepState::OutOfBounds);

    part1_map.draw();

    //println!("");
    //part1_map.draw_bonks();

    part1_map.sanity_check();

    // 5811 too high
    // 5176 too low



    println!("uniq g: {}", part1_map.guard_history.len());
    println!("part 1: {}", part1_map.pos_history.len());

    let infinite_loops = part1_map.find_infinite_loops();

    println!("part 2: {}", infinite_loops.count());

    Ok(())
}
