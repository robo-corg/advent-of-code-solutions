use std::collections::{BTreeMap, HashMap, HashSet, BinaryHeap};
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
    let re = Regex::new(r"Valve ([A-Z]{2}) has flow rate=(\d+); tunnels? leads? to valves? (.+)*$")
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

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash)]
struct PlanState {
    cur_location: NodeName,
    elephant_location: Option<NodeName>,
    preasure_released: i32,
    time_elapsed: i32,
    opened_valves: BTreeMap<NodeName, i32>,
}

impl PlanState {
    fn new(cur_location: NodeName, elephant: bool) -> Self {
        PlanState {
            cur_location,
            elephant_location: if elephant { Some(cur_location) } else { None },
            preasure_released: 0,
            time_elapsed: 0,
            opened_valves: Default::default(),
        }
    }

    fn open_valve(&self, actor: Actor, flow: i32) -> Self {
        let mut new_state = self.clone();
        //new_state.tick();
        new_state.opened_valves.insert(self.location_of(actor), flow);
        new_state
    }

    fn tick(&mut self) {
        let released: i32 = self.opened_valves.values().sum();

        self.preasure_released += released;
        self.time_elapsed += 1;
    }


    fn has_time_left(&self) -> bool {
        self.time_elapsed < self.max_time()
    }

    fn can_do_action(&self) -> bool {
        self.time_elapsed < self.max_time()
    }

    fn location_of(&self, actor: Actor) -> NodeName {
        match actor {
            Actor::Human => self.cur_location,
            Actor::Elephant => self.elephant_location.unwrap(),
        }
    }

    fn is_valve_open(&self, actor: Actor) -> bool {
        self.opened_valves.contains_key(&self.location_of(actor))
    }

    fn best_score(&self) -> i32 {
        let s: i32 = self.opened_valves.values().sum();
        self.preasure_released + s * (self.max_time() - self.time_elapsed)
    }

    fn max_time(&self) -> i32 {
        if self.elephant_location.is_some() {
            26
        }
        else {
            30
        }
    }

    fn apply_action(&mut self, actor: Actor, action: Action) {
        match action {
            Action::OpenValve(flow) => {
                *self = self.open_valve(actor, flow);
            },
            Action::MoveTo(new_location) => {
                match actor {
                    Actor::Human => { self.cur_location = new_location; },
                    Actor::Elephant => {self.elephant_location = Some(new_location); },
                }
            },
        }
    }

    fn remaining_ticks(&self) -> i32 {
        self.max_time() - self.time_elapsed
    }

    fn estimate_best(&self, map: &World) -> i32 {
        let mut hypothetical = self.clone();

        for (valve_name, flow) in map.valves.iter().take((self.remaining_ticks() * 2) as usize) {
            hypothetical.opened_valves.insert(*valve_name, *flow);
        }

        hypothetical.best_score()
    }

    fn next_actions<'a>(&self, map: &'a World, actor: Actor) -> impl Iterator<Item=Action> + 'a {
        let cur_room_valve_flow = map.valves[&self.location_of(actor)];

        let valve_open_action = if cur_room_valve_flow > 0 && !self.is_valve_open(actor) && self.can_do_action() {
            Some(Action::OpenValve(cur_room_valve_flow))
        }
        else {
            None
        };

        valve_open_action.into_iter().chain(
            map.connections.neighbors(self.location_of(actor)).map(|neighbor| {
                Action::MoveTo(neighbor)
            })
        )
    }
}

fn pick_best_plan(a: PlanState, b: PlanState) -> PlanState {
    if a.preasure_released > b.preasure_released {
        a
    } else {
        b
    }
}

#[derive(Debug, Eq)]
struct VisitItem {
    estimated_best: i32,
    state: PlanState,
}

impl PartialEq for VisitItem {
    fn eq(&self, other: &Self) -> bool {
        self.estimated_best == other.estimated_best
    }
}

impl PartialOrd for VisitItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for VisitItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.estimated_best.cmp(&other.estimated_best)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum Actor {
    Human,
    Elephant
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum ActionType {
    OpenValve,
    MoveTo
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum Action {
    OpenValve(i32),
    MoveTo(NodeName)
}

fn plan(map: &World, explored: &mut HashSet<PlanState>, mut status: PlanState) -> PlanState {
    let mut visit_queue = BinaryHeap::new();
    let mut best_new_plan: PlanState = status.clone();

    visit_queue.push(VisitItem {
        estimated_best: status.estimate_best(map),
        state: status,
    });
    //costs.insert(start_pos, get_cost(&map, start_pos));

    while let Some(visit_item) = visit_queue.pop() {
        let VisitItem {
            estimated_best: current_estimated_cost,
            state: status,
        } = visit_item;

        // if visit_queue.len() > 1000000 {
        //     println!("Cleaning queue");
        //     let old_count = visit_queue.len();
        //     let mut new_queue = BinaryHeap::new();
        //     for item in visit_queue.drain() {
        //         if best_new_plan.preasure_released < item.state.estimate_best(map) {
        //             new_queue.push(item);
        //         }
        //     }

        //     println!("Removed {} items", old_count - new_queue.len());

        //     visit_queue = new_queue;
        // }

        if best_new_plan.preasure_released < status.preasure_released {
            println!("{} queue:{} estimate:{}", status.preasure_released, visit_queue.len(), current_estimated_cost);
        }

        best_new_plan = pick_best_plan(best_new_plan, status.clone());

        if explored.contains(&status) {
            continue;
        }

        if best_new_plan.preasure_released > status.estimate_best(map) {
            continue;
        }

        explored.insert(status.clone());

        //dbg!(explored.len(), status.preasure_released);

        if !status.can_do_action() {
            continue;
        }


        let human_actions: Vec<Action> = status.next_actions(map, Actor::Human).collect();
        let elephant_actions: Vec<Action> = status.next_actions(map, Actor::Elephant).collect();

        for human_action in human_actions.iter() {
            for elephant_action in elephant_actions.iter() {
                let mut new_plan = status.clone();

                new_plan.tick();

                new_plan.apply_action(Actor::Human, *human_action);
                new_plan.apply_action(Actor::Elephant, *elephant_action);

                if !explored.contains(&new_plan) {
                    if best_new_plan.preasure_released < new_plan.estimate_best(map) {
                        visit_queue.push(VisitItem {
                            estimated_best: new_plan.estimate_best(map),
                            state: new_plan,
                        });
                    }
                }
            }
        }
    }

    best_new_plan
}

// fn plan1(
//     map: &World,
//     explored: &mut HashSet<PlanState>,
//     mut status: PlanState,
// ) -> Option<PlanState> {
//     let mut best_new_plan = None;

//     let cur_room_valve_flow = map.valves[&status.cur_location];

//     if explored.contains(&status) {
//         return Some(status);
//     }

//     explored.insert(status.clone());

//     //dbg!(explored.len(), status.preasure_released);

//     if !status.can_do_action() {
//         return Some(status);
//     }

//     //dbg!(status.explored.len(), status.time_elapsed, status.preasure_released);

//     let open_valve_plan =
//         if cur_room_valve_flow > 0 && !status.is_valve_open() && status.can_do_action() {
//             Some(status.open_valve(cur_room_valve_flow))
//         } else {
//             None
//         };

//     let mut base_plans = [Some(status), open_valve_plan];

//     for base_plan in base_plans.iter().filter_map(|p| p.as_ref()) {
//         best_new_plan = pick_best_plan(best_new_plan, Some(base_plan.clone()));
//     }

//     for base_plan in base_plans.iter().filter_map(|p| p.as_ref()) {
//         if !base_plan.can_do_action() {
//             continue;
//         }

//         for neighbor in map.connections.neighbors(base_plan.cur_location) {
//             // if base_plan.explored.contains(&neighbor) {
//             //     continue;
//             // }
//             let mut new_plan = base_plan.clone();

//             new_plan.tick();
//             new_plan.cur_location = neighbor;

//             best_new_plan = pick_best_plan(best_new_plan, plan(map, explored, new_plan));
//         }
//     }

//     best_new_plan
// }

fn main() {
    let input = {
        let stdin = io::stdin();
        let stdin_lock = stdin.lock();
        parse_input(stdin_lock)
    };

    dbg!(&input);

    let mut explored = HashSet::new();

    let best_plan = plan(
        &input,
        &mut explored,
        PlanState::new(parse_node_name("AA"), true),
    );

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
