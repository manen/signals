use crate::world::*;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub enum Block {
	#[default]
	Nothing,
	Wire(Direction),
	Switch(bool),
	// true if powered
	Not(bool),
}
impl Block {
	/// syntax: push_move(relative_x, relative_y, signal)
	pub fn pass(
		&self,
		signal: Signal,
		from: Option<Direction>,
		mut push_move: impl FnMut(i32, i32, Signal),
	) -> Option<Self> {
		match self {
			Self::Wire(dir) => {
				if from.map(|from| from == *dir).unwrap_or(false) {
				} else {
					let (rx, ry) = dir.rel();
					push_move(rx, ry, signal);
					return Some(Self::Wire(*dir));
				}
			}
			Self::Not(_) => return Some(Self::Not(true)),
			Self::Switch(_) => {}
			Self::Nothing => {}
		}
		None
	}
	pub fn tick(&self, mut push_move: impl FnMut(i32, i32, Signal)) -> Option<Self> {
		let mut all_directions = || {
			push_move(1, 0, Signal);
			push_move(0, 1, Signal);
			push_move(-1, 0, Signal);
			push_move(0, -1, Signal);
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

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Direction {
	Right,
	Bottom,
	Left,
	Top,
}
impl Direction {
	/// clockwise
	pub fn rotate(self) -> Self {
		match self {
			Direction::Right => Direction::Bottom,
			Direction::Bottom => Direction::Left,
			Direction::Left => Direction::Top,
			Direction::Top => Direction::Right,
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
}
