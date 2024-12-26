use crate::{
	game::Game,
	world::{Block, Direction, World},
};

pub const TOOLS: &[(&str, Tool)] = &[
	("place wire", Tool::PlaceWire { start: None }),
	("place switch", Tool::Place(Block::Switch(false))),
	("place not", Tool::Place(Block::Not(false))),
	("place input", Tool::PlaceInput),
	("place output", Tool::PlaceOutput),
	("remove", Tool::Place(Block::Nothing)),
	("rotate", Tool::Rotate),
	("interact", Tool::Interact),
];
pub const FOREIGNS: &[(&str, Tool)] = &[
	("foreign to 1", Tool::PlaceForeign(1)),
	("foreign to 2", Tool::PlaceForeign(2)),
	("foreign to 3", Tool::PlaceForeign(3)),
	("foreign to 4", Tool::PlaceForeign(4)),
	("foreign to 5", Tool::PlaceForeign(5)),
	("foreign to 6", Tool::PlaceForeign(6)),
	("foreign to 7", Tool::PlaceForeign(7)),
	("foreign to 8", Tool::PlaceForeign(8)),
	("foreign to 9", Tool::PlaceForeign(9)),
];

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum Tool {
	PlaceWire {
		start: Option<(i32, i32)>,
	},
	Place(Block),
	Rotate,
	PlaceInput,
	PlaceOutput,
	PlaceForeign(usize), // world id
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
		match self {
			Self::Place(block) => {
				let ptr = game.main.as_mut().mut_at(x, y);
				*ptr = *block;
				game.main.as_mut().io_blocks_fix();
			}
			Self::PlaceInput => {
				*game.main.as_mut().mut_at(x, y) =
					Block::Input(game.main.as_mut().io_blocks_inputs_len());
				game.main.as_mut().io_blocks_fix();
				// TODO if io_blocks_inputs_len() worked properly we wouldn't need to fix io blocks
				// immediately afterwards
			}
			Self::PlaceOutput => {
				*game.main.as_mut().mut_at(x, y) =
					Block::Output(game.main.as_mut().io_blocks_outputs_len());
				game.main.as_mut().io_blocks_fix();
				// TODO if io_blocks_outputs_len() worked properly we wouldn't need to fix io blocks
				// immediately afterwards
			}
			Self::PlaceForeign(wid) => {
				// this part has to do suprisingly lot:
				// - if there exists no foreign to wid, create one and link to it. (simplest case)
				// - if there exists a foreign to wid, check if the highest id foreign to wid is the most a foreign id for that wid can be
				//   - if there can be higher, create a reference to the already existing wid, with id highest id + 1
				//   - if there can't be higher, create a new world with wid and id 1

				// some part of this logic should be abstracted into Game

				// like a game.fix_foreigns, that does all the foreign housekeeping:
				// - check if there are any IngameWorld's with 0 references to them, if there are delete them
				// - .
				// idk this part is sm harder than it seemed

				// just to test if things work:

				game.moves.children.push(crate::game::IngameWorld {
					id: Some(1),
					..Default::default()
				});
				*game.main.as_mut().mut_at(x, y) = Block::Foreign(0, 0)

				// this creates a foreign reference to 1
			}
			_ => {}
		}
	}
	pub fn pressed(&mut self, x: i32, y: i32, world: &mut World) {
		match self {
			Self::Rotate => world.map_at(x, y, |i| match i {
				Block::Wire(dir) => Block::Wire(dir.rotate()),
				_ => i,
			}),
			Self::PlaceWire { start } if *start == None => *start = Some((x, y)),
			Self::Interact => world.mut_at(x, y).interact(),
			_ => {}
		};
	}
	pub fn released(&mut self, x: i32, y: i32, world: &mut World) {
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

						*world.mut_at(x, y) = {
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
