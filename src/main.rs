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

trait UniqueSet<T> {
    fn insert(&mut self, value: T) -> bool;
}

impl UniqueSet<Vec<u8>> for FxHashSet<Vec<u8>> {
    fn insert(&mut self, value: Vec<u8>) -> bool {
        self.insert(value)
    }
}


struct UniqueWithCap {
    lines: FxHashSet<Vec<u8>>,
    cap: usize,
}

impl UniqueWithCap {
    fn new(cap: usize) -> Self {
        UniqueWithCap {
            lines: FxHashSet::default(),
            cap,
        }
    }
}

impl UniqueSet<Vec<u8>> for UniqueWithCap {
    fn insert(&mut self, value: Vec<u8>) -> bool {
        if self.lines.insert(value) {
            if self.lines.len() > self.cap {
                panic!("Cache capacity exceeded!");
            }
            true
        } else {
            false
        }
    }
}

struct UniqueWithOverride {
    set: FxHashSet<Vec<u8>>,
    queue: VecDeque<Vec<u8>>,
    cap: usize,
}

impl UniqueWithOverride {
    fn new(cap: usize) -> Self {
        UniqueWithOverride {
            set: FxHashSet::default(),
            queue: VecDeque::new(),
            cap,
        }
    }
}


impl UniqueSet<Vec<u8>> for UniqueWithOverride {
    fn insert(&mut self, value: Vec<u8>) -> bool {
        if self.set.insert(value.clone()) {
            if self.set.len() > self.cap {
                self.set.remove(&self.queue.pop_front().unwrap());
            }

            self.queue.push_back(value);
            true
        } else {
            false
        }
    }
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


struct ExcludeFilter {
    re: Regex,
}

impl ExcludeFilter {
    fn new(regex: &str) -> Result<Self, UqError> {
        match Regex::new(regex) {
            Ok(re) => Ok(ExcludeFilter { re }),
            Err(_) => Err(UqError::InvalidRegex { regex: regex.to_string() }),
        }
    }

    fn filter(&self, line: &[u8]) -> Option<Vec<u8>> {
        Some(self.re.replace_all(line, &b""[..]).to_vec())
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

impl LineFilter for ExcludeFilter {
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
        .arg(
            Arg::with_name("exclude")
                .long("exclude")
                .help("Regex capture to exclude for matching")
                .value_name("exclude")
                .conflicts_with("include")
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

    let mut unique_filter: Box<UniqueSet<Vec<u8>>> = match (capacity, matches.is_present("override")) {
        (Some(capacity), true) => Box::new(UniqueWithOverride::new(capacity)),
        (Some(capacity), false) => Box::new(UniqueWithCap::new(capacity)),
        _ => Box::new(FxHashSet::default()),
    };


    let (_in, _out) = (std::io::stdin(), std::io::stdout());
    let (input, mut output) = (_in.lock(), _out.lock());

    let mut stdin_reader = StdinReader::new(input);

    let filter: Option<Box<LineFilter>> = match (matches.value_of("include"),
                                                 matches.value_of("exclude")) {
        (Some(include), _) => Some(Box::new(IncludeFilter::new(include)?)),
        (_, Some(exclude)) => Some(Box::new(ExcludeFilter::new(exclude)?)),
        _ => None,
    };


    while let Some(line) = stdin_reader.next_line() {
        let is_unique = match &filter {
            Some(filter) =>
                match filter.apply(line) {
                    Some(line) => unique_filter.insert(line.clone()),
                    None => false,
                }
            None =>
                unique_filter.insert(line.clone()),
        };


        if is_unique {
            output.write_all(line).expect("Failed writing line");
        }
    }

    Ok(())
}
