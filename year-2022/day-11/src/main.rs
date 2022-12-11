use std::io::{self, BufRead};

mod monkey_num;

use monkey_num::MonkeyNum;

type Input = Vec<Monkey>;

type ValueType = MonkeyNum;

#[derive(Clone, Debug)]
enum Value {
    Old,
    Const(ValueType),
}

#[derive(Clone, Debug)]
enum Operation {
    Add(Value),
    Mul(Value),
}

impl Operation {
    fn eval(&self, old: &ValueType) -> ValueType {
        match self {
            Operation::Add(val) => {
                old + match val {
                    Value::Old => old,
                    Value::Const(c) => c,
                }
            }
            Operation::Mul(val) => {
                old * match val {
                    Value::Old => old,
                    Value::Const(c) => c,
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
enum Test {
    DivisibleBy(i64),
}

impl Test {
    fn eval(&self, new_item: &ValueType) -> bool {
        match self {
            Test::DivisibleBy(amount) => {
                match new_item {
                    MonkeyNum::Modular(modular) => {
                        for (val, val_mod) in modular.iter() {
                            if val_mod == amount {
                                return *val == 0;
                            }
                        }

                        panic!("Attempting to mod monkeynum by unknown divisor");
                    },
                    MonkeyNum::Plain(val) => {
                        (val % amount) == 0
                    },
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
enum Action {
    ThrowTo(usize),
}

#[derive(Clone, Debug)]
pub struct Monkey {
    id: usize,
    items: Vec<ValueType>,
    op: Operation,
    test: Test,
    if_true: Action,
    if_false: Action,
    inspect_count: usize,
}

const NEW_PLUS_OLD: &str = "  Operation: new = old";
const STARTING_ITEMS: &str = "  Starting items: ";
const DIVISIBLE_BY: &str = "  Test: divisible by ";
const IF_TRUE_THROW: &str = "    If true: throw to monkey ";
const IF_FALSE_THROW: &str = "    If false: throw to monkey ";

fn parse_input(mut reader: impl BufRead) -> Input {
    let mut lines = reader.lines().map(|l| l.unwrap());

    let mut monkeys = Vec::new();

    while let Some(monkey_name) = lines.next() {
        if monkey_name == "" {
            continue;
        }

        let starting_items_s = lines.next().unwrap();
        let operation_s = lines.next().unwrap();
        let test_s = lines.next().unwrap();
        let true_branch_s = lines.next().unwrap();
        let false_branch_s = lines.next().unwrap();

        let starting_items: Vec<ValueType> = starting_items_s[STARTING_ITEMS.len()..]
            .split(", ")
            .map(|item| i32::from_str_radix(item, 10).unwrap().into())
            .collect();

        let (operator_s, amount_s) = operation_s[NEW_PLUS_OLD.len() + 1..]
            .split_once(' ')
            .unwrap();
        let amount = match amount_s {
            "old" => Value::Old,
            const_amount => Value::Const(i32::from_str_radix(amount_s, 10).unwrap().into()),
        };
        let op = match operator_s {
            "+" => Operation::Add(amount),
            "*" => Operation::Mul(amount),
            other => panic!("Invalid operation {}", operator_s),
        };

        let test = Test::DivisibleBy(
            i32::from_str_radix(&test_s[DIVISIBLE_BY.len()..], 10)
                .unwrap()
                .into(),
        );

        let if_true = Action::ThrowTo(
            usize::from_str_radix(&true_branch_s[IF_TRUE_THROW.len()..], 10).unwrap(),
        );
        let if_false = Action::ThrowTo(
            usize::from_str_radix(&false_branch_s[IF_FALSE_THROW.len()..], 10).unwrap(),
        );

        monkeys.push(Monkey {
            id: monkeys.len(),
            items: starting_items,
            op,
            test,
            if_true,
            if_false,
            inspect_count: 0,
        });
    }

    monkeys
}

fn execute_round(monkeys: &Vec<Monkey>, stress_reducer: bool) -> Vec<Monkey> {
    let mut next_monkeys = monkeys.clone();

    for cur_monkey_id in 0..next_monkeys.len() {
        let mut monkey = next_monkeys[cur_monkey_id].clone();

        let inspect_amount = monkey.items.len();

        for mut item in monkey.items.drain(..) {
            if !stress_reducer {
                item = item.to_modular(monkeys);
            }

            let new_item: ValueType = monkey.op.eval(&item) / if stress_reducer { 3 } else { 1 };
            let test_res = monkey.test.eval(&new_item);

            let action = if test_res {
                &monkey.if_true
            } else {
                &monkey.if_false
            };

            match action {
                Action::ThrowTo(target_id) => {
                    next_monkeys[*target_id].items.push(new_item);
                }
            }
        }

        next_monkeys[cur_monkey_id].inspect_count += inspect_amount;
        next_monkeys[cur_monkey_id].items = Vec::new();
    }

    next_monkeys
}

fn main() {
    let input = {
        let stdin = io::stdin();
        let stdin_lock = stdin.lock();
        parse_input(stdin_lock)
    };

    dbg!(&input);

    // dbg!(execute_round(&input));
    // dbg!(execute_round(&execute_round(&input)));

    {
        let mut cur_monkeys = input.clone();

        for _ in 0..20 {
            cur_monkeys = execute_round(&cur_monkeys, true);
        }

        //dbg!(&cur_monkeys);

        let mut inspections: Vec<usize> = cur_monkeys.iter().map(|m| m.inspect_count).collect();
        inspections.sort();
        inspections.reverse();
        dbg!(inspections[0] * inspections[1]);
    }

    {
        let mut cur_monkeys = input.clone();

        for _ in 0..10000 {
            cur_monkeys = execute_round(&cur_monkeys, false);
        }

        //dbg!(&cur_monkeys);

        let mut inspections: Vec<usize> = cur_monkeys.iter().map(|m| m.inspect_count).collect();
        inspections.sort();
        inspections.reverse();
        dbg!(&inspections);
        dbg!(inspections[0] * inspections[1]);
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
        let test_data = get_test_input();
    }
}
