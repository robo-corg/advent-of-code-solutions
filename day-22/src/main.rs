use std::io::{self, BufRead};
use std::mem;

type Vec3 = nalgebra::Vector3<i32>;
use building_blocks::core::prelude::*;
use building_blocks::storage::{prelude::*, ChunkMap2x1, ChunkHashMap};
type Input = Vec<Command>;

fn parse_input(mut reader: impl BufRead) -> Input {
    reader.lines().map(|line| {
        Command::from_str(line.unwrap().as_str())
    }).collect()
}

#[derive(Clone, Debug, PartialEq)]
struct Cuboid {
    min: Vec3,
    max: Vec3
}

impl Cuboid {
    fn from_str(s: &str) -> Self {
        let mut components = s.split(",").map(|c| {
            let (_, r) = c.split_once("=").unwrap();

            let (min, max) = r.split_once("..").unwrap();
            (
                i32::from_str_radix(min, 10).unwrap(),
                i32::from_str_radix(max, 10).unwrap(),
            )
        });

        let x_r = components.next().unwrap();
        let y_r = components.next().unwrap();
        let z_r = components.next().unwrap();

        Cuboid {
            min: Vec3::new(x_r.0, y_r.0, z_r.0),
            max: Vec3::new(x_r.1+1, y_r.1+1, z_r.1+1)
        }
    }

    fn to_extent(&self) -> Extent3i {
        Extent3i::from_corners(to_grid_point(self.min), to_grid_point(self.max))
    }

    fn get_volume(&self) -> usize {
        let delta = (self.max - self.min).abs();
        (delta[0] as usize) * (delta[1] as usize) * (delta[2] as usize)
    }
}


#[derive(Clone, Debug)]
struct Command {
    on: bool,
    cuboid: Cuboid
}

impl Command {
    fn from_str(s: &str) -> Self {
        let (on_off_str, cuboid_str) = s.split_once(" ").unwrap();

        Command {
            on:  match on_off_str {
                "on" => true,
                "off" => false,
                other => panic!("Invalid command must start with on or off")
            },
            cuboid: Cuboid::from_str(cuboid_str)
        }
    }
}

fn to_grid_point(v: Vec3) -> PointN<[i32; 3]> {
    PointN(v.data.0[0])
}


/// Undirected axis aligned plane
#[derive(Debug)]
enum AAPlane {
    X(i32),
    Y(i32),
    Z(i32)
}

impl AAPlane {
    fn from_axis_num(axis: usize, val: i32) -> Self {
        match axis {
            0 => AAPlane::X(val),
            1 => AAPlane::Y(val),
            2 => AAPlane::Z(val),
            other => panic!("Invalid axis: {}", other)
        }
    }

    fn axis(&self) -> usize {
        match self {
            AAPlane::X(_) => 0,
            AAPlane::Y(_) => 1,
            AAPlane::Z(_) => 2,
        }
    }
}

/// Axis aligned plane with a facing direction
#[derive(Debug)]
struct HalfSpace(bool, AAPlane);

impl HalfSpace {
    fn split(&self, cube: Cuboid) -> (Option<Cuboid>, Option<Cuboid>) {
        let (left, right) = self.1.split(cube);

        if self.0 {
            (left, right)
        }
        else {
            (right, left)
        }
    }
}

impl AAPlane {
    fn split(&self, cube: Cuboid) -> (Option<Cuboid>, Option<Cuboid>) {
        match self {
            AAPlane::X(x) => {
                if cube.min[0] < *x && *x <= cube.max[0] {
                    (
                        Some(Cuboid {
                            min: cube.min,
                            max: Vec3::new(*x, cube.max[1], cube.max[2])
                        }),
                        Some(Cuboid {
                            min: Vec3::new(*x, cube.min[1], cube.min[2]),
                            max: cube.max
                        })
                    )
                }
                else if cube.max[0] < *x {
                    (Some(cube), None)
                }
                else if cube.min[0] >= *x {
                    (None, Some(cube))
                }
                else {
                    panic!("Invalid x split: {:?} by {}", cube, x);
                }
            }
            AAPlane::Y(y) => {
                if cube.min[1] < *y && *y <= cube.max[1] {
                    (
                        Some(Cuboid {
                            min: cube.min,
                            max: Vec3::new(cube.max[0], *y, cube.max[2])
                        }),
                        Some(Cuboid {
                            min: Vec3::new(cube.min[0], *y, cube.min[2]),
                            max: cube.max
                        })
                    )
                }
                else if cube.max[1] < *y {
                    (Some(cube), None)
                }
                else if cube.min[1] >= *y {
                    (None, Some(cube))
                }
                else {
                    panic!("Invalid y split: {:?} by {}", cube, y);
                }
            }
            AAPlane::Z(z) => {
                if cube.min[2] < *z && *z <= cube.max[2] {
                    (
                        Some(Cuboid {
                            min: cube.min,
                            max: Vec3::new(cube.max[0], cube.max[1], *z)
                        }),
                        Some(Cuboid {
                            min: Vec3::new(cube.min[0], cube.min[1], *z),
                            max: cube.max
                        })
                    )
                }
                else if cube.max[2] < *z {
                    (Some(cube), None)
                }
                else if cube.min[2] >= *z {
                    (None, Some(cube))
                }
                else {
                    panic!("Invalid z split: {:?} by {}", cube, z);
                }
            }
        }
    }

    fn split_multiple(&self, cubes: impl Iterator<Item=Cuboid>) -> (Vec<Cuboid>, Vec<Cuboid>) {
        let (left, right): (Vec<Option<Cuboid>>, Vec<Option<Cuboid>>) = cubes.map(|c| self.split(c)).unzip();

        let output = (
            left.into_iter().filter_map(|c| c).collect(),
            right.into_iter().filter_map(|c| c).collect()
        );



        output
    }
}

/// Iterator for all the half spaces representing the sides of the cube
fn cube_planes(cube: Cuboid) -> impl Iterator<Item=HalfSpace> {
    (0..3).flat_map(move |dim| {
        (0..2).map(move |side| {
            let v = if side == 0 {
                cube.min
            }
            else {
                cube.max
            };

            HalfSpace(
                side==0,
                match dim {
                    0 => AAPlane::X(v[dim]),
                    1 => AAPlane::Y(v[dim]),
                    2 => AAPlane::Z(v[dim]),
                    other => panic!("Invalid dimension: {}", other)
                }
            )
        })
    })
}

///
fn split_cubes(splitter: Cuboid, target: Cuboid) -> (Vec<Cuboid>, Vec<Cuboid>) {
    let mut left = Vec::new();
    let mut inside = vec![target];

    for half_space in cube_planes(splitter) {
        //dbg!(&half_space);
        let mut new_inside = Vec::new();

        for cur_inside_cube in inside.iter() {
            let (maybe_outside, maybe_inside) = half_space.split(cur_inside_cube.clone());

            //dbg!(&maybe_inside, &maybe_outside);

            if let Some(outside) = maybe_outside {
                if outside.get_volume() > 0 {
                    left.push(outside);
                }
            }

            if let Some(inside) = maybe_inside {
                if inside.get_volume() > 0 {
                    new_inside.push(inside);
                }
            }

        }

        inside = new_inside;
    }

    (left, inside)
}

#[derive(Default)]
struct Splits( [Option<i32>; 3]);

enum KDTreeNode {
    Branch { split: AAPlane, left: Box<KDTreeNode>, right: Box<KDTreeNode> },
    Leaf(Vec<Cuboid>)
}

impl KDTreeNode {
    fn insert(&mut self, cmd: Command) {
        // TODO: Make this actually split nodes
        match self {
            KDTreeNode::Leaf(cubes ) => {
                if cmd.on {
                    let cubes_to_merge = mem::replace(cubes, Vec::new());

                    *cubes = cubes_to_merge.into_iter().flat_map(|on_cuboid| {
                        let (outside, _inside) = split_cubes(cmd.cuboid.clone(), on_cuboid);
                        outside.into_iter()
                    }).collect();

                    cubes.push(cmd.cuboid);
                }
                else {
                    let cubes_to_split = mem::replace(cubes, Vec::new());

                    *cubes = cubes_to_split.into_iter().flat_map(|on_cuboid| {
                        let (outside, _inside) = split_cubes(cmd.cuboid.clone(), on_cuboid);
                        outside.into_iter()
                    }).collect();
                }
            },
            KDTreeNode::Branch { split, left, right} => {
                let on = cmd.on;
                let (maybe_split_left, maybe_split_right) = split.split(cmd.cuboid);

                if let Some(split_left) = maybe_split_left {
                    left.insert(Command { on, cuboid: split_left });
                }

                if let Some(split_right) = maybe_split_right {
                    right.insert(Command { on, cuboid: split_right });
                }
            }
        }
    }

    fn balance(&mut self, parent_split_axis: Option<usize>) {
        let mut replacement = None;

        match self {
            KDTreeNode::Leaf(nodes) if nodes.len() > 5 => {
                let old_volume: usize = nodes.iter().map(Cuboid::get_volume).sum();

                let split_axis = parent_split_axis.map(|a| (a + 1) % 3).unwrap_or(0);

                let mut partition_candidates: Vec<i32> = nodes.iter().flat_map(|cube| [cube.min[split_axis], cube.max[split_axis]].into_iter()).collect();
                partition_candidates.sort();

                let median = partition_candidates[partition_candidates.len()/2];

                let old_nodes = nodes.clone(); //mem::replace(nodes, Vec::new());

                let split = AAPlane::from_axis_num(split_axis, median);
                let (left, right) = split.split_multiple(old_nodes.into_iter());

                if left.len() == 0 || right.len() == 0 {
                    return;
                }

                let mut new_node = KDTreeNode::Branch {
                    split,
                    left: Box::new(KDTreeNode::Leaf(left)),
                    right: Box::new(KDTreeNode::Leaf(right))
                };

                assert_eq!(new_node.get_volume(), old_volume);

                new_node.balance(Some(split_axis));

                assert_eq!(new_node.get_volume(), old_volume);

                replacement = Some(new_node);
            },
            KDTreeNode::Branch { split, left, right } => {
                left.balance(Some(split.axis()));
                right.balance(Some(split.axis()));
            }
            _ => {}
        }

        if let Some(replacement) = replacement {
            *self = replacement;
        }
    }

    fn get_volume(&self) -> usize {
        match self {
            KDTreeNode::Leaf(cubes) => {
                let total: usize = cubes.iter().map(Cuboid::get_volume).sum();
                total as usize
            }
            KDTreeNode::Branch { left, right, .. } => {
                left.get_volume() + right.get_volume()
            }
        }
    }
}

struct KDTree(KDTreeNode);

impl KDTree {
    fn new() -> Self {
        KDTree(KDTreeNode::Leaf(Vec::new()))
    }

    fn insert(&mut self, cmd: Command) {
        self.0.insert(cmd);
    }

    fn get_volume(&self) -> usize {
        self.0.get_volume()
    }

    pub fn balance(&mut self) {
        self.0.balance(None);
    }
}

fn main() {
    let input = {
        let stdin = io::stdin();
        let stdin_lock = stdin.lock();
        parse_input(stdin_lock)
    };

    dbg!(&input);

    let extent = Extent3i::from_min_and_shape(Point3i::fill(-50), Point3i::fill(101));

    let mut map: Array3x1<i32> = Array3x1::fill(extent, 0);

    for cmd in input.iter() {
        let cmd_extent = cmd.cuboid.to_extent();

        dbg!(&cmd_extent);

        let val = if cmd.on {
            1
        }
        else {
            0
        };

        dbg!(val);

        //*map.get_mut(PointN([0, 0, 0])) = 1;
        map.fill_extent(&cmd_extent, val);
    }


    let mut lit_cells = 0;

    dbg!(map.extent());

    map.for_each(&map.extent(), |p: Point3i, val: i32| {
        if val != 0 {
            lit_cells += 1;
        }
    });

    println!("on cubes: {}", lit_cells);



    let mut kdtree = KDTree::new();

    for cmd in input.iter() {
        kdtree.insert(cmd.clone());
        kdtree.balance();
    }

    println!("on cubes (part2): {}", kdtree.get_volume());

    // 1235164413198198
    // 1110237838901716
    // 226687381203442

    // let chunk_shape = Point3i::fill(16);
    // //let ambient_value = 0;
    // let builder = ChunkMapBuilder3x1::new(chunk_shape, 0);
    // let mut map = builder.build_with_hash_map_storage();

    // let mut lod0 = map.lod_view_mut(0);


    // for cmd in input.iter() {
    //     let cmd_extent = cmd.cuboid.to_extent();

    //     dbg!(&cmd_extent);

    //     let val = if cmd.on {
    //         1
    //     }
    //     else {
    //         0
    //     };

    //     dbg!(val);

    //     //*map.get_mut(PointN([0, 0, 0])) = 1;
    //     lod0.fill_extent(&cmd_extent, val);
    // }


    // let mut lit_cells = 0;

    // //dbg!(map.extent());

    // map.visit_occupied_chunks(0, &map.bounding_extent(0), |chunk| {
    //     chunk.for_each(chunk.extent(), |_: Point3i, val| {
    //         if val != 0 {
    //             lit_cells += 1;
    //         }
    //     })
    // });

    // println!("on cubes: {}", lit_cells);
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use crate::{parse_input, split_cubes, Input, Cuboid, Vec3, HalfSpace, AAPlane, KDTree, Command};

    fn get_test_input() -> Input {
        let test_data_str = include_str!("../test_input.txt");

        let test_data_reader = Cursor::new(test_data_str.to_owned());

        parse_input(test_data_reader)
    }

    #[test]
    fn test_parse() {
        let test_data = get_test_input();
    }

    #[test]
    fn split_cube_with_halfspace() {
        let cube = Cuboid {
            min: Vec3::new(-5, -5, -5),
            max: Vec3::new(5, 5, 5)
        };

        let halfspace = HalfSpace(true, AAPlane::X(0));

        let (maybe_a, maybe_b) = halfspace.split(cube);

        let a = maybe_a.unwrap();
        let b = maybe_b.unwrap();
    }

    #[test]
    fn split_cube_with_halfspace_min() {
        let cube = Cuboid {
            min: Vec3::new(-5, -5, -5),
            max: Vec3::new(5, 5, 5)
        };

        let halfspace = HalfSpace(true, AAPlane::X(-5));

        let (maybe_a, maybe_b) = halfspace.split(cube);

        assert_eq!(maybe_a, None);
        let b = maybe_b.unwrap();
    }

    #[test]
    fn split_cube_with_halfspace_max() {
        let cube = Cuboid {
            min: Vec3::new(-5, -5, -5),
            max: Vec3::new(5, 5, 5)
        };

        let halfspace = HalfSpace(true, AAPlane::X(6));

        let (maybe_a, maybe_b) = halfspace.split(cube);

        let a = maybe_a.unwrap();
        assert_eq!(maybe_b, None);
    }

    #[test]
    fn test_split_cubes() {
        let cube_a = Cuboid {
            min: Vec3::new(-5, -5, -5),
            max: Vec3::new(5, 5, 5)
        };

        let cube_b = Cuboid {
            min: Vec3::new(0, -5, -5),
            max: Vec3::new(5, 5, 5)
        };

        let (cube_a_outside, cube_a_inside) = split_cubes(cube_b, cube_a);

        dbg!(&cube_a_outside);
        dbg!(&cube_a_inside);

        assert_eq!(cube_a_outside.len(), 1);
        assert_eq!(cube_a_inside.len(), 1);

        assert_eq!(cube_a_outside[0].max[0], 0);
        assert_eq!(cube_a_inside[0].min[0], 0);

        assert_eq!(cube_a_outside[0].get_volume(), 5*10*10);
        assert_eq!(cube_a_inside[0].get_volume(), 5*10*10);
    }



    #[test]
    fn test_split_cubes_inside() {
        let cube_a = Cuboid {
            min: Vec3::new(0, 0, 0),
            max: Vec3::new(3, 3, 3)
        };

        let cube_b = Cuboid {
            min: Vec3::new(1, 1, 1),
            max: Vec3::new(2, 2, 2)
        };

        let (cube_a_outside, cube_a_inside) = split_cubes(cube_b, cube_a);

        dbg!(&cube_a_outside);
        dbg!(&cube_a_inside);

        let inside_vol: usize = cube_a_inside.iter().map(|c| c.get_volume()).sum();
        let outside_vol: usize = cube_a_outside.iter().map(|c| c.get_volume()).sum();

        assert_eq!(inside_vol, 1);
        assert_eq!(outside_vol, 3*3*3-1);

        //assert_eq!(cube_a_outside.len(), 1);
        assert_eq!(cube_a_inside.len(), 1);
    }

    #[test]
    fn test_kd_tree_single_node() {
        let mut kdtree = KDTree::new();

        let cmd = Command::from_str("on x=10..12,y=10..12,z=10..12");

        kdtree.insert(cmd);

        assert_eq!(kdtree.get_volume(), 3*3*3);
    }

    #[test]
    fn test_kd_tree_two_overlapping_nodes() {
        let mut kdtree = KDTree::new();

        let cmd_1 = Command::from_str("on x=10..12,y=10..12,z=10..12");
        let cmd_2 = Command::from_str("on x=11..13,y=11..13,z=11..13");

        kdtree.insert(cmd_1);
        kdtree.insert(cmd_2);

        assert_eq!(kdtree.get_volume(), 46);
    }

    #[test]
    fn test_kd_tree_two_overlapping_nodes_one_off() {
        let mut kdtree = KDTree::new();

        let cmd_1 = Command::from_str("on x=10..12,y=10..12,z=10..12");
        let cmd_2 = Command::from_str("on x=11..13,y=11..13,z=11..13");
        let cmd_3 = Command::from_str("off x=9..11,y=9..11,z=9..11");

        kdtree.insert(cmd_1);
        kdtree.insert(cmd_2);
        kdtree.insert(cmd_3);

        assert_eq!(kdtree.get_volume(), 38);
    }
}
