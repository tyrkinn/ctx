use anyhow::{Result, anyhow};
use std::{
    fs::File,
    io::{BufReader, Read},
};

use crate::context::Context;

mod cli;
mod commands;
mod context;

fn main() {
    let f = File::open("./cfg_sample.kdl").unwrap();
    let mut s = String::new();
    BufReader::new(f).read_to_string(&mut s).unwrap();
    let cfg = Context::try_from(s.as_str()).unwrap();
}
