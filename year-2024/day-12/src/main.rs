use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::io::{self, BufRead};
use std::{i32, mem};
use anyhow::{bail, Result};
use ndarray::{s, Array2};
use svg::Document;
use svg::node::element::Path;
use svg::node::element::path::Data;

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
struct PlotNode {
    ty: char,
    pos: Pos

}

#[repr(u8)]
enum EdgeType {
    Right = 1 << 0,
    Bottom = 1 << 1,
    Left = 1 << 2,
    Top = 1 << 3
}

struct EdgeTypeSet(u8);

impl EdgeTypeSet {
    fn empty() -> Self {
        EdgeTypeSet(0)
    }

    fn insert(&mut self, poly_edge: EdgeType) {
        self.0 |= poly_edge as u8;
    }

    fn is_empty(&self) -> bool {
        self.0 == 0
    }

}

#[derive(Debug)]
struct Graph {
    nodes: Vec<PlotNode>,
    node_pos: HashMap<Pos, usize>,
    edges: HashMap<usize, HashSet<usize>>
}

impl Graph {
    fn build_from_map(map: Array2<Option<char>>) -> Graph {
        let map_shape = map.shape();

        let mut padded_map: Array2<Option<char>> = Array2::from_shape_fn((map_shape[0] + 2, map_shape[1] + 2), |_| None);

        padded_map.slice_mut(s![1..-1, 1..-1]).assign(&map);

        let padded_shape = padded_map.shape();

        dbg!(&padded_map);


        let mut nodes = Vec::new();
        let mut node_pos= HashMap::new();
        let mut edges: HashMap<usize, HashSet<usize>> = HashMap::new();

        for y in 1..(padded_shape[1]-1) {
            for x in 1..(padded_shape[0]-1) {
                let pos = Pos::new(x as i32, y as i32);

                if arr_get(&padded_map, pos).is_none() {
                    continue;
                }

                let val = arr_get(&padded_map, pos).clone().unwrap();

                let node_id = nodes.len();
                nodes.push(PlotNode {
                    ty: val,
                    pos,
                });
                node_pos.insert(pos, node_id);

                let mut p = 0;

                for neigh_dir in [Vec2::new(-1, 0), Vec2::new(0, -1)].into_iter() {
                    let neigh_pos = pos + neigh_dir;
                    let maybe_neigh_val = arr_get(&padded_map, neigh_pos).clone();

                    if let Some(neigh_val) = maybe_neigh_val {
                        if neigh_val != val {
                            continue;
                        }

                        let neigh_node_id: usize = node_pos[&neigh_pos];

                        edges.entry(node_id).or_default().insert(neigh_node_id);
                        edges.entry(neigh_node_id).or_default().insert(node_id);
                    }
                }

            }
        }

        Graph {
            nodes,
            node_pos,
            edges,
        }
    }

    fn neighbors(&self, node_id: usize) -> Option<&HashSet<usize>> {
        self.edges.get(&node_id)
    }

    fn is_connected_pos(&self, a: Pos, b: Pos) -> bool {
        let a_ind = self.node_pos[&a];
        let b_ind = if let Some(ind) = self.node_pos.get(&b) { ind } else { return false };

        if let Some(neighbors) = self.edges.get(&a_ind) {
            return neighbors.contains(&b_ind);
        }

        false
    }

    fn find_all_connected(&self, start_pos: Pos) -> HashSet<Pos> {
        let mut fringe = vec![start_pos];
        let mut fringe_set: HashSet<Pos> = fringe.iter().copied().collect();
        let mut seen = HashSet::new();


        while let Some(cur_pos) = fringe.pop() {
            fringe_set.remove(&cur_pos);
            seen.insert(cur_pos);

            let cur_node_id = self.node_pos.get(&cur_pos).unwrap().clone();

            if let Some(neighbors) = self.edges.get(&cur_node_id) {
                for neighbor_id in neighbors.iter().copied() {
                    let neighbor_pos = self.nodes[neighbor_id].pos;
                    if !seen.contains(&neighbor_pos) && !fringe_set.contains(&neighbor_pos) {
                        fringe.push(neighbor_pos);
                        fringe_set.insert(neighbor_pos);
                    }
                }
            }
        }

        seen
    }

    fn find_regions(&self) -> Vec<HashSet<Pos>> {
        let mut regions = Vec::new();
        let mut explored: HashSet<Pos> = HashSet::new();

        for check_node in self.nodes.iter() {
            if explored.contains(&check_node.pos) {
                continue;
            }

            let region = self.find_all_connected(check_node.pos);
            explored.extend(&region);
            regions.push(region);
        }

        regions
    }

    fn is_edge(&self, pos: Pos) -> EdgeTypeSet {
        let edge_mapping = [
            (Vec2::new(1, 0), EdgeType::Right),
            (Vec2::new(0, 1), EdgeType::Bottom),
            (Vec2::new(-1, 0), EdgeType::Left),
            (Vec2::new(0, -1), EdgeType::Top)
        ].into_iter();

        let mut edgeset = EdgeTypeSet::empty();

        for (neigh_pos, poly_edge) in edge_mapping {
            if !self.is_connected_pos(pos, pos + neigh_pos) {
                edgeset.insert(poly_edge);
            }
        }

        edgeset
    }

    fn find_edges(&self, region: &HashSet<Pos>) -> HashMap<Pos, Pos> {
        let mut edges: HashMap<Pos, Pos> = HashMap::new();

        for pos in region.iter().copied() {

            let edge_mapping = [
                (Vec2::new(1, 0), (Vec2::new(1, 0), Vec2::new(1, 1))),
                (Vec2::new(0, 1), (Vec2::new(1, 1), Vec2::new(0, 1))),
                (Vec2::new(-1, 0), (Vec2::new(0, 1), Vec2::new(0, 0))),
                (Vec2::new(0, -1), (Vec2::new(0, 0), Vec2::new(1, 0)))
            ].into_iter();

            for (neigh_pos, poly_edge) in edge_mapping {
                if !self.is_connected_pos(pos, pos + neigh_pos) {
                    let a_poly_pos = pos + poly_edge.0;
                    let b_poly_pos = pos + poly_edge.1;

                    edges.insert(a_poly_pos, b_poly_pos);
                }
            }
        }

        edges
    }
}

fn count_corners(region: &HashSet<Pos>) -> usize {
    let mut corners = 0;

    let mut region_end = Vec2::new(0, 0);
    let mut region_start = Vec2::new(i32::MAX, i32::MAX);

    for pos in region {
        region_end[0] = i32::max(region_end[0], pos[0]);
        region_end[1] = i32::max(region_end[1], pos[1]);

        region_start[0] = i32::min(region_start[0], pos[0]);
        region_start[1] = i32::min(region_start[1], pos[1]);
    }

    for y in region_start[1]-1..region_end[1]+2 {
        for x in region_start[0]-1..region_end[0]+2 {
            let w_pos = Pos::new(x, y);

            let mask: u8 = (0..2).flat_map(|y_off| {
                (0..2).map(move |x_off| {
                    let off = Vec2::new(x_off, y_off);
                    w_pos + off
                })
            }).enumerate().map(|(n, p)| {
                if region.contains(&p) {
                    1 << n
                }
                else {
                    0
                }
            }).reduce(|acc, el|{
                acc | el
            }).unwrap();


            let bit_count = mask.count_ones();

            let corners_found = if bit_count == 0 {
                0
            }
            else if bit_count == 1 || bit_count == 3 {
                1
            }
            else {
                match mask {
                    0b1001 => 2,
                    0b0110 => 2,
                    _ => 0
                }
            };

            corners += corners_found as usize;
        }
    }

    corners
}

fn count_edges(region: &HashSet<Pos>) -> usize {
    let mut edges = 0;

    let mut region_end = Vec2::new(0, 0);
    let mut region_start = Vec2::new(i32::MAX, i32::MAX);

    for pos in region {
        region_end[0] = i32::max(region_end[0], pos[0]);
        region_end[1] = i32::max(region_end[1], pos[1]);

        region_start[0] = i32::min(region_start[0], pos[0]);
        region_start[1] = i32::min(region_start[1], pos[1]);
    }

    dbg!(region_start, region_end);

    for y in (region_start[1]-1)..(region_end[1]+2) {
        for x in (region_start[0]-1)..(region_end[0]+2) {
            let w_pos = Pos::new(x, y);

            dbg!(w_pos);

            let mask: u8 = (0..2).flat_map(|y_off| {
                (0..2).map(move |x_off| {
                    let off = Vec2::new(x_off, y_off);
                    w_pos + off
                })
            }).enumerate().map(|(n, p)| {

                //dbg!(&p);
                if region.contains(&p) {
                    1 << n
                }
                else {
                    0
                }
            }).reduce(|acc, el|{
                acc | el
            }).unwrap();

            if w_pos == Pos::new(2, 1) {
                dbg!(mask);
            }

            if mask & 0b0001 != 0 {
                if mask & 0b0100 == 0  {
                    edges += 1;
                }

                if mask & 0b0010 == 0 {
                    edges += 1;
                }

                // let m = mask | 0b10011111;
                // let found_edges = m.count_zeros();
                // dbg!(found_edges);
                // edges += found_edges;
            }
            else {
                if mask & 0b0100 != 0  {
                    edges += 1;
                }

                if mask & 0b0010 != 0  {
                    edges += 1;
                }
            }
        }
    }

    edges as usize
}

fn main() -> Result<()> {
    let map = {
        let stdin = io::stdin();
        let stdin_lock = stdin.lock();
        parse_input(stdin_lock)?
    };

    let graph = Graph::build_from_map(map);

    dbg!(&graph);


    dbg!("Finding regions...");

    let regions = graph.find_regions();

    dbg!("Finding edges...");

    dbg!(regions.len());

    let mut part1_cost= 0;
    let mut part2_cost = 0;


    for region in regions.iter() {
        let ty = {
            let node_id = graph.node_pos.get(region.iter().next().unwrap()).unwrap();
            graph.nodes[*node_id].ty
        };


        //let edges = graph.find_edges(region);
        dbg!(region);


        let area = region.len();
        let perimiter = count_edges(region);
        let sides = count_corners(region);

        println!("{} * {} sides: {}", area, perimiter, sides);

        part1_cost += area * perimiter;
        part2_cost += area * sides;

        //dbg!(&edges);

        // for edge in edges.iter() {
        //     println!("{:?} -> {:?}", edge.0, edge.1);
        // }

    }

    println!("part1: {}", part1_cost);
    println!("part2: {}", part2_cost);


    // XXX
    //  X

    // let mut connected_plots: HashMap<u32, HashSet<u32>> = HashMap::new();


    // for plot in plots.iter() {
    //     for id in plot.ids.iter() {
    //         let connected = connected_plots.entry(*id).or_default();
    //         connected.extend(&plot.ids);
    //     }
    // }


    // let keys: Vec<u32> = connected_plots.keys().copied().collect();

    // dbg!("Closure");

    // loop {
    //     let mut updates = 0;

    //     for plot_id in keys.iter().copied() {
    //         let connected = connected_plots.entry(plot_id).or_default().clone();

    //         for connected_id in connected.iter().copied() {
    //             let conns = connected_plots.entry(connected_id).or_default();

    //             let old_len = conns.len();

    //             conns.extend(&connected);

    //             if old_len != conns.len() {
    //                 updates += 1;
    //             }
    //         }
    //     }

    //     dbg!(updates);

    //     if updates == 0 {
    //         break;
    //     }
    // }

    // dbg!("merging...");


    // let mut merged_plots: Vec<Plot> = Vec::new();
    // let mut processed_plots = HashSet::new();
    // for connected_plot in connected_plots {
    //     if !connected_plot.1.is_disjoint(&processed_plots) {
    //         continue;
    //     }

    //     let mut plot = Plot::default();

    //     for plot_id in connected_plot.1.iter().copied() {
    //         let i = plot_id as usize;
    //         plot.area += plots[i].area;
    //         plot.perimiter += plots[i].perimiter;
    //         plot.ty = plots[i].ty;
    //     }

    //     merged_plots.push(plot);
    //     processed_plots.extend(connected_plot.1);
    // }


    // let part1_cost: i32 = merged_plots.iter().map(|plot| plot.area * plot.perimiter).sum();

    // for plot in merged_plots.iter() {
    //     if plot.area == 0 && plot.perimiter == 0 {
    //         continue;
    //     }

    //     println!("{} {} * {}", plot.ty, plot.area, plot.perimiter);
    // }

    // println!("part1: {}", part1_cost);

    Ok(())
}
