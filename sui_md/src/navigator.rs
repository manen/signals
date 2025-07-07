use std::{cell::RefCell, path::PathBuf, rc::Rc};

use sui::{comp::Space, DynamicLayable, Layable, LayableExt};

use crate::*;

#[derive(Clone, Debug)]
pub struct Navigator<P: NavProvider> {
	prov: P,
	page: Rc<RefCell<(String, DynamicLayable<'static>)>>,
}
impl<P: NavProvider> Navigator<P> {
	pub fn new(provider: P, page_path: String) -> Result<Self> {
		let nav = Navigator {
			prov: provider,
			page: Rc::new(RefCell::new((
				format!("UNSET"),
				DynamicLayable::new(Space::new(0, 0)),
			))),
		};
		nav.navigate_res(page_path)?;
		Ok(nav)
	}
	fn to_provider_path(&self, path: String) -> String {
		if path.starts_with('/') {
			path.replacen('/', "", 1)
		} else {
			let mut folder = PathBuf::from(&self.page.borrow().0);
			folder.pop();
			let folder = format!("{}", folder.display());
			if folder.len() > 0 {
				format!("{folder}/{path}")
			} else {
				path
			}
		}
	}
	pub fn navigate_res(&self, path: String) -> Result<()> {
		let path = self.to_provider_path(path);

		let page = self.prov.page(&path)?;
		let page = md_to_page(&page);
		let page = DynamicLayable::new(page);

		let mut s = self.page.borrow_mut();
		*s = (path, page);
		Ok(())
	}
	pub fn navigate(&self, path: String) {
		match self.navigate_res(path.clone()) {
			Ok(a) => a,
			Err(err) => {
				let err_page = sui::text(format!("{err}"), 16).margin(4);
				let err_page = sui::div([
					sui::custom(sui::text(format!("error loading page {path}:"), 24).centered()),
					sui::custom(err_page),
				])
				.centered();

				let mut page = self.page.borrow_mut();
				*page = (format!("/_error"), DynamicLayable::new(err_page));
			}
		}
	}
}
impl<P: NavProvider> Layable for Navigator<P> {
	fn size(&self) -> (i32, i32) {
		(0, 0)
	}
	fn render(&self, d: &mut sui::Handle, det: sui::Details, scale: f32) {
		self.page.borrow().1.render(d, det, scale);
	}
	fn pass_event(
		&self,
		event: sui::core::Event,
		det: sui::Details,
		scale: f32,
	) -> Option<sui::core::ReturnEvent> {
		let ret = self.page.borrow().1.pass_event(event, det, scale);
		// if there is a return event and it's a NavigateCommand, navigate to the page requested
		if ret
			.as_ref()
			.map(|a| a.can_take::<NavigateCommand>())
			.unwrap_or_default()
		{
			let ret = ret.expect("we just checked there is a return event");
			let nav = ret
				.take::<NavigateCommand>()
				.expect("we just checked if this is a NaviagteCommand");
			self.navigate(nav.0);
			None
		} else {
			ret
		}
	}
}

#[derive(Clone, Debug)]
pub struct NavigateCommand(pub String);

pub trait NavProvider {
	fn page(&self, path: &str) -> Result<String>;
}
