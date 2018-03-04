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

fn unique_filter() -> Box<FnMut(&String) -> bool> {
    let mut lines: HashSet<String> = HashSet::new();

    return Box::new(move |line| lines.insert(line.clone()));
}

fn unique_filter_with_cap(capacity: usize) -> Box<FnMut(&String) -> bool> {
    let mut lines: HashSet<String> = HashSet::new();

    return Box::new(move |line| {
        if lines.insert(line.clone()) {
            if lines.len() > capacity {
                panic!("Cache capacity exceeded!");
            }
            return true;
        }
        return false;
    });
}

fn unique_filter_with_override(capacity: usize) -> Box<FnMut(&String) -> bool> {
    let mut set = HashSet::new();
    let mut queue = VecDeque::new();

    return Box::new(move |line| {
        if set.insert(line.clone()) {
            if set.len() > capacity {
                set.remove(&queue.pop_front().unwrap());
            }

            queue.push_back(line.clone());
            return true;
        }
        return false;
    });
}

fn unique() {
    let mut filter_obj = unique_filter();

    for line in stdin_reader().filter(|x| filter_obj(&x)) {
        print!("{}", line);
    }
}

fn unique_and_die(capacity: usize) {
    let mut filter_obj = unique_filter_with_cap(capacity);

    for line in stdin_reader().filter(|x| filter_obj(&x)) {
        print!("{}", line);
    }
}

fn unique_and_overwrite(capacity: usize) {
    let mut filter_obj = unique_filter_with_override(capacity);

    for line in stdin_reader().filter(|x| filter_obj(&x)) {
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
