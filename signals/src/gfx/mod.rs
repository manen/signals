use std::borrow::Cow;

use raylib::prelude::{RaylibDraw, RaylibDrawHandle};

use crate::world::{self, Chunk, World};
use raylib::color::Color;

pub const fn color(r: u8, g: u8, b: u8, a: u8) -> Color {
	Color { r, g, b, a }
}
pub const BACKGROUND: Color = Color::BLACK;
pub const WIRE_ON: Color = color(230, 200, 200, 255);
pub const WIRE_OFF: Color = color(80, 80, 80, 255);
pub const SWITCH_ON: Color = color(200, 200, 200, 255);
pub const SWITCH_OFF: Color = color(100, 100, 100, 255);
pub const NOT_BASE: Color = color(39, 143, 86, 255);
pub const NOT_ON: Color = color(82, 81, 80, 255);
pub const NOT_OFF: Color = color(255, 255, 255, 255);
pub const REST_ON: Color = color(150, 150, 150, 255);

// pub const WIRE_ON: Color = color(207, 109, 173, 255);
// pub const WIRE_OFF: Color = color(105, 38, 81, 255);
// pub const SWITCH_ON: Color = color(255, 161, 236, 255);
// pub const SWITCH_OFF: Color = color(232, 90, 203, 255);
// pub const NOT_BASE: Color = color(212, 8, 103, 255);
// pub const NOT_ON: Color = color(237, 109, 126, 255);
// pub const NOT_OFF: Color = color(255, 74, 116, 255);
// pub const REST_ON: Color = color(224, 4, 147, 255);

pub const DEBUG_WIRES: bool = false;
pub const DEBUG_CHUNKS: bool = false;
pub const DEBUG_NOT: bool = false;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Default)]
/// this is the enum that is used to determine what type of a block should be rendered at a position
pub enum DrawType {
	#[default]
	Off,
	On,
	/// junction stores values for both axis, they should be reset before every render,
	/// and entering false won't actually set it to false if it's already true from that same tick
	Junction {
		vertical: bool,
		horizontal: bool,
	},
}
impl From<bool> for DrawType {
	fn from(value: bool) -> Self {
		if value {
			Self::On
		} else {
			Self::Off
		}
	}
}
impl DrawType {
	pub fn apply_new(self, new: Self) -> Self {
		match new {
			DrawType::Junction {
				vertical: new_vert,
				horizontal: new_horizontal,
			} => match self {
				DrawType::Junction {
					vertical: prev_vert,
					horizontal: prev_horizontal,
				} => DrawType::Junction {
					vertical: prev_vert || new_vert,
					horizontal: prev_horizontal || new_horizontal,
				},
				_ => new,
			},
			_ => new,
		}
	}
}
pub type Drawmap = world::Chunk<DrawType>;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PosInfo {
	pub base: (i32, i32),
	pub scale: f32,
}
impl Default for PosInfo {
	fn default() -> Self {
		PosInfo {
			base: (0, 0),
			scale: 1.0,
		}
	}
}
impl PosInfo {
	pub fn transform(&self, x: i32, y: i32) -> Self {
		self.add(self.scale(x), self.scale(y))
	}
	pub fn add(&self, x: i32, y: i32) -> Self {
		PosInfo {
			base: (self.base.0 + x, self.base.1 + y),
			scale: self.scale,
		}
	}
	pub fn scale(&self, n: i32) -> i32 {
		(self.scale * n as f32) as i32
	}
}

pub fn render_basic_world(world: &world::World, d: &mut RaylibDrawHandle, pos_info: PosInfo) {
	render_any_world(
		world,
		d,
		pos_info,
		|_| Cow::Owned(Default::default()),
		false,
	);
}
/// ticks and renders the given world
pub fn render_world(
	world: &world::World,
	d: &mut RaylibDrawHandle,
	pos_info: PosInfo,
	drawmap: &World<DrawType>,
) {
	render_any_world(
		world,
		d,
		pos_info,
		|coords| {
			drawmap
				.chunk(coords)
				.map(|a| Cow::Borrowed(a))
				.unwrap_or_default()
		},
		true,
	);
}
fn render_any_world<'a>(
	world: &world::World,
	d: &mut RaylibDrawHandle,
	pos_info: PosInfo,
	drawmap_at: impl Fn((i32, i32)) -> Cow<'a, Chunk<DrawType>>,
	draw_misc: bool,
) {
	for (coords, chunk) in world.chunks() {
		render_chunk(
			&chunk,
			drawmap_at(*coords).as_ref(),
			d,
			pos_info.transform(
				coords.0 * world::CHUNK_SIZE as i32 * world::BLOCK_SIZE as i32,
				coords.1 * world::CHUNK_SIZE as i32 * world::BLOCK_SIZE as i32,
			),
			draw_misc,
		);
	}
}

pub fn render_chunk(
	chunk: &world::Chunk,
	drawmap: &Drawmap,
	d: &mut RaylibDrawHandle,
	pos_info: PosInfo,
	draw_misc: bool,
) {
	for px in 0..world::CHUNK_SIZE as i32 {
		for py in 0..world::CHUNK_SIZE as i32 {
			let pos_info = pos_info.transform(px * world::BLOCK_SIZE, py * world::BLOCK_SIZE);
			let (base_x, base_y) = pos_info.base;
			chunk.at(px, py).map(|b| {
				render_block(
					b,
					&drawmap
						.at(px, py)
						.expect("drawmap chunks are smaller than regular chunks(how)"),
					d,
					pos_info,
					draw_misc,
				)
			});

			if DEBUG_CHUNKS && draw_misc {
				use raylib::prelude::RaylibDraw;
				d.draw_text(
					&format!("{px} {py}"),
					base_x,
					base_y,
					(6.0 * pos_info.scale) as i32,
					raylib::color::Color::WHITE,
				);
			}
		}
	}
}

pub fn render_block(
	block: &world::Block,
	dt: &DrawType,
	d: &mut RaylibDrawHandle,
	pos_info: PosInfo,
	draw_misc: bool,
) {
	match block {
		world::Block::Nothing => {}
		world::Block::Wire(dir) => {
			let horizontal = match dir {
				world::Direction::Bottom | world::Direction::Top => false,
				_ => true,
			};
			let off = world::BLOCK_SIZE / 4;
			let x_off = if !horizontal { off } else { 0 };
			let y_off = if horizontal { off } else { 0 };

			let color = if let DrawType::On = dt {
				WIRE_ON
			} else {
				WIRE_OFF
			};

			let pos_info = pos_info.transform(x_off, y_off);
			d.draw_rectangle(
				pos_info.base.0,
				pos_info.base.1,
				pos_info.scale(world::BLOCK_SIZE - x_off * 2),
				pos_info.scale(world::BLOCK_SIZE - y_off * 2),
				color,
			);

			if DEBUG_WIRES && draw_misc {
				use raylib::color::Color;
				let c_dir = match dir {
					world::Direction::Right => "r",
					world::Direction::Bottom => "b",
					world::Direction::Left => "l",
					world::Direction::Top => "t",
				};
				let c = format!("{c_dir}");
				d.draw_text(
					&c,
					pos_info.base.0,
					pos_info.base.1,
					pos_info.scale(8),
					Color::WHITE,
				);
			}
		}
		world::Block::Switch(state) => {
			d.draw_rectangle(
				pos_info.base.0,
				pos_info.base.1,
				pos_info.scale(world::BLOCK_SIZE),
				pos_info.scale(world::BLOCK_SIZE),
				if *state { SWITCH_ON } else { SWITCH_OFF },
			);
		}
		world::Block::Not(_) => {
			d.draw_rectangle(
				pos_info.base.0,
				pos_info.base.1,
				pos_info.scale(world::BLOCK_SIZE),
				pos_info.scale(world::BLOCK_SIZE),
				NOT_BASE,
			);

			let excl_color = if let DrawType::On = dt {
				NOT_ON
			} else {
				NOT_OFF
			};
			let excl_width = 6;
			let excl_height = 24;
			let excl_point = 4;

			let excl_start_x =
				pos_info.base.0 + pos_info.scale(world::BLOCK_SIZE / 2 - excl_width / 2);
			let excl_start_y =
				pos_info.base.1 + pos_info.scale(world::BLOCK_SIZE - excl_height / 2);

			d.draw_rectangle(
				excl_start_x,
				excl_start_y,
				pos_info.scale(excl_width),
				pos_info.scale(excl_height - excl_point * 2),
				excl_color,
			);
			d.draw_rectangle(
				excl_start_x,
				excl_start_y + excl_height - excl_point,
				pos_info.scale(excl_width),
				pos_info.scale(excl_point),
				excl_color,
			);

			if DEBUG_NOT && draw_misc {
				d.draw_text(
					&format!("{block:?}"),
					pos_info.base.0,
					pos_info.base.0,
					pos_info.scale(6),
					excl_color,
				);
			}
		}
		world::Block::Junction => {
			let (vert_dt, horiz_dt) = match dt {
				DrawType::Junction {
					vertical,
					horizontal,
				} => (*vertical, *horizontal),
				_ => (false, false),
			};

			render_block(
				&world::Block::Wire(world::Direction::Bottom),
				&vert_dt.into(),
				d,
				pos_info,
				draw_misc,
			);
			render_block(
				&world::Block::Wire(world::Direction::Right),
				&horiz_dt.into(),
				d,
				pos_info,
				draw_misc,
			);
		}
		world::Block::Error(typ) => {
			d.draw_rectangle(
				pos_info.base.0,
				pos_info.base.1,
				pos_info.scale(world::BLOCK_SIZE),
				pos_info.scale(world::BLOCK_SIZE),
				SWITCH_OFF,
			);
			if draw_misc {
				d.draw_text(
					&format!("{typ}"),
					pos_info.base.0,
					pos_info.base.1,
					12,
					SWITCH_ON,
				)
			}
		}
		world::Block::Foreign(wid, inst_id, id) => {
			let color = if *dt == DrawType::On {
				REST_ON
			} else {
				SWITCH_OFF
			};
			d.draw_rectangle(
				pos_info.base.0,
				pos_info.base.1,
				pos_info.scale(world::BLOCK_SIZE),
				pos_info.scale(world::BLOCK_SIZE),
				color,
			);
			if draw_misc {
				d.draw_text(
					&format!("{}\n{inst_id}|{id}", wid.short()),
					pos_info.base.0,
					pos_info.base.1,
					12,
					SWITCH_ON,
				)
			}
		}
		rest => {
			let color = if *dt == DrawType::On {
				REST_ON
			} else {
				SWITCH_OFF
			};
			d.draw_rectangle(
				pos_info.base.0,
				pos_info.base.1,
				pos_info.scale(world::BLOCK_SIZE),
				pos_info.scale(world::BLOCK_SIZE),
				color,
			);
			if draw_misc {
				d.draw_text(
					&format!("{rest:?}"),
					pos_info.base.0,
					pos_info.base.1,
					12,
					SWITCH_ON,
				)
			}
		}
	}
}
