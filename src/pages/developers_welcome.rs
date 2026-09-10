use resuma::prelude::*;

use crate::family::canonical_url;

pub fn page(req: FlowRequest) -> View {
    set_page_title("Your API key | YouTubeForge");
    set_page_description("Reveal your YouTubeForge API key after Lemon Squeezy checkout.");
    set_page_canonical(canonical_url("/developers/welcome"));
    set_page_robots("noindex, nofollow");

    let token = req.query_param("t").unwrap_or("").to_string();
    let has_token = token.starts_with("ft_") && token.len() >= 12;
    let token_attr = if has_token { token } else { String::new() };

    let body = if has_token {
        view! {
            <div class="welcome-panel" data-forge-welcome="" data-forge-token={token_attr}>
                <p class="hint" data-welcome-status="">"Confirming payment and provisioning your key…"</p>
                <pre class="code-block" data-welcome-key="" hidden=""></pre>
                <p class="hint" data-welcome-usage="" hidden=""></p>
                <p class="error-actions">
                    <NavLink href="/developers" class="btn btn-primary">"API docs"</NavLink>
                    <NavLink href="/pricing" class="btn btn-ghost">"Plans"</NavLink>
                </p>
            </div>
        }
    } else {
        view! {
            <p class="hint">
                "Missing checkout token. Start from "
                <NavLink href="/pricing">"Pricing"</NavLink>
                " or open the redirect link in your Lemon Squeezy receipt."
            </p>
        }
    };

    view! {
        <main class="content-section docs-page">
            <p class="eyebrow">"Welcome"</p>
            <h1>"Your API key"</h1>
            <p class="hero-lead">"Payment is processed by Lemon Squeezy. We only show the secret key once."</p>
            {body}
        </main>
    }
}
