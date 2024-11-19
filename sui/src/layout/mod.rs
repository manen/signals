use crate::Details;
use raylib::prelude::RaylibDrawHandle;

pub mod page;
pub use page::Page;

pub trait Layable {
	fn size(&self) -> (i32, i32);

	fn render(&self, d: &mut RaylibDrawHandle, det: Details, scale: f32);
}
