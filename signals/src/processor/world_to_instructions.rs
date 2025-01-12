use crate::{
	game::Game,
	world::{Block, PushMoveTo, Signal, World},
};

use super::Instruction;

/// returns none if world doesn't exist
pub fn world_to_instructions(game: &Game, world_id: Option<usize>) -> Option<Vec<Instruction>> {
	let mut vec = vec![];
	let world = game.world_opt(world_id)?;

	// let eqs = world.outputs().map(|coords, id| output_to_eq(world, ));

	// start from the outputs:
	// go back, and we need to write this shit out backwards
	// for example, for a simple and gate:

	// output = flipped(flipped(input_0) || flipped(input_1))

	let eq = Equation::not(Equation::or(
		Equation::not(Equation::Input(0)),
		Equation::not(Equation::Input(1)),
	));

	eq.to_insts(0, 0, &mut vec);
	Some(vec)
}

/// returns whether that given block in a world is on or off as an equation
pub fn world_block_to_eq(
	game: &Game,
	world_id: Option<usize>,
	coords: (i32, i32),
) -> Option<Equation> {
	fn internal(world: &World, (b_x, b_y): (i32, i32)) -> Option<Equation> {
		let b = world.at(b_x, b_y)?;

		match b {
			&Block::Wire(_) => {
				// only stopping for a commit
			}
		}
	}

	let world = game.world_opt(world_id)?;
	internal(world, coords)
}

/// Equation represents how we get a value ingame. (like outputs)
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Equation {
	Input(usize),
	Or(Box<Equation>, Box<Equation>),
	Not(Box<Equation>),
}
impl Equation {
	pub fn or(a: Equation, b: Equation) -> Self {
		Self::Or(Box::new(a), Box::new(b))
	}
	pub fn not(val: Equation) -> Self {
		Equation::Not(Box::new(val))
	}

	/// stack_top is where the empty memory starts
	pub fn to_insts(&self, out_ptr: usize, stack_top: usize, insts: &mut Vec<Instruction>) {
		match self {
			&Equation::Input(id) => insts.push(Instruction::SummonInput { id, out: out_ptr }),
			Equation::Not(n_eq) => {
				n_eq.to_insts(out_ptr, stack_top, insts);
				insts.push(Instruction::Not {
					ptr: out_ptr,
					out: out_ptr,
				})
			}
			Equation::Or(a_eq, b_eq) => {
				a_eq.to_insts(stack_top, stack_top + 2, insts);
				b_eq.to_insts(stack_top + 1, stack_top + 2, insts);

				insts.push(Instruction::Or {
					a: stack_top,
					b: stack_top + 1,
					out: out_ptr,
				})
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::super::*;
	use super::*;

	#[test]
	fn equations_and() {
		let and = Equation::not(Equation::or(
			Equation::not(Equation::Input(0)),
			Equation::not(Equation::Input(1)),
		));

		let mut insts = vec![];
		and.to_insts(0, 2, &mut insts);

		let mut mem = Memory::default();

		let mut run = |a: bool, b: bool| -> bool {
			mem.execute(&insts, &[a, b]);

			mem.get(0)
		};

		assert_eq!(run(false, false), false);
		assert_eq!(run(true, false), false);
		assert_eq!(run(false, true), false);
		assert_eq!(run(true, true), true);
	}

	#[test]
	fn equations_xor() {
		let make_and = |a: Equation, b: Equation| -> Equation {
			Equation::not(Equation::or(Equation::not(a), Equation::not(b)))
		};

		let xor = make_and(
			Equation::or(Equation::Input(0), Equation::Input(1)),
			Equation::not(make_and(Equation::Input(0), Equation::Input(1))),
		);

		let mut insts = vec![];
		xor.to_insts(0, 2, &mut insts);

		let mut mem = Memory::default();

		let mut run = |a: bool, b: bool| -> bool {
			mem.execute(&insts, &[a, b]);
			mem.get(0)
		};

		assert_eq!(run(false, false), false);
		assert_eq!(run(true, false), true);
		assert_eq!(run(false, true), true);
		assert_eq!(run(true, true), false);
	}
}
