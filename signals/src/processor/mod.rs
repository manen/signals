// essentially a computer. has some memory, runs instructions which change the memory
// implementation's pretty basic and straightforward (for now)

#[derive(Clone, Debug, PartialEq, Eq)]
// keeps the memory on the stack (not a vec or anything)
pub struct Memory {
	mem: [bool; 512],
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
	pub fn map(&mut self, i: usize, f: impl FnOnce(bool) -> bool) {
		self.mem[i] = f(self.mem[i]);
	}

	pub fn execute(&mut self, instructions: &[Instruction]) {
		for inst in instructions {
			match inst {
				&Instruction::Not { ptr, out } => {
					let val = self.get(ptr);

					self.set(out, !val)
				}
				&Instruction::Or { a, b, out } => {
					let a = self.get(a);
					let b = self.get(b);

					self.set(out, a || b)
				}
				&Instruction::And { a, b, out } => {
					let a = self.get(a);
					let b = self.get(b);

					self.set(out, a && b);
				}
			}
		}
	}
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Instruction {
	// base set
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

	// extended set
	And {
		a: usize,
		b: usize,
		out: usize,
	},
}
impl Instruction {
	pub fn extended_set_to_base_set<'a>(
		iter: impl IntoIterator<Item = &'a Instruction>,
	) -> Vec<Instruction> {
		let iter = iter.into_iter();

		let mut vec = Vec::with_capacity(iter.size_hint().0);

		for inst in iter {
			match inst {
				&Instruction::And { a, b, out } => {
					vec.push(Instruction::Not { ptr: a, out: a });
					vec.push(Instruction::Not { ptr: b, out: b });
					vec.push(Instruction::Or { a, b, out });
					vec.push(Instruction::Not { ptr: out, out });

					// if we knew more context about these instructions we could potentially skip these two
					vec.push(Instruction::Not { ptr: a, out: a });
					vec.push(Instruction::Not { ptr: b, out: b })
				}
				_ => vec.push(*inst),
			}
		}

		vec
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_processor_and() {
		let mut mem = Memory::default();
		let mut and_in_processor = |a: bool, b: bool| -> bool {
			let instructions = [
				Instruction::Not { ptr: 0, out: 0 },
				Instruction::Not { ptr: 1, out: 1 },
				Instruction::Or { a: 0, b: 1, out: 2 },
				Instruction::Not { ptr: 2, out: 2 },
			];
			// 2 = 0 && 1

			mem.set(0, a);
			mem.set(1, b);
			mem.execute(&instructions);

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
				Instruction::And { a: 0, b: 1, out: 2 },
				Instruction::Or { a: 0, b: 1, out: 3 },
				Instruction::Not { ptr: 2, out: 2 },
				Instruction::And { a: 3, b: 2, out: 4 },
			];

			mem.set(0, a);
			mem.set(1, b);
			mem.execute(&instructions);
			let extended_set_result = mem.get(4);

			let instructions = Instruction::extended_set_to_base_set(&instructions);
			mem.set(0, a);
			mem.set(1, b);
			mem.execute(&instructions);
			let base_set_result = mem.get(4);

			assert_eq!(extended_set_result, base_set_result);

			base_set_result
		};

		assert_eq!(xor_in_processor(false, false), false);
		assert_eq!(xor_in_processor(true, false), true);
		assert_eq!(xor_in_processor(false, true), true);
		assert_eq!(xor_in_processor(true, true), false);
	}
}
