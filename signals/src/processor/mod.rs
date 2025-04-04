pub mod world_to_instructions;
use std::ops::{Index, Range};

pub use world_to_instructions::world_to_instructions;

pub mod eq;
pub mod program;
pub mod stack;

// essentially a computer. has some memory, runs instructions which change the memory
// implementation's pretty basic and straightforward (for now)

#[derive(Clone, Debug, PartialEq, Eq)]
// keeps the memory on the stack (not a vec or anything)
pub struct Memory {
	mem: [bool; 512],
}
impl Index<Range<usize>> for Memory {
	type Output = [bool];
	fn index(&self, index: Range<usize>) -> &Self::Output {
		Index::index(&self.mem, index)
	}
}
impl Default for Memory {
	fn default() -> Self {
		let mem = std::mem::MaybeUninit::zeroed();
		let mem = unsafe { mem.assume_init() };
		Self { mem }
	}
}
impl Memory {
	pub fn get(&mut self, i: usize) -> bool {
		self.mem[i]
	}
	pub fn set(&mut self, i: usize, v: bool) {
		self.mem[i] = v;
	}

	pub fn execute(&mut self, instructions: &[Instruction], inputs: &[bool]) {
		for inst in instructions {
			match inst {
				&Instruction::SummonInput { id, out } => {
					if let Some(val) = inputs.iter().nth(id).copied() {
						self.set(out, val)
					} else {
						eprintln!("program tried to access an input that doesn't exist");
						self.set(out, false)
					}
				}

				&Instruction::Not { ptr, out } => {
					let val = self.get(ptr);

					self.set(out, !val)
				}
				&Instruction::Or { a, b, out } => {
					let a = self.get(a);
					let b = self.get(b);

					self.set(out, a || b)
				}
				&Instruction::Set { ptr, val } => self.set(ptr, val),
				&Instruction::Copy { src_ptr, dst_ptr } => {
					let val = self.get(src_ptr);
					self.set(dst_ptr, val)
				}

				&Instruction::And { a, b, out } => {
					let a = self.get(a);
					let b = self.get(b);

					self.set(out, a && b);
				}
				&Instruction::Xor { a, b, out } => {
					let a = self.get(a);
					let b = self.get(b);

					self.set(out, a ^ b);
				}
			}
		}
	}
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Instruction {
	// base set
	SummonInput {
		id: usize,
		out: usize,
	},

	Or {
		a: usize,
		b: usize,
		out: usize,
	},
	/// flip the value in place
	Not {
		ptr: usize,
		out: usize,
	},
	Set {
		ptr: usize,
		val: bool,
	},
	Copy {
		src_ptr: usize,
		dst_ptr: usize,
	},

	// extended set
	And {
		a: usize,
		b: usize,
		out: usize,
	},
	Xor {
		a: usize,
		b: usize,
		out: usize,
	},
}
#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_processor_and() {
		let mut mem = Memory::default();
		let mut and_in_processor = |a: bool, b: bool| -> bool {
			let instructions = [
				Instruction::SummonInput { id: 0, out: 0 },
				Instruction::SummonInput { id: 1, out: 1 },
				Instruction::Not { ptr: 0, out: 0 },
				Instruction::Not { ptr: 1, out: 1 },
				Instruction::Or { a: 0, b: 1, out: 2 },
				Instruction::Not { ptr: 2, out: 2 },
			];
			// 2 = 0 && 1

			mem.execute(&instructions, &[a, b]);

			mem.get(2)
		};

		assert_eq!(and_in_processor(false, false), false);
		assert_eq!(and_in_processor(true, false), false);
		assert_eq!(and_in_processor(false, true), false);
		assert_eq!(and_in_processor(true, true), true);
	}

	#[test]
	fn test_processor_xor() {
		let mut mem = Memory::default();
		let mut xor_in_processor = |a: bool, b: bool| -> bool {
			// make this and also make a couple of other logic gates in instructions by hand just to get a sense for the instructionset we'll need

			// memory:
			// 0,1: inputs
			// 2: 0 && 1
			// 3: 0 || 1
			// 4: output, 3 && !2

			let instructions = [
				Instruction::SummonInput { id: 0, out: 0 },
				Instruction::SummonInput { id: 1, out: 1 },
				Instruction::And { a: 0, b: 1, out: 2 },
				Instruction::Or { a: 0, b: 1, out: 3 },
				Instruction::Not { ptr: 2, out: 2 },
				Instruction::And { a: 3, b: 2, out: 4 },
			];

			mem.execute(&instructions, &[a, b]);
			let extended_set_result = mem.get(4);

			extended_set_result
		};

		assert_eq!(xor_in_processor(false, false), false);
		assert_eq!(xor_in_processor(true, false), true);
		assert_eq!(xor_in_processor(false, true), true);
		assert_eq!(xor_in_processor(true, true), false);
	}
}
