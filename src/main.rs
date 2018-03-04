extern crate clap;
use clap::{App, Arg};

use std::collections::{HashSet, VecDeque};

struct StdinReader;

impl Iterator for StdinReader {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        let mut line = String::new();

        match std::io::stdin().read_line(&mut line) {
            Ok(0) => None,
            Ok(_) => Some(line),
            Err(e) => panic!("Failed reading line: {}", e),
        }
    }
}

fn stdin_reader() -> StdinReader {
    StdinReader {}
}

trait UniqueTest {
    fn is_unique(&mut self, line: &String) -> bool;
}

struct UnlimitedUnique {
    lines: HashSet<String>,
}

impl UnlimitedUnique {
    fn new() -> Self {
        UnlimitedUnique {
            lines: HashSet::new(),
        }
    }
}

impl UniqueTest for UnlimitedUnique {
    fn is_unique(&mut self, line: &String) -> bool {
        self.lines.insert(line.clone())
    }
}

struct CapacityUnique {
    lines: HashSet<String>,
    capacity: usize,
}

impl CapacityUnique {
    fn new(capacity: usize) -> Self {
        CapacityUnique {
            lines: HashSet::new(),
            capacity: capacity,
        }
    }
}

impl UniqueTest for CapacityUnique {
    fn is_unique(&mut self, line: &String) -> bool {
        if self.lines.insert(line.clone()) {
            if self.lines.len() > self.capacity {
                panic!("Cache capacity exceeded!");
            }
            return true;
        }
        return false;
    }
}

struct OverrideUnique {
    capacity: usize,
    set: HashSet<String>,
    queue: VecDeque<String>,
}

impl OverrideUnique {
    fn new(capacity: usize) -> Self {
        OverrideUnique {
            set: HashSet::new(),
            capacity: capacity,
            queue: VecDeque::new(),
        }
    }
}

impl UniqueTest for OverrideUnique {
    fn is_unique(&mut self, line: &String) -> bool {
        if self.set.insert(line.clone()) {
            if self.set.len() > self.capacity {
                self.set.remove(&self.queue.pop_front().unwrap());
            }

            self.queue.push_back(line.clone());
            return true;
        }
        return false;
    }
}

fn unique() {
    let mut filter_obj = UnlimitedUnique::new();

    for line in stdin_reader().filter(|x| filter_obj.is_unique(&x)) {
        print!("{}", line);
    }
}

fn unique_and_die(capacity: usize) {
    let mut filter_obj = CapacityUnique::new(capacity);

    for line in stdin_reader().filter(|x| filter_obj.is_unique(&x)) {
        print!("{}", line);
    }
}

fn unique_and_overwrite(capacity: usize) {
    let mut filter_obj = OverrideUnique::new(capacity);

    for line in stdin_reader().filter(|x| filter_obj.is_unique(&x)) {
        print!("{}", line);
    }
}

fn main() {
    let matches = App::new("uq (lostutils)")
        .arg(
            Arg::with_name("capacity")
                .short("n")
                .help("Number of unique entries to remember.")
                .value_name("capacity")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("override")
                .short("r")
                .help("Override old unique entries when capacity reached.\nWhen not used, uq will die when the capacity is exceeded.")
                .requires("capacity")
                .value_name("override")
                .takes_value(false),
        )
        .get_matches();

    let capacity = match matches.value_of("capacity") {
        Some(n) => match n.parse::<usize>() {
            Ok(n) => Some(n),
            Err(_) => None,
        },
        None => None,
    };

    match (capacity, matches.is_present("override")) {
        (Some(capacity), true) => unique_and_overwrite(capacity),
        (Some(capacity), false) => unique_and_die(capacity),
        _ => unique(),
    }
}
