use crate::{Details, Layable};

use raylib::{
	color::Color,
	math::{Rectangle, Vector2},
	prelude::{RaylibDraw, RaylibDrawHandle},
	texture::{RaylibTexture2D, Texture2D},
};

pub fn render_to_raylib_tex<L: Layable>(
	layable: &L,
	d: &mut crate::Handle,
	det: Details,
	scale: f32,
) -> Texture2D {
	let target = unsafe {
		raylib::ffi::LoadRenderTexture(
			(det.aw as f32 * scale) as i32,
			(det.ah as f32 * scale) as i32,
		)
	};

	// whole lotta unsafe but safe raylib crate is stupid
	{
		unsafe {
			raylib::ffi::BeginTextureMode(target);
		};

		d.clear_background(Color::new(0, 0, 0, 0));
		layable.render(d, Details { x: 0, y: 0, ..det }, scale);

		unsafe {
			raylib::ffi::EndTextureMode();
		}
	}

	let tex = target.texture;
	let tex = unsafe { Texture2D::from_raw(tex) };

	let mut image = tex
		.load_image()
		.expect("tex.load_image failed in render_to_raylib_tex");
	image.flip_vertical();

	let flipped_tex = unsafe { raylib::ffi::LoadTextureFromImage(image.to_raw()) };

	unsafe { Texture2D::from_raw(flipped_tex) }
}

#[derive(Debug)]
pub struct Texture {
	tex: Texture2D,
}
impl Clone for Texture {
	fn clone(&self) -> Self {
		let image = self
			.tex
			.load_image()
			.expect("Texture.tex.load_image failed");
		let image = unsafe { image.unwrap() };
		let new_tex = unsafe { raylib::ffi::LoadTextureFromImage(image) };
		let new_tex = unsafe { Texture2D::from_raw(new_tex) };

		Self { tex: new_tex }
	}
}
impl Texture {
	pub fn new_from_raylib(tex: Texture2D) -> Self {
		Self { tex }
	}
	pub fn from_layable<L: Layable>(d: &mut crate::Handle, layable: &L) -> Self {
		let (w, h) = layable.size();
		let tex = render_to_raylib_tex(
			layable,
			d,
			Details {
				aw: w,
				ah: h,
				..Default::default()
			},
			1.0,
		);
		Self::new_from_raylib(tex)
	}
}
impl Layable for Texture {
	fn size(&self) -> (i32, i32) {
		(self.tex.width, self.tex.height)
	}
	fn render(&self, d: &mut crate::Handle, det: Details, scale: f32) {
		d.draw_texture_pro(
			&self.tex,
			Rectangle {
				x: 0.0,
				y: 0.0,
				width: self.tex.width as f32,
				height: self.tex.height as f32,
			},
			Rectangle {
				x: det.x as f32,
				y: det.y as f32,
				width: det.aw as f32 * scale,
				height: det.ah as f32 * scale,
			},
			Vector2::default(),
			Default::default(),
			Color::new(255, 255, 255, 255),
		);
	}
	fn pass_event(
		&self,
		_: crate::core::Event,
		_: Details,
		_: f32,
	) -> Option<crate::core::ReturnEvent> {
		None
	}
}
