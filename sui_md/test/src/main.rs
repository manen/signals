fn main() {
	let text = include_str!("../test.md");
	let md = sui_md::md_to_page(text);

	let mut ctx = sui_runner::ctx(md);
	ctx.start()
}
