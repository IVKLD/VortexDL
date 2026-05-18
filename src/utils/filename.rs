use deunicode::deunicode_with_tofu;

pub fn clean_filename(filename: &str) -> String {
    let ascii_str = deunicode_with_tofu(filename, "");

    let mut cleaned: String = ascii_str
        .chars()
        .filter(|c| !c.is_control())
        .map(|c| match c {
            '/' | '\\' | ':' | '?' | '"' | '<' | '>' | '|' | '*' => '_',
            other => other,
        })
        .collect();

    while cleaned.contains("__") {
        cleaned = cleaned.replace("__", "_");
    }
    while cleaned.contains("  ") {
        cleaned = cleaned.replace("  ", " ");
    }

    cleaned.trim().to_string()
}

pub fn format_track_filename(artist: &str, title: &str) -> String {
    clean_filename(&format!("{} - {}", artist, title))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_filename() {
        assert_eq!(clean_filename("hello/world"), "hello_world");
        assert_eq!(clean_filename("a:b?c\"d<e>f|g*h"), "a_b_c_d_e_f_g_h");
        assert_eq!(clean_filename("normal.mp3"), "normal.mp3");
        assert_eq!(
            clean_filename("NEFFEX - Fight Back 👊 🔥 [Copyright Free]"),
            "NEFFEX - Fight Back punch fire [Copyright Free]"
        );
        assert_eq!(clean_filename("maji* & vai5000"), "maji_ & vai5000");
        assert_eq!(clean_filename("𝒌𝒃 𝒋𝒖𝒏𝒊𝒐𝒓™"), "kb juniortm");
        assert_eq!(clean_filename("743⁺Aether*✧"), "743+Aether_");
    }
}
