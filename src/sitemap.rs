//! Custom `/sitemap.xml` for English landings and static pages.

use crate::family::{canonical_url, Mode};

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn url_entry(loc: &str, priority: &str, changefreq: &str) -> String {
    let mut out = String::from("<url><loc>");
    out.push_str(&escape_xml(loc));
    out.push_str("</loc><changefreq>");
    out.push_str(changefreq);
    out.push_str("</changefreq><priority>");
    out.push_str(priority);
    out.push_str("</priority></url>");
    out
}

pub fn sitemap_body() -> String {
    let home = canonical_url("/");
    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?><urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">"#,
    );
    xml.push_str(&url_entry(&home, "1.0", "weekly"));

    for mode in Mode::all() {
        xml.push_str(&url_entry(
            &canonical_url(mode.landing_path()),
            "0.9",
            "monthly",
        ));
    }

    for path in [
        "/extension",
        "/pricing",
        "/developers",
        "/privacy",
        "/terms",
    ] {
        xml.push_str(&url_entry(&canonical_url(path), "0.6", "monthly"));
    }

    xml.push_str("</urlset>");
    xml
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sitemap_lists_english_landings_only() {
        let body = sitemap_body();
        assert!(body.contains("/youtube-to-text"));
        assert!(body.contains("/youtube-to-audio"));
        assert!(!body.contains("/youtube-a-texto"));
        assert!(!body.contains("hreflang"));
        assert!(!body.contains("/developers/welcome"));
    }
}
