use std::{
    io::{self, BufRead},
    iter::Peekable, fmt,
};

use itertools::iproduct;


#[derive(PartialEq, Clone)]
enum PairItem {
    Num(i32),
    Pair(Box<Pair>),
}

impl fmt::Debug for PairItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Num(arg0) => write!(f, "{}", arg0),
            Self::Pair(arg0) => write!(f, "{:?}", arg0),
        }
    }
}

impl From<i32> for PairItem {
    fn from(n: i32) -> Self {
        PairItem::Num(n)
    }
}

impl From<Pair> for PairItem {
    fn from(p: Pair) -> Self {
        PairItem::Pair(Box::new(p))
    }
}

impl PairItem {
    fn pair(&self) -> Option<&Pair> {
        if let PairItem::Pair(p) = self {
            Some(p)
        }
        else {
            None
        }
    }

    fn num(&self) -> Option<i32> {
        if let PairItem::Num(n) = self {
            Some(*n)
        }
        else {
            None
        }
    }

    fn magnitude(&self) -> i32 {
        match self {
            PairItem::Num(n) => *n,
            PairItem::Pair(p) => p.magnitude()
        }
    }
}

#[derive(PartialEq, Clone)]
struct Pair(PairItem, PairItem);

impl fmt::Debug for Pair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{:?},{:?}]", self.0, self.1)
    }
}

impl Pair {
    fn new(a: impl Into<PairItem>, b: impl Into<PairItem>) -> Self {
        Pair(a.into(), b.into())
    }

    fn left_pair(&self) -> Option<&Pair> {
        self.0.pair()
    }

    fn right_pair(&self) -> Option<&Pair> {
        self.1.pair()
    }

    fn right_num(&self) -> Option<i32> {
        self.1.num()
    }

    fn magnitude(&self) -> i32 {
        self.0.magnitude() * 3 + self.1.magnitude() * 2
    }
}

type Input = Vec<Pair>;

#[derive(Copy, Clone, Debug, PartialEq)]
enum Tok {
    OpenBracket,
    CloseBracket,
    Comma,
    Digit(i32),
}

fn tokenize(ch: char) -> Tok {
    match ch {
        '[' => Tok::OpenBracket,
        ']' => Tok::CloseBracket,
        ',' => Tok::Comma,
        digit if digit.is_digit(10) => Tok::Digit(digit.to_digit(10).unwrap() as i32),
        other => panic!("Unexpected token `{}`", other),
    }
}

fn parse_input(mut reader: impl BufRead) -> Input {
    reader
        .lines()
        .map(|maybe_line| {
            let line = maybe_line.unwrap();

            let mut tokens_iter = line.chars().map(tokenize).peekable();

            parse_pair(&mut tokens_iter)
        })
        .collect()
}

fn parse_pair_item(tokens: &mut Peekable<impl Iterator<Item = Tok>>) -> PairItem {
    if tokens.peek().copied() == Some(Tok::OpenBracket) {
        PairItem::Pair(Box::new(parse_pair(tokens)))
    } else {

        let mut digits = Vec::new();

        while let Some(Tok::Digit(_)) = tokens.peek() {
            if let Tok::Digit(d) = tokens.next().unwrap() {
                digits.push(d);
            }
        }

        digits.reverse();

        let mut num = 0;

        for (place, d) in digits.into_iter().enumerate() {
            num += d * 10i32.pow(place as u32);
        }

        PairItem::Num(num)
    }
}

fn expect_token(tokens: &mut Peekable<impl Iterator<Item = Tok>>, tok: Tok) {
    let actual = tokens.next();

    if actual != Some(tok) {
        panic!("Unexpected token `{:?}`, expected `{:?}`", actual, tok);
    }
}

fn parse_pair_str(s: &str) -> Pair {
    let mut tokens_iter = s.chars().map(tokenize).peekable();
    parse_pair(&mut tokens_iter)
}

fn parse_pair(tokens: &mut Peekable<impl Iterator<Item = Tok>>) -> Pair {
    expect_token(tokens, Tok::OpenBracket);

    let lhs = parse_pair_item(tokens);
    expect_token(tokens, Tok::Comma);
    let rhs = parse_pair_item(tokens);
    let ret = Pair::new(lhs, rhs);

    expect_token(tokens, Tok::CloseBracket);

    ret
}

struct ExplodeResult {
    left_over: i32,
    right_over: i32,
    pair: Pair
}

fn add_left_pair(pair: &mut Pair, amount: i32) {
    add_left(&mut pair.0, amount)
}

fn add_left(pair_item: &mut PairItem, amount: i32) {
    match pair_item {
        PairItem::Num(n) => { *n += amount },
        PairItem::Pair(p) => {
            add_left_pair(p, amount);
        }
    }
}

fn add_right_pair(pair: &mut Pair, amount: i32) {
    add_right(&mut pair.1, amount)
}

fn add_right(pair_item: &mut PairItem, amount: i32) {
    match pair_item {
        PairItem::Num(n) => { *n += amount },
        PairItem::Pair(p) => {
            add_right_pair(p, amount);
        }
    }
}

fn explode_pair(pair: &Pair, depth: u32) -> Option<ExplodeResult> {
    if depth < 3 {
        if let Some(explosion) = pair.left_pair().and_then(|child| explode_pair(child, depth + 1)) {
            let mut new_right = pair.1.clone();

            if explosion.right_over != 0 {
                add_left(&mut new_right, explosion.right_over);
            }

            let new_pair = Pair::new(
                PairItem::Pair(Box::new(explosion.pair)),
                new_right,
            );

            return Some(ExplodeResult {
                left_over: explosion.left_over,
                right_over: 0,
                pair: new_pair
            });
        }


        if let Some(explosion) = pair.right_pair().and_then(|child| explode_pair(child, depth + 1)) {
            let mut new_left = pair.0.clone();

            if explosion.left_over != 0 {
                add_right(&mut new_left, explosion.left_over);
            }

            let new_pair = Pair::new(
                new_left,
                PairItem::Pair(Box::new(explosion.pair))
            );

            return Some(ExplodeResult {
                left_over: 0,
                right_over: explosion.right_over,
                pair: new_pair
            });
        }

        None
    }
    else {
        if let Some(left_pair) = pair.left_pair() {
            let mut new_right = pair.1.clone();

            add_left(&mut new_right, left_pair.1.num().unwrap());

            let new_pair = Pair::new(
                PairItem::Num(0),
                new_right
            );

            return Some(ExplodeResult {
                left_over: left_pair.0.num().unwrap(),
                right_over: 0,
                pair: new_pair
            });
        }
        else if let Some(right_pair) = pair.right_pair() {
            let mut new_left = pair.0.clone();

            add_right(&mut new_left, right_pair.0.num().unwrap());

            let new_pair = Pair::new(
                new_left,
                PairItem::Num(0)
            );

            return Some(ExplodeResult {
                left_over: 0,
                right_over: right_pair.1.num().unwrap(),
                pair: new_pair
            });
        }
        else {
            None
        }
    }
}

fn split_num(n: i32) -> Pair {
    Pair::new(
        n / 2,
        n / 2 + i32::from(n % 2 != 0)
    )
}

fn split_pair_item(pair_item: &PairItem) -> Option<Pair> {
    match pair_item {
        PairItem::Num(left_num) => {
            if *left_num > 9 {
                Some(split_num(*left_num))
            }
            else {
                None
            }
        },
        PairItem::Pair(p) => {
            split_pair(&p)
        }
    }
}

fn split_pair(pair: &Pair) -> Option<Pair> {
    split_pair_item(&pair.0).map(|left_split| Pair::new(left_split, pair.1.clone()))
    .or_else(|| split_pair_item(&pair.1).map(|right_split| Pair::new(pair.0.clone(), right_split)))
}

fn eval_reduce_pair(pair: &Pair) -> Pair {
    let mut cur_pair = pair.clone();

    loop {
        let maybe_next_pair = explode_pair(&cur_pair, 0).map(|e| e.pair).or_else(|| split_pair(&cur_pair));

        if let Some(next_pair) = maybe_next_pair {
            // eprintln!("Reduced:");
            // eprintln!("{:?}", cur_pair);
            // eprintln!("{:?}", next_pair);
            cur_pair = next_pair;
        }
        else {
            break;
        }
    }

    cur_pair
}

fn add_pair(lhs: &Pair, rhs: &Pair) -> Pair {
    let lhs_reduced = eval_reduce_pair(lhs);
    let rhs_reduced = eval_reduce_pair(rhs);

    Pair::new(lhs_reduced, rhs_reduced)
}

fn best_pair_magnitude(pairs: &[Pair]) -> (i32, &Pair, &Pair) {
    iproduct!(pairs, pairs).filter_map(|(a, b)| {
        if a != b {
            let reduced_sum = eval_reduce_pair(&add_pair(a, b));
            Some((reduced_sum.magnitude(), a, b))
        }
        else {
            None
        }
    }).max_by_key(|(m, _, _)| *m).unwrap()
}

fn main() {
    let input = {
        let stdin = io::stdin();
        let stdin_lock = stdin.lock();
        parse_input(stdin_lock)
    };

    dbg!(&input);

    let total = eval_reduce_pair(&input.clone().into_iter().reduce(|a, b| add_pair(&a, &b)).unwrap());

    println!("total: {:?}", &total);

    println!("Magnitude: {}", total.magnitude());

    let (best_mag, best_lhs, best_rhs) = best_pair_magnitude(&input);

    println!("Best pair magnitude (part 2): {}", best_mag);
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use crate::{parse_input, Input, parse_pair_str, explode_pair, Pair, PairItem, split_pair, split_num, add_pair, eval_reduce_pair};

    fn get_test_input() -> Input {
        let test_data_str = include_str!("../test_input.txt");

        let test_data_reader = Cursor::new(test_data_str.to_owned());

        parse_input(test_data_reader)
    }

    #[test]
    fn test_parse() {
        let test_data = get_test_input();
    }


    #[test]
    fn test_parse_str_pair() {
        let pair = parse_pair_str("[4,2]");

        assert_eq!(pair, Pair::new(PairItem::Num(4), PairItem::Num(2)))
    }

    #[test]
    fn test_parse_str_pair_multidigit() {
        let pair = parse_pair_str("[14,2]");

        assert_eq!(pair, Pair::new(PairItem::Num(14), PairItem::Num(2)))
    }


    #[test]
    fn test_explode_1() {
        let pair = parse_pair_str("[[[[[9,8],1],2],3],4]");
        let pair_post_explosion = parse_pair_str("[[[[0,9],2],3],4]");
        let explode_results = explode_pair(&pair, 0).expect("Should explode");

        assert_eq!(explode_results.pair, pair_post_explosion);
    }


    #[test]
    fn test_explode_2() {
        let pair = parse_pair_str("[7,[6,[5,[4,[3,2]]]]]");
        let pair_post_explosion = parse_pair_str("[7,[6,[5,[7,0]]]]");
        let explode_results = explode_pair(&pair, 0).expect("Should explode");

        assert_eq!(explode_results.pair, pair_post_explosion);
    }

    #[test]
    fn test_explode_3() {
        let pair = parse_pair_str("[[6,[5,[4,[3,2]]]],1]");
        let pair_post_explosion = parse_pair_str("[[6,[5,[7,0]]],3]");
        let explode_results = explode_pair(&pair, 0).expect("Should explode");

        assert_eq!(explode_results.pair, pair_post_explosion);
    }

    #[test]
    fn test_explode_4() {
        let pair = parse_pair_str("[[3,[2,[1,[7,3]]]],[6,[5,[4,[3,2]]]]]");
        let pair_post_explosion = parse_pair_str("[[3,[2,[8,0]]],[9,[5,[4,[3,2]]]]]");
        let explode_results = explode_pair(&pair, 0).expect("Should explode");

        assert_eq!(explode_results.pair, pair_post_explosion);
    }


    #[test]
    fn test_explode_5() {
        let pair = parse_pair_str("[[3,[2,[8,0]]],[9,[5,[4,[3,2]]]]]");
        let pair_post_explosion = parse_pair_str("[[3,[2,[8,0]]],[9,[5,[7,0]]]]");
        let explode_results = explode_pair(&pair, 0).expect("Should explode");

        assert_eq!(explode_results.pair, pair_post_explosion);
    }

    #[test]
    fn test_split_num_odd() {
        assert_eq!(split_num(11), Pair::new(5, 6));
    }

    #[test]
    fn test_split_1() {
        let pair = parse_pair_str("[[[[0,7],4],[15,[0,13]]],[1,1]]");
        let expected_pair_post_split = parse_pair_str("[[[[0,7],4],[[7,8],[0,13]]],[1,1]]");
        let actual_pair_post_split = split_pair(&pair).expect("Should split");

        assert_eq!(actual_pair_post_split, expected_pair_post_split);
    }

    #[test]
    fn test_reduce_pair_1() {
        let pair = parse_pair_str("[[[[[4,3],4],4],[7,[[8,4],9]]],[1,1]]");
        let expected_reduced = parse_pair_str("[[[[0,7],4],[[7,8],[6,0]]],[8,1]]");
        let reduced = eval_reduce_pair(&pair);

        assert_eq!(reduced, expected_reduced);
    }

    #[test]
    fn test_reduce_pair_2() {
        let pair = parse_pair_str("[[[[0,[4,5]],[0,0]],[[[4,5],[2,6]],[9,5]]],[7,[[[3,7],[4,3]],[[6,3],[8,8]]]]]");
        let expected_reduced = parse_pair_str("[[[[4,0],[5,4]],[[7,7],[6,0]]],[[8,[7,7]],[[7,9],[5,0]]]]");
        let reduced = eval_reduce_pair(&pair);

        assert_eq!(reduced, expected_reduced);
    }


    #[test]
    fn test_add() {
        let lhs_pair = parse_pair_str("[[[0,[4,5]],[0,0]],[[[4,5],[2,6]],[9,5]]]");
        let rhs_pair = parse_pair_str("[7,[[[3,7],[4,3]],[[6,3],[8,8]]]]");

        let expected = parse_pair_str("[[[[4,0],[5,4]],[[7,7],[6,0]]],[[8,[7,7]],[[7,9],[5,0]]]]");

        let actual = eval_reduce_pair(&add_pair(&lhs_pair, &rhs_pair));

        assert_eq!(actual, expected);
    }


}
