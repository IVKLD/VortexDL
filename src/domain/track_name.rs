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
        let is_tag = suffix_lower.split(|c: char| !c.is_alphanumeric()).any(|w| {
            matches!(
                w,
                "remix" | "edit" | "vip" | "cover" | "mix" | "bootleg" | "flip" | "dub"
            )
        });

        if !is_tag {
            return (prefix.trim().to_string(), suffix.trim().to_string());
        }
    }

    (uploader.to_string(), raw_title.to_string())
}
