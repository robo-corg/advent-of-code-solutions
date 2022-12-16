use std::collections::{HashMap, HashSet};
use std::io::{self, BufRead};

use petgraph::prelude::UnGraphMap;
use regex::Regex;

type Input = World;

type NodeName = [char; 2];

type Graph = UnGraphMap<NodeName, ()>;

#[derive(Debug, Default)]
struct World {
    connections: Graph,
    valves: HashMap<NodeName, i32>,
}

fn parse_node_name(name: &str) -> NodeName {
    let mut chars = name.chars();
    [chars.next().unwrap(), chars.next().unwrap()]
}

fn parse_input(mut reader: impl BufRead) -> Input {
    // Valve AA has flow rate=0; tunnels lead to valves DD, II, BB
    let re = Regex::new(
        r"Valve ([A-Z]{2}) has flow rate=(\d+); tunnels? leads? to valves? (.+)*$",
    )
    .unwrap();

    let mut lines = reader.lines().map(|l| l.unwrap());

    let mut world = World::default();

    for line in lines {
        let captures = re.captures(&line).unwrap();

        dbg!(&captures);

        let mut captures_iter = captures.iter().skip(1);

        let cur_room = parse_node_name(captures_iter.next().unwrap().unwrap().as_str());
        let flow_rate =
            i32::from_str_radix(captures_iter.next().unwrap().unwrap().as_str(), 10).unwrap();

        world.connections.add_node(cur_room);

        let connections_s = captures_iter.next().unwrap().unwrap().as_str();


        for connection in connections_s.split(", ").map(parse_node_name) {
            dbg!((cur_room, connection));
            world.connections.add_node(connection);
            world.connections.add_edge(cur_room, connection, ());
        }

        world.valves.insert(cur_room, flow_rate);
    }

    world
}

#[derive(Debug, Clone, Default)]
struct PlanState {
    explored: HashSet<NodeName>,
    preasure_released: i32,
    time_elapsed: i32,
    opened_valves: HashMap<NodeName, i32>,
}

impl PlanState {
    fn open_valve(&self, valve_name: NodeName, flow: i32) -> Self {
        let mut new_state = self.clone();
        new_state.tick();
        new_state.opened_valves.insert(valve_name, flow);
        new_state
    }

    fn tick(&mut self) {
        let released: i32 = self.opened_valves.values().sum();

        self.preasure_released += released;
        self.time_elapsed += 1;
    }

    fn has_time_left(&self) -> bool {
        self.time_elapsed < 30
    }

    fn can_do_action(&self) -> bool {
        self.time_elapsed < 29
    }

    fn is_valve_open(&self, valve_name: NodeName) -> bool {
        self.opened_valves.contains_key(&valve_name)
    }
}

fn pick_best_plan(a: Option<PlanState>, b: Option<PlanState>) -> Option<PlanState> {
    match (a, b) {
        (None, None) => None,
        (None, Some(a)) => Some(a),
        (Some(a), None) => Some(a),
        (Some(a), Some(b)) => if a.preasure_released > b.preasure_released {
            Some(a)
        } else {
            Some(b)
        },
    }
}

fn plan(map: &World, start_pos: NodeName, mut status: PlanState) -> Option<PlanState> {
    let mut best_new_plan = None;

    let cur_room_valve_flow = map.valves[&start_pos];

    status.explored.insert(start_pos);

    if status.explored.len() == map.connections.node_count() || !status.can_do_action() {
        return Some(status);
    }

    //dbg!(status.explored.len(), status.time_elapsed);

    let open_valve_plan = if cur_room_valve_flow > 0 && !status.is_valve_open(start_pos) && status.can_do_action() {
        Some(status.open_valve(start_pos, cur_room_valve_flow))
    } else {
        None
    };

    let mut base_plans = [
        Some(status),
        open_valve_plan
    ];

    for base_plan in base_plans.iter().filter_map(|p| p.as_ref()) {
        best_new_plan = pick_best_plan(best_new_plan, Some(base_plan.clone()));
    }

    for base_plan in base_plans.iter().filter_map(|p| p.as_ref()) {
        if !base_plan.can_do_action() {
            continue;
        }

        for neighbor in map.connections.neighbors(start_pos) {
            // if base_plan.explored.contains(&neighbor) {
            //     continue;
            // }
            let mut new_plan = base_plan.clone();

            new_plan.tick();

            best_new_plan = pick_best_plan(best_new_plan, plan(map, neighbor, new_plan));
        }
    }

    best_new_plan
}

fn main() {
    let input = {
        let stdin = io::stdin();
        let stdin_lock = stdin.lock();
        parse_input(stdin_lock)
    };

    dbg!(&input);

    let best_plan = plan(&input, parse_node_name("AA"), PlanState::default());

    dbg!(best_plan);
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
