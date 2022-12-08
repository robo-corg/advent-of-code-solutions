use std::io::{self, BufRead};

use ndarray::{s, Array1, Array2};

type Map = Array2<i32>;

type Input = Map;

fn parse_input(mut reader: impl BufRead) -> Input {
    let map_vec: Vec<Array1<_>> = reader
        .lines()
        .map(|line| {
            Array1::from_vec(
                line.unwrap()
                    .chars()
                    .map(|ch| ch.to_digit(10).unwrap() as i32)
                    .collect(),
            )
        })
        .collect();

    dbg!(&map_vec);

    let width = map_vec[0].len();

    let mut map = Map::zeros((0, width));

    for row in map_vec.into_iter() {
        map.push_row(row.view()).unwrap();
    }

    map
}

fn find_visible_trees<'a>(map: &'a Map) -> impl Iterator<Item = (i32, usize, usize)> + 'a {
    let shape = map.shape();

    let row_count = shape[0];
    let col_count = shape[1];

    let positions = (0..row_count).flat_map(move |row| (0..col_count).map(move |col| (col, row)));

    positions.filter_map(|(x, y)| {
        let height = map[(y, x)];

        //dbg!((x, y), map.column(x).slice(s![0..x]), map.column(x).slice(s![x..]));

        let visible = map.row(y).slice(s![0..x]).iter().all(|v| *v < height)
            | map.row(y).slice(s![x + 1..]).iter().all(|v| *v < height)
            | map.column(x).slice(s![0..y]).iter().all(|v| *v < height)
            | map.column(x).slice(s![y + 1..]).iter().all(|v| *v < height);

        if visible {
            Some((height, x, y))
        } else {
            None
        }
    })
}

fn trees_visible(height: i32, mut iter: impl Iterator<Item = i32>) -> usize {
    let mut count = 0;

    while let Some(v) = iter.next() {
        if v >= height {
            return count + 1;
        }

        count += 1;
    }

    return count;
}

fn scenic_score<'a>(map: &'a Map) -> impl Iterator<Item = (usize, usize, usize)> + 'a {
    let shape = map.shape();

    let row_count = shape[0];
    let col_count = shape[1];

    let positions = (0..row_count).flat_map(move |row| (0..col_count).map(move |col| (col, row)));

    positions.filter_map(|(x, y)| {
        let height = map[(y, x)];

        let s1 = trees_visible(height, map.row(y).slice(s![0..x;-1]).iter().copied());
        let s2 = trees_visible(height, map.row(y).slice(s![x + 1..]).iter().copied());
        let s3 = trees_visible(height, map.column(x).slice(s![0..y;-1]).iter().copied());
        let s4 = trees_visible(height, map.column(x).slice(s![y + 1..]).iter().copied());

        let score = s1 * s2 * s3 * s4;

        if score > 0 {
            Some((score, x, y))
        } else {
            None
        }
    })
}

fn main() {
    let input = {
        let stdin = io::stdin();
        let stdin_lock = stdin.lock();
        parse_input(stdin_lock)
    };

    dbg!(&input);

    let visible: Vec<(i32, usize, usize)> = find_visible_trees(&input).collect();
    //dbg!(&visible);

    let visible_count = visible.len();
    dbg!(visible_count);

    let scenic_scores: Vec<(usize, usize, usize)> = scenic_score(&input).collect();

    let best_tree = scenic_scores
        .iter()
        .max_by_key(|(score, x, y)| *score)
        .unwrap();
    dbg!(best_tree);
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
        let test_data = get_test_input();
    }
}
