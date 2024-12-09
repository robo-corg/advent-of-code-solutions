use std::collections::{HashMap, HashSet, VecDeque};
use std::io::{self, BufRead};
use anyhow::Result;

type Pos = nalgebra::Point2<i32>;
type Vec2 = nalgebra::Vector2<i32>;

type Input = DiskMap;

#[derive(Clone, Debug)]
struct File {
    id: u32,
    used: u8,
    free: u8,
}

#[derive(Debug)]
struct DiskMap(Vec<File>);

fn parse_ch(ch: char) -> Result<u8> {
    let s= ch.to_string();
    Ok(u8::from_str_radix(&s, 10)?)
}

fn parse_input(mut reader: impl BufRead) -> Result<Input> {
    let mut buf = String::new();
    reader.read_to_string(&mut buf)?;

    let mut chars = buf.trim().chars();
    let mut disk_map = DiskMap(Vec::new());

    let mut id = 0;

    while let Some(used_ch) = chars.next() {
        let free_ch = chars.next().unwrap_or('0');

        dbg!(used_ch, free_ch);

        disk_map.0.push(File {
            id,
            used: parse_ch(used_ch)?,
            free: parse_ch(free_ch)?
        });

        id += 1;
    }

    Ok(disk_map)
}

fn get_compacted_blocks(dm: &DiskMap) -> u64 {
    let mut files = VecDeque::from(dm.0.clone());
    let mut checksum = 0;
    let mut offset = 0;
    let mut moving_file: Option<File> = None;

    let mut move_fails = 0;
    let mut move_file_retries = 0;

    let mut compacted_size = 0;

    for f in dm.0.iter() {
        compacted_size += f.used as u32;
    }

    while let Some(cur_file) = files.pop_front() {
        for _ in 0..cur_file.used {
            //println!("{} {}", offset, cur_file.id);
            checksum += (offset * cur_file.id) as u64;
            offset += 1;
        }

        for _ in 0..cur_file.free {
            loop {
                if let Some(mf) = moving_file.as_ref() {
                    if mf.used > 0 {
                        break;
                    }
                }

                moving_file = files.pop_back();

                if moving_file.is_none() {
                    break;
                }

                move_file_retries += 1;
            }

            if let Some(mf) = moving_file.as_mut() {
                //println!("{} {}", offset, mf.id);
                checksum += (offset * mf.id) as u64;
                mf.used -= 1;
                offset += 1;
            }
            else {
                //dbg!(offset, cur_file.id);
                move_fails += 1;
            }
        }

        if files.is_empty() {
            if let Some(mf) = moving_file.take() {
                files.push_back(mf);
            }
        }
    }

    assert_eq!(offset, compacted_size);
    assert!(files.is_empty());
    assert!(moving_file.is_none());

    // dbg!(offset);
    // dbg!(move_fails);
    // dbg!(move_file_retries);

    checksum
}



fn get_compacted_blocks_part_2(dm: &DiskMap) -> u64 {
    let mut checksum = 0;


    let mut compacted_size = 0;

    struct FileOffset {
        id: u32,
        size: u32,
        offset: u32,
        next_file: Option<u32>
    }

    struct EmptySpace {
        prev_file: u32,
        size: u32,
        offset: u32
    }

    let (mut files, mut empty_space) = {
        let mut files: Vec<FileOffset> = Vec::new();
        let mut empty_space = Vec::new();

        let mut offset = 0;
        for f in dm.0.iter() {
            files.push(FileOffset { id: f.id, size: f.used as u32, offset: offset, next_file: Some(f.id + 1) });
            empty_space.push(EmptySpace {
                prev_file: f.id,
                size: f.free as u32,
                offset: offset + (f.used as u32)
            });
            offset += (f.used as u32)+ (f.free as u32);
        }
        (files, empty_space)
    };

    for move_id in (0..files.len()).rev() {
        let cur_move = &mut files[move_id];

        for possible_dest in empty_space.iter_mut() {
            if possible_dest.offset >= cur_move.offset {
                break;
            }

            if possible_dest.size < cur_move.size {
                continue;
            }

            cur_move.offset = possible_dest.offset;

            possible_dest.size -= cur_move.size;
            possible_dest.offset += cur_move.size;
        }
    }

    files.sort_by_key(|f| f.offset);

    let checksum: u64 = files.iter().map(|f| {
        let s: u64 = (0..f.size).map(|n| (f.id * (f.offset + n)) as u64 ).sum();
        s
    }).sum();

    checksum
}


fn main() -> Result<()> {
    let map = {
        let stdin = io::stdin();
        let stdin_lock = stdin.lock();
        parse_input(stdin_lock)?
    };

    dbg!(&map);

    let part1_checksum = get_compacted_blocks(&map);

    println!("part1: {}", part1_checksum);

    let part2_checksum = get_compacted_blocks_part_2(&map);

    println!("part2: {}", part2_checksum);

    Ok(())
}
