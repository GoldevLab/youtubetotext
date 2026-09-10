//! Server actions, loaders, and form submits.

use resuma::prelude::*;

use crate::family::{app_href, Mode};
use crate::guard::{self, PAGE};
use crate::parse::parse_video_id;
use crate::youtube::{load_transcript, pick_audio, AudioMeta, TranscriptDoc};

#[data]
pub struct OpenForm {
    url: String,
    mode: Option<String>,
    website: Option<String>,
    pow_salt: Option<String>,
    pow_challenge: Option<String>,
    pow_number: Option<String>,
    pow_maxnumber: Option<String>,
    pow_signature: Option<String>,
    pow_exp: Option<String>,
    pow_kind: Option<String>,
}

#[submit]
pub async fn open_transcript(
    form: OpenForm,
    req: &FlowRequest,
) -> std::result::Result<Redirect, SubmitError> {
    if guard::honeypot_tripped(form.website.as_deref()) {
        return Ok(redirect("/"));
    }
    guard::check_req(req, PAGE).map_err(|m| SubmitError::new(m))?;
    let mut ticket: Option<String> = None;
    let has_pow = form.pow_salt.as_deref().is_some_and(|s| !s.is_empty());
    if has_pow {
        let number = form
            .pow_number
            .as_deref()
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| SubmitError::new("Confirm you are not a bot, then try again."))?;
        let maxnumber = form
            .pow_maxnumber
            .as_deref()
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| SubmitError::new("Confirm you are not a bot, then try again."))?;
        let exp = form
            .pow_exp
            .as_deref()
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| SubmitError::new("Confirm you are not a bot, then try again."))?;
        let cap = guard::verify_pow(
            form.pow_salt.as_deref().unwrap_or(""),
            form.pow_challenge.as_deref().unwrap_or(""),
            number,
            maxnumber,
            form.pow_signature.as_deref().unwrap_or(""),
            exp,
            form.pow_kind.as_deref().unwrap_or("media"),
            &guard::client_ip(req),
        )
        .map_err(|d| SubmitError::new(d.message))?;
        ticket = Some(guard::issue_ticket(&guard::client_ip(req), cap));
    }
    let id = parse_video_id(&form.url).ok_or_else(|| {
        SubmitError::new("Paste a YouTube URL or the 11-character video id.")
            .field("url", "Not a YouTube link")
    })?;
    let mode = Mode::parse(form.mode.as_deref().unwrap_or("text"));
    let mut out = redirect(app_href(&id, mode));
    if let Some(t) = ticket {
        out = out.with_cookie(guard::ticket_cookie_header(&t));
    }
    Ok(out)
}

#[load]
pub async fn audio_pick(req: &FlowRequest) -> std::result::Result<AudioMeta, LoaderError> {
    let raw = req.query_param("v").unwrap_or("");
    let id = parse_video_id(raw).ok_or_else(|| LoaderError::new(400, "Missing video id."))?;
    match crate::guard::check_req(req, crate::guard::PAGE) {
        Ok(()) => {}
        Err(m) => return Err(LoaderError::new(429, m)),
    }
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
    crate::guard::check_req(req, crate::guard::PAGE)
        .map_err(|m| LoaderError::new(429, m))?;
    match load_transcript(&id, lang, tlang).await {
        Ok(doc) => Ok((*doc).clone()),
        Err(e) => Err(LoaderError::new(e.status, e.message)),
    }
}
