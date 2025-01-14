use crate::world::*;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Default)]
pub enum Block {
	#[default]
	Nothing,
	Wire(Direction),
	Router, // for now, two routers next to each are gonna power each other indefinitely
	Junction,
	Switch(bool), // true if powered
	Not(bool),
	Input(usize),
	Output(usize),
	Foreign(Option<usize>, usize, usize), // (world_id (for redundancy), inst_id, input_and_output_id)
	Error(&'static str),                  // error contains an error.
}
impl Block {
	/// syntax: push_move(relative_x, relative_y, signal)
	pub fn pass(
		&self,
		signal: Signal,
		from: Option<Direction>,
		mut push_move: impl FnMut(PushMoveTo, Signal),
	) -> Option<Self> {
		let mut all_directions = |signal: Signal| {
			push_move(PushMoveTo::Rel(1, 0), signal.clone());
			push_move(PushMoveTo::Rel(0, 1), signal.clone());
			push_move(PushMoveTo::Rel(-1, 0), signal.clone());
			push_move(PushMoveTo::Rel(0, -1), signal);
		};
		match self {
			Self::Wire(dir) => {
				// only accept the signal if it's coming from any other direction than the wire is pointing to
				if from.map(|from| from == *dir).unwrap_or(false) {
				} else {
					let (rx, ry) = dir.rel();
					push_move(PushMoveTo::Rel(rx, ry), signal);
				}
			}
			Self::Router => {
				all_directions(Default::default());
			}
			Self::Junction => {
				if let Some(dir) = from {
					let (rel_x, rel_y) = dir.reverse().rel();
					push_move(PushMoveTo::Rel(rel_x, rel_y), signal)
				}
			}
			Self::Not(_) => return Some(Self::Not(true)),
			Self::Switch(_) => {}
			Self::Input(_) => {
				match signal {
					Signal::ExternalPoweron => {
						all_directions(Signal::Default);
					}
					_ => (), // do nothing unless externally powered on
				}
			}
			Self::Output(id) => push_move(PushMoveTo::OutputID(*id), signal),
			Self::Foreign(_, inst_id, id) => match signal {
				Signal::Default => {
					push_move(
						PushMoveTo::Foreign {
							inst_id: *inst_id,
							id: *id,
						},
						signal,
					);
				}
				Signal::DefaultIf(f) => {
					if f(*self) {
						push_move(
							PushMoveTo::Foreign {
								inst_id: *inst_id,
								id: *id,
							},
							signal,
						);
					}
				}
				Signal::ExternalPoweron => {
					fn cause(block: Block) -> bool {
						match block {
							Block::Foreign(_, _, _) => false,
							_ => true,
						}
					}
					all_directions(Signal::DefaultIf(cause));
				}
			},
			Self::Nothing | Block::Error(_) => {}
		}
		None
	}
	pub fn tick(&self, mut push_move: impl FnMut(PushMoveTo, Signal)) -> Option<Self> {
		let mut all_directions = || {
			push_move(PushMoveTo::Rel(1, 0), Default::default());
			push_move(PushMoveTo::Rel(0, 1), Default::default());
			push_move(PushMoveTo::Rel(-1, 0), Default::default());
			push_move(PushMoveTo::Rel(0, -1), Default::default());
			None
		};
		match self {
			Self::Switch(true) => all_directions(),
			Self::Not(true) => Some(Self::Not(false)),
			Self::Not(false) => all_directions(),
			_ => None,
		}
	}

	pub fn interact(&mut self) {
		match self {
			Self::Switch(s) => *s = !*s,
			_ => {}
		}
	}
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Direction {
	Right,
	Bottom,
	Left,
	Top,
}
impl Direction {
	/// clockwise
	pub fn rotate_r(self) -> Self {
		match self {
			Direction::Right => Direction::Bottom,
			Direction::Bottom => Direction::Left,
			Direction::Left => Direction::Top,
			Direction::Top => Direction::Right,
		}
	}
	pub fn rotate_l(self) -> Self {
		match self {
			Direction::Right => Direction::Top,
			Direction::Bottom => Direction::Right,
			Direction::Left => Direction::Bottom,
			Direction::Top => Direction::Left,
		}
	}
	pub fn reverse(self) -> Self {
		match self {
			Direction::Right => Direction::Left,
			Direction::Bottom => Direction::Top,
			Direction::Left => Direction::Right,
			Direction::Top => Direction::Bottom,
		}
	}

	pub fn rel(self) -> (i32, i32) {
		match self {
			Self::Right => (1, 0),
			Self::Bottom => (0, 1),
			Self::Left => (-1, 0),
			Self::Top => (0, -1),
		}
	}
	pub fn from_rel(rel: (i32, i32)) -> Option<Self> {
		match rel {
			(1, 0) => Some(Self::Right),
			(0, 1) => Some(Self::Bottom),
			(-1, 0) => Some(Self::Left),
			(0, -1) => Some(Self::Top),
			_ => None,
		}
	}

	pub fn all() -> impl Iterator<Item = Self> {
		[Self::Right, Self::Bottom, Self::Left, Self::Top].into_iter()
	}
}
