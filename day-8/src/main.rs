use std::{
    collections::HashSet,
    io::{self, BufRead},
    ops::RangeBounds,
};

type Input = Vec<(Vec<String>, Vec<String>)>;

fn parse_input(mut reader: impl BufRead) -> Input {
    reader
        .lines()
        .map(|line| {
            let line = line.unwrap();
            let (left, right) = line.trim().split_once("|").unwrap();

            (
                left.trim().split_whitespace().map(str::to_string).collect(),
                right
                    .trim()
                    .split_whitespace()
                    .map(str::to_string)
                    .collect(),
            )
        })
        .collect()
}

const DIGIT_COUNTS: [usize; 10] = [
    6, // 0
    2, // 1 *
    5, // 2
    5, // 3
    4, // 4 *
    5, // 5
    6, // 6
    3, // 7
    7, // 8
    6, // 9
];

const DIGITS: [Segments; 10] = [
    0b1110111, // 0
    0b0010010, // 1
    0b1011101, // 2
    0b1011011, // 3
    0b0111010, // 4
    0b1101011, // 5
    0b1101111, // 6
    0b1010010, // 7
    0b1111111, 0b1111011,
];

type SegmentIndex = u8;
type Segments = u8;

fn char_to_segment(ch: char) -> SegmentIndex {
    match ch {
        'a' => 0,
        'b' => 1,
        'c' => 2,
        'd' => 3,
        'e' => 4,
        'f' => 5,
        'g' => 6,
        invalid => panic!("invalid digit {}", invalid),
    }
}

fn encode_number(num: u8) -> Segments {
    DIGITS[num as usize].reverse_bits() >> 1
}

/// Encode array of active segment indices to bitmask
fn encode_segments(segments: &[SegmentIndex]) -> Segments {
    let mut active: u8 = 0;

    for s in segments {
        active |= (1 << s);
    }

    active
}

fn first_active_segment(s: Segments) -> SegmentIndex {
    let eight_leading_zeros = u8::leading_zeros(0b1111111);
    7 - (u8::leading_zeros(s) - eight_leading_zeros) as u8 - 1
}

fn check_solution(decoding: &Vec<(&str, Vec<u8>)>) -> bool {
    if !decoding.iter().all(|entry| entry.1.len() == 1) {
        return false;
    }

    let digits: HashSet<u8> = decoding
        .iter()
        .filter_map(|entry| entry.1.first().copied())
        .collect();

    digits.len() == decoding.len()
}

fn check_solution_open(decoding: &Vec<(&str, Vec<u8>)>) -> bool {
    decoding.iter().all(|entry| entry.1.len() >= 1)
}

fn decode_preamble(
    preamble: &Vec<String>,
    mut full_segment_mappings: [Option<SegmentIndex>; 7],
) -> (Vec<(&str, Vec<u8>)>, [Option<SegmentIndex>; 7]) {
    let mut letter_segment_mapping: [Segments; 7] = [0b1111111; 7];
    let mut decoded_segment_mapping: [Segments; 7] = [0b1111111; 7];

    for bit in 0..7 {
        if full_segment_mappings[bit].is_some() {
            letter_segment_mapping[bit] = 0;
        }
    }

    let known_outputs: Vec<(&str, u8)> = preamble
        .iter()
        .filter_map(|s| {
            let l = s.len();

            if l == DIGIT_COUNTS[1] {
                Some((s.as_str(), 1))
            } else if l == DIGIT_COUNTS[4] {
                Some((s.as_str(), 4))
            } else if l == DIGIT_COUNTS[7] {
                Some((s.as_str(), 7))
            } else if l == DIGIT_COUNTS[8] {
                Some((s.as_str(), 8))
            }
            else {
                None
            }
        })
        .collect();

    for (s, decoded) in known_outputs.iter().copied() {
        let active_segments = DIGITS[decoded as usize];

        let letter_segments: Vec<u8> = s.chars().map(char_to_segment).collect();
        let letter_segments_mask = encode_segments(&letter_segments);

        let decoded_segments_mask = encode_number(decoded);


        for bit in 0..7 {
            let active = (letter_segments_mask & (1 << bit)) != 0;


            if !active {
                letter_segment_mapping[bit] &= !decoded_segments_mask;
            } else {
                letter_segment_mapping[bit] &= decoded_segments_mask;
            }
        }

        loop {
            let mut solved_letter_mask = 0;

            for (segment_index, segment_mask) in letter_segment_mapping.iter().copied().enumerate()
            {
                if segment_mask.count_ones() == 1 {
                    solved_letter_mask |= segment_mask;
                    let mapped_segment = first_active_segment(segment_mask);

                    full_segment_mappings[segment_index] = Some(mapped_segment);
                }
            }

            if solved_letter_mask != 0 {
                // Remove solved segments from the mappings
                for segment_mask in letter_segment_mapping.iter_mut() {
                    *segment_mask &= !solved_letter_mask;
                }
            } else {
                break;
            }

            let mut count_masks = [0; 10];

            for n in 0u8..10 {
                let encoded = encode_number(n);

                let segment_count = u8::count_ones(encoded);
                count_masks[segment_count as usize] |= (!encoded) & 0b1111111;
            }
        }

        //dbg!(full_segment_mappings);
    }

    // Attempt to decode outputting possible candidates
    let decoded_outputs: Vec<(&str, Vec<u8>)> = preamble
        .iter()
        .map(|s| {
            let l = s.len();
            let mut candidates = Vec::new();

            let letter_segments: Vec<u8> = s.chars().map(char_to_segment).collect();
            let letter_segments_mask = encode_segments(&letter_segments);

            'find_digit: for n in 0u8..10 {
                //dbg!(n);
                if DIGIT_COUNTS[n as usize] != l {
                    // dbg!("length wrong");
                    continue;
                }

                let encoded = encode_number(n);

                for bit in 0..7 {
                    let letter_segment_expected_bit = (letter_segments_mask >> bit) & 1;

                    if let Some(encoded_bit) = full_segment_mappings[bit] {
                        let expected_encoded_bit = encoded >> encoded_bit & 1;


                        // if bit does not match it can't be n
                        if letter_segment_expected_bit != expected_encoded_bit {
                            continue 'find_digit;
                        }
                    }
                }

                candidates.push(n);
            }

            (s.as_str(), candidates)
        })
        .collect();

    if check_solution(&decoded_outputs) {
        return (decoded_outputs, full_segment_mappings);
    } else if check_solution_open(&decoded_outputs) {
        let used_encoded_segments: Vec<u8> = full_segment_mappings
            .iter()
            .copied()
            .filter_map(|v| v)
            .collect();

        for (letter_segment, maybe_encoded_segment) in full_segment_mappings.iter().enumerate() {
            if maybe_encoded_segment.is_none() {
                //dbg!(letter_segment);

                for possible_subs in 0..7 {
                    if used_encoded_segments.contains(&possible_subs) {
                        continue;
                    }

                    //dbg!(possible_subs);

                    let mut new_full_segment_mappings: [Option<SegmentIndex>; 7] =
                        full_segment_mappings;
                    new_full_segment_mappings[letter_segment] = Some(possible_subs);

                    let (new_decoded_outputs, new_updated_mappings) =
                        decode_preamble(preamble, new_full_segment_mappings);

                    if check_solution(&new_decoded_outputs) {
                        return (new_decoded_outputs, new_updated_mappings);
                    }
                }

                //full_segment_mappings[letter_segment]
            }
        }
    }

    return (decoded_outputs, full_segment_mappings);
}

fn main() {
    let input = {
        let stdin = io::stdin();
        let stdin_lock = stdin.lock();
        parse_input(stdin_lock)
    };

    dbg!(&input);

    let total: usize = input
        .iter()
        .map(|(_, outputs)| {
            outputs
                .iter()
                .filter(|s| {
                    let l = s.len();

                    l == DIGIT_COUNTS[1]
                        || l == DIGIT_COUNTS[4]
                        || l == DIGIT_COUNTS[7]
                        || l == DIGIT_COUNTS[8]
                })
                .count()
        })
        .sum();

    dbg!(total);

    let mut total = 0;

    //let total: usize = input.iter().flat_map(|(_, outputs)| {
    for (preamble, outputs) in input.iter() {
        let mut full_segment_mappings: [Option<SegmentIndex>; 7] = [None; 7];

        let (decoded_outputs, mapping) = decode_preamble(preamble, full_segment_mappings);

        dbg!(&decoded_outputs);

        if check_solution(&decoded_outputs) {
            let (decoded, _) = decode_preamble(outputs, mapping);

            let digits: Vec<u8> = decoded
                .iter()
                .map(|item| *item.1.first().unwrap())
                .collect();

            let s = digits
                .into_iter()
                .map(|d| format!("{}", d))
                .reduce(|a, b| format!("{}{}", a, b))
                .unwrap();

            dbg!(&s);

            let num = i64::from_str_radix(&s, 10).unwrap();

            total += num;

            // for digit in digits {
            //     print!("{}", digit);
            // }
        }

        //dbg!(decoded_outputs);
    }

    println!("total: {}", total);
    //});
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use crate::{
        encode_number, first_active_segment, parse_input, test, Input, DIGITS, DIGIT_COUNTS, encode_segments,
    };

    fn get_test_input() -> Input {
        let test_data_str = include_str!("../test_input.txt");

        let test_data_reader = Cursor::new(test_data_str.to_owned());

        parse_input(test_data_reader)
    }

    #[test]
    fn test_parse() {
        let test_data = get_test_input();

        assert_eq!(test_data.len(), 10);
    }

    #[test]
    fn test_digit_counts() {
        for (digit_mask, expect_count) in DIGITS.iter().zip(DIGIT_COUNTS.iter()) {
            assert_eq!(digit_mask.count_ones() as usize, *expect_count);
        }
    }

    #[test]
    fn test_encode_number() {
        assert_eq!(encode_number(8), 0b1111111);
    }

    #[test]
    fn test_first_active_segment() {
        assert_eq!(first_active_segment(0b1000000), 6);
    }

    #[test]
    fn test_encode_segments() {
        let segments_vec = vec![2, 0, 6, 4, 3, 1];

        let encoded = encode_segments(&segments_vec);

        dbg!(format!("{:b}", encoded));

        assert_eq!(encoded, 0b1011111);
    }
}
