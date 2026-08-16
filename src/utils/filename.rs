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
        .fold((String::new(), None::<char>), |(mut acc, last), c| {
            match (last, c) {
                (Some(' ' | '_'), ' ' | '_') => {
                    if last == Some('_') || c == '_' {
                        acc.pop();
                        acc.push('_');
                        (acc, Some('_'))
                    } else {
                        (acc, last)
                    }
                }
                _ => {
                    acc.push(c);
                    (acc, Some(c))
                }
            }
        })
        .0;

    cleaned.trim().trim_end_matches('.').trim().to_string()
}

fn normalize_name(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_alphanumeric())
        .collect::<String>()
        .to_lowercase()
}

pub fn parse_track_metadata(raw_title: &str, uploader: &str) -> (String, String) {
    let raw_title = raw_title.trim();
    let uploader = uploader.trim();

    if let Some((prefix, suffix)) = raw_title.split_once(" - ") {
        let prefix_norm = normalize_name(prefix);
        let uploader_norm = normalize_name(uploader);

        if !prefix_norm.is_empty()
            && (prefix_norm == uploader_norm
                || uploader_norm.contains(&prefix_norm)
                || prefix_norm.contains(&uploader_norm))
        {
            return (uploader.to_string(), suffix.trim().to_string());
        }

        let suffix_lower = suffix.to_lowercase();
        let is_tag = suffix_lower.contains("remix")
            || suffix_lower.contains("edit")
            || suffix_lower.contains("vip")
            || suffix_lower.contains("cover")
            || suffix_lower.contains("mix")
            || suffix_lower.contains("bootleg");

        if !is_tag {
            return (prefix.trim().to_string(), suffix.trim().to_string());
        }
    }

    (uploader.to_string(), raw_title.to_string())
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
        assert_eq!(clean_filename("maji* & vai5000"), "maji_& vai5000");
        assert_eq!(clean_filename("𝒌𝒃 𝒋𝒖𝒏𝒊𝒐𝒓™"), "kb juniortm");
        assert_eq!(clean_filename("743⁺Aether*✧"), "743+Aether_");
        assert_eq!(clean_filename("track _ _ name"), "track_name");
        assert_eq!(clean_filename("track   _name"), "track_name");
        assert_eq!(clean_filename("track_   name"), "track_name");
    }

    #[test]
    fn test_parse_track_metadata() {
        assert_eq!(
            parse_track_metadata("NEFFEX - Fight Back", "NEFFEX"),
            ("NEFFEX".to_string(), "Fight Back".to_string())
        );
        assert_eq!(
            parse_track_metadata("NEFFEX - Fight Back", "NEFFEX Music"),
            ("NEFFEX Music".to_string(), "Fight Back".to_string())
        );
        assert_eq!(
            parse_track_metadata("NEFFEX - Fight Back", "Copyright Free Channel"),
            ("NEFFEX".to_string(), "Fight Back".to_string())
        );
        assert_eq!(
            parse_track_metadata("Fight Back - Remix", "NEFFEX"),
            ("NEFFEX".to_string(), "Fight Back - Remix".to_string())
        );
        assert_eq!(
            parse_track_metadata("Fight Back", "NEFFEX"),
            ("NEFFEX".to_string(), "Fight Back".to_string())
        );
    }
}
