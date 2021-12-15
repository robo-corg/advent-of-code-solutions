use std::{io::{self, BufRead}, collections::HashMap};

type Input = (String, Vec<Rule>);

#[derive(Debug)]
struct Rule([char; 2], char);

fn parse_input(mut reader: impl BufRead) -> Input {
    let mut lines_iter = reader.lines();

    let temalate = lines_iter.next().unwrap().unwrap();
    let _blank_line = lines_iter.next().unwrap().unwrap();

    let rules: Vec<Rule> = lines_iter.map(|maybe_line| {
        let line = maybe_line.unwrap();
        let (a, b) = line.split_once(" -> ").unwrap();

        let mut a_chars = a.chars();

        Rule(
            [a_chars.next().unwrap(), a_chars.next().unwrap()],
            b.chars().next().unwrap()
        )
    }).collect();

    (temalate, rules)
}

type LetterCounts = HashMap<char, usize>;

fn merge_counts(a: Option<&LetterCounts>, b: Option<&LetterCounts>) -> LetterCounts {
    let mut merged_counts = LetterCounts::new();

    for (key, value) in a.iter().flat_map(|count| count.iter()).chain(b.iter().flat_map(|count| count.iter())) {
        let c = merged_counts.entry(*key).or_insert(0);
        *c += value;
    }

    merged_counts
}

/// Recursively counts expansions of rules, starting with a single pair from the polymer
fn expand_and_count(rules: &Vec<Rule>, memo: &mut HashMap<(char, char, u8), LetterCounts>, a: char, b: char, depth: u8) {
    if memo.contains_key(&(a, b, depth))  {
        return;
    }

    let pred = [
       a,b
    ];

    for rule in rules.iter() {
        if rule.0 == pred {
            let mut new_counts = if depth > 0 {
                expand_and_count(rules, memo, a, rule.1, depth - 1);
                expand_and_count(rules, memo, rule.1, b, depth - 1);

                let a_counts = memo.get(&(a, rule.1, depth - 1));
                let b_counts = memo.get(&(rule.1, b, depth - 1));

                merge_counts(a_counts, b_counts)
            }
            else {
                LetterCounts::new()
            };

            {
                let c= new_counts.entry(rule.1).or_insert(0);
                *c += 1;
            }

            memo.insert((a, b, depth), new_counts);

            return;
        }
    }


    let new_counts = if depth > 0 {
        dbg!((a, b, depth - 1));
        memo.get(&(a, b, depth - 1)).unwrap().clone()
    }
    else {
        LetterCounts::new()
    };

    memo.insert((a, b, depth), new_counts);
}

fn main() {
    let (input_polymer, rules) = {
        let stdin = io::stdin();
        let stdin_lock = stdin.lock();
        parse_input(stdin_lock)
    };

    dbg!(&input_polymer);

    let mut polymer = input_polymer;

    let mut element_counts = HashMap::new();
    let mut memo = HashMap::new();

    for ch in polymer.chars() {
        let el_count = element_counts.entry(ch).or_insert(0);
        *el_count += 1;
    }

    // for pass_count in 0..10 {
    //     //let mut next_polymer = polymer.clone();
    //     let mut insert_pos = 1;

    //     let mut chars = polymer.chars().peekable();

    //     let mut next_polymer_buf = String::with_capacity(polymer.len()*2);

    //     while let (Some(ch_a), Some(ch_b)) = (chars.next(), chars.peek()) {
    //         //dbg!(ch_a, ch_b);

    //         next_polymer_buf.push(ch_a);

    //         let pred = [
    //             ch_a,
    //             *ch_b
    //         ];

    //         for rule in rules.iter() {
    //             if rule.0 == pred {
    //                 //next_polymer.insert(insert_pos, rule.1);
    //                 next_polymer_buf.push(rule.1);
    //                 insert_pos += 1;

    //                 let el_count = element_counts.entry(rule.1).or_insert(0);
    //                 *el_count += 1;
    //                 break;
    //             }
    //         }

    //         insert_pos += 1;
    //     }

    //     next_polymer_buf.push(polymer.chars().last().unwrap());

    //     //assert_eq!(&next_polymer, &next_polymer_buf);

    //     polymer = next_polymer_buf;

    //     println!("{}/40 {}", pass_count, polymer.len());
    // }

    let mut chars = polymer.chars().peekable();

    //let mut next_polymer_buf = String::with_capacity(polymer.len()*2);

    let PASS_COUNT = 39;

    while let (Some(ch_a), Some(ch_b)) = (chars.next(), chars.peek()) {
        expand_and_count(&rules, &mut memo, ch_a, *ch_b, PASS_COUNT);

        let counts_output = memo.get(&(ch_a, *ch_b, PASS_COUNT));

        element_counts = merge_counts(Some(&element_counts), counts_output);

        println!("{}", memo.len());
    }

    //dbg!(&polymer);
    dbg!(&element_counts);

    let mut counts_sorted: Vec<_> = element_counts.values().collect();
    counts_sorted.sort();

    dbg!(counts_sorted);


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
        let (template, rules) = get_test_input();

        assert_eq!(rules.len(), 16);
    }
}
