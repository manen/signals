// the stack is entirely compile-time, as in it ceases to exists after the instructions are generated

// we need:
// - short-term stack
// - long-term reserved memory

// the way the memory is laid out:
// 1. output bits
// 2. reserved memory (once checked in, that bit becomes like an output bit)
// -- these two cannot shrink, only grow
// 3. the actual stack, which grows and shrinks as the program runs

// note: the stack is pretty dynamic!
// code running before a reservation is made will use the soon-to-be-checked-in memory as if it were just the stack
// (because until that point it is the stack)

// terminology (taken from restaurants n shi)
// reserve: reserve bits of memory for later use
// check-in: claim a reserved bit for use

#[derive(Clone, Debug)]
pub struct Stack {
	shared: sui::core::Store<StackShared>,
	short_term: usize,
}
impl Stack {
	#[allow(unused)]
	pub fn new(start_ptr: usize) -> Self {
		Self::with_reserved(start_ptr, 0)
	}
	pub fn with_reserved(start_ptr: usize, reserve: usize) -> Self {
		Self {
			shared: sui::core::Store::new(StackShared {
				stack_bottom: start_ptr,
				reserved: reserve,
			}),
			short_term: 0,
		}
	}

	pub fn top(&self) -> usize {
		self.shared
			.with_borrow(|shared| shared.stack_bottom + shared.reserved + self.short_term)
	}

	pub fn grow(&self, by: usize) -> Self {
		Self {
			shared: self.shared.clone(),
			short_term: self.short_term + by,
		}
	}

	#[allow(unused)]
	pub fn reserve(&self, by: usize) {
		self.shared.with_mut_borrow(|shared| shared.reserved += by)
	}
	/// claim a bit of reserved memory \
	/// returns a pointer to said bit
	pub fn check_in(&self) -> Option<usize> {
		self.shared.with_mut_borrow(|shared| {
			let reserved = shared.stack_bottom;
			if shared.reserved > 0 {
				shared.stack_bottom += 1;
				shared.reserved -= 1;

				Some(reserved)
			} else {
				None
			}
		})
	}
}

#[derive(Clone, Debug)]
pub struct StackShared {
	stack_bottom: usize,
	reserved: usize,
}
