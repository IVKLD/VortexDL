pub mod local;
pub mod url;
pub mod webdav;

pub use local::LocalStrategy;
pub use url::UrlStrategy;
pub use webdav::WebDavStrategy;
