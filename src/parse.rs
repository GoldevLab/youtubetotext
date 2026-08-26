//! Pull a YouTube video id out of the mess of URLs people actually paste.

/// 11-character YouTube video id.
pub fn parse_video_id(input: &str) -> Option<String> {
    let raw = input.trim();
    if raw.is_empty() {
        return None;
    }
    if is_id(raw) {
        return Some(raw.to_string());
    }

    let candidate = if raw.contains("://") {
        raw.to_string()
    } else if raw.starts_with("www.") || raw.starts_with("youtube.") || raw.starts_with("youtu.be")
    {
        format!("https://{raw}")
    } else {
        raw.to_string()
    };

    if let Ok(url) = url::Url::parse(&candidate) {
        if let Some(id) = from_url(&url) {
            return Some(id);
        }
    }

    // Last resort: first 11-char token that looks like an id.
    for token in raw.split(|c: char| c.is_whitespace() || c == '"' || c == '\'') {
        let t = token.trim_matches(|c: char| !is_id_char(c));
        if is_id(t) {
            return Some(t.to_string());
        }
    }
    None
}

fn from_url(url: &url::Url) -> Option<String> {
    let host = url.host_str()?.to_ascii_lowercase();
    let youtube = host == "youtu.be"
        || host == "www.youtu.be"
        || host.ends_with("youtube.com")
        || host.ends_with("youtube-nocookie.com")
        || host == "youtube.com";
    if !youtube {
        return None;
    }

    if let Some(v) = url
        .query_pairs()
        .find(|(k, _)| k == "v" || k == "vi")
        .map(|(_, v)| v.into_owned())
    {
        if is_id(&v) {
            return Some(v);
        }
    }

    let mut segs = url.path_segments()?.filter(|s| !s.is_empty());
    let first = segs.next()?;
    match first {
        "embed" | "shorts" | "live" | "v" | "watch" => {
            if let Some(next) = segs.next() {
                let id = next.split('&').next().unwrap_or(next);
                if is_id(id) {
                    return Some(id.to_string());
                }
            }
        }
        _ if host.contains("youtu.be") && is_id(first) => return Some(first.to_string()),
        _ => {}
    }
    None
}

fn is_id_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '-'
}

pub fn is_id(s: &str) -> bool {
    s.len() == 11 && s.chars().all(is_id_char)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_common_shapes() {
        let id = "dQw4w9WgXcQ";
        let samples = [
            id,
            &format!("https://www.youtube.com/watch?v={id}"),
            &format!("https://www.youtube.com/watch?v={id}&t=12s"),
            &format!("https://youtu.be/{id}"),
            &format!("https://youtu.be/{id}?t=30"),
            &format!("https://www.youtube.com/embed/{id}"),
            &format!("https://www.youtube.com/shorts/{id}"),
            &format!("https://m.youtube.com/watch?v={id}"),
            &format!("https://www.youtube-nocookie.com/embed/{id}"),
            &format!("https://music.youtube.com/watch?v={id}"),
            &format!("youtube.com/live/{id}"),
            &format!("  {id}  "),
        ];
        for s in samples {
            assert_eq!(parse_video_id(s).as_deref(), Some(id), "failed: {s}");
        }
    }

    #[test]
    fn rejects_garbage() {
        assert!(parse_video_id("").is_none());
        assert!(parse_video_id("https://example.com/watch?v=nope").is_none());
        assert!(parse_video_id("hello world").is_none());
    }
}
