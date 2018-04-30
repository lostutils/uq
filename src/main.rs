extern crate clap;
use clap::{App, Arg};

extern crate fxhash;
use fxhash::FxHashSet;

use std::collections::VecDeque;
use std::io::{BufRead, StdinLock, Write};

struct StdinReader<'a> {
    buffer: Vec<u8>,
    input: StdinLock<'a>,
}

impl<'a> StdinReader<'a> {
    fn new(input: StdinLock<'a>) -> Self {
        Self {
            buffer: Vec::new(),
            input: input,
        }
    }

    fn next_line(&mut self) -> Option<&Vec<u8>> {
        self.buffer.clear();
        match self.input.read_until(b'\n', &mut self.buffer) {
            Ok(0) => None,
            Ok(_) => Some(&self.buffer),
            Err(e) => panic!("Failed reading line: {}", e),
        }
    }
}

fn unique_filter() -> Box<FnMut(&Vec<u8>) -> bool> {
    let mut lines = FxHashSet::default();

    return Box::new(move |line| lines.insert(line.clone()));
}

fn unique_filter_with_cap(capacity: usize) -> Box<FnMut(&Vec<u8>) -> bool> {
    let mut lines = FxHashSet::default();

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

fn unique_filter_with_override(capacity: usize) -> Box<FnMut(&Vec<u8>) -> bool> {
    let mut set = FxHashSet::default();
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

    let mut unique_filter = match (capacity, matches.is_present("override")) {
        (Some(capacity), true) => unique_filter_with_override(capacity),
        (Some(capacity), false) => unique_filter_with_cap(capacity),
        _ => unique_filter(),
    };

    let (_in, _out) = (std::io::stdin(), std::io::stdout());
    let (input, mut output) = (_in.lock(), _out.lock());

    let mut stdin_reader = StdinReader::new(input);
    while let Some(line) = stdin_reader.next_line() {
        if unique_filter(line) {
            output.write(line).expect("Failed writing line");
        }
    }
}
