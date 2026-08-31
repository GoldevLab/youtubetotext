//! Chapter-aware extractive recap from caption cues.

use crate::youtube::{Chapter, Cue, TranscriptDoc};

pub fn extractive_summary(doc: &TranscriptDoc) -> String {
    if doc.cues.is_empty() {
        return String::new();
    }
    if doc.chapters.len() >= 2 {
        let mut out = String::new();
        for (i, ch) in doc.chapters.iter().enumerate() {
            let end = doc
                .chapters
                .get(i + 1)
                .map(|n| n.start_ms)
                .unwrap_or(u64::MAX);
            let slice: Vec<&Cue> = doc
                .cues
                .iter()
                .filter(|c| c.start_ms >= ch.start_ms && c.start_ms < end)
                .collect();
            let body = first_sentences(&join_cues(&slice), 2);
            if body.is_empty() {
                continue;
            }
            if !out.is_empty() {
                out.push_str("\n\n");
            }
            out.push_str("## ");
            out.push_str(&ch.title);
            out.push('\n');
            out.push_str(&body);
        }
        if !out.is_empty() {
            return out;
        }
    }
    first_sentences(&join_cues(&doc.cues.iter().collect::<Vec<_>>()), 8)
}

fn join_cues(cues: &[&Cue]) -> String {
    cues.iter()
        .map(|c| c.text.as_str())
        .collect::<Vec<_>>()
        .join(" ")
}

fn first_sentences(text: &str, max: usize) -> String {
    let mut parts = Vec::new();
    let mut buf = String::new();
    for ch in text.chars() {
        buf.push(ch);
        if matches!(ch, '.' | '!' | '?') && buf.split_whitespace().count() >= 4 {
            parts.push(buf.trim().to_string());
            buf.clear();
            if parts.len() >= max {
                break;
            }
        }
    }
    if parts.len() < max {
        let rest = buf.trim();
        if !rest.is_empty() {
            parts.push(rest.to_string());
        }
    }
    if parts.is_empty() {
        text.split_whitespace().take(80).collect::<Vec<_>>().join(" ")
    } else {
        parts.join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::youtube::{CaptionTrack, Cue, TranscriptDoc};

    fn doc(cues: Vec<Cue>, chapters: Vec<Chapter>) -> TranscriptDoc {
        TranscriptDoc {
            video_id: "dQw4w9WgXcQ".into(),
            title: "Talk".into(),
            author: "A".into(),
            channel_id: String::new(),
            duration_secs: 120,
            track: CaptionTrack {
                lang: "en".into(),
                name: "English".into(),
                kind: String::new(),
                translatable: true,
            },
            tracks: vec![],
            translations: vec![],
            cues,
            chapters,
        }
    }

    #[test]
    fn chapters_split_recap() {
        let d = doc(
            vec![
                Cue {
                    start_ms: 0,
                    duration_ms: 1000,
                    text: "Welcome to the intro section of this talk.".into(),
                },
                Cue {
                    start_ms: 60_000,
                    duration_ms: 1000,
                    text: "Now we discuss the results in detail here.".into(),
                },
            ],
            vec![
                Chapter {
                    start_ms: 0,
                    title: "Intro".into(),
                },
                Chapter {
                    start_ms: 60_000,
                    title: "Results".into(),
                },
            ],
        );
        let s = extractive_summary(&d);
        assert!(s.contains("## Intro"), "{s}");
        assert!(s.contains("## Results"), "{s}");
    }
}
