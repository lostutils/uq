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

fn unique_and_die(capacity: Option<usize>) {
    let mut lines = HashSet::new();

    for line in stdin_reader() {
        if lines.insert(line.clone()) {
            if let Some(capacity) = capacity {
                if lines.len() > capacity {
                    panic!("Cache capacity exceeded!");
                }
            }

            print!("{}", line);
        }
    }
}

fn unique_and_overwrite(capacity: usize) {
    let mut set = HashSet::new();
    let mut queue = VecDeque::new();

    for line in stdin_reader() {
        if set.insert(line.clone()) {
            if set.len() > capacity {
                set.remove(&queue.pop_front().unwrap());
            }

            queue.push_back(line.clone());
            print!("{}", line);
        }
    }
}

fn main() {
    let matches = App::new("uq")
        .arg(
            Arg::with_name("capacity")
                .short("n")
                .value_name("capacity")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("override")
                .short("r")
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

    if matches.is_present("override") {
        if let Some(capacity) = capacity {
            unique_and_overwrite(capacity);
        } else {
            panic!("Override requires capacity!");
        }
    } else {
        unique_and_die(capacity);
    }
}
