pub mod md_to_page;
pub use md_to_page::md_to_page;

#[derive(Clone, Debug)]
pub struct NavigateCommand(pub String);
