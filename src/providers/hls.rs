use reqwest::{Response as ReqwestResponse, header::CONTENT_TYPE};
use url::Url;

use crate::providers::error::ProviderError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HlsSegment {
    pub url: String,
    pub byte_range: Option<(u64, u64)>,
}

#[derive(Clone, Debug)]
pub struct HlsPlaylist {
    pub segments: Vec<HlsSegment>,
    pub total_size: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ByteRange {
    Full,
    Partial(u64, u64),
    Unsatisfiable,
}

impl ByteRange {
    pub fn parse(header_str: Option<&str>, total_size: u64) -> Self {
        let Some(range_str) = header_str else {
            return Self::Full;
        };
        let Some(bytes_spec) = range_str.trim().strip_prefix("bytes=") else {
            return Self::Full;
        };
        if bytes_spec.contains(',') {
            return Self::Full;
        }
        let Some((start_str, end_str)) = bytes_spec.split_once('-') else {
            return Self::Full;
        };

        if total_size == 0 {
            return Self::Unsatisfiable;
        }

        if start_str.is_empty() {
            let Ok(suffix_len) = end_str.parse::<u64>() else {
                return Self::Unsatisfiable;
            };
            if suffix_len == 0 {
                return Self::Unsatisfiable;
            }
            let start = total_size.saturating_sub(suffix_len);
            return Self::Partial(start, total_size - 1);
        }

        let Ok(start) = start_str.parse::<u64>() else {
            return Self::Unsatisfiable;
        };
        if start >= total_size {
            return Self::Unsatisfiable;
        }

        let end = if end_str.is_empty() {
            total_size - 1
        } else {
            let Ok(parsed_end) = end_str.parse::<u64>() else {
                return Self::Unsatisfiable;
            };
            if parsed_end < start {
                return Self::Unsatisfiable;
            }
            parsed_end.min(total_size - 1)
        };

        Self::Partial(start, end)
    }
}

impl HlsPlaylist {
    pub fn is_hls_url(url: &str) -> bool {
        Url::parse(url)
            .map(|u| u.path().to_ascii_lowercase().ends_with(".m3u8"))
            .unwrap_or_else(|_| url.to_ascii_lowercase().contains(".m3u8"))
    }

    pub fn is_hls_response(res: &ReqwestResponse, url: &str) -> bool {
        if Self::is_hls_url(url) {
            return true;
        }
        res.headers()
            .get(CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .is_some_and(|ct| ct.to_ascii_lowercase().contains("mpegurl"))
    }

    pub fn parse(manifest_text: &str, base_url: &str) -> Result<Self, ProviderError> {
        let mut raw_urls = Vec::new();

        for line in manifest_text.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Some(rest) = trimmed.strip_prefix("#EXT-X-MAP:") {
                if let Some(uri) = Self::extract_quoted_attr(rest, "URI") {
                    raw_urls.push(Self::resolve_url(base_url, uri));
                }
                continue;
            }
            if trimmed.starts_with('#') {
                continue;
            }
            raw_urls.push(Self::resolve_url(base_url, trimmed));
        }

        if raw_urls.is_empty() {
            return Err(ProviderError::InvalidStream(
                "HLS playlist contains no media segments".to_string(),
            ));
        }

        let raw_ranges: Vec<Option<(u64, u64)>> = raw_urls
            .iter()
            .map(|u| Self::extract_media_numbers(u))
            .collect();

        let (segments, total_size) = if raw_ranges.iter().all(Option::is_some) {
            let pairs: Vec<(u64, u64)> = raw_ranges.into_iter().map(Option::unwrap).collect();
            Self::normalize_media_ranges(&raw_urls, &pairs)
        } else {
            let segs = raw_urls
                .into_iter()
                .map(|url| HlsSegment {
                    url,
                    byte_range: None,
                })
                .collect();
            (segs, None)
        };

        Ok(Self {
            segments,
            total_size,
        })
    }

    pub fn single_nested_playlist_url(&self) -> Option<&str> {
        if self.segments.len() == 1 && Self::is_hls_url(&self.segments[0].url) {
            Some(&self.segments[0].url)
        } else {
            None
        }
    }

    pub fn segments_in_range(&self, start: u64, end: u64) -> Vec<HlsSegment> {
        self.segments
            .iter()
            .filter(|seg| seg.byte_range.is_some_and(|(s, e)| e >= start && s <= end))
            .cloned()
            .collect()
    }

    fn normalize_media_ranges(
        urls: &[String],
        pairs: &[(u64, u64)],
    ) -> (Vec<HlsSegment>, Option<u64>) {
        if pairs.is_empty() || pairs[0].0 != 0 {
            return Self::unranged_segments(urls);
        }

        let is_inclusive = pairs
            .windows(2)
            .all(|w| w[0].1 >= w[0].0 && w[1].0 == w[0].1 + 1);

        let is_exclusive = !is_inclusive
            && pairs
                .windows(2)
                .all(|w| w[0].1 > w[0].0 && w[1].0 == w[0].1);

        if is_inclusive {
            let total = pairs.last().map(|&(_, end)| end + 1);
            let segs = urls
                .iter()
                .cloned()
                .zip(pairs.iter().copied())
                .map(|(url, (start, end))| HlsSegment {
                    url,
                    byte_range: Some((start, end)),
                })
                .collect();
            (segs, total)
        } else if is_exclusive {
            let total = pairs.last().map(|&(_, end)| end);
            let segs = urls
                .iter()
                .cloned()
                .zip(pairs.iter().copied())
                .map(|(url, (start, end))| HlsSegment {
                    url,
                    byte_range: Some((start, end - 1)),
                })
                .collect();
            (segs, total)
        } else {
            Self::unranged_segments(urls)
        }
    }

    fn unranged_segments(urls: &[String]) -> (Vec<HlsSegment>, Option<u64>) {
        let segs = urls
            .iter()
            .cloned()
            .map(|url| HlsSegment {
                url,
                byte_range: None,
            })
            .collect();
        (segs, None)
    }

    fn resolve_url(base_url: &str, target: &str) -> String {
        if let Ok(u) = Url::parse(target) {
            return u.to_string();
        }
        if let Ok(base) = Url::parse(base_url)
            && let Ok(joined) = base.join(target)
        {
            return joined.to_string();
        }
        target.to_string()
    }

    fn extract_quoted_attr<'a>(input: &'a str, attr: &str) -> Option<&'a str> {
        let pattern = format!("{attr}=\"");
        let start = input.find(&pattern)? + pattern.len();
        let rest = &input[start..];
        let end = rest.find('"')?;
        Some(&rest[..end])
    }

    fn extract_media_numbers(url_str: &str) -> Option<(u64, u64)> {
        let parsed = Url::parse(url_str).ok()?;
        let parts: Vec<&str> = parsed.path().split('/').filter(|s| !s.is_empty()).collect();
        for window in parts.windows(3) {
            if window[0] == "media"
                && let (Ok(start), Ok(end)) = (window[1].parse::<u64>(), window[2].parse::<u64>())
                && end >= start
            {
                return Some((start, end));
            }
        }
        None
    }
}
