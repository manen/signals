use crate::{
	game::{Game, IngameWorld, WorldId},
	world::{Block, Direction},
};

pub const TOOLS: &[(&str, Tool)] = &[
	("place wire", Tool::PlaceWire { start: None }),
	("place switch", Tool::Place(Block::Switch(false))),
	("place junction", Tool::Place(Block::Junction)),
	("place router", Tool::Place(Block::Router)),
	("place not", Tool::Place(Block::Not(false))),
	("place input", Tool::PlaceInput),
	("place output", Tool::PlaceOutput),
	("remove", Tool::Place(Block::Nothing)),
	("rotate", Tool::Rotate),
	("interact", Tool::Interact),
];

macro_rules! main_or_return {
	($game:expr) => {{
		match $game.worlds.at($game.main_id) {
			Some(a) => a,
			None => return,
		}
	}};
	(mut $game:expr) => {{
		match $game.worlds.at_mut($game.main_id) {
			Some(a) => a,
			None => return,
		}
	}};
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum Tool {
	PlaceWire {
		start: Option<(i32, i32)>,
	},
	Place(Block),
	Rotate,
	PlaceInput,
	PlaceOutput,
	PlaceForeign(WorldId), // world id
	#[default]
	Interact,
}
impl Tool {
	// pub fn rotate(self) -> Self {
	// 	match self {
	// 		Self::PlaceWire { .. } => Self::Place(Block::Nothing),
	// 		Self::Place(Block::Nothing) => Self::Place(Block::Switch(false)),
	// 		Self::Place(Block::Switch(_)) => Self::Place(Block::Not(false)),
	// 		Self::Place(Block::Not(_)) => Self::Rotate,
	// 		Self::Rotate => Self::Interact,
	// 		Self::Interact => Self::PlaceWire { start: None },
	// 		_ => Self::PlaceWire { start: None },
	// 	}
	// }

	pub fn down(&mut self, x: i32, y: i32, game: &mut Game) {
		let main = main_or_return!(mut game);
		match self {
			Self::Place(block) => {
				let ptr = main.mut_at(x, y);
				*ptr = *block;
				main.io_blocks_fix();
			}
			_ => {}
		}
	}
	pub fn pressed(&mut self, x: i32, y: i32, game: &mut Game) {
		let main = main_or_return!(mut game);
		match self {
			Self::Rotate => main.map_at(x, y, |i| match i {
				Block::Wire(dir) => Block::Wire(dir.rotate_r()),
				_ => i,
			}),
			Self::PlaceWire { start } if *start == None => *start = Some((x, y)),
			Self::Interact => main.mut_at(x, y).interact(),
			Self::PlaceInput => {
				*main.mut_at(x, y) = Block::Input(main.inputs_count());
				main.io_blocks_fix();
				// TODO if io_blocks_inputs_len() worked properly we wouldn't need to fix io blocks
				// immediately afterwards
			}
			Self::PlaceOutput => {
				*main.mut_at(x, y) = Block::Output(main.outputs_count());
				main.io_blocks_fix();
				// TODO if io_blocks_outputs_len() worked properly we wouldn't need to fix io blocks
				// immediately afterwards
			}
			Self::PlaceForeign(wid) => {
				// i want to make it so that a one block gap is okay
				// so the area we need to scan:
				//     x
				//   x x x
				// x x   x x
				//   x x x
				//     x
				// i'm gonna do all this with iterators

				let main = match game.worlds.at_mut(game.main_id) {
					Some(a) => a,
					None => return,
				};

				let make_line = |dir, rng: std::ops::Range<i32>| rng.map(move |a| (dir, a));
				let make_line = |dir| make_line(dir, 1..3);

				let lines = Direction::all().map(make_line).flatten();
				let lines = lines.map(|(dir, mul)| dir.rel_mul(mul));
				let to_scan = lines.chain([(-1, -1), (-1, 1), (1, -1), (1, 1)]);

				let to_scan = to_scan.map(|(rx, ry)| (x + rx, y + ry));
				for (x, y) in to_scan {
					*main.mut_at(x, y) = Block::Switch(true);
				}

				// we have the pattern of what counts as attached to a clump, but we still need a clump detection algo cause imagine this is ingame:

				// foreign inst_id id
				//         0       3
				//         0       2
				//         0       1
				//         0       0
				//
				//                    <- and here comes the new foreign! if we don't do proper clump detection, this would gladly place
				//                       down an inst_id 0 id 1 cause it doesn't know they're already one clump
			}
			_ => {}
		};
	}
	pub fn released(&mut self, x: i32, y: i32, game: &mut Game) {
		let main = main_or_return!(mut game);
		match self {
			Self::PlaceWire { start } => {
				if let Some(start) = start {
					let x_diff = x - start.0;
					let y_diff = y - start.1;

					let (horizontal, oldfrom, oldto) = if x_diff.abs() >= y_diff.abs() {
						(true, start.0, x)
					} else {
						(false, start.1, y)
					};

					let reverse = oldfrom > oldto;
					let from = oldfrom.min(oldto);
					let to = oldfrom.max(oldto);

					for i in from..to + 1 {
						let x = if horizontal { i } else { start.0 };
						let y = if horizontal { start.1 } else { i };

						*main.mut_at(x, y) = {
							if horizontal {
								Block::Wire(if !reverse {
									Direction::Right
								} else {
									Direction::Left
								})
							} else {
								Block::Wire(if !reverse {
									Direction::Bottom
								} else {
									Direction::Top
								})
							}
						};
					}
				};
				*start = None;
			}
			_ => {}
		}
	}
}
