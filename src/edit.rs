use crate::world::{Block, Chunk, Direction};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Tool {
	Place(Block),
	Rotate,
}
impl Tool {
	pub fn rotate(self) -> Self {
		match self {
			Self::Place(Block::Nothing) => Self::Place(Block::Wire(Direction::Right, false)),
			Self::Place(Block::Wire(Direction::Right, s)) => {
				Self::Place(Block::Wire(Direction::Bottom, s))
			}
			Self::Place(Block::Wire(Direction::Bottom, s)) => {
				Self::Place(Block::Wire(Direction::Left, s))
			}
			Self::Place(Block::Wire(Direction::Left, s)) => {
				Self::Place(Block::Wire(Direction::Top, s))
			}
			Self::Place(Block::Wire(Direction::Top, s)) => Self::Rotate,
			Self::Rotate => Self::Place(Block::Nothing),
		}
	}

	pub fn down(&self, x: usize, y: usize, chunk: &mut Chunk) {
		match self {
			Self::Place(block) => chunk.map_at(x, y, |_| *block),
			_ => {}
		}
	}
	pub fn pressed(&self, x: usize, y: usize, chunk: &mut Chunk) {
		match self {
			Self::Rotate => chunk.map_at(x, y, |i| match i {
				Block::Wire(dir, s) => Block::Wire(dir.rotate(), s),
				_ => i,
			}),
			_ => {}
		}
	}
}
