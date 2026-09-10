use resuma::prelude::*;

use crate::billing::{billing_ready, Plan};
use crate::family::canonical_url;

pub fn page(_req: FlowRequest) -> View {
    set_page_title("Pricing | YouTubeForge API");
    set_page_description(
        "Free web transcripts with ads. Paid API: Basic $9/mo (5,000 transcripts) or Pro $29/mo (25,000). Powered by Lemon Squeezy.",
    );
    set_page_canonical(canonical_url("/pricing"));

    let ready = billing_ready();
    let basic_cta = if ready {
        view! {
            <a class="btn btn-primary" href="/api/billing/checkout?plan=basic">"Subscribe Basic — $9/mo"</a>
        }
    } else {
        view! {
            <span class="btn btn-primary is-disabled" aria-disabled="true">"Coming soon"</span>
        }
    };
    let pro_cta = if ready {
        view! {
            <a class="btn btn-primary" href="/api/billing/checkout?plan=pro">"Subscribe Pro — $29/mo"</a>
        }
    } else {
        view! {
            <span class="btn btn-primary is-disabled" aria-disabled="true">"Coming soon"</span>
        }
    };
    let notice = if ready {
        view! { <p class="hint">"Checkout is handled by Lemon Squeezy (Merchant of Record). After payment you get an API key on the welcome page — copy it once."</p> }
    } else {
        view! { <p class="hint">"Subscriptions are almost ready. The free web tool stays available. Follow /developers for the API shape."</p> }
    };

    view! {
        <main class="content-section pricing-page">
            <p class="eyebrow">"YouTubeForge"</p>
            <h1>"Simple API pricing"</h1>
            <p class="hero-lead">
                "More transcripts per dollar than typical competitors. Audio and video stay hard-capped so the service stays fast and solvent."
            </p>
            {notice}
            <ul class="pricing-grid">
                <li class="pricing-card">
                    <h2>"Free web"</h2>
                    <p class="price">"$0"</p>
                    <p>"Paste on forgeyt.com. Ads may appear. Rate limits by network. No public API key."</p>
                    <ul class="pricing-features">
                        <li>"Unlimited casual transcripts in the browser"</li>
                        <li>"SRT / VTT / TXT / Markdown export"</li>
                        <li>"Audio & video with daily caps + bot check"</li>
                    </ul>
                    <NavLink href="/" class="btn btn-ghost">"Open the tool"</NavLink>
                </li>
                <li class="pricing-card is-featured">
                    <h2>{Plan::Basic.label()}</h2>
                    <p class="price">{format!("${}/mo", Plan::Basic.price_usd())}</p>
                    <p>{format!("{} transcripts / month · {}/min", Plan::Basic.transcripts_month(), Plan::Basic.rate_per_min())}</p>
                    <ul class="pricing-features">
                        <li>{format!("{} caption fetches (JSON / SRT / VTT / TXT / MD)", Plan::Basic.transcripts_month())}</li>
                        <li>{format!("{} translate calls", Plan::Basic.translates_month())}</li>
                        <li>{format!("{} audio downloads", Plan::Basic.audio_month())}</li>
                        <li>"No video API (use the web tool)"</li>
                        <li>"x-api-key · no ads · no PoW"</li>
                    </ul>
                    {basic_cta}
                </li>
                <li class="pricing-card">
                    <h2>{Plan::Pro.label()}</h2>
                    <p class="price">{format!("${}/mo", Plan::Pro.price_usd())}</p>
                    <p>{format!("{} transcripts / month · {}/min", Plan::Pro.transcripts_month(), Plan::Pro.rate_per_min())}</p>
                    <ul class="pricing-features">
                        <li>{format!("{} caption fetches", Plan::Pro.transcripts_month())}</li>
                        <li>{format!("{} translate calls", Plan::Pro.translates_month())}</li>
                        <li>{format!("{} audio · {} SD video · {}×720p", Plan::Pro.audio_month(), Plan::Pro.video_sd_month(), Plan::Pro.video_720_month())}</li>
                        <li>"HD/4K stays web-only"</li>
                        <li>"x-api-key · higher rate limits"</li>
                    </ul>
                    {pro_cta}
                </li>
            </ul>
            <p class="hint">
                <NavLink href="/developers">"API docs"</NavLink>
                " · Compare: typical rivals sell ~1k transcripts near $10. Basic gives 5k for $9."
            </p>
        </main>
    }
}
