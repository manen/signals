use crate::world::{Block, Chunk, Direction};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Tool {
	Place(Block),
	Rotate,
	PlaceWire { start: Option<(usize, usize)> },
}
impl Tool {
	pub fn rotate(self) -> Self {
		match self {
			Self::Place(Block::Nothing) => Self::Rotate,
			Self::Rotate => Self::PlaceWire { start: None },
			Self::PlaceWire { .. } => Self::Place(Block::Nothing),
			_ => Self::PlaceWire { start: None },
		}
	}

	pub fn down(&mut self, x: usize, y: usize, chunk: &mut Chunk) {
		match self {
			Self::Place(block) => chunk.map_at(x, y, |_| *block),
			_ => {}
		}
	}
	pub fn pressed(&mut self, x: usize, y: usize, chunk: &mut Chunk) {
		match self {
			Self::Rotate => chunk.map_at(x, y, |i| match i {
				Block::Wire(dir, s) => Block::Wire(dir.rotate(), s),
				_ => i,
			}),
			Self::PlaceWire { start } if *start == None => *start = Some((x, y)),
			_ => {}
		}
	}
	pub fn released(&mut self, x: usize, y: usize, chunk: &mut Chunk) {
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

						chunk.map_at(x, y, |_| {
							if horizontal {
								Block::Wire(
									if !reverse {
										Direction::Right
									} else {
										Direction::Left
									},
									false,
								)
							} else {
								Block::Wire(
									if !reverse {
										Direction::Bottom
									} else {
										Direction::Top
									},
									false,
								)
							}
						})
					}
				};
				*start = None;
			}
			_ => {}
		}
	}
}
