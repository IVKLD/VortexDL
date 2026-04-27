pub fn clean_filename(filename: &str) -> String {
    filename
        .replace("/", "_")
        .replace("\\", "_")
        .replace(":", "_")
        .replace("?", "_")
        .replace("\"", "_")
        .replace("<", "_")
        .replace(">", "_")
        .replace("|", "_")
}

pub fn ensure_likes_suffix(url_str: &str) -> String {
    let mut base = url_str.trim_end_matches('/');

    if let Some(stripped) = base.strip_suffix("likes") {
        base = stripped.trim_end_matches('/');
    }

    format!("{}/likes", base)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_filename() {
        assert_eq!(clean_filename("hello/world"), "hello_world");
        assert_eq!(clean_filename("a:b?c\"d<e>f|g"), "a_b_c_d_e_f_g");
        assert_eq!(clean_filename("normal.mp3"), "normal.mp3");
    }
}
