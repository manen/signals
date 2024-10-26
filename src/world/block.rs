use crate::world::*;

use crate::consts;
use raylib::{drawing::RaylibDrawHandle, prelude::RaylibDraw};

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub enum Block {
	#[default]
	Nothing,
	Wire(Direction, u8),
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
			Self::Wire(dir, _) => {
				if from.map(|from| from == *dir).unwrap_or(false) {
				} else {
					let (rx, ry) = dir.rel();
					push_move(rx, ry, signal);
					return Some(Self::Wire(*dir, 0));
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
			Self::Wire(dir, ticks) => {
				Some(Self::Wire(*dir, if *ticks > 200 { 100 } else { ticks + 1 }))
			}
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

	pub fn draw_at(&self, d: &mut RaylibDrawHandle, base_x: i32, base_y: i32) {
		match self {
			Block::Nothing => {}
			Block::Wire(dir, ticks) => {
				let horizontal = match dir {
					Direction::Bottom | Direction::Top => false,
					_ => true,
				};
				let off = BLOCK_SIZE / 4;
				let x_off = if !horizontal { off } else { 0 };
				let y_off = if horizontal { off } else { 0 };

				let color = if *ticks < 3 {
					consts::WIRE_ON
				} else {
					consts::WIRE_OFF
				};

				d.draw_rectangle(
					base_x + x_off,
					base_y + y_off,
					BLOCK_SIZE - x_off * 2,
					BLOCK_SIZE - y_off * 2,
					color,
				);

				use raylib::color::Color;
				let c_dir = match dir {
					Direction::Right => "r",
					Direction::Bottom => "b",
					Direction::Left => "l",
					Direction::Top => "t",
				};
				let c = format!("{c_dir} {ticks}");
				d.draw_text(&c, base_x + x_off, base_y + y_off, 8, Color::WHITE);
			}
			Block::Switch(state) => {
				d.draw_rectangle(
					base_x,
					base_y,
					BLOCK_SIZE,
					BLOCK_SIZE,
					if *state {
						consts::SWITCH_ON
					} else {
						consts::SWITCH_OFF
					},
				);
			}
			Block::Not(state) => {
				d.draw_rectangle(base_x, base_y, BLOCK_SIZE, BLOCK_SIZE, consts::NOT_BASE);

				let excl_color = if *state {
					consts::NOT_ON
				} else {
					consts::NOT_OFF
				};
				let excl_width = 6;
				let excl_height = 24;
				let excl_point = 4;

				let excl_start_x = base_x + BLOCK_SIZE / 2 - excl_width / 2;
				let excl_start_y = base_y + (BLOCK_SIZE - excl_height) / 2;

				d.draw_rectangle(
					excl_start_x,
					excl_start_y,
					excl_width,
					excl_height - excl_point * 2,
					excl_color,
				);
				d.draw_rectangle(
					excl_start_x,
					excl_start_y + excl_height - excl_point,
					excl_width,
					excl_point,
					excl_color,
				);

				d.draw_text(&format!("{self:?}"), base_x, base_y, 6, excl_color);
			}
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
