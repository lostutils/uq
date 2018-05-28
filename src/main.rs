extern crate clap;
extern crate fxhash;
extern crate regex;
extern crate itertools;
extern crate failure;

#[macro_use]
extern crate failure_derive;
#[macro_use]
extern crate serde_derive;

use clap::{App, Arg};
use fxhash::FxHashSet;
use regex::bytes::Regex;
use itertools::Itertools;
use std::collections::VecDeque;
use std::io::{BufRead, StdinLock, Write};
use failure::Error;

#[derive(Debug, Fail)]
enum UqError {
    #[fail(display = "invalid regular expression: {}", regex)]
    InvalidRegex {
        regex: String,
    },
}

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

    Box::new(move |line| lines.insert(line.clone()))
}

fn unique_filter_with_cap(capacity: usize) -> Box<FnMut(&Vec<u8>) -> bool> {
    let mut lines = FxHashSet::default();

    Box::new(move |line| {
        if lines.insert(line.clone()) {
            if lines.len() > capacity {
                panic!("Cache capacity exceeded!");
            }
            true
        } else {
            false
        }
    })
}

fn unique_filter_with_override(capacity: usize) -> Box<FnMut(&Vec<u8>) -> bool> {
    let mut set = FxHashSet::default();
    let mut queue = VecDeque::new();

    Box::new(move |line| {
        if set.insert(line.clone()) {
            if set.len() > capacity {
                set.remove(&queue.pop_front().unwrap());
            }

            queue.push_back(line.clone());
            true
        } else {
            false
        }
    })
}

struct IncludeFilter {
    re: Regex,
}

impl IncludeFilter {
    fn new(regex: &str) -> Result<Self, UqError> {
        match Regex::new(regex) {
            Ok(re) => Ok(IncludeFilter { re }),
            Err(_) => Err(UqError::InvalidRegex { regex: regex.to_string() }),
        }
    }

    fn filter(&self, line: &[u8]) -> Option<Vec<u8>> {
        let mut x: Vec<u8> = Vec::new();
        if let Some(captures) = self.re.captures(line) {
            let iter = if captures.len() == 1 {
                captures.iter()
            } else {
                captures.iter().dropping(1)
            };

            for match_str in iter.filter_map(|opt_match| match opt_match {
                Some(m) => Some(m.as_bytes()),
                None => None,
            }) {
                x.extend(match_str);
            }

            Some(x)
        } else {
            None
        }
    }
}

trait LineFilter {
    fn apply(&self, line: &[u8]) -> Option<Vec<u8>>;
}

impl LineFilter for IncludeFilter {
    fn apply(&self, line: &[u8]) -> Option<Vec<u8>> {
        self.filter(line)
    }
}


fn main() -> Result<(), UqError> {
    let matches = App::new("uq (lostutils)")
        .arg(
            Arg::with_name("capacity")
                .long("capacity")
                .help("Number of unique entries to remember.")
                .value_name("capacity")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("override")
                .long("override")
                .help("Override old unique entries when capacity reached.\nWhen not used, uq will die when the capacity is exceeded.")
                .requires("capacity")
                .value_name("override")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("include")
                .long("include")
                .help("Regex capture to use for matching")
                .value_name("include")
                .takes_value(true),
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
    let filter = match matches.value_of("include") {
        Some(include) => Some(IncludeFilter::new(include)?),
        None => None,
    };

    while let Some(line) = stdin_reader.next_line() {
        let is_unique = if let Some(filter) = &filter {
            if let Some(line) = filter.apply(line) {
                unique_filter(&line)
            } else {
                false
            }
        } else {
            unique_filter(&line)
        };


        if is_unique {
            output.write_all(line).expect("Failed writing line");
        }
    }

    Ok(())
}
