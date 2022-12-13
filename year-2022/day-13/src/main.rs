use std::io::{self, BufRead};
use std::iter;
use std::str::FromStr;

use nom::branch::alt;
use nom::character::complete::one_of;
use nom::character::is_digit;
use nom::combinator::{complete, map, map_res, recognize};
use nom::multi::{many1, separated_list0};
use nom::sequence::terminated;
use nom::{
    // see the "streaming/complete" paragraph lower for an explanation of these submodules
    character::complete::char,
    sequence::delimited,
    IResult,
};

#[derive(Debug, Eq, PartialEq, Ord)]
enum Value {
    List(Vec<Value>),
    Scalar(i32),
}

impl Value {
    fn divider(val: i32) -> Self {
        Value::List(vec![Value::List(vec![Value::Scalar(val)])])
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Value::List(lhs), Value::List(rhs)) => lhs.partial_cmp(rhs),
            (Value::List(lhs), Value::Scalar(rhs)) => lhs.partial_cmp(&vec![Value::Scalar(*rhs)]),
            (Value::Scalar(lhs), Value::List(rhs)) => vec![Value::Scalar(*lhs)].partial_cmp(rhs),
            (Value::Scalar(lhs), Value::Scalar(rhs)) => lhs.partial_cmp(rhs),
        }
    }
}

fn decimal(input: &str) -> IResult<&str, &str> {
    recognize(many1(one_of("0123456789")))(input)
}

fn number(input: &str) -> IResult<&str, i32> {
    map_res(decimal, |out: &str| i32::from_str_radix(out, 10))(input)
}

fn parens_or_scaler(input: &str) -> IResult<&str, Value> {
    alt((map(number, Value::Scalar), parens))(input)
}

fn parens(input: &str) -> IResult<&str, Value> {
    let (i, items): (_, Vec<Value>) = delimited(
        char('['),
        separated_list0(char(','), parens_or_scaler),
        char(']'),
    )(input)?;

    Ok((i, Value::List(items)))
}

impl Value {}

impl FromStr for Value {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (_, value) =
            complete(parens_or_scaler)(s).map_err(|e| anyhow::anyhow!("Parse error: {:?}", e))?;
        Ok(value)
    }
}

type Input = Vec<(Value, Value)>;

fn parse_input(mut reader: impl BufRead) -> Input {
    let mut lines = reader.lines().map(|l| l.unwrap());

    let mut output = Vec::new();

    while let Some(first_line) = lines.next() {
        let second_line = lines.next().unwrap();
        let _skip_blank = lines.next();

        output.push((
            Value::from_str(&first_line).unwrap(),
            Value::from_str(&second_line).unwrap(),
        ));
    }

    output
}

fn flatten_pairs<'a>(pairs: &'a [(Value, Value)]) -> impl Iterator<Item = &'a Value> + 'a {
    pairs
        .iter()
        .flat_map(|p| iter::once(&p.0).chain(iter::once(&p.1)))
}

fn ordered_packets<'a>(packets_iter: impl Iterator<Item = &'a Value> + 'a) -> Vec<&'a Value> {
    let mut packets: Vec<&'a Value> = packets_iter.collect();
    packets.sort();
    packets
}

fn part_2<'a>(pairs: &'a [(Value, Value)]) -> usize {
    let divider_2 = Value::divider(2);
    let divider_6 = Value::divider(6);

    let ordered_packets = ordered_packets(
        flatten_pairs(pairs)
            .chain(iter::once(&divider_2))
            .chain(iter::once(&divider_6)),
    );

    let divider_2_pos = ordered_packets
        .iter()
        .position(|p| *p == &divider_2)
        .unwrap();
    let divider_6_pos = ordered_packets
        .iter()
        .position(|p| *p == &divider_6)
        .unwrap();

    (divider_2_pos + 1) * (divider_6_pos + 1)
}

fn main() {
    let input = {
        let stdin = io::stdin();
        let stdin_lock = stdin.lock();
        parse_input(stdin_lock)
    };

    dbg!(&input);

    let c_indices: Vec<usize> = correct_indices(&input).collect();
    dbg!(&c_indices);
    let correct_sum: usize = c_indices.iter().sum();

    dbg!(correct_sum);

    dbg!(part_2(&input));
}

fn correct_indices<'a>(pairs: &'a [(Value, Value)]) -> impl Iterator<Item = usize> + 'a {
    pairs.iter().enumerate().filter_map(
        |(ind, pair)| {
            if pair.0 < pair.1 {
                Some(ind + 1)
            } else {
                None
            }
        },
    )
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

        {
            let (left, right) = &test_data[0];
            assert!(left < right);
        }

        {
            let (left, right) = &test_data[1];
            assert!(left > right);
        }
    }
}
