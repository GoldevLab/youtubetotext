//! Server actions, loaders, and form submits.

use resuma::prelude::*;

use crate::family::{app_href, Mode};
use crate::parse::parse_video_id;
use crate::youtube::{load_transcript, pick_audio, AudioMeta, TranscriptDoc};

#[data]
pub struct OpenForm {
    url: String,
    mode: Option<String>,
}

#[submit]
pub async fn open_transcript(
    form: OpenForm,
    _req: &FlowRequest,
) -> std::result::Result<Redirect, SubmitError> {
    let id = parse_video_id(&form.url).ok_or_else(|| {
        SubmitError::new("Paste a YouTube URL or the 11-character video id.")
            .field("url", "Not a YouTube link")
    })?;
    let mode = Mode::parse(form.mode.as_deref().unwrap_or("text"));
    Ok(redirect(app_href(&id, mode)))
}

#[load]
pub async fn audio_pick(req: &FlowRequest) -> std::result::Result<AudioMeta, LoaderError> {
    let raw = req.query_param("v").unwrap_or("");
    let id = parse_video_id(raw).ok_or_else(|| LoaderError::new(400, "Missing video id."))?;
    match pick_audio(&id).await {
        Ok(doc) => Ok(doc.meta()),
        Err(e) => Err(LoaderError::new(e.status, e.message)),
    }
}

#[load]
pub async fn video_doc(
    req: &FlowRequest,
) -> std::result::Result<TranscriptDoc, LoaderError> {
    let raw = req
        .query_param("v")
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| req.param("id").unwrap_or(""));
    let id = parse_video_id(raw).unwrap_or_else(|| raw.to_string());
    let lang = req.query_param("lang").filter(|s| !s.is_empty());
    let tlang = req.query_param("tlang").filter(|s| !s.is_empty());
    match load_transcript(&id, lang, tlang).await {
        Ok(doc) => Ok((*doc).clone()),
        Err(e) => Err(LoaderError::new(e.status, e.message)),
    }
}
