//! Custom `/sitemap.xml` with xhtml hreflang pairs for EN/ES landings.

use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::IntoResponse;

use crate::family::{canonical_url, Mode};

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn url_entry(loc: &str, priority: &str, changefreq: &str, alternates: &[(&str, &str)]) -> String {
    let mut out = String::from("<url><loc>");
    out.push_str(&escape_xml(loc));
    out.push_str("</loc>");
    for (lang, href) in alternates {
        out.push_str(r#"<xhtml:link rel="alternate" hreflang=""#);
        out.push_str(lang);
        out.push_str(r#"" href=""#);
        out.push_str(&escape_xml(href));
        out.push_str(r#""/>"#);
    }
    out.push_str("<changefreq>");
    out.push_str(changefreq);
    out.push_str("</changefreq><priority>");
    out.push_str(priority);
    out.push_str("</priority></url>");
    out
}

pub fn sitemap_body() -> String {
    let home = canonical_url("/");
    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?><urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9" xmlns:xhtml="http://www.w3.org/1999/xhtml">"#,
    );
    xml.push_str(&url_entry(&home, "1.0", "weekly", &[]));

    for mode in Mode::all() {
        let en = canonical_url(mode.landing_path());
        let es = canonical_url(mode.es_path());
        let alts = [("en", en.as_str()), ("es", es.as_str()), ("x-default", en.as_str())];
        xml.push_str(&url_entry(&en, "0.9", "monthly", &alts));
        xml.push_str(&url_entry(&es, "0.9", "monthly", &alts));
    }

    for path in [
        "/extension",
        "/pricing",
        "/developers",
        "/privacy",
        "/terms",
    ] {
        xml.push_str(&url_entry(
            &canonical_url(path),
            "0.6",
            "monthly",
            &[],
        ));
    }

    xml.push_str("</urlset>");
    xml
}

pub async fn sitemap() -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/xml; charset=utf-8"),
    );
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=3600"),
    );
    (StatusCode::OK, headers, sitemap_body())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sitemap_lists_es_and_hreflang() {
        let body = sitemap_body();
        assert!(body.contains("/youtube-a-texto"));
        assert!(body.contains("/youtube-to-text"));
        assert!(body.contains(r#"hreflang="es""#));
        assert!(body.contains(r#"hreflang="x-default""#));
        assert!(!body.contains("/developers/welcome"));
    }
}
