use deunicode::deunicode_with_tofu;

pub fn clean_filename(filename: &str) -> String {
    let ascii_str = deunicode_with_tofu(filename, "");

    let cleaned: String = ascii_str
        .chars()
        .filter(|c| !c.is_control())
        .map(|c| match c {
            '/' | '\\' | ':' | '?' | '"' | '<' | '>' | '|' | '*' => '_',
            other => other,
        })
        .fold(String::new(), |mut acc, c| {
            let dominated = c == '_' || c == ' ';
            if dominated && acc.ends_with(c) {
                return acc;
            }
            acc.push(c);
            acc
        });

    cleaned.trim().trim_end_matches('.').trim().to_string()
}

pub fn clean_title(title: &str) -> String {
    title
        .split_once(" - ")
        .map(|(_, name)| name.trim())
        .unwrap_or(title.trim())
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_filename() {
        assert_eq!(clean_filename("hello/world"), "hello_world");
        assert_eq!(clean_filename("a:b?c\"d<e>f|g*h"), "a_b_c_d_e_f_g_h");
        assert_eq!(clean_filename("normal.mp3"), "normal.mp3");
        assert_eq!(clean_filename("Pilot."), "Pilot");
        assert_eq!(clean_filename("KOVEN."), "KOVEN");
        assert_eq!(
            clean_filename("NEFFEX - Fight Back 👊 🔥 [Copyright Free]"),
            "NEFFEX - Fight Back punch fire [Copyright Free]"
        );
        assert_eq!(clean_filename("maji* & vai5000"), "maji_ & vai5000");
        assert_eq!(clean_filename("𝒌𝒃 𝒋𝒖𝒏𝒊𝒐𝒓™"), "kb juniortm");
        assert_eq!(clean_filename("743⁺Aether*✧"), "743+Aether_");
    }

    #[test]
    fn test_clean_title() {
        assert_eq!(clean_title("NEFFEX - Fight Back"), "Fight Back");
        assert_eq!(clean_title("Fight Back"), "Fight Back");
        assert_eq!(clean_title("Artist - Track - Remix"), "Track - Remix");
    }
}
