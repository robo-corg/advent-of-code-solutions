use std::collections::{BinaryHeap, HashMap, HashSet, LinkedList};
use std::io::{self, BufRead};
use std::str::FromStr;
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Mutex};
use std::thread;
use anyhow::Result;
use bigdecimal::{BigDecimal, FromPrimitive};
use rayon::prelude::*;

type Pos = nalgebra::Point2<i32>;
type Vec2 = nalgebra::Vector2<i32>;

type Input = StoneStore;

struct Stone {
    num: u64,
    exp_2024: u32
}


#[derive(Debug, Clone)]
struct StoneStore {
    len: usize,
    chunks: LinkedList<Vec<BigDecimal>>
}

impl StoneStore {
    fn from_vec(v: Vec<BigDecimal>) -> Self {
        let mut chunks = LinkedList::new();

        let len = v.len();

        chunks.push_back(v);

        StoneStore {
            len,
            chunks
        }
    }

    fn len(&self) -> usize {
        self.len
    }

    fn add_chunk(&mut self, v: Vec<BigDecimal>) {
        self.len += v.len();
        self.chunks.push_back(v);
    }

    fn iter_mut<'a>(&'a mut self) -> impl Iterator<Item=&'a mut BigDecimal> {
        self.chunks.iter_mut().flat_map(|chunk| chunk.iter_mut())
    }

    fn into_iter(self) -> impl Iterator<Item=BigDecimal>  {
        self.chunks.into_iter().flat_map(|chunk| chunk.into_iter())
    }
}

fn parse_input(mut reader: impl BufRead) -> Result<Input> {
    let mut buf = String::new();
    reader.read_to_string(&mut buf)?;

    let nums = buf.trim().split(' ').map(|num_s| BigDecimal::from_str(num_s).unwrap()).collect();

    Ok(StoneStore::from_vec(nums))
}

fn step(nums: &mut StoneStore) {
    let mut n = 0;
    let length = nums.len();


    let mut new_stones = Vec::with_capacity(1024);

    let zero = BigDecimal::from_i32(0).unwrap();
    let one = BigDecimal::from_i32(1).unwrap();
    let magic = BigDecimal::from_i32(2024).unwrap();

    for num in nums.iter_mut() {
        if *num == zero {
            *num = one.clone();
        }
        else if num.digits() % 2 == 0 {
            let nums_str = num.to_plain_string();

            let mid = nums_str.len()/2;

            let a_s = &nums_str[0..mid];
            let b_s = &nums_str[mid..];

            let a = BigDecimal::from_str(a_s).unwrap();
            let b = BigDecimal::from_str(b_s).unwrap();

            *num = a;
            new_stones.push(b);
        }
        else {
            *num *= magic.clone();
        }
    }

    nums.add_chunk(new_stones);
}

fn step_par(nums: &mut StoneStore) {
    let mut n = 0;
    let length = nums.len();


    let mut new_stones = Vec::with_capacity(1024);

    let zero = BigDecimal::from_i32(0).unwrap();
    let one = BigDecimal::from_i32(1).unwrap();
    let magic = BigDecimal::from_i32(2024).unwrap();

    for num in nums.iter_mut() {
        if *num == zero {
            *num = one.clone();
        }
        else if num.digits() % 2 == 0 {
            let nums_str = num.to_plain_string();

            let mid = nums_str.len()/2;

            let a_s = &nums_str[0..mid];
            let b_s = &nums_str[mid..];

            let a = BigDecimal::from_str(a_s).unwrap();
            let b = BigDecimal::from_str(b_s).unwrap();

            *num = a;
            new_stones.push(b);
        }
        else {
            *num *= magic.clone();
        }
    }

    nums.add_chunk(new_stones);
}

struct Task {
    stones: StoneStore,
    steps_needed: u32
}

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Task {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.steps_needed.cmp(&other.steps_needed).reverse()
    }
}

impl PartialEq for Task {
    fn eq(&self, other: &Self) -> bool {
        self.steps_needed.eq(&other.steps_needed)
    }
}

impl Eq for Task {

}

struct TaskSource {
    tasks: Mutex<BinaryHeap<Task>>
}

impl TaskSource {
    fn new(stones: StoneStore, steps_needed: u32) -> TaskSource {
        let mut tasks = BinaryHeap::new();

        tasks.push(Task { stones, steps_needed });

        TaskSource { tasks: Mutex::new(tasks) }
    }

    fn pop_tasks(&self, count: usize) -> Vec<Task> {
        let mut tasks_locked = self.tasks.lock().unwrap();

        let mut ret_tasks = Vec::new();

        for _ in 0..count {
            if let Some(task) = tasks_locked.pop() {
                ret_tasks.push(task);
            }
            else {
                break;
            }
        }

        drop(tasks_locked);

        ret_tasks
    }

    fn push_tasks<I>(&self, tasks: I)
        where I: Iterator<Item=Task>
    {
        let mut tasks_locked = self.tasks.lock().unwrap();
        tasks_locked.extend(tasks);
    }
}

fn task_worker(source: Arc<TaskSource>, total: Arc<AtomicUsize>) {
    loop {
        let tasks = source.pop_tasks(64);

        'outer: for mut task in tasks.into_iter() {
            //dbg!(task.steps_needed, tasks.len(), total);
            while task.steps_needed > 0 {
                if task.stones.len() > 100000 {
                    source.push_tasks(task.stones.into_iter().map(|num| {
                        Task { stones: StoneStore::from_vec(vec![num]), steps_needed: task.steps_needed }
                    }));

                    continue 'outer;
                }

                step(&mut task.stones);

                task.steps_needed -= 1;
            }

            total.fetch_add(task.stones.len(), std::sync::atomic::Ordering::SeqCst);
        }
    }
}

fn run_stones(stones: StoneStore, steps_needed: u32) -> u128 {
    let mut tasks = BinaryHeap::new();

    tasks.push(Task { stones, steps_needed });

    let mut total: u128 = 0;

    'outer: while let Some(mut task) = tasks.pop() {
        //dbg!(task.steps_needed, tasks.len(), total);
        while task.steps_needed > 0 {
            if task.stones.len() > 10000 {
                for num in task.stones.into_iter() {
                    tasks.push(Task { stones: StoneStore::from_vec(vec![num]), steps_needed: task.steps_needed });
                }

                continue 'outer;
            }

            step(&mut task.stones);

            task.steps_needed -= 1;
        }

        total.saturating_add(task.stones.len() as u128);
    }

    total
}


fn run_stones_par(stones: StoneStore, steps_needed: u32) -> usize {
    let source = Arc::new(TaskSource::new(stones, steps_needed));
    let total = Arc::new(AtomicUsize::new(0));

    let mut join_handles = Vec::new();

    for _ in 0..12 {
        let source = source.clone();
        let total = total.clone();
        join_handles.push(thread::spawn(move || {
            task_worker(source, total);
        }));
    }

    for join_handle in join_handles.into_iter() {
        join_handle.join().unwrap();
    }

    total.load(std::sync::atomic::Ordering::SeqCst)
}


fn main() -> Result<()> {
    let nums = {
        let stdin = io::stdin();
        let stdin_lock = stdin.lock();
        parse_input(stdin_lock)?
    };

    dbg!(&nums);

    let mut part1_nums = nums.clone();

    for _ in 0..25 {
        step(&mut part1_nums);
    }

    println!("part1: {}", part1_nums.len());

    drop(part1_nums);

    let mut part2_nums = nums.clone();

    println!("starting part2...");

    //let mut part2_total = run_stones(part2_nums, 50);

    let part2_total = run_stones(part2_nums, 50);

    println!();

    println!("part2: {}", part2_total);

    Ok(())
}
