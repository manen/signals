use crate::{consts, world::*};
use raylib::{drawing::RaylibDrawHandle, prelude::RaylibDraw};

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub enum Block {
	#[default]
	Nothing,
	Wire(Direction, u8),
	Switch(bool),
}
impl Block {
	/// syntax: push_move(relative_x, relative_y, signal)
	pub fn pass(
		&self,
		signal: Signal,
		mut push_move: impl FnMut(i32, i32, Signal),
	) -> Option<Self> {
		match self {
			Self::Wire(Direction::Right, _) => push_move(1, 0, signal),
			Self::Wire(Direction::Bottom, _) => push_move(0, 1, signal),
			Self::Wire(Direction::Left, _) => push_move(-1, 0, signal),
			Self::Wire(Direction::Top, _) => push_move(0, -1, signal),
			Self::Switch(_) => {}
			Self::Nothing => {}
		}
		if let Self::Wire(dir, _) = self {
			Some(Self::Wire(*dir, 0))
		} else {
			None
		}
	}
	pub fn tick(&self, mut push_move: impl FnMut(i32, i32, Signal)) -> Option<Self> {
		match self {
			Self::Switch(true) => {
				push_move(1, 0, Signal);
				push_move(0, 1, Signal);
				push_move(-1, 0, Signal);
				push_move(0, -1, Signal);
				None
			}
			Self::Wire(dir, ticks) => {
				Some(Self::Wire(*dir, if *ticks > 200 { 100 } else { ticks + 1 }))
			}
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

				// use raylib::color::Color;
				// let c_dir = match dir {
				// 	Direction::Right => "r",
				// 	Direction::Bottom => "b",
				// 	Direction::Left => "l",
				// 	Direction::Top => "t",
				// };
				// let c = format!("{c_dir} {ticks}");
				// d.draw_text(&c, base_x + x_off, base_y + y_off, 8, Color::WHITE);
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
}
