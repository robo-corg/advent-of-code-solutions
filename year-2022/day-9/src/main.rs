use std::collections::HashSet;
use std::io::{self, BufRead};
type Point = nalgebra::Point2<i32>;
type Vec2 = nalgebra::Vector2<i32>;

type Input = Vec<Vec2>;

fn parse_input(mut reader: impl BufRead) -> Input {
    reader
        .lines()
        .map(|l| l.unwrap())
        .map(|line| {
            let (dir_s, amount_s) = line.split_once(' ').unwrap();

            let dir = match dir_s {
                "U" => Vec2::new(0, 1),
                "D" => Vec2::new(0, -1),
                "R" => Vec2::new(1, 0),
                "L" => Vec2::new(-1, 0),
                other => panic!("{} not a valid direction!", other),
            };

            let amount = i32::from_str_radix(amount_s, 10).unwrap();

            dir * amount
        })
        .collect()
}


#[derive(Default, Clone, Debug, PartialEq)]
struct SimState {
    head: Point,
    tail: Point,
}

fn signum_vec(v: Vec2) -> Vec2 {
    Vec2::new(i32::signum(v.x), i32::signum(v.y))
}

fn movement_steps(movement: Vec2) -> impl Iterator<Item=Vec2> {
    let step_dir = signum_vec(movement);

    let count = i32::abs(movement.x) + i32::abs(movement.y);

    (0..count).map(move |_|
        step_dir
    )
}

impl SimState {
    fn unit_step(&self, unit_movement: Vec2) -> SimState {
        let new_head = self.head + unit_movement;

        let delta = new_head - self.tail;

        let travel_delta = if i32::abs(delta.x) <= 1 && i32::abs(delta.y) <= 1 {
            // touching
            Vec2::new(0, 0)
        } else {
            Vec2::new(i32::signum(delta.x), i32::signum(delta.y))
        };

        SimState {
            head: new_head,
            tail: self.tail + travel_delta,
        }
    }

    fn apply_movement(&self, movement: Vec2) -> impl Iterator<Item=SimState> {
        let mut states = Vec::new();
        let mut cur_state = self.clone();

        for unit_step in movement_steps(movement) {
            let new_state = cur_state.unit_step(unit_step);
            states.push(new_state.clone());
            cur_state = new_state;
        }

        states.into_iter()
    }

    fn apply_movements(&self, movements: &[Vec2]) -> impl Iterator<Item=SimState> {
        let mut states = Vec::new();
        let mut cur_state = self.clone();

        for movement in movements.iter() {
            states.extend(cur_state.apply_movement(*movement));
            cur_state = states.last().unwrap().clone();
        }

        states.into_iter()
    }
}

fn main() {
    let input = {
        let stdin = io::stdin();
        let stdin_lock = stdin.lock();
        parse_input(stdin_lock)
    };

    dbg!(&input);

    let state = SimState::default();

    let states: Vec<SimState> = state.apply_movements(&input).collect();

    dbg!(&states);

    let mut tail_positions = HashSet::new();

    for st in states.iter() {
        tail_positions.insert(st.tail);
    }

    dbg!(tail_positions.len());
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use crate::{parse_input, Input, Vec2, SimState, Point};

    fn get_test_input() -> Input {
        let test_data_str = include_str!("../test_input.txt");

        let test_data_reader = Cursor::new(test_data_str.to_owned());

        parse_input(test_data_reader)
    }

    #[test]
    fn test_parse() {
        let test_data = get_test_input();

        assert_eq!(test_data[0], Vec2::new(4, 0));
    }

    #[test]
    fn sim_step() {
        let state = SimState {
            head: Point::new(1, 1),
            tail: Point::new(0, 0),
        };

        let new_state = state.unit_step(Vec2::new(0, 1));

        assert_eq!(new_state, SimState {
            head: Point::new(1, 2),
            tail: Point::new(1, 1),
        });
    }
}
