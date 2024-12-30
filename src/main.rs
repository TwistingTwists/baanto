mod message;

use message::*;
use serde::{Deserialize, Serialize};
use std::io::{self, Read};

fn main() {
    println!("Hello, world!");
}

fn read_from_stdin() -> Result<String, io::Error> {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;
    Ok(buffer)
}

#[derive(Debug)]
pub struct Node {}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub src: String,
    pub dest: String,
    pub body: MessageBody,
}
