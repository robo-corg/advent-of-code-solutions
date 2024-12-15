use std::io::{self, BufRead};
use anyhow::{bail, Result};
use ndarray::Array2;

type Pos = nalgebra::Point2<i32>;
type Vec2 = nalgebra::Vector2<i32>;

#[derive(Debug, Copy, Clone, PartialEq)]
enum Tile {
    Wall,
    Box,
    WideBoxLeft,
    WideBoxRight,
}
impl Tile {
    fn from_ch(ch: char) -> Option<Tile> {
        match ch {
            '#' => Some(Tile::Wall),
            'O' => Some(Tile::Box),
            '.' => None,
            '@' => None,
            bad => panic!("Invalid tile: {:?}", bad)
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum MoveDir {
    Up,
    Right,
    Down,
    Left
}

impl MoveDir {
    fn from_ch(ch: char) -> Self {
        match ch {
            '^' => MoveDir::Up,
            '>' => MoveDir::Right,
            'v' => MoveDir::Down,
            '<' => MoveDir::Left,
            bad => panic!("Invalid move dir: {:?}", bad)
        }
    }

    fn as_vec2(&self) -> Vec2 {
        match self {
            MoveDir::Up => Vec2::new(0, -1),
            MoveDir::Right => Vec2::new(1, 0),
            MoveDir::Down => Vec2::new(0, 1),
            MoveDir::Left => Vec2::new(-1, 0),
        }
    }
}

type Map = Array2<Option<Tile>>;

#[derive(Debug, Clone)]
struct Robot {
    pos: Pos,
    moves: Vec<MoveDir>
}

type Input = (Map, Robot);

fn parse_char_grid<T, Conv>(reader: impl BufRead, conv: Conv) -> Result<(Pos, ndarray::Array2<T>)>
where
    Conv: Fn(char) -> T,
{
    let mut elems = Vec::new();

    let mut width = 0;
    let mut height = 0;
    let mut robo_pos = None;

    for line in reader.lines() {
        let line = line.unwrap();

        if line.is_empty() {
            break;
        }

        if let Some(robo_x) = line.find('@') {
            robo_pos = Some(Pos::new(robo_x as i32, height as i32 ));
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

    Ok((robo_pos.unwrap(), map.reversed_axes()))
}


fn map_get(map: &Map, pos: Pos) -> Option<Tile> {
    map[(pos[0] as usize, pos[1] as usize)]
}


fn map_set(map: &mut Map, pos: Pos, value: Option<Tile>) {
    map[(pos[0] as usize, pos[1] as usize)] = value;
}


fn parse_input(mut reader: impl BufRead) -> Result<Input> {
    let (pos, map) = parse_char_grid(reader.by_ref(), |ch| Tile::from_ch(ch))?;

    let mut moves: Vec<MoveDir> = Vec::new();

    for line in reader.lines() {
        let line = line.unwrap();
        moves.extend(line.chars().map(MoveDir::from_ch));
    }

    Ok((
        map,
        Robot {
            pos,
            moves,
        }
    ))
}

fn get_box_positions<'a>(map: &'a Map) -> impl Iterator<Item=Pos> + 'a {

    let shape = map.shape();
    (0..shape[1]).flat_map(move |y| {
        (0..shape[0]).filter_map(move |x| {
            match map[(x, y)] {
                Some(Tile::Box) | Some(Tile::WideBoxLeft) => Some(Pos::new(x as i32, y as i32)),
                _ => None,
            }
        })
    })
}

fn do_step(map: &mut Map, robot: &mut Robot, step: usize) -> bool {
    if robot.moves.len() <= step {
        return false;
    }

    let robo_move = robot.moves[step];
    let dir = robo_move.as_vec2();
    let next_pos = robot.pos + dir;

    let mut push_lanes = vec![next_pos];

    let mut box_moves = Vec::new();

    'outer: loop {
        let mut next_push_lanes = Vec::new();

        if push_lanes.is_empty() {
            break;
        }

        for check_pos in push_lanes.iter().copied() {
            match map_get(map, check_pos) {
                Some(Tile::Box) => {
                    box_moves.push((Tile::Box, check_pos, check_pos + dir));
                    next_push_lanes.push(check_pos + dir);
                },
                Some(Tile::WideBoxLeft) => {
                    box_moves.push((Tile::WideBoxLeft, check_pos, check_pos + dir));
                    next_push_lanes.push(check_pos + dir);

                    if dir[1] != 0{
                        let off = Vec2::new(1, 0);
                        box_moves.push((Tile::WideBoxRight, check_pos + off, check_pos + dir + off));
                        next_push_lanes.push(check_pos + dir + off);
                    }
                },
                Some(Tile::WideBoxRight) => {
                    box_moves.push((Tile::WideBoxRight, check_pos, check_pos + dir));
                    next_push_lanes.push(check_pos + dir);

                    if dir[1] != 0{
                        let off = Vec2::new(-1, 0);
                        box_moves.push((Tile::WideBoxLeft, check_pos + off, check_pos + dir + off));
                        next_push_lanes.push(check_pos + dir + off);
                    }
                },
                Some(Tile::Wall) => {
                    break 'outer;
                },
                None => {}
            }
        }

        push_lanes = next_push_lanes;
    }

    if push_lanes.is_empty() {
        robot.pos = next_pos;

        for box_move in box_moves.iter().rev() {
            map_set(map, box_move.1, None);
            map_set(map, box_move.2, Some(box_move.0));
        }
    }

    (step + 1) < robot.moves.len()
}

fn print_world(map: &Map, robot: &Robot) {
    let map_size = map.shape();

    for y in 0..map_size[1] {
        for x in 0..map_size[0] {
            let pos = Pos::new(x as i32, y as i32);

            if robot.pos == pos {
                print!("@");
                continue;
            }

            match map_get(map, pos) {
                Some(Tile::Box) => print!("O"),
                Some(Tile::Wall) => print!("#"),
                Some(Tile::WideBoxLeft) => print!("["),
                Some(Tile::WideBoxRight) => print!("]"),
                None => print!("."),
            }
        }

        println!();
    }
}

fn create_wide_map(map: &Map) -> Map {
    let old_shape = map.shape();
    let new_shape = (
        old_shape[0] * 2,
        old_shape[1]
    );

    let mut wide_map = Map::from_elem(new_shape, None);

    for y in 0..old_shape[1] {
        for x in 0..old_shape[0] {
            let pos = Pos::new(x as i32, y as i32);
            let val = map_get(map, pos);

            for (xx, wide_tile) in [Tile::WideBoxLeft, Tile::WideBoxRight].into_iter().enumerate() {
                let w_pos = Pos::new(pos[0]*2, pos[1]) + Vec2::new(xx as i32, 0);

                if val == Some(Tile::Box) {
                    map_set(&mut wide_map, w_pos, Some(wide_tile));
                }
                else {
                    map_set(&mut wide_map, w_pos, val);
                }
            }
        }
    }

    wide_map
}

fn main() -> Result<()> {
    let (map, robot) = {
        let stdin = io::stdin();
        let stdin_lock = stdin.lock();
        parse_input(stdin_lock)?
    };

    print_world(&map, &robot);
    {
        let mut robot = robot.clone();
        let mut map = map.clone();
        let mut step = 0;
        while do_step(&mut map, &mut robot, step) {
            //print_world(&map, &robot);
            step += 1;
        }



        let part1_value: i32 = get_box_positions(&map).map(|b| b[0] + b[1]*100).sum();

        println!("part1: {}", part1_value);
    }

    {
        let mut map = create_wide_map(&map);
        let mut robot = robot.clone();

        robot.pos[0] *= 2;


        print_world(&map, &robot);

        let mut step = 0;
        while do_step(&mut map, &mut robot, step) {
            //print_world(&map, &robot);
            step += 1;
        }

        print_world(&map, &robot);

        let part2_value: i32 = get_box_positions(&map).map(|b| b[0] + b[1]*100).sum();
        println!("part2: {}", part2_value);
    }

    Ok(())
}
