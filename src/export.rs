//! Transcript file formats.

use crate::youtube::{Cue, TranscriptDoc};

pub fn as_txt(doc: &TranscriptDoc, timestamps: bool) -> String {
    let mut out = String::new();
    if !doc.title.is_empty() {
        out.push_str(&doc.title);
        out.push('\n');
        if !doc.author.is_empty() {
            out.push_str(&doc.author);
            out.push('\n');
        }
        out.push_str("https://www.youtube.com/watch?v=");
        out.push_str(&doc.video_id);
        out.push_str("\n\n");
    }
    for cue in &doc.cues {
        if timestamps {
            out.push('[');
            out.push_str(&format_clock(cue.start_ms, false));
            out.push_str("] ");
        }
        out.push_str(cue.text.trim());
        out.push('\n');
    }
    out
}

fn one_line(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn as_markdown(doc: &TranscriptDoc) -> String {
    let mut out = String::new();
    out.push_str("# ");
    out.push_str(&if doc.title.is_empty() {
        "Transcript".to_string()
    } else {
        one_line(&doc.title)
    });
    out.push_str("\n\n");
    if !doc.author.is_empty() {
        out.push_str("*");
        out.push_str(&one_line(&doc.author));
        out.push_str("*\n\n");
    }
    out.push_str("[Watch on YouTube](https://www.youtube.com/watch?v=");
    out.push_str(&doc.video_id);
    out.push_str(")\n\n");
    for cue in &doc.cues {
        let secs = cue.start_ms / 1000;
        out.push_str(&format!(
            "- [{ts}](https://www.youtube.com/watch?v={id}&t={secs}s) {text}\n",
            ts = format_clock(cue.start_ms, false),
            id = doc.video_id,
            text = cue.text.trim()
        ));
    }
    out
}

pub fn as_srt(cues: &[Cue]) -> String {
    let mut out = String::new();
    for (i, cue) in cues.iter().enumerate() {
        let end = cue.start_ms.saturating_add(cue.duration_ms.max(500));
        out.push_str(&(i + 1).to_string());
        out.push('\n');
        out.push_str(&format_clock(cue.start_ms, true));
        out.push_str(" --> ");
        out.push_str(&format_clock(end, true));
        out.push('\n');
        out.push_str(cue.text.trim());
        out.push_str("\n\n");
    }
    out
}

pub fn as_vtt(cues: &[Cue]) -> String {
    let mut out = String::from("WEBVTT\n\n");
    for cue in cues {
        let end = cue.start_ms.saturating_add(cue.duration_ms.max(500));
        out.push_str(&format_clock(cue.start_ms, true).replace(',', "."));
        out.push_str(" --> ");
        out.push_str(&format_clock(end, true).replace(',', "."));
        out.push('\n');
        out.push_str(cue.text.trim());
        out.push_str("\n\n");
    }
    out
}

pub fn format_clock(ms: u64, srt: bool) -> String {
    let h = ms / 3_600_000;
    let m = (ms / 60_000) % 60;
    let s = (ms / 1000) % 60;
    let frac = ms % 1000;
    if srt {
        format!("{h:02}:{m:02}:{s:02},{frac:03}")
    } else if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m}:{s:02}")
    }
}

pub fn reading_minutes(text: &str) -> u32 {
    let words = text.split_whitespace().count() as u32;
    if words == 0 {
        0
    } else {
        words.div_ceil(200)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::youtube::{CaptionTrack, Cue, TranscriptDoc};

    fn sample() -> TranscriptDoc {
        TranscriptDoc {
            video_id: "dQw4w9WgXcQ".into(),
            title: "Never Gonna Give You Up".into(),
            author: "Rick Astley".into(),
            channel_id: String::new(),
            duration_secs: 213,
            track: CaptionTrack {
                lang: "en".into(),
                name: "English".into(),
                kind: String::new(),
                translatable: true,
            },
            tracks: vec![],
            translations: vec![],
            cues: vec![Cue {
                start_ms: 1500,
                duration_ms: 2000,
                text: "Hello there".into(),
            }],
            chapters: vec![],
        }
    }

    #[test]
    fn srt_round_shape() {
        let s = as_srt(&sample().cues);
        assert!(s.contains("00:00:01,500 --> 00:00:03,500"));
        assert!(s.contains("Hello there"));
    }

    #[test]
    fn markdown_has_timestamp_links() {
        let md = as_markdown(&sample());
        assert!(md.contains("&t=1s"));
        assert!(md.contains("Never Gonna Give You Up"));
    }

    #[test]
    fn markdown_flattens_title_newlines() {
        let mut doc = sample();
        doc.title = "Line one\nLine two".into();
        let md = as_markdown(&doc);
        assert!(md.starts_with("# Line one Line two\n"));
        assert!(!md.contains("# Line one\nLine two"));
    }

    #[test]
    fn reading_minutes_empty_is_zero() {
        assert_eq!(reading_minutes(""), 0);
        assert_eq!(reading_minutes("one two"), 1);
    }
}
