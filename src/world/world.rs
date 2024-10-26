use crate::world::*;

use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq)]
pub struct Signal;
pub type Moves = Vec<Move>;
#[derive(Debug, PartialEq, Eq)]
pub struct Move {
	pub to: (i32, i32),
	pub from: Option<Direction>,
	pub signal: Signal,
}
impl Move {
	pub fn new(to: (i32, i32), from: Option<Direction>, signal: Signal) -> Self {
		Self { to, from, signal }
	}
}

pub struct World {
	chunks: HashMap<(i32, i32), Chunk>,
}
