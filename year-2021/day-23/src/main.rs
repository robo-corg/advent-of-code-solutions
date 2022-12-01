use std::collections::{HashMap, HashSet, BinaryHeap};
use std::fmt;
use std::io::{self, BufRead};

use building_blocks::core::prelude::*;
use building_blocks::storage::{prelude::*, ChunkHashMap, ChunkMap2x1};

type Map = Array2x1<u8>;

type Vec2 = nalgebra::Vector2<i32>;
type Input = Vec<i32>;

fn parse_input(mut reader: impl BufRead) -> Input {
    unimplemented!()
}

const HALLWAY_LENGTH: i32 = 11;

const ROOM_X_POSITIONS: [i32; 4] = [3, 5, 7, 9];

const ROOM_Y: i32 = 2;
const ROOM_SIZE: i32 = 4;

const NEIGHBORS_DIRS: [Vec2; 4] = [
    Vec2::new(0, -1),
    Vec2::new(0, 1),
    Vec2::new(-1, 0),
    Vec2::new(1, 0),
];

fn hallway_positions() -> impl Iterator<Item = Vec2> {
    (0..HALLWAY_LENGTH).map(|x| Vec2::new(x + 1, 1))
}

fn load_map() -> Map {
    let extent = Extent2i::from_min_and_shape(Point2i::fill(0), Point2i::fill(16));

    let mut map: Array2x1<u8> = Array2x1::fill(extent, 0);

    for x in 0..HALLWAY_LENGTH {
        *map.get_mut(PointN([x + 1, 1])) = 1;
    }

    for room_x in ROOM_X_POSITIONS {
        for room_y in ROOM_Y..ROOM_Y + ROOM_SIZE {
            *map.get_mut(PointN([room_x, room_y])) = 2;
        }
    }

    map
}

fn room_positions() -> impl Iterator<Item = Vec2> {
    ROOM_X_POSITIONS
        .iter()
        .copied()
        .flat_map(|x| (ROOM_Y..ROOM_Y + ROOM_SIZE).map(move |y| Vec2::new(x, y)))
}

fn spawn_amphipods(types: &[AmphipodType]) -> Vec<Amphipod> {
    types
        .iter()
        .copied()
        .zip(room_positions())
        .map(|(ty, pos)| Amphipod {
            pos,
            ty,
            state: State::Fresh,
        })
        .collect()
}

struct DisplayMap<'a>(&'a Map, &'a [Amphipod]);

impl<'a> fmt::Display for DisplayMap<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let shape = self.0.extent();

        for y in shape.minimum.y()..(shape.minimum.y() + shape.shape.y()) {
            for x in shape.minimum.x()..(shape.minimum.x() + shape.shape.x()) {
                let p = Vec2::new(x, y);

                let maybe_occupied_by =
                    self.1
                        .iter()
                        .find_map(|pod| if pod.pos == p { Some(pod.ty) } else { None });

                let display_char = maybe_occupied_by
                    .map(|pod_ty| pod_ty.as_letter())
                    .unwrap_or_else(|| match self.0.get(PointN([x, y])) {
                        0 => "#",
                        1 => ".",
                        other => "?",
                    });

                write!(f, "{}", display_char)?;
            }

            writeln!(f, "")?;
        }

        Ok(())
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord)]
enum AmphipodType {
    Amber,
    Bronze,
    Copper,
    Desert,
}

impl AmphipodType {
    fn as_letter(&self) -> &'static str {
        match self {
            AmphipodType::Amber => "A",
            AmphipodType::Bronze => "B",
            AmphipodType::Copper => "C",
            AmphipodType::Desert => "D",
        }
    }

    fn move_cost(&self) -> i32 {
        match self {
            AmphipodType::Amber => 1,
            AmphipodType::Bronze => 10,
            AmphipodType::Copper => 100,
            AmphipodType::Desert => 1000,
        }
    }
}

fn get_map_at(map: &Map, pos: Vec2) -> u8 {
    map.get(PointN(pos.data.0[0]))
}

fn is_walkable(map: &Map, pos: Vec2) -> bool {
    get_map_at(map, pos) != 0
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
enum State {
    Fresh,
    WaitingInHall,
    Done,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
struct Amphipod {
    pos: Vec2,
    ty: AmphipodType,
    state: State,
}

impl Amphipod {
    fn next_moves(&self) -> impl Iterator<Item = Vec2> {
        let pos = self.pos;
        NEIGHBORS_DIRS.iter().map(move |dir| pos + dir)
    }

    fn next_legal_moves<'a>(&self, map: &'a Map) -> impl Iterator<Item = Vec2> + 'a {
        let state = self.state;
        self.next_moves()
            .filter(move |pos| is_walkable(map, *pos) && state != State::Done)
    }

    fn move_cost(&self) -> i32 {
        self.ty.move_cost()
    }

    fn in_hallway(&self, map: &Map) -> bool {
        let on_tile = get_map_at(map, self.pos);
        on_tile == 1
    }

    fn update_hall_state(&mut self, map: &Map) {
        let on_tile = get_map_at(map, self.pos);

        if self.state == State::Fresh && on_tile == 1 {
            self.state = State::WaitingInHall;
        }
    }

    fn check_path_segment(&self, occupied_locations: &HashSet<Vec2>, start_pos: Vec2, move_goal: Vec2) -> bool {
        let mut cur_pos = start_pos;

        // room -> hallway
        if move_goal[1] == 1 {
            while cur_pos[1] > 1 {
                cur_pos[1] -= 1;

                if occupied_locations.contains(&cur_pos) {
                    return false;
                }
            }

            while cur_pos[0] != move_goal[0] {
                cur_pos[0] += (move_goal[0] - cur_pos[0]).signum();

                if occupied_locations.contains(&cur_pos) {
                    return false;
                }
            }
        }
        // hallway -> room
        else if move_goal[1] > 1 && start_pos[1] == 1 {
            while cur_pos[0] != move_goal[0] {
                cur_pos[0] += (move_goal[0] - cur_pos[0]).signum();

                if occupied_locations.contains(&cur_pos) {
                    return false;
                }
            }

            while cur_pos[1] != move_goal[1] {
                cur_pos[1] += (move_goal[1] - cur_pos[1]).signum();

                if occupied_locations.contains(&cur_pos) {
                    return false;
                }
            }
        }
        // room -> room
        else {
            let mut in_hall = start_pos;
            in_hall[1] = 1;

            if !self.check_path_segment(occupied_locations, start_pos, in_hall) {
                return false;
            }

            //dbg!(&move_goal);
            if !self.check_path_segment(occupied_locations, in_hall, move_goal) {
                return false;
            }
        }

        true
    }

    fn check_path(&self, occupied_locations: &HashSet<Vec2>, move_goal: Vec2) -> bool {
        self.check_path_segment(occupied_locations, self.pos, move_goal)
    }

    fn next_states(
        &self,
        map: &Map,
        goal_rooms: &HashMap<AmphipodType, HashSet<i32>>,
        availble_rooms: &Vec<bool>,
        occupied_locations: &HashSet<Vec2>
    ) -> impl Iterator<Item = (i32, Amphipod)> {
        let mut next_states = Vec::new();

        match self.state {
            State::Fresh => {
                for hallway_pos in hallway_positions() {
                    if !ROOM_X_POSITIONS
                        .iter()
                        .copied()
                        .all(|room_x| hallway_pos[0] != room_x)
                    {
                        continue;
                    }

                    // Check if path to hallway is blocked
                    if !self.check_path(occupied_locations, hallway_pos) {
                        continue;
                    }

                    let mut new_state = self.clone();
                    new_state.pos = hallway_pos;
                    new_state.state = State::WaitingInHall;

                    let dist = (self.pos - hallway_pos).abs().sum();
                    let cost = dist * self.move_cost();
                    next_states.push((cost, new_state));
                }
            },
            _ => {}
        }

        match self.state {
            State::Fresh | State::WaitingInHall => {
                for (room_x, availible) in ROOM_X_POSITIONS
                    .iter()
                    .copied()
                    .zip(availble_rooms.iter().copied())
                {
                    if !availible {
                        continue;
                    }

                    let is_goal_room = goal_rooms
                        .get(&self.ty)
                        .map(|rooms| rooms.contains(&room_x))
                        .unwrap_or(false);

                    if !is_goal_room {
                        continue;
                    }

                    for room_y in ROOM_Y..ROOM_Y + ROOM_SIZE {
                        let room_pos = Vec2::new(room_x, room_y);

                        if self.pos != room_pos && occupied_locations.contains(&room_pos) {
                            continue;
                        }

                        if !self.check_path(occupied_locations, room_pos) {
                            continue;
                        }

                        let mut new_state = self.clone();
                        new_state.pos = room_pos;
                        new_state.state = State::Done;

                        let d = dist(self.pos, room_pos);
                        let cost = d * self.move_cost();
                        next_states.push((cost, new_state));
                    }
                }
            }
            State::Done => {}
        }

        next_states.into_iter()
    }
}

fn dist(a: Vec2, b: Vec2) -> i32 {
    if a[0] != b[0] && a[1] > 1 && b[1] > 1 {
        let mut to_hallway = a;
        to_hallway[1]  = 1;

        return dist(a, to_hallway) + dist(to_hallway, b);
    }

    let dist = (a - b).abs().sum();
    dist
}

fn check_goal(goal: &[Amphipod], state: &[Amphipod]) -> bool {
    assert_eq!(goal.len(), state.len());

    // let goal_set: HashSet<_> = goal.iter().map(|pod| (pod.ty, pod.pos)).collect();
    // let state_set: HashSet<_> = state.iter().map(|pod| (pod.ty, pod.pos)).collect();

    // goal_set == state_set

    state.iter().all(|pod| pod.state == State::Done)
}

fn determine_available_rooms(goal: &Vec<Amphipod>, pods: &Vec<Amphipod>) -> Vec<bool> {
    let pod_ty_by_pos: HashMap<Vec2, AmphipodType> =
        pods.iter().map(|pod| (pod.pos, pod.ty)).collect();
    let goal_ty_by_pos: HashMap<Vec2, AmphipodType> =
        goal.iter().map(|pod| (pod.pos, pod.ty)).collect();

    ROOM_X_POSITIONS
        .iter()
        .copied()
        .map(|room_x| {
            (ROOM_Y..ROOM_Y + ROOM_SIZE).all(|room_y| {
                let pos = Vec2::new(room_x, room_y);

                if let Some(occupied_type) = pod_ty_by_pos.get(&pos) {
                    goal_ty_by_pos.get(&pos) == Some(occupied_type)
                } else {
                    true
                }
            })
        })
        .collect()
}


#[derive(Debug, Eq)]
struct VisitItem {
    estimated_cost: i32,
    state: Vec<Amphipod>,
}

impl PartialEq for VisitItem {
    fn eq(&self, other: &Self) -> bool {
        self.estimated_cost == other.estimated_cost
    }
}

impl PartialOrd for VisitItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for VisitItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.estimated_cost.cmp(&self.estimated_cost)
    }
}

fn heuristic(goal_rooms: &HashMap<AmphipodType, HashSet<i32>>, pods: &Vec<Amphipod>) -> i32 {
    pods.iter().map(|pod| {
        let hallway_cost = (i32::abs(pod.pos[1] - 1) + 1) * pod.move_cost();
        match pod.state {
            State::Fresh => {
                let in_goal_already = goal_rooms.get(&pod.ty).map(|rooms| rooms.contains(&pod.pos[0])).unwrap_or(false);
                if in_goal_already { 0 } else { hallway_cost * 2 }
            },
            State::WaitingInHall => hallway_cost,
            State::Done => 0
        }
    }).sum()
}

fn find_lowest_energy_plan(
    map: &Map,
    goal: &Vec<Amphipod>,
    pods: Vec<Amphipod>,
) -> (i32, Vec<Amphipod>) {
    let goal_rooms_iter = goal.iter().map(|pod| (pod.ty, pod.pos[0]));

    let mut goal_rooms = HashMap::new();

    for (room_ty, room_x) in goal_rooms_iter {
        let rooms = goal_rooms.entry(room_ty).or_insert_with(|| HashSet::new());
        rooms.insert(room_x);
    }

    let mut visit_queue: BinaryHeap<VisitItem> = BinaryHeap::new();
    //let mut open: Vec<Vec<Amphipod>> = Vec::new();
    let mut visited: HashSet<(i32, Vec<Amphipod>)> = HashSet::default();

    let mut goal_configs = Vec::new();

    let mut costs: HashMap<Vec<Amphipod>, i32> = HashMap::new();

    let mut previous_links: HashMap<Vec<Amphipod>, Vec<Amphipod>> = HashMap::default();

    costs.insert(pods.clone(), 0);
    //let start_cost_and_state = (0, pods);
    //open.push(pods);
    visit_queue.push(VisitItem { estimated_cost: 0, state: pods });

    let mut cheapest_solution = i32::MAX;

    //visited.insert(start_cost_and_state);

    while let Some(VisitItem { state: cur_state, .. }) = visit_queue.pop() {

        if costs.len() % 100000 == 0 {
            println!(
                "explored: {} solutions: {} cheapest: {} visit queue: {}",
                costs.len(),
                goal_configs.len(),
                cheapest_solution,
                visit_queue.len()
            );

            //println!("{}", DisplayMap(map, &cur_state));
        }

        let cur_cost = costs[&cur_state];

        //dbg!(&cur_cost);

        if check_goal(goal, &cur_state) {
            goal_configs.push((cur_cost, cur_state));
            cheapest_solution = i32::min(cheapest_solution, cur_cost);
            continue;
        }

        if cur_cost > cheapest_solution {
            continue;
        }

        let available_rooms = determine_available_rooms(goal, &cur_state);

        let occupied_locations: HashSet<Vec2> = cur_state.iter().map(|pod| pod.pos).collect();

        for (cur_pod_id, pod) in cur_state.iter().enumerate() {
            for (action_cost, next_action) in pod.next_states(map, &goal_rooms, &available_rooms, &occupied_locations) {
                let next_cost = action_cost + cur_cost;
                let mut next_state = cur_state.clone();
                next_state[cur_pod_id] = next_action;



                if next_cost < costs.get(&next_state).copied().unwrap_or(i32::MAX) {
                    //open.push(next_state.clone());
                    previous_links.insert(next_state.clone(), cur_state.clone());
                    let estimated_cost = next_cost + heuristic(&goal_rooms, &next_state);
                    visit_queue.push(VisitItem { estimated_cost, state: next_state.clone()});
                    costs.insert(next_state, next_cost);

                }
            }
        }
    }

    let (min_cost, min_state) = goal_configs.into_iter().min_by_key(|(cost, state)| *cost).unwrap();


    println!("***** replay start *****");

    let mut states = Vec::new();
    let mut maybe_cur_state = Some(min_state.clone());
    states.push(min_state.clone());

    while let Some(cur_state) = maybe_cur_state {
        let next_state = previous_links.get(&cur_state).cloned();

        if let Some(next_state) = next_state.clone() {
            states.push(next_state);
        }

        maybe_cur_state = next_state;
    }

    states.reverse();

    for (turn, state) in states.iter().enumerate() {
        println!("Turn: {} cost: {}", turn, costs[state]);
        println!("{}", DisplayMap(map, &state));
    }

    println!("***** replay end *****");

    (min_cost, min_state)
}

fn main() {
    let map = load_map();

    // let pods = spawn_amphipods(&[
    //     AmphipodType::Bronze,
    //     AmphipodType::Amber,
    //     AmphipodType::Copper,
    //     AmphipodType::Desert,
    //     AmphipodType::Bronze,
    //     AmphipodType::Copper,
    //     AmphipodType::Desert,
    //     AmphipodType::Amber,
    // ]);

    // #############
    // #...........#
    // ###D#A#C#C###
    //   #D#A#B#B#
    //   #########


    // let pods = spawn_amphipods(&[
    //     AmphipodType::Desert,
    //     AmphipodType::Desert,
    //     AmphipodType::Amber,
    //     AmphipodType::Amber,
    //     AmphipodType::Copper,
    //     AmphipodType::Bronze,
    //     AmphipodType::Copper,
    //     AmphipodType::Bronze,
    // ]);

    // #############
    // #...........#
    // ###D#A#C#C###
    //   #D#C#B#A#
    //   #D#B#A#C#
    //   #D#A#B#B#
    //   #########


    let pods = spawn_amphipods(&[
        AmphipodType::Desert,
        AmphipodType::Desert,
        AmphipodType::Desert,
        AmphipodType::Desert,

        AmphipodType::Amber,
        AmphipodType::Copper,
        AmphipodType::Bronze,
        AmphipodType::Amber,

        AmphipodType::Copper,
        AmphipodType::Bronze,
        AmphipodType::Amber,
        AmphipodType::Bronze,

        AmphipodType::Copper,
        AmphipodType::Amber,
        AmphipodType::Copper,
        AmphipodType::Bronze,
    ]);



    println!("{}", DisplayMap(&map, &pods));

    let mut goal_ty: Vec<AmphipodType> = pods.iter().map(|p| p.ty).collect();
    goal_ty.sort();
    let mut goal = spawn_amphipods(&goal_ty);

    for pod in goal.iter_mut() {
        pod.state = State::Done;
    }

    println!("{}", DisplayMap(&map, &goal));

    let maybe_best_plan = find_lowest_energy_plan(&map, &goal, pods);

    let (best_plan_cost, best_plan) = maybe_best_plan;

    //println!("{}", DisplayMap(&map, &best_plan));

    println!("cost: {}", best_plan_cost);
}

#[cfg(test)]
mod test {
}
