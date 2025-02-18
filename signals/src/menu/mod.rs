use nfde::{
	DefaultPathDialogBuilder, DialogResult, FilterableDialogBuilder, SingleFileDialogBuilder,
};
use sui::{comp, core::Store, form::typable::TypableData, Comp, Layable, LayableExt};

use crate::ui::SignalsEvent;

pub mod title;
pub use title::title;

pub fn menu() -> Comp<'static> {
	let title = title::title().centered();

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

	let new = |at| {
		// a textbox for name/path and a cancel and submit button
		// just copy the println one

		let text_store = Store::new(TypableData::with_default("new_world".to_owned()));

		let title = comp::Text::new("enter world name", 16).margin(3);
		let textbox = sui::form::textbox(text_store.clone(), 32)
			.margin(5)
			.margin_h(15);

		let cancel = |_| sui::dialog::Command::Close;
		let cancel = comp::Text::new("cancel", 12).clickable(cancel);

		let create = {
			let text_store = text_store.clone();
			move |_| {
				let path = text_store.with_borrow(|a| {
					if a.text.ends_with(".snsv") {
						format!("{}", a.text)
					} else {
						format!("{}.snsv", a.text)
					}
				});
				SignalsEvent::LoadSave(path.into())
			}
		};
		let create = comp::Text::new("create world", 12)
			.clickable(create)
			.to_right();

		let actions = cancel.overlay(create);

		let comp = comp::Div::vertical([
			sui::custom(title),
			sui::custom(textbox),
			sui::custom(actions),
		]);

		let comp = sui::custom(comp);
		SignalsEvent::Multiple(vec![
			SignalsEvent::DialogCommand(sui::dialog::Command::Open(sui::dialog::Instance {
				comp,
				at,
				scale: 1.0,
			})),
			SignalsEvent::FocusCommand(sui::form::FocusCommand::Request(
				text_store.with_borrow(|a| a.uid),
			)),
		])
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
