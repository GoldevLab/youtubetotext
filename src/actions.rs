//! Server actions, loaders, and form submits.

use resuma::prelude::*;

use crate::parse::parse_video_id;
use crate::youtube::{load_transcript, TranscriptDoc};

#[data]
pub struct OpenForm {
    url: String,
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
    Ok(redirect(format!("/v/{id}")))
}

#[load]
pub async fn video_doc(
    req: &FlowRequest,
) -> std::result::Result<TranscriptDoc, LoaderError> {
    let id = req.param("id").unwrap_or("");
    let lang = req.query_param("lang").filter(|s| !s.is_empty());
    let tlang = req.query_param("tlang").filter(|s| !s.is_empty());
    match load_transcript(id, lang, tlang).await {
        Ok(doc) => Ok((*doc).clone()),
        Err(e) => Err(LoaderError::new(e.status, e.message)),
    }
}
