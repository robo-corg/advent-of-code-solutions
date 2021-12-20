use std::{
    collections::{HashMap, HashSet},
    io::{self, BufRead},
    mem,
};
use nalgebra::ComplexField;
use petgraph::{graphmap::{UnGraphMap, DiGraphMap}, algo::{connected_components, k_shortest_path, all_simple_paths, min_spanning_tree}};
use ndarray::prelude::*;
use petgraph::data::FromElements;
use petgraph::Direction;
//use ndarray::{Array2, Array1, ArrayView2};

type Scanner = Array2<i32>;

type Input = Vec<Array2<i32>>;

fn parse_input(mut reader: impl BufRead) -> Input {
    let mut scanners = Vec::new();
    let mut cur_scanner = Array2::zeros((0, 3));

    for maybe_line in reader.lines() {
        let line = maybe_line.unwrap();

        if line.starts_with("---") {
            continue;
        }

        if line == "" {
            let finished_scanner = mem::replace(&mut cur_scanner, Array2::zeros((0, 3)));
            scanners.push(finished_scanner);
            continue;
        }

        let coord: Array1<i32> = line
            .split(",")
            .map(|comp| i32::from_str_radix(comp, 10).unwrap())
            .collect();
        cur_scanner.push_row(coord.view()).unwrap();
    }

    let finished_scanner = mem::replace(&mut cur_scanner, Array2::zeros((0, 3)));
    scanners.push(finished_scanner);

    scanners
}

type DistsByBeacon = HashMap<usize, HashSet<i32>>;
type BeaconsByDist = HashMap<i32, HashSet<usize>>;

fn compute_paired_distances(
    scanner: ArrayView2<i32>,
) -> (Array2<i32>, DistsByBeacon, BeaconsByDist) {
    let shape = scanner.shape();
    let l = shape[0];

    let mut dist = Array2::zeros((l, l));

    let mut dists_by_beacon = HashMap::new();
    let mut beacons_by_dist = HashMap::new();

    for i in 0..l {
        for j in 0..l {
            if i < j {
                continue;
            }
            //array![1, 2, 3] - array![1, 2, 3];

            let a = scanner.row(i).into_owned();
            let b = scanner.row(j).into_owned();
            let delta: Array1<i32> = (b - a).mapv(|x| x.pow(2));
            let d = delta.sum();

            dists_by_beacon.entry(i).or_insert(HashSet::new()).insert(d);
            dists_by_beacon.entry(j).or_insert(HashSet::new()).insert(d);

            let beacon_dists = beacons_by_dist.entry(d).or_insert(HashSet::new());

            beacon_dists.insert(i);
            beacon_dists.insert(j);

            dist[(i, j)] = delta.sum();
        }
    }

    (dist, dists_by_beacon, beacons_by_dist)
}

type BeaconTransform = (Array2<i32>, Array1<i32>);

fn find_compatible_coordinate_system(scanner_a: &Scanner, scanner_b: &Scanner, equivalent: &HashSet<(usize, usize)>) -> Option<BeaconTransform> {

    let mut a_mat_int = Array2::zeros((0, 3));
    let mut b_mat_int = Array2::zeros((0, 3));


    for (a, b) in equivalent.iter().copied() {
        a_mat_int.push_row(scanner_a.row(a)).unwrap();
        b_mat_int.push_row(scanner_b.row(b)).unwrap();
    }

    for x_col in 0..3 {
        for y_col in 0..3 {
            if y_col == x_col {
                continue;
            }

            for z_col in 0..3 {
                if z_col == x_col || z_col == y_col {
                    continue;
                }

                let mut rot_mat = Array2::zeros((3, 3));

                for x_sign in [-1, 1] {
                    for y_sign in [-1, 1] {
                        for z_sign in [-1, 1] {
                            rot_mat.row_mut(x_col)[0] = x_sign;
                            rot_mat.row_mut(y_col)[1] = y_sign;
                            rot_mat.row_mut(z_col)[2] = z_sign;

                            let beacon_deltas = &a_mat_int - b_mat_int.dot(&rot_mat);

                            let first_delta = beacon_deltas.row(0);

                            let all_matching = beacon_deltas.rows().into_iter().all(|b| b == first_delta);

                            if all_matching {
                                //dbg!(&a_mat_int - b_mat_int.dot(&rot_mat));
                                return Some((rot_mat, first_delta.to_owned()));
                            }
                        }
                    }
                }
            }
        }
    }




    // let b_mat_f64 = b_mat_int.mapv(|v| v as f64);
    // let b_inv_f64 = b_mat_f64.inv().unwrap();
    // let b_inv = b_inv_f64.mapv(|v| v.round() as i32);

    // dbg!(b_inv);

    //Array2::zeros((4, 4))
    None
}

fn find_equivalent_beacons(scanner_a: &Scanner, scanner_b: &Scanner) -> HashSet<(usize, usize)> {
    let (dists_a, dist_by_beacon_a, beacons_by_dist_a) = compute_paired_distances(scanner_a.view());
    let dists_set_a: HashSet<i32> = dists_a.iter().copied().collect();

    let (dists_b, dist_by_beacon_b, beacons_by_dist_b) = compute_paired_distances(scanner_b.view());
    let dists_set_b: HashSet<i32> = dists_b.iter().copied().collect();

    let intersection: Vec<i32> = dists_set_a.intersection(&dists_set_b).copied().collect();

    // dbg!(intersection.len());

    let mut beacon_equivalence_map = HashSet::new();

    for common_dist in intersection {
        let empty = HashSet::new();

        let a_beaons = beacons_by_dist_a.get(&common_dist).unwrap_or(&empty);
        let b_beacons = beacons_by_dist_b.get(&common_dist).unwrap_or(&empty);

        // println!(
        //     "dist {} has {} a beacons and {} b beacons",
        //     common_dist,
        //     a_beaons.len(),
        //     b_beacons.len()
        // );

        if a_beaons.len() >= 2 && b_beacons.len() >= 2 {
            // println!("   a");
            // for a_beacon in a_beaons.iter() {
            //     println!("       {}", scanner_a.row(*a_beacon));
            // }

            // println!("   b");
            // for b_beacon in b_beacons.iter() {
            //     println!("       {}", scanner_b.row(*b_beacon));
            // }

            for (i, a_beacon) in a_beaons.iter().copied().enumerate() {
                for (j, b_beacon) in b_beacons.iter().copied().enumerate() {
                    let common: Vec<i32> = dist_by_beacon_a[&a_beacon]
                        .intersection(&dist_by_beacon_b[&b_beacon])
                        .copied()
                        .collect();
                    //println!("    {} ^ {} = {}", i, j, common.len());

                    if common.len() >= 12 {
                        assert_eq!(common.len(), 12);

                        beacon_equivalence_map.insert((a_beacon, b_beacon));
                    }
                }
            }
        }
    }

    //dbg!(&beacon_equivalence_map, beacon_equivalence_map.len());

    beacon_equivalence_map
}


fn transform_beacons(beacons: &Scanner, transform: &BeaconTransform) -> Scanner {
    beacons.dot(&transform.0) + &transform.1
}


fn transform_scanner_tree(graph: &UnGraphMap<usize, ()>, input: &[Scanner], vistied: HashSet<usize>, scanner_a: usize) -> (HashSet<Array1<i32>>, Vec<Array1<i32>>) {
    dbg!(scanner_a);

    let mut beacons = HashSet::new();
    let mut scanner_positions = Vec::new();

    for beacon in input[scanner_a].rows() {
        beacons.insert(beacon.to_owned());
    }

    scanner_positions.push(array!(0, 0, 0));

    for scanner_b in graph.neighbors(scanner_a) {
        if vistied.contains(&scanner_b) {
            continue;
        }

        let mut new_visited = vistied.clone();
        new_visited.insert(scanner_b);

        let common_beacons = find_equivalent_beacons(&input[scanner_a], &input[scanner_b]);

        assert!(common_beacons.len() > 0);

        let t = find_compatible_coordinate_system(&input[scanner_a], &input[scanner_b], &common_beacons).unwrap();
        //dbg!(&input[scanner_a]);
        //dbg!(transform_beacons(&input[scanner_b], &t));

        let (child_beacons, child_scanner_positions) = transform_scanner_tree(graph, input, new_visited, scanner_b);

        for child_scanner_position in child_scanner_positions {
            let transformed_scanner = child_scanner_position.dot(&t.0) + &t.1;
            scanner_positions.push(transformed_scanner);
        }

        for beacon in child_beacons {
            let transformed_beacon = beacon.dot(&t.0) + &t.1;
            beacons.insert(transformed_beacon);
        }
    }

    (beacons, scanner_positions)
}

fn main() {
    let input = {
        let stdin = io::stdin();
        let stdin_lock = stdin.lock();
        parse_input(stdin_lock)
    };

    dbg!(&input);

    //find_equivalent_beacons(&input[0], &input[1]);

    //let scanner_beacon_equivalences = HashMap::new();

    let mut scanner_connectivity = UnGraphMap::new();
    let mut egraph = UnGraphMap::new();

    for scanner_idx in 0..input.len() {
        scanner_connectivity.add_node(scanner_idx);
        let scanner = &input[scanner_idx];
        for beacon_idx in 0..scanner.shape()[0] {
            egraph.add_node((scanner_idx, beacon_idx));
        }
    }

    for scanner_a in 0..input.len() {
        for scanner_b in 0..input.len() {
            if scanner_b >= scanner_a {
                break;
            }

            let common_beacons = find_equivalent_beacons(&input[scanner_a], &input[scanner_b]);

            if common_beacons.len() > 0 {
                //let t = find_compatible_coordinate_system(&input[scanner_a], &input[scanner_b], &common_beacons).unwrap();

                // dbg!(&input[scanner_a]);
                // dbg!(transform_beacons(&input[scanner_b], &t));

                scanner_connectivity.add_edge(scanner_a, scanner_b, ());
            }

            println!("common scanner {} & {}: {}", scanner_a, scanner_b, common_beacons.len());

            for (a, b) in common_beacons.iter().copied() {
                let pos_a = input[scanner_a].row(a).into_owned();
                let pos_b = input[scanner_b].row(b).into_owned();

                println!("{} {} {}", &pos_a, &pos_b, &pos_b - &pos_a);
                egraph.add_edge((scanner_a, a), (scanner_b, b), ());
            }
        }
    }

    let distinct_beacons = connected_components(&egraph);
    let isolated_scanner_groups = connected_components(&scanner_connectivity);

    println!("isolated scanners: {} (should be 1)", isolated_scanner_groups);
    println!("Distinct beacons (pre-coordinate-adjust): {}", distinct_beacons);

    let scanner_tree = UnGraphMap::from_elements(min_spanning_tree(&scanner_connectivity));

    println!("isolated scanners (tree): {} (should be 1)", connected_components(&scanner_tree));

    // for n in scanner_tree.nodes() {
    //     dbg!(n, scanner_tree.neighbors_directed(n, Direction::Incoming).count());
    // }

    let (beacons, scanner_positions) = transform_scanner_tree(&scanner_tree, &input, vec![0].into_iter().collect(), 0);

    let mut farthest_scanner = 0;

    for i in 0..scanner_positions.len() {
        for j in 0..scanner_positions.len() {
            let dist = (&scanner_positions[i] - &scanner_positions[j]).mapv(|v| v.abs()).sum();

            farthest_scanner = i32::max(dist, farthest_scanner);
        }
    }

    dbg!(scanner_positions);

    println!("Distinct beacons: {}", beacons.len());
    println!("Farthest appart scanners: {}", farthest_scanner);


    //let path = all_simple_paths(scanner_connectivity, 0, 1 0, None).next().unwrap();

    // for scanner_idx in 1..input.len() {
    //     for neighbor in scanner_connectivity.neighbors(cur_node) {
    //         if visited.contains(&neighbor) {
    //             continue;
    //         }

    //         neighbor.
    //     }
    // }
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
