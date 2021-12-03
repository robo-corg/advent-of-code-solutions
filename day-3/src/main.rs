use std::{io::{self, BufRead}, iter::repeat};
use anyhow::{Error, anyhow};

const BIT_SIZE: usize = 12;

fn old_main() -> Result<(), anyhow::Error> {
    let lines: Vec<String> = {
        let stdin = io::stdin();
        let stdin_lock = stdin.lock();
        stdin_lock.lines().collect::<Result<Vec<_>, _>>()?
    };

    let reading: Vec<u32> = lines.into_iter().map(|s| {
        u32::from_str_radix(s.trim(), 2)
    }).collect::<Result<Vec<_>, _>>()?;

    dbg!(&reading);

    let counts_init: Vec<i32> = repeat(0).take(BIT_SIZE).collect();

    let counts = reading.into_iter().fold(counts_init, |mut counts, reading| {
        for bit_index in 0..BIT_SIZE {
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

    dbg!(&counts);

    let gamma: u32 = counts.into_iter().enumerate().map(|(n, count)| {
        let bit = if count > 0 {
            1
        }
        else if count < 0 {
            0
        }
        else {
            panic!("bit is even");
        };
        bit << n
    }).reduce(|a, b| a | b).unwrap();

    //3813416

    let epsilon = (!gamma) & ((1 << (BIT_SIZE )) - 1);

    dbg!(gamma);
    dbg!(epsilon);

    println!("{}", gamma * epsilon);

    Ok(())
}

fn calc(readings: &Vec<u32>) -> Vec<i32> {
    let counts_init: Vec<i32> = repeat(0).take(BIT_SIZE).collect();

    let counts = readings.iter().fold(counts_init, |mut counts, reading| {
        for bit_index in 0..BIT_SIZE {
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

fn main() -> Result<(), anyhow::Error> {
    let lines: Vec<String> = {
        let stdin = io::stdin();
        let stdin_lock = stdin.lock();
        stdin_lock.lines().collect::<Result<Vec<_>, _>>()?
    };

    let reading: Vec<u32> = lines.into_iter().map(|s| {
        u32::from_str_radix(s.trim(), 2)
    }).collect::<Result<Vec<_>, _>>()?;

    dbg!(&reading);


    let mut oxygen_candidate_readings = reading.clone();

    for bit_index_n in 0..BIT_SIZE {
        if oxygen_candidate_readings.len() == 1 {
            break;
        }

        let bit_index = BIT_SIZE - bit_index_n - 1;
        let counts = calc(&oxygen_candidate_readings);

        let new_oxygen_candidate_readings: Vec<u32> = oxygen_candidate_readings.iter().copied().filter(|reading| {
            let needed_bit = if counts[bit_index] == 0 {
                1
            }
            else {
                if counts[bit_index] > 0 {
                    1
                }
                else {
                    0
                }
            };


            let reading_bit = (reading >> bit_index) & 1;
            reading_bit == needed_bit
        }).collect();

        oxygen_candidate_readings = new_oxygen_candidate_readings;
    }

    dbg!(&oxygen_candidate_readings);

    println!("oxygen_reading: {}", oxygen_candidate_readings[0]);


    let mut co2_candidate_readings = reading.clone();

    for bit_index_n in 0..BIT_SIZE {
        if co2_candidate_readings.len() == 1 {
            break;
        }

        let bit_index = BIT_SIZE - bit_index_n - 1;
        let counts = calc(&co2_candidate_readings);

        //dbg!(&co2_candidate_readings);
        dbg!(&counts);

        let new_co2_candidate_readings: Vec<u32> = co2_candidate_readings.iter().copied().filter(|reading| {
            let needed_bit = if counts[bit_index] == 0 {
                0
            }
            else {
                if counts[bit_index] > 0 {
                    0
                }
                else {
                    1
                }
            };


            let reading_bit = (reading >> bit_index) & 1;
            //let needed_bit = (gamma >> bit_index) & 1;
            //dbg!(reading, bit_index, reading_bit, needed_bit);
            reading_bit == needed_bit
        }).collect();

        co2_candidate_readings = new_co2_candidate_readings;
    }

    dbg!(&co2_candidate_readings);

    println!("c02_reading: {}", co2_candidate_readings[0]);

    println!("rating: {}", oxygen_candidate_readings[0] * co2_candidate_readings[0]);

    Ok(())
}