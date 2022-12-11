
use std::ops::{Add, Div, Mul, Rem};

use crate::{Monkey, Test};


#[derive(Debug, Clone, PartialEq)]
pub enum MonkeyNum {
    Modular(Vec<(i64, i64)>),
    Plain(i64),
}

impl MonkeyNum {
    pub fn zero() -> Self {
        MonkeyNum::Plain(0)
    }

    pub fn to_modular(self, monkeys: &[Monkey]) -> Self {
        match self {
            MonkeyNum::Plain(num) => MonkeyNum::Modular(
                monkeys
                    .iter()
                    .map(|monkey| match monkey.test {
                        Test::DivisibleBy(arm) => (num, arm as i64),
                    })
                    .collect(),
            ),
            already_converted => already_converted,
        }
    }
}

// impl Zero for MonkeyNum {
//     fn zero() -> Self {
//         MonkeyNum::Plain(0)
//     }

//     fn is_zero(&self) -> bool {
//         match self {
//             MonkeyNum::Modular(_) => todo!(),
//             MonkeyNum::Plain(val) => *val == 0,
//         }
//     }
// }

impl From<i32> for MonkeyNum {
    fn from(v: i32) -> Self {
        MonkeyNum::Plain(v as i64)
    }
}

impl<'a> Add<i64> for &'a MonkeyNum {
    type Output = MonkeyNum;

    fn add(self, rhs: i64) -> Self::Output {
        match self {
            MonkeyNum::Modular(_) => todo!(),
            MonkeyNum::Plain(val) => MonkeyNum::Plain(val + rhs),
        }
    }
}

impl Div<i64> for MonkeyNum {
    type Output = MonkeyNum;

    fn div(self, rhs: i64) -> Self::Output {
        match self {
            MonkeyNum::Modular(m) => if rhs == 1 { MonkeyNum::Modular(m) } else { todo!() },
            MonkeyNum::Plain(val) => MonkeyNum::Plain(val / rhs),
        }
    }
}

impl<'a> Add for &'a MonkeyNum {
    type Output = MonkeyNum;

    fn add(self, rhs: Self) -> Self::Output {
        match self {
            MonkeyNum::Modular(modular) => match rhs {
                MonkeyNum::Modular(_) => todo!(),
                MonkeyNum::Plain(rhs_val) => MonkeyNum::Modular(
                    modular
                        .iter()
                        .map(|(val, val_mod)|((*val + rhs_val) % *val_mod, *val_mod))
                        .collect(),
                ),
            },
            MonkeyNum::Plain(val) => match rhs {
                MonkeyNum::Modular(_) => todo!(),
                MonkeyNum::Plain(rhs_val) => MonkeyNum::Plain(*val + *rhs_val),
            },
        }
    }
}

impl<'a> Mul for &'a MonkeyNum {
    type Output = MonkeyNum;

    fn mul(self, rhs: Self) -> Self::Output {
        match self {
            MonkeyNum::Modular(modular) => match rhs {
                MonkeyNum::Modular(modular_rhs) => {
                    MonkeyNum::Modular(modular.iter().zip(modular_rhs).map(|(lhs, rhs)| {
                        assert_eq!(lhs.1, rhs.1);
                        ((lhs.0 * rhs.0) % lhs.1, lhs.1)
                    }).collect())
                },
                MonkeyNum::Plain(rhs_val) => MonkeyNum::Modular(
                    modular
                        .iter()
                        .map(|(val, val_mod)|((*val * (rhs_val % *val_mod)) % *val_mod, *val_mod))
                        .collect(),
                ),
            },
            MonkeyNum::Plain(val) => match rhs {
                MonkeyNum::Modular(_) => todo!(),
                MonkeyNum::Plain(rhs_val) => MonkeyNum::Plain(*val * *rhs_val),
            },
        }
    }
}

// impl<'a> Rem for &'a MonkeyNum {
//     type Output = MonkeyNum;

//     fn rem(self, rhs: Self) -> Self::Output {
//         match self {
//             MonkeyNum::Modular(modular) => {

//             },
//             MonkeyNum::Plain(val) => match rhs {
//                 MonkeyNum::Modular(_) => todo!(),
//                 MonkeyNum::Plain(rhs_val) => MonkeyNum::Plain(*val % *rhs_val),
//             },
//         }
//     }
// }
