pub mod extract;
pub mod resolve;

pub use extract::{extract_artist, extract_title};
pub use resolve::{get_likes, resolve_with_feedback, show_feedback};
