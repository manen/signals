macro_rules! module {
	($name:ident) => {
		mod $name;
		pub use $name::*;
	};
}

module!(world);
module!(chunk);
module!(block);
