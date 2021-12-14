use std::{io::{self, BufRead}, collections::HashSet};

use petgraph::{graphmap::UnGraphMap, visit};

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Copy)]
enum Cave {
    Start,
    Small([u8; 2]),
    Large([u8; 2]),
    End
}

impl Cave {
    fn is_small(&self) -> bool {
        match self {
            Cave::Small(_) => true,
            _ => false
        }
    }
}

impl Default for Cave {
    fn default() -> Self {
        Cave::Start
    }
}

type Input = Vec<(Cave, Cave)>;

type Graph = UnGraphMap<Cave, ()>;

fn parse_node(node_str: &str) -> Cave {
    if node_str == "start" {
        return Cave::Start;
    }

    if node_str == "end" {
        return Cave::End;
    }

    let upper = node_str.chars().all(|ch| ch.is_uppercase());
    let id_slice = node_str.as_bytes();
    let id: [u8; 2] = [
        id_slice[0],
        id_slice[1]
    ];

    if upper {
        Cave::Large(id)
    }
    else {
        Cave::Small(id)
    }
}

fn parse_input(mut reader: impl BufRead) -> Input {
    reader.lines().map(|line| {
        let line = line.unwrap();
        let (a, b) = line.split_once("-").unwrap();

        (
            parse_node(a),
            parse_node(b)
        )
    }).collect()
}

fn path_count(graph: &Graph, pos: Cave, visited: HashSet<Cave>, small_cave_visited: bool) -> usize {
    let mut count = 0;

    for neighbor in graph.neighbors(pos) {
        let next_small_cave_visited = if visited.contains(&neighbor) && !small_cave_visited && neighbor.is_small() {
            dbg!(neighbor);
            true
        }
        else if visited.contains(&neighbor) {
            continue;
        }
        else {
            small_cave_visited
        };

        match neighbor {
            Cave::End => {
                count += 1;
                //dbg!("End");
            }
            Cave::Start => {
                continue;
            }
            Cave::Small(_) => {
                let mut visited = visited.clone();
                visited.insert(neighbor);
                count += path_count(graph, neighbor, visited, next_small_cave_visited);
                //dbg!(neighbor);
            }
            Cave::Large(_) => {
                let visited = visited.clone();
                count += path_count(graph, neighbor, visited, next_small_cave_visited);
                //dbg!(neighbor);
            }
        }
    }

    count
}

fn main() {
    let input = {
        let stdin = io::stdin();
        let stdin_lock = stdin.lock();
        parse_input(stdin_lock)
    };

    dbg!(&input);

    let graph: Graph = UnGraphMap::from_edges(input.into_iter());

    //dbg!(&graph);

    let paths = path_count(&graph, Cave::Start, HashSet::new(), true);
    println!("{}", paths);



    let paths_part_2 = path_count(&graph, Cave::Start, HashSet::new(), false);
    println!("{}", paths_part_2);
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use crate::{parse_input, Input, test};

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
}
