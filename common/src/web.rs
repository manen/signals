// some web fixes fuck the web

use std::ops::Deref;

use raylib::{prelude::RaylibDrawHandle, RaylibHandle};

pub const WEB_BUILD: bool = cfg!(target_arch = "wasm32");

pub fn cursor<H: Handle>(rl: &H) -> (i32, i32) {
	let (mouse_x, mouse_y) = (rl.get_mouse_x(), rl.get_mouse_y());
	if !WEB_BUILD {
		(mouse_x, mouse_y)
	} else {
		(
			(mouse_x as f32 / 640.0 * rl.get_render_width() as f32) as i32,
			(mouse_y as f32 / 480.0 * rl.get_render_height() as f32) as i32,
		)
	}
}

pub trait Handle {
	fn get_mouse_x(&self) -> i32;
	fn get_mouse_y(&self) -> i32;
	fn get_render_width(&self) -> i32;
	fn get_render_height(&self) -> i32;
}
impl Handle for RaylibHandle {
	fn get_mouse_x(&self) -> i32 {
		self.get_mouse_x()
	}
	fn get_mouse_y(&self) -> i32 {
		self.get_mouse_y()
	}
	fn get_render_width(&self) -> i32 {
		self.get_render_width()
	}
	fn get_render_height(&self) -> i32 {
		unsafe { raylib::ffi::GetRenderHeight() }
	}
}
impl<'a> Handle for RaylibDrawHandle<'a> {
	fn get_mouse_x(&self) -> i32 {
		Deref::deref(self).get_mouse_x()
	}
	fn get_mouse_y(&self) -> i32 {
		Deref::deref(self).get_mouse_y()
	}
	fn get_render_width(&self) -> i32 {
		Deref::deref(self).get_render_width()
	}
	fn get_render_height(&self) -> i32 {
		unsafe { raylib::ffi::GetRenderHeight() }
	}
}
