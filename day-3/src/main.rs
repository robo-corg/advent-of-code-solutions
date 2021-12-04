use std::{io::{self, BufRead}, iter::repeat};

const BIT_SIZE: usize = 12;

fn most_common_bit(count: i32) -> Option<u32> {
    if count == 0 {
        None
    }
    else if count > 0 {
        Some(1)
    }
    else {
        Some(0)
    }
}

fn gamma(counts: &[i32]) -> u32 {
    counts.into_iter().enumerate().map(|(n, count)| {
        most_common_bit(*count).unwrap() << n
    }).reduce(|a, b| a | b).unwrap()
}

fn get_counts(bit_size: usize, readings: &[u32]) -> Vec<i32> {
    let counts_init: Vec<i32> = repeat(0).take(bit_size).collect();

    let counts = readings.iter().fold(counts_init, |mut counts, reading| {
        for bit_index in 0..bit_size {
            let inc = if (reading & (1 << bit_index)) != 0 {
                1
            }
            else {
                -1
            };

            counts[bit_index] += inc;
        }

        counts
    });

    counts
}

fn get_system_reading<F>(readings: Vec<u32>, criteria: F) -> u32
    where F: Fn(i32) -> u32

{
    let mut candidate_readings = readings;

    for bit_index_n in 0..BIT_SIZE {
        if candidate_readings.len() == 1 {
            break;
        }

        let bit_index = BIT_SIZE - bit_index_n - 1;
        let counts = get_counts(BIT_SIZE, &candidate_readings);
        let needed_bit = criteria(counts[bit_index]);

        let new_candidate_readings: Vec<u32> = candidate_readings.iter().copied().filter(|reading| {
            let reading_bit = (reading >> bit_index) & 1;
            reading_bit == needed_bit
        }).collect();

        candidate_readings = new_candidate_readings;
    }

    assert_eq!(candidate_readings.len(), 1);
    candidate_readings.into_iter().next().expect("System should have at least one remaining valid reading")
}

fn parse_readings(lines: Vec<String>) -> Vec<u32> {
    lines.into_iter().map(|s| {
        u32::from_str_radix(s.trim(), 2)
    }).collect::<Result<Vec<_>, _>>().unwrap()
}

fn main() {
    let lines: Vec<String> = {
        let stdin = io::stdin();
        let stdin_lock = stdin.lock();
        stdin_lock.lines().collect::<Result<Vec<_>, _>>().unwrap()
    };

    let reading = parse_readings(lines);


    let counts = get_counts(BIT_SIZE, &reading);
    let gamma: u32 = gamma(&counts);

    let epsilon = (!gamma) & ((1 << (BIT_SIZE )) - 1);

    dbg!(gamma);
    dbg!(epsilon);

    println!("{}", gamma * epsilon);

    let oxygen_reading = get_system_reading(reading.clone(), |bit_count| most_common_bit(bit_count).unwrap_or(1));
    println!("oxygen_reading: {}", oxygen_reading);


    let co2_reading = get_system_reading(reading.clone(), |bit_count| !most_common_bit(bit_count).unwrap_or(1) & 1);
    println!("c02_reading: {}", co2_reading);

    println!("rating: {}", oxygen_reading * co2_reading);
}

#[cfg(test)]
mod test {
    use crate::{parse_readings, get_counts, gamma};

    fn get_test_readings() -> Vec<u32> {
        let test_data_str = include_str!("../test_input.txt");
        let lines: Vec<String> = test_data_str.lines().map(str::to_string).collect();
        parse_readings(lines)
    }

    #[test]
    fn test_parse() {
        let actual = get_test_readings();

        assert_eq!(actual, vec![
            0b00100,
            0b11110,
            0b10110,
            0b10111,
            0b10101,
            0b01111,
            0b00111,
            0b11100,
            0b10000,
            0b11001,
            0b00010,
            0b01010
        ]);
    }

    #[test]
    fn test_gamma() {
        let readings = get_test_readings();
        let counts = get_counts(5, &readings);
        let gamma = gamma(&counts);

        assert_eq!(gamma, 22);
    }
}