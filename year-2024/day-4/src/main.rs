use std::io::{self, BufRead};

use ndarray::{Array1, Array2};

type Map = Array2<char>;
type Input = Map;



fn parse_input(mut reader: impl BufRead) -> Input {
    let map_vec: Vec<Array1<_>> = reader
        .lines()
        .map(|line| {
            Array1::from_vec(
                line.unwrap()
                    .chars()
                    .collect(),
            )
        })
        .collect();

    dbg!(&map_vec);

    let width = map_vec[0].len();

    let mut map = Map::from_shape_fn((0, width), |_| '.');

    for row in map_vec.into_iter() {
        map.push_row(row.view()).unwrap();
    }


    map
}

fn parse_input_str(s: &str) -> Input {
    parse_input(s.as_bytes())
}

const XMAS: &str = "XMAS";

fn count_xmas<I: Iterator<Item=char>>(mut iter: I) -> usize {
    let mut iter = iter.peekable();

    let mut count = 0;
    'outer: loop {
        for xmas_ch in XMAS.chars() {
            if let Some(ch) = iter.peek().copied() {
                if xmas_ch != ch {
                    if ch != 'X' {
                        iter.next();
                    }
                    continue 'outer;
                }
                iter.next();
            }
            else {
                return count;
            }
        }

        count += 1;
    }

    count
}

fn diag1(shape: &[usize]) -> impl Iterator<Item=Vec<(usize, usize)>> {
    let w = shape[0];
    let h = shape[1];

    (0..w+h-1).map(move |i| {
        let mut x = usize::min(w-1, i);
        let mut y = i.saturating_sub(x);

        let mut d = Vec::new();

        loop {
            d.push((x, y));

            if x == 0 || y == (h-1) {
                break;
            }

            x -= 1;
            y += 1;
        }

        d
    })
}

fn diag2(shape: &[usize]) -> impl Iterator<Item=Vec<(usize, usize)>> {
    let h = shape[1];

    diag1(shape).map(move |d| {
        d.into_iter().map(|(x, y)| (y, h - x - 1)).collect()
    })
}

fn all_diags(shape: &[usize]) -> impl Iterator<Item=Vec<(usize, usize)>>  {
    diag1(shape).chain(diag2(shape))
}

// fn diag2(shape: &[usize]) -> impl Iterator<Item=Vec<(usize, usize)>> {
//     let w = shape[0];
//     let h = shape[1];

//     (0..h).map(|_x| {
//         (0..start_x+1).map(move |y| (start_x - y, y)).collect()
//     })
// }

fn count_xmas_on_map(map: &Map) -> usize {
    let mut count = 0;

    for col in map.columns() {
        count += count_xmas(col.iter().copied());
        count += count_xmas(col.iter().rev().copied());
    }

    for row in map.rows() {
        count += count_xmas(row.iter().copied());
        count += count_xmas(row.iter().rev().copied());
    }

    //map.slice(info)

    for d in all_diags(map.shape()) {
        let d_vals: Vec<char> = d.into_iter().map(|p| map[p]).collect();

        count += count_xmas(d_vals.iter().copied());
        count += count_xmas(d_vals.iter().rev().copied());
    }

    count
}

fn main() -> anyhow::Result<()> {
    let map = {
        let stdin = io::stdin();
        let stdin_lock = stdin.lock();
        parse_input(stdin_lock)
    };

    dbg!(&map);

    let count = count_xmas_on_map(&map);

    println!("Count: {}", count);

    let mut w_count = 0;

    for w in map.windows((3, 3)) {
        let a = [w[(0,0)], w[(1,1)], w[(2,2)]];
        let b = [w[(2,0)], w[(1,1)], w[(0,2)]];

        let a_match = a == ['M', 'A', 'S'] || a == ['S', 'A', 'M'];
        let b_match = b == ['M', 'A', 'S'] || b == ['S', 'A', 'M'];

        if a_match && b_match {
            w_count += 1;
        }
    }

        println!("Count (part2): {}", w_count);

    Ok(())
}


#[cfg(test)]
mod tests {
    use crate::{count_xmas, count_xmas_on_map, diag1, diag2, parse_input_str};

    #[test]
    fn test_count_xmas_one() {
        let count = count_xmas("XMAS".chars());

        assert_eq!(count, 1);
    }

    #[test]
    fn test_diag1() {
        let diags: Vec<Vec<(usize, usize)>> = diag1(&[10, 10]).collect();
        let expected: Vec<Vec<(usize, usize)>> = vec![];

        dbg!(&diags);

        assert_eq!(diags.len(), 19);

        assert_eq!(diags[0], vec![(0, 0)]);
        assert_eq!(diags[18], vec![(9, 9)]);

        assert_eq!(diags[0].len(), 1);
        assert_eq!(diags[9].len(), 10);
        assert_eq!(diags[10].len(), 9);
        assert_eq!(diags[18].len(), 1);

        //assert_eq!(diags, expected);
    }

    #[test]
    fn test_diag2() {
        let diags: Vec<Vec<(usize, usize)>> = diag2(&[10, 10]).collect();
        let expected: Vec<Vec<(usize, usize)>> = vec![];

        dbg!(&diags);

        assert_eq!(diags.len(), 19);

        assert_eq!(diags[0], vec![(0, 9)]);
        assert_eq!(diags[9], vec![(0, 0), (1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6), (7, 7), (8, 8), (9, 9)]);
        assert_eq!(diags[18], vec![(9, 0)]);

        assert_eq!(diags[0].len(), 1);
        assert_eq!(diags[9].len(), 10);
        assert_eq!(diags[10].len(), 9);
        assert_eq!(diags[18].len(), 1);

        //assert_eq!(diags, expected);
    }

    #[test]
    fn count_xmas_horiz() {
        let map = parse_input_str(
            r#"XMAS
SAMX
XMAS
...."#);

        let count = count_xmas_on_map(&map);

        assert_eq!(count, 3);
    }

    #[test]
    fn count_xmas_vert() {
        let map = parse_input_str(
            r#"XS..
MA..
AM..
SX.."#);

        let count = count_xmas_on_map(&map);

        assert_eq!(count, 2);
    }

    #[test]
    fn count_xmas_diag1() {
        let map = parse_input_str(
            r#"...X
..M.
.A..
S..."#);

        let count = count_xmas_on_map(&map);

        assert_eq!(count, 1);
    }

    #[test]
    fn count_xmas_diag2() {
        let map = parse_input_str(
            r#"X...
.M..
..A.
...S"#);

        let count = count_xmas_on_map(&map);

        assert_eq!(count, 1);
    }

    #[test]
    fn count_xmas_diag1_big_left() {
        let map = parse_input_str(
            r#"...X.
..M..
.A...
S....
....."#);

        let count = count_xmas_on_map(&map);

        assert_eq!(count, 1);
    }

    #[test]
    fn count_xmas_diag1_big_right() {
        let map = parse_input_str(
            r#"....X
...M.
..A..
.S...
....."#);

        let count = count_xmas_on_map(&map);

        assert_eq!(count, 1);
    }

    #[test]
    fn count_xmas_diag2_big_bot() {
        let map = parse_input_str(
            r#".....
X....
.M...
..A..
...S."#);

        let count = count_xmas_on_map(&map);

        assert_eq!(count, 1);
    }

    #[test]
    fn count_xmas_diag2_big_top() {
        let map = parse_input_str(
            r#"X....
.M...
..A..
...S.
....."#);

        let count = count_xmas_on_map(&map);

        assert_eq!(count, 1);
    }


    #[test]
    fn test_check_no_remove() {
        let map = parse_input_str(
            r#"....XXMAS.
.SAMXMS...
...S..A...
..A.A.MS.X
XMASAMX.MM
X.....XA.A
S.S.S.S.SS
.A.A.A.A.A
..M.M.M.MM
.X.X.XMASX"#);

    let count = count_xmas_on_map(&map);

        for x in 0..10 {
            for y in 0..10 {
                if map[(x, y)] != '.' {
                    let mut m = map.clone();
                    m[(x, y)] = '.';

                    let remove_count = count_xmas_on_map(&m);

                    dbg!((x, y));
                    assert_ne!(count, remove_count);
                }
            }
        }
    }
}