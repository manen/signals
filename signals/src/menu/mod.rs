use nfde::{
	DefaultPathDialogBuilder, DialogResult, FilterableDialogBuilder, SingleFileDialogBuilder,
};
use sui::{comp, core::Store, form::typable::TypableData, Comp, Layable, LayableExt};

use crate::ui::SignalsEvent;

pub mod title;
pub use title::title;

pub fn menu() -> Comp<'static> {
	let title = title().centered();

	let open = |_| {
		let nfd = nfde::Nfd::new().expect("failed to init nfde for file picking");

		match nfd
			.open_file()
			.default_path(&".")
			.expect("yaya")
			.add_filter("signals saves", "snsv")
			.expect("failed to build file picker")
			.show()
		{
			DialogResult::Ok(p) => SignalsEvent::LoadSave(p.to_path_buf()),
			DialogResult::Cancel => SignalsEvent::DialogFallback,
			DialogResult::Err(err) => {
				eprintln!("{err}");
				SignalsEvent::DialogFallback
			}
		}
	};
	let open = comp::Text::new("open file", 24).centered().clickable(open);

	let new = |_| {
		let nfd = nfde::Nfd::new().expect("failed to init nfde for file picking");

		match nfd
			.save_file()
			.default_path(&".")
			.expect("bruh")
			.add_filter("signals saves", "snsv")
			.expect("failed to add file save filter")
			.show()
		{
			DialogResult::Ok(p) => {
				let path = p.to_path_buf();

				SignalsEvent::LoadSave(path)
			}
			DialogResult::Cancel => SignalsEvent::DialogFallback,
			DialogResult::Err(err) => {
				eprintln!("{err}");
				SignalsEvent::DialogFallback
			}
		}
	};
	let new = comp::Text::new("new world", 24).centered().clickable(new);

	let title = title.margin(5);

	let open = open.margin(5);
	let new = new.margin(5);

	// let open = open.with_background(bg);
	// let new = new.with_background(bg);

	let open = open.margin(5);
	let new = new.margin(5);

	let actions = sui::div_h([sui::custom(open), sui::custom(new)]);

	let page = comp::Div::vertical([sui::custom(title), sui::custom(actions)]);
	sui::custom(page.centered())
}
