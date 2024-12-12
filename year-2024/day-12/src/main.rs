use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::io::{self, BufRead};
use std::mem;
use anyhow::{bail, Result};
use ndarray::{s, Array2};

type Pos = nalgebra::Point2<i32>;
type Vec2 = nalgebra::Vector2<i32>;


type Input = Array2<Option<char>>;

fn parse_char_grid<T, Conv>(mut reader: impl BufRead, conv: Conv) -> Result<ndarray::Array2<T>>
where
    Conv: Fn(char) -> T,
{
    let mut elems = Vec::new();

    let mut width = 0;
    let mut height = 0;

    for line in reader.lines() {
        let line = line.unwrap();

        let new_width = line.len();

        if width != 0 && new_width != width {
            bail!("Width of rows does not match");
        }

        width = new_width;

        elems.extend(line.chars().map(|ch| conv(ch)));

        height += 1;
    }

    let map = Array2::from_shape_vec((width, height), elems)?;

    Ok(map)
}


fn parse_input(mut reader: impl BufRead) -> Result<Input> {
    parse_char_grid(reader,|ch| Some(ch))
}

fn arr_get<'a, T>(a: &'a Array2<T>, p: Pos) -> &T {
    &a[(p[0] as usize, p[1] as usize)]
}

#[derive(Debug, Default)]
struct Plot {
    ty: char,
    perimiter: i32,
    area: i32,
    ids: HashSet<u32>
}

fn main() -> Result<()> {
    let map = {
        let stdin = io::stdin();
        let stdin_lock = stdin.lock();
        parse_input(stdin_lock)?
    };

    let map_shape = map.shape();

    let mut padded_map: Array2<Option<char>> = Array2::from_shape_fn((map_shape[0] + 2, map_shape[1] + 2), |_| None);

    padded_map.slice_mut(s![1..-1, 1..-1]).assign(&map);

    let padded_shape = padded_map.shape();

    dbg!(&padded_map);

    let mut plot_ids: HashMap<Pos, u32> = HashMap::new();
    let mut plots: Vec<Plot> = Vec::new();

    for y in 1..(padded_shape[1]-1) {
        for x in 1..(padded_shape[0]-1) {
            let pos = Pos::new(x as i32, y as i32);

            if arr_get(&padded_map, pos).is_none() {
                continue;
            }

            let val = arr_get(&padded_map, pos).clone().unwrap();
            let mut found_plot_id = None;

            let mut p = 0;

            for neigh_dir in [Vec2::new(1, 0), Vec2::new(0, 1), Vec2::new(-1, 0), Vec2::new(0, -1)].into_iter() {
                let neigh_pos = pos + neigh_dir;
                let maybe_neigh_val = arr_get(&padded_map, neigh_pos).clone();

                if let Some(neigh_val) = maybe_neigh_val {
                    if neigh_val != val {
                        p += 1;
                    }
                    else {
                        let plot_id = plot_ids.entry(neigh_pos).or_insert_with(|| {
                            let id = plots.len() as u32;
                            plots.push(Plot {
                                ty: val,
                                perimiter: 0,
                                area: 0,
                                ids: vec![id].into_iter().collect(),
                            });
                            id
                        }).clone();

                        if let Some(existing_plot_id) = found_plot_id {
                            if existing_plot_id != plot_id {
                                let mut plot = &mut plots[plot_id as usize];
                                plot.ids.insert(existing_plot_id);
                            }
                        }

                        plot_ids.insert(pos,  plot_id);
                        found_plot_id = Some(plot_id);
                        plots[plot_id as usize].ty = val;
                    }
                }
                else {
                    p += 1;
                }
            }

            let plot_id = if let Some(found_plot_id) = found_plot_id {
                found_plot_id
            } else {
                let id = plots.len() as u32;
                plots.push(Plot::default());
                plot_ids.insert(pos,  id);
                id
            };

            let mut plot = &mut plots[plot_id as usize];

            plot.ty = val;
            plot.area += 1;
            plot.perimiter += p;
            plot.ids.insert(plot_id);
        }
    }

    let mut connected_plots: HashMap<u32, HashSet<u32>> = HashMap::new();


    for plot in plots.iter() {
        for id in plot.ids.iter() {
            let connected = connected_plots.entry(*id).or_default();
            connected.extend(&plot.ids);
        }
    }


    let keys: Vec<u32> = connected_plots.keys().copied().collect();

    dbg!("Closure");

    loop {
        let mut updates = 0;

        for plot_id in keys.iter().copied() {
            let connected = connected_plots.entry(plot_id).or_default().clone();

            for connected_id in connected.iter().copied() {
                let conns = connected_plots.entry(connected_id).or_default();

                let old_len = conns.len();

                conns.extend(&connected);

                if old_len != conns.len() {
                    updates += 1;
                }
            }
        }

        dbg!(updates);

        if updates == 0 {
            break;
        }
    }

    dbg!("merging...");


    let mut merged_plots: Vec<Plot> = Vec::new();
    let mut processed_plots = HashSet::new();
    for connected_plot in connected_plots {
        if !connected_plot.1.is_disjoint(&processed_plots) {
            continue;
        }

        let mut plot = Plot::default();

        for plot_id in connected_plot.1.iter().copied() {
            let i = plot_id as usize;
            plot.area += plots[i].area;
            plot.perimiter += plots[i].perimiter;
            plot.ty = plots[i].ty;
        }

        merged_plots.push(plot);
        processed_plots.extend(connected_plot.1);
    }


    let part1_cost: i32 = merged_plots.iter().map(|plot| plot.area * plot.perimiter).sum();

    for plot in merged_plots.iter() {
        if plot.area == 0 && plot.perimiter == 0 {
            continue;
        }

        println!("{} {} * {}", plot.ty, plot.area, plot.perimiter);
    }

    println!("part1: {}", part1_cost);

    Ok(())
}
