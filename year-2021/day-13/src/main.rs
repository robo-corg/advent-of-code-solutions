use std::{io::{self, BufRead}, ops::Index, collections::HashSet};

type Vec2 = nalgebra::Vector2<i32>;
use nalgebra::DimMax;
use ndarray::{s, Array1, Array2};

type Input = (Vec<Vec2>, Vec<Fold>);

#[derive(Debug)]
enum Fold {
    X(i32),
    Y(i32)
}

fn parse_input(mut reader: impl BufRead) -> Input {
    let lines: Vec<String> = reader.lines().collect::<Result<Vec<_>, _>>().unwrap();

    dbg!(&lines);

    let folds_partition = lines.iter().position(|line| line.trim() == "").unwrap();
    dbg!(folds_partition);

    let positions: Vec<Vec2> = lines[..folds_partition].iter().map(|line| {
        let (a, b) = line.split_once(",").unwrap();
        Vec2::new(i32::from_str_radix(a, 10).unwrap(), i32::from_str_radix(b, 10).unwrap())
    }).collect();

    let folds: Vec<Fold> = lines[folds_partition+1..].iter().map(|fold| {
        if !fold.starts_with("fold along ") {
            panic!("line `{}` not a fold instruction", fold);
        }

        let fold_inst = &fold["fold along ".len()..];

        let (axis, amount) = fold_inst.split_once("=").unwrap();
        let amount_parsed = i32::from_str_radix(amount, 10).unwrap();

        match axis {
            "x" => Fold::X(amount_parsed),
            "y" => Fold::Y(amount_parsed),
            other => panic!("Invalid fold axis: `{}`", other)
        }

    }).collect();

    (positions, folds)
}

fn main() {
    let (input_positions, folds) = {
        let stdin = io::stdin();
        let stdin_lock = stdin.lock();
        parse_input(stdin_lock)
    };

    //dbg!(&input);

    let mut positions: HashSet<Vec2> = input_positions.into_iter().collect();

    for fold in folds[0..1].iter() {
        let mut new_positions = HashSet::new();

        for pos in positions.iter() {
            match fold {
                &Fold::X(fold_pos) => {
                    let x = if pos[0] >= fold_pos {
                        2*fold_pos - pos[0]
                    }
                    else {
                        pos[0]
                    };

                    new_positions.insert(Vec2::new(x, pos[1]));
                }
                &Fold::Y(fold_pos) => {
                    let y = if pos[1] >= fold_pos {
                        2*fold_pos - pos[1]
                    }
                    else {
                        pos[1]
                    };

                    new_positions.insert(Vec2::new(pos[0], y));
                }
            }
        }

        positions = new_positions;
    }

    println!("{}", positions.len());



    for fold in folds.iter() {
        let mut new_positions = HashSet::new();

        for pos in positions.iter() {
            match fold {
                &Fold::X(fold_pos) => {
                    let x = if pos[0] >= fold_pos {
                        2*fold_pos - pos[0]
                    }
                    else {
                        pos[0]
                    };

                    new_positions.insert(Vec2::new(x, pos[1]));
                }
                &Fold::Y(fold_pos) => {
                    let y = if pos[1] >= fold_pos {
                        2*fold_pos - pos[1]
                    }
                    else {
                        pos[1]
                    };

                    new_positions.insert(Vec2::new(pos[0], y));
                }
            }
        }

        positions = new_positions;
    }

    let x_size = positions.iter().map(|pos| pos[0]).max().unwrap() + 1;
    let y_size = positions.iter().map(|pos| pos[1]).max().unwrap() + 1;

    let mut grid = Array2::zeros((y_size  as usize, x_size as usize));


    for p in positions.iter() {
        grid[(p[1] as usize, p[0] as usize)] = 1;
    }

    //println!("{:?}", grid);

    for row in 0..y_size {
        for col in 0..x_size {
            let p: Vec2 = Vec2::new(col as i32, row as i32);

            if positions.contains(&p) {
                print!("*")
            }
            else {
                print!(" ");
            }
        }

        println!("");
    }
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use crate::{parse_input, Input};

    fn get_test_input() -> Input {
        let test_data_str = include_str!("../test_input.txt");

        let test_data_reader = Cursor::new(test_data_str.to_owned());

        parse_input(test_data_reader)
    }

    #[test]
    fn test_parse() {
        let (positions, folds) = get_test_input();

        assert_eq!(positions.len(), 18);
        assert_eq!(folds.len(), 2);
    }
}
