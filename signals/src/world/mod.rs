macro_rules! module {
	($name:ident) => {
		mod $name;
		pub use $name::*;
	};
}

module!(world);
module!(chunk);
module!(block);

#[macro_export]
macro_rules! continue_on_none {
	($expr:expr) => {
		match $expr {
			None => continue,
			Some(a) => a,
		}
	};
}
