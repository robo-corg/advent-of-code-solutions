use core::fmt;
use std::io::{self, BufRead};
use std::ops;
use std::str::FromStr;
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Mutex};
use anyhow::Result;
use bigdecimal::{BigDecimal, FromPrimitive};
use rayon::prelude::*;
use rustc_hash::FxHashMap;


type Input = StoneStore;
//type Stone = BigDecimal;
pub type StoneStore = Vec<Stone>;

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Stone {
    num: u64,
    digits: u16
}

fn get_digits(num: u64) -> u16 {
    if num == 0 {
        return 1;
    }

    f64::log10(num as f64).floor() as u16 + 1
}

impl Stone {
    fn from_i32(num: i32) -> Stone {
        Stone {
            num: num as u64,
            digits: get_digits(num as u64)
        }
    }

    fn from_u64(num: u64) -> Stone {
        Stone {
            num: num as u64,
            digits: get_digits(num)
        }
    }

    fn from_str(s: &str) -> Result<Stone> {
        let num = u64::from_str_radix(s, 10)?;
        Ok(Self::from_u64(num))
    }

    fn digits(&self) -> usize {
        assert_eq!(self.digits as usize, self.to_string().len());
        self.digits as usize
    }
}

impl ops::Mul for Stone {
    type Output = Stone;

    fn mul(self, rhs: Self) -> Self::Output {
        let num = self.num * rhs.num;
        Stone::from_u64(num)
    }
}

impl fmt::Display for Stone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.num)
    }
}

pub fn parse_input(mut reader: impl BufRead) -> Result<Input> {
    let mut buf = String::new();
    reader.read_to_string(&mut buf)?;

    parse_input_str(&buf)
}

pub fn parse_input_str(buf: &str) -> Result<StoneStore> {
    let nums = buf.trim().split(' ').map(|num_s| Stone::from_str(num_s).unwrap()).collect();
    Ok(nums)
}

fn memoized_run_stone(num: Stone, steps_needed: u32, cache: &mut FxHashMap<(Stone, u32), usize>) -> usize {
    if steps_needed == 0 {
        return 1;
    }

    let cache_key = (num, steps_needed);

    if let Some(res) = cache.get(&cache_key) {
        return *res;
    }

    let num = cache_key.0;

    let zero = Stone::from_i32(0);
    let one = Stone::from_i32(1);
    let magic = Stone::from_i32(2024);

    let new_nums = if num == zero {
        vec![one.clone()]
    }
    else if num.digits() % 2 == 0 {
        let nums_str = num.to_string();

        let mid = nums_str.len()/2;

        let a_s = &nums_str[0..mid];
        let b_s = &nums_str[mid..];

        let a = Stone::from_str(a_s).unwrap();
        let b = Stone::from_str(b_s).unwrap();

        vec![a, b]
    }
    else {
        vec![num * magic]
    };

    let mut total = 0;

    for new_num in new_nums.into_iter() {
        let res = memoized_run_stone(new_num, steps_needed - 1, cache);
        total += res;
    }

    cache.insert((num, steps_needed), total);

    total
}

pub fn memoized_run_stones(stones: &[Stone], steps_needed: u32) -> u128 {
    let mut cache = FxHashMap::default();

    let mut total = 0;

    for stone in stones.into_iter() {
        total += memoized_run_stone(stone.clone(), steps_needed, &mut cache) as u128;
    }

    total
}