use crate::world::{Block, Chunk, Direction};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Tool {
	PlaceWire { start: Option<(i32, i32)> },
	Place(Block),
	Rotate,
	Interact,
}
impl Default for Tool {
	fn default() -> Self {
		Self::PlaceWire { start: None }
	}
}
impl Tool {
	pub fn rotate(self) -> Self {
		match self {
			Self::PlaceWire { .. } => Self::Place(Block::Nothing),
			Self::Place(Block::Nothing) => Self::Place(Block::Switch(false)),
			Self::Place(Block::Switch(_)) => Self::Place(Block::Not(false)),
			Self::Place(Block::Not(_)) => Self::Rotate,
			Self::Rotate => Self::Interact,
			Self::Interact => Self::PlaceWire { start: None },
			_ => Self::PlaceWire { start: None },
		}
	}

	pub fn down(&mut self, x: i32, y: i32, chunk: &mut Chunk) -> Option<()> {
		match self {
			Self::Place(block) => *chunk.mut_at(x, y) = *block,
			_ => {}
		}
		Some(())
	}
	pub fn pressed(&mut self, x: i32, y: i32, chunk: &mut Chunk) -> Option<()> {
		match self {
			Self::Rotate => chunk.map_at(x, y, |i| match i {
				Block::Wire(dir, s) => Block::Wire(dir.rotate(), s),
				_ => i,
			}),
			Self::PlaceWire { start } if *start == None => *start = Some((x, y)),
			Self::Interact => chunk.mut_at(x, y)?.interact(),
			_ => {}
		};
		Some(())
	}
	pub fn released(&mut self, x: i32, y: i32, chunk: &mut Chunk) -> Option<()> {
		match self {
			Self::PlaceWire { start } => {
				if let Some(start) = start {
					let x_diff = x as isize - start.0 as isize;
					let y_diff = y as isize - start.1 as isize;

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

						*chunk.mut_at(x, y) = {
							if horizontal {
								Block::Wire(
									if !reverse {
										Direction::Right
									} else {
										Direction::Left
									},
									100,
								)
							} else {
								Block::Wire(
									if !reverse {
										Direction::Bottom
									} else {
										Direction::Top
									},
									100,
								)
							}
						};
					}
				};
				*start = None;
			}
			_ => {}
		}
		Some(())
	}
}
