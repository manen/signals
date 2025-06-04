use sui::LayableExt;
use sui_md::Navigator;
use sui_md_fs::FsProvider;

fn main() {
	let prov = FsProvider::new(std::env::current_dir().unwrap());
	let navigator = Navigator::new(prov, "./README.md".into()).unwrap();

	let state = Default::default();
	let md = navigator.scrollable(state);

	let mut ctx = sui_runner::ctx(md);
	ctx.start()
}
