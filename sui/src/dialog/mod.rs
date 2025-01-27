use crate::{Comp, Details, RootContext};

const DEFAULT_COMP: Comp = Comp::Space(crate::comp::Space::new(0, 0));

#[derive(Clone, Debug)]
/// DialogHandler is the struct that handles everything about having a dialog
pub struct Handler {
	frame_f: fn(Comp<'static>) -> Comp<'static>,
	inst: Option<Instance>,
}
impl Default for Handler {
	fn default() -> Self {
		fn frame_f(comp: Comp<'static>) -> Comp<'static> {
			comp
		}
		Self::new(frame_f)
	}
}
impl Handler {
	pub fn new(frame_f: fn(Comp<'static>) -> Comp<'static>) -> Self {
		Self {
			frame_f,
			inst: None,
		}
	}

	pub fn run(&mut self, command: Command) {
		match command {
			Command::Open(inst) => self.inst = Some(inst.with_framer(self.frame_f)),
			Command::Close => self.inst = None,
		}
	}
	pub fn root_context(&self) -> RootContext<Comp<'static>> {
		match &self.inst {
			Some(Instance { comp, det, scale }) => RootContext::new(comp, *det, *scale),
			None => RootContext::new(&DEFAULT_COMP, Default::default(), 1.0),
		}
	}
}

#[derive(Clone, Debug)]
pub struct Instance {
	pub comp: Comp<'static>,
	pub det: Details,
	pub scale: f32,
}
impl Instance {
	pub fn with_framer(self, f: fn(Comp<'static>) -> Comp<'static>) -> Self {
		let Instance {
			mut comp,
			det,
			scale,
		} = self;
		comp = f(comp);
		return Instance { comp, det, scale };
	}
}

#[derive(Clone, Debug)]
pub enum Command {
	Open(Instance),
	Close,
}
