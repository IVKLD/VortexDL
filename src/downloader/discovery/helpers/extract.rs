use crate::utils::filename::clean_title;

pub trait AsUsername {
    fn username(&self) -> Option<&str>;
}

impl AsUsername for soundcloud_rs::UserSummary {
    fn username(&self) -> Option<&str> {
        self.username.as_deref()
    }
}

impl AsUsername for crate::models::UserInfo {
    fn username(&self) -> Option<&str> {
        Some(&self.username)
    }
}

pub fn extract_artist<T: AsUsername>(user: Option<&T>) -> String {
    user.and_then(|u| u.username())
        .unwrap_or("Unknown")
        .to_string()
}

pub fn extract_title(title: Option<&str>) -> String {
    clean_title(title.unwrap_or("Unknown"))
}
