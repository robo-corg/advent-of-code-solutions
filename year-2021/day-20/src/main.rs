use std::fmt;
use std::io::{self, BufRead};

use bitvec::bitarr;
use building_blocks::core::prelude::*;
use building_blocks::storage::{prelude::*, ChunkMap2x1, ChunkHashMap};
use bitvec::prelude::*;

type Input = (Enhancement, Map);

fn clone_map(map: &Map, ambient_value: i32) -> Map {
    let shape = map.bounding_extent(0);
    let mut new_map = create_map(ambient_value);

    map.visit_occupied_chunks(0, &shape, |chunk| {
        let chunk_key = ChunkKey::new(0, chunk.extent().minimum);
        new_map.write_chunk(chunk_key, chunk.clone());
        //let chunk_extent = chunk.extent();

        //copy_extent(&chunk_extent, chunk, &mut new_map);
    });

    new_map
}

fn parse_input(mut reader: impl BufRead) -> Input {
    let mut lines = reader.lines();

    let enhancement_str = lines.next().unwrap().unwrap();
    let enhancement = parse_enhancement_str(&enhancement_str);

    let _blank_line = lines.next().unwrap().unwrap();

    (
        enhancement,
        parse_map(lines.map(|line| line.unwrap()))
    )
}

type Map = ChunkHashMap<[i32; 2], i32, ChunkMapBuilder2x1<i32>>;
type Enhancement = BitArr!(for 512, in Lsb0, u32);

struct DisplayMap<'a>(&'a Map);

impl <'a> fmt::Display for DisplayMap<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let shape = self.0.bounding_extent(0);
        let lod0 = self.0.lod_view(0);

        for y in shape.minimum.y()..(shape.minimum.y()+shape.shape.y()) {
            for x in shape.minimum.x()..(shape.minimum.x()+shape.shape.x()) {
                match lod0.get(PointN([x, y])) {
                    0 => write!(f, ".")?,
                    1 => write!(f, "#")?,
                    other => write!(f, "?")?
                }
            }

            writeln!(f, "")?;
        }

        Ok(())
    }
}

fn parse_cell(cell: char) -> bool {
    match cell {
        '#' => true,
        '.' => false,
        other => panic!("Invlaid cell `{}`", cell)
    }
}

fn parse_enhancement_str(s: &str) -> Enhancement {
    let mut bits = bitarr![Lsb0, u32; 0; 512];

    for (n, bit) in s.chars().map(parse_cell).enumerate() {
        bits.set(n, bit);
    }

    bits
}

fn parse_map(lines: impl Iterator<Item=String>) -> Map {
    let mut map = create_map(0);

    let mut lod0 = map.lod_view_mut(0);

    for (row, line) in lines.enumerate() {
        for (col, ch) in line.chars().enumerate() {
            *lod0.get_mut(PointN([col as i32, row as i32])) = parse_cell(ch) as i32;
        }
    }

    map
}

fn create_map(ambient_value: i32) -> Map {
    let chunk_shape = Point2i::fill(16);
    //let ambient_value = 0;
    let builder = ChunkMapBuilder2x1::new(chunk_shape, ambient_value);
    let mut map = builder.build_with_hash_map_storage();

    map
}

fn lookup_enhancement(enhancement: &Enhancement, key: u32) -> i32 {
    let bit = enhancement.get(key as usize).as_deref().copied().unwrap();
    bit as i32
}

fn enhance_map(map: &Map, enhancement: &Enhancement) -> Map {
    let ambient = map.ambient_value();

    let new_ambient = if ambient != 0 {
        lookup_enhancement(enhancement, 0b111111111)
    }
    else {
        lookup_enhancement(enhancement, 0)
    };

    let shape = map.bounding_extent(0);
    let mut next_map = create_map(new_ambient);//clone_map(map, new_ambient);
    let source_lod0 = map.lod_view(0);
    let mut next_lod0 = next_map.lod_view_mut(0);

    let sx = shape.minimum.x() - 2;
    let sy = shape.minimum.y() - 2;
    let ex = shape.minimum.x() + shape.shape.x() + 3;
    let ey = shape.minimum.y() + shape.shape.y() + 3;

    for y in sy..ey {
        for x in sx..ex {
            let mut enhancement_key = 0;
            let mut key_n = 0;
            for wy_offset_flipped in -1..2 {
                let wy_offset = -wy_offset_flipped;
                for wx_offset_flipped in -1..2 {
                    let wx_offset = -wx_offset_flipped;

                    let wx = wx_offset + x;
                    let wy = wy_offset + y;

                    let w_val = source_lod0.get(PointN([wx, wy]));

                    if w_val != 0 {
                        enhancement_key |= 1 << key_n;
                    }

                    key_n += 1;
                }
            }

            let pixel_value = lookup_enhancement(enhancement, enhancement_key);
            if pixel_value != new_ambient {
                *next_lod0.get_mut(PointN([x, y])) = pixel_value;
            }
        }
    }

    next_map
}

fn enhance_n(map: &Map, enhancement: &Enhancement, n: usize) -> Map {
    let mut cur_map = clone_map(map, map.ambient_value());

    for _ in 0..n {
        cur_map = enhance_map(&cur_map, &enhancement);
    }

    cur_map
}

fn count_lit_cells(map: &Map) -> usize {
    let lod0 = map.lod_view(0);

    let mut lit_cells = 0;

    lod0.for_each(&map.bounding_extent(0), |_, val| {
        if val != 0 {
            lit_cells += 1;
        }
    });

    lit_cells
}

fn main() {
    let (enhancement, map) = {
        let stdin = io::stdin();
        let stdin_lock = stdin.lock();
        parse_input(stdin_lock)
    };


    dbg!(enhancement);

    dbg!(map.bounding_extent(0));

    println!("{}", DisplayMap(&map));

    let enhanced_map_1 = enhance_map(&map, &enhancement);
    println!("enhanced:\n{}", DisplayMap(&enhanced_map_1));

    let enhanced_map_2 = enhance_n(&map, &enhancement, 2);

    //dbg!(&input);

    println!("enhanced 2x:\n{}", DisplayMap(&enhanced_map_2));

    let lit_cells_2 = count_lit_cells(&enhanced_map_2);

    println!("Lit pixels (2 times): {}", lit_cells_2);

    let enhanced_map_50 = enhance_n(&map, &enhancement, 50);
    println!("enhanced 50x:\n{}", DisplayMap(&enhanced_map_50));
    let lit_cells_50 = count_lit_cells(&enhanced_map_50);
    println!("Lit pixels (50 times): {}", lit_cells_50);
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
