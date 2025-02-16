use nfde::{
	DefaultPathDialogBuilder, DialogResult, FilterableDialogBuilder, SingleFileDialogBuilder,
};
use sui::{comp, Comp, LayableExt};

use crate::ui::SignalsEvent;

pub fn menu() -> Comp<'static> {
	let title = comp::Text::new("signals", 32).centered();

	let open = comp::Text::new("open file", 24).centered().clickable(|_| {
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
	});

	// rfd crate for file selection

	let page = comp::Div::vertical([sui::custom(title), sui::custom(open)]);
	sui::custom(page)
}
