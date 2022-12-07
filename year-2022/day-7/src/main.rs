use std::collections::{BTreeMap, HashMap};
use std::io::{self, BufRead};
use std::str::FromStr;

#[derive(Debug)]
enum Command {
    ChangeDir(String),
    List,
}

#[derive(Debug)]
enum EntryInfo {
    Dir,
    File(usize),
}

#[derive(Debug)]
struct ListEntry {
    entry: EntryInfo,
    name: String,
}

#[derive(Debug)]
enum TermOutput {
    Command(Command),
    ListEntry(ListEntry),
}

impl FromStr for TermOutput {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("$ ") {
            let cmd_str = &s[2..];

            if cmd_str == "ls" {
                return Ok(TermOutput::Command(Command::List));
            }

            let (cmd, arg) = cmd_str.split_once(' ').unwrap();

            if cmd == "cd" {
                return Ok(TermOutput::Command(Command::ChangeDir(arg.to_string())));
            }

            return Err(anyhow::anyhow!("Unknown command: {}", cmd));
        }

        let (entry_str, name) = s.split_once(' ').unwrap();

        let entry = if entry_str == "dir" {
            EntryInfo::Dir
        } else {
            EntryInfo::File(usize::from_str_radix(entry_str, 10).unwrap())
        };

        Ok(TermOutput::ListEntry(ListEntry {
            entry,
            name: name.to_string(),
        }))
    }
}

type Input = Vec<TermOutput>;

#[derive(Default, Debug)]
struct Directory {
    parent: usize,
    children: BTreeMap<String, usize>,
    file_sizes: BTreeMap<String, usize>,
}

fn parse_input(mut reader: impl BufRead) -> Input {
    reader
        .lines()
        .map(|l| l.unwrap())
        .map(|line| TermOutput::from_str(&line).unwrap())
        .collect()
}

fn build_fs(term_output: &[TermOutput]) -> Vec<Directory> {
    let mut dirs = vec![Directory::default()];

    let mut cur_dir = 0;

    for term_line in term_output.iter() {
        match term_line {
            TermOutput::Command(Command::ChangeDir(d)) => {
                if d == "/" {
                    cur_dir = 0;
                } else if d == ".." {
                    cur_dir = dirs[cur_dir].parent;
                } else {
                    if !dirs[cur_dir].children.contains_key(d) {
                        let dir_handle = dirs.len();
                        dirs.push(Directory {
                            parent: cur_dir,
                            ..Default::default()
                        });

                        dirs[cur_dir].children.insert(d.to_string(), dir_handle);
                        cur_dir = dir_handle;
                    } else {
                        cur_dir = dirs[cur_dir].children[d];
                    }
                }
            }
            TermOutput::Command(Command::List) => {}
            TermOutput::ListEntry(ListEntry { entry, name }) => match entry {
                EntryInfo::Dir => {}
                EntryInfo::File(filesize) => {
                    dirs[cur_dir].file_sizes.insert(name.to_string(), *filesize);
                }
            },
        }
    }

    dirs
}

fn print_fs(dirs: &[Directory], dir: usize, indent: usize) {
    for (child_name, dir_id) in dirs[dir].file_sizes.iter() {
        for _ in 0..indent {
            print!(" ");
        }

        println!("{}", child_name);
    }

    for (child_name, dir_id) in dirs[dir].children.iter() {
        for _ in 0..indent {
            print!(" ");
        }

        println!("{}/", child_name);
        print_fs(dirs, *dir_id, indent + 4);
    }
}

fn get_sizes_inner(dirs: &[Directory], sizes: &mut Vec<usize>, cur_dir: usize) -> usize {
    let mut cur_total = dirs[cur_dir].file_sizes.values().sum();

    for child_id in dirs[cur_dir].children.values() {
        cur_total += get_sizes_inner(dirs, sizes, *child_id);
    }

    sizes[cur_dir] = cur_total;
    cur_total
}

fn get_sizes(dirs: &[Directory]) -> Vec<usize> {
    let mut sizes = vec![0usize; dirs.len()];
    get_sizes_inner(dirs, &mut sizes, 0);
    sizes
}

fn main() {
    let input = {
        let stdin = io::stdin();
        let stdin_lock = stdin.lock();
        parse_input(stdin_lock)
    };

    dbg!(&input);

    let fs = build_fs(&input);

    print_fs(&fs, 0, 0);

    let mut sizes = get_sizes(&fs);

    let sizes_under_100000: usize = sizes.iter().copied().filter(|sz| *sz < 100000).sum();

    dbg!(sizes_under_100000);

    let space_free = (70000000 - sizes[0]);
    let space_needed = 30000000 - space_free;

    sizes.sort();
    dbg!(&sizes);

    dbg!(space_free, space_needed);

    let dir_size_delete = sizes
        .iter()
        .copied()
        .find(|dir_size| *dir_size >= space_needed);

    dbg!(dir_size_delete);
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
