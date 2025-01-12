use crate::game::Game;

use super::Instruction;

pub fn world_to_instructions(game: &Game, world_id: Option<usize>) -> Vec<Instruction> {
	let vec = vec![Instruction::And { a: 1, b: 2, out: 3 }];

	vec
}
