use std::fmt::Debug;

use markdown::{Block, ListItem, Span};
use sui::{Color, Comp, Div, Layable, LayableExt};

pub fn md_to_page(text: &str) -> impl Layable + Debug + Clone {
	let blocks = markdown::tokenize(text);

	let components = blocks.into_iter().map(from_block).collect::<Vec<_>>();

	sui::div(components)
}

fn from_block(block: Block) -> sui::Comp<'static> {
	match block {
		Block::Hr => sui::custom(
			sui::comp::Space::new(0, 0)
				.fix_h(1)
				.with_background(sui::comp::Color::new(sui::Color::WHITESMOKE))
				.margin(8),
		),
		Block::Raw(text) => sui::text(text, 13),
		Block::Header(spans, lvl) => sui::custom(
			spans
				.into_iter()
				.map(from_span)
				.collect::<sui::Div<_>>()
				.scale((7.0 - lvl as f32 - 2.0) * 0.5)
				.margin_h(4),
		),
		Block::Paragraph(spans) => sui::Comp::Div(sui::div(
			spans.into_iter().map(from_span).collect::<Vec<_>>(),
		)),
		Block::Blockquote(blocks) => sui::Comp::Div(sui::div(
			blocks.into_iter().map(from_block).collect::<Vec<_>>(),
		)),
		Block::CodeBlock(lang, text) => sui::text(
			format!("'''{}\n{text}\n'''", lang.unwrap_or_else(String::new)),
			13,
		),
		Block::OrderedList(list, _typ) => sui::custom(sui::div(
			list.into_iter()
				.enumerate()
				.map(|(i, x)| (i + 1, x))
				.map(|(index, item)| {
					let item = match item {
						ListItem::Simple(spans) => sui::Comp::Div(sui::div(
							spans.into_iter().map(from_span).collect::<Vec<_>>(),
						)),
						ListItem::Paragraph(blocks) => sui::Comp::Div(sui::div(
							blocks.into_iter().map(from_block).collect::<Vec<_>>(),
						)),
					};
					sui::div_h([sui::text(format!("{index}. "), 13), item])
				})
				.collect::<Vec<_>>(),
		)),
		Block::UnorderedList(list) => sui::Comp::Div(
			list.into_iter()
				.map(|item| {
					let item = match item {
						ListItem::Simple(spans) => sui::Comp::Div(sui::div(
							spans.into_iter().map(from_span).collect::<Vec<_>>(),
						)),
						ListItem::Paragraph(blocks) => sui::Comp::Div(sui::div(
							blocks.into_iter().map(from_block).collect::<Vec<_>>(),
						)),
					};
					sui::custom(sui::div_h([sui::text(" - ", 13), item]))
				})
				.collect::<Div<Vec<Comp>>>(),
		),
	}
}

fn from_span(span: Span) -> sui::Comp<'static> {
	match span {
		Span::Break => sui::Comp::Space(sui::comp::Space::new(0, 0)),
		Span::Text(text) => sui::text(text, 13),
		Span::Code(code) => sui::text(code, 13),
		Span::Link(text, url, _) => sui::custom(
			sui::comp::Text::new_colored(text, 13, Color::BLUEVIOLET)
				.clickable_fallback(move |_| crate::NavigateCommand(url.clone())),
		),
		Span::Image(title, _href, alt) => {
			sui::text(format!("[image] {}", alt.unwrap_or(title)), 13)
		}

		Span::Strong(spans) => sui::Comp::Div(sui::div(
			spans.into_iter().map(from_span).collect::<Vec<_>>(),
		)),
		Span::Emphasis(spans) => sui::Comp::Div(sui::div(
			spans.into_iter().map(from_span).collect::<Vec<_>>(),
		)),
	}
}
