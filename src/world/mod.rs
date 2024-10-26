macro_rules! module {
	($name:ident) => {
		mod $name;
		pub use $name::*;
	};
}

module!(chunk);
module!(block);
