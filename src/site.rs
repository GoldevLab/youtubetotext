//! Contact, measurement, and Search Console extras (all optional env).

use resuma::prelude::*;

pub fn contact_email() -> Option<String> {
    std::env::var("CONTACT_EMAIL")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| s.contains('@') && !s.contains(' '))
}

pub fn chrome_store_url() -> Option<String> {
    std::env::var("CHROME_STORE_URL")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| s.starts_with("https://"))
}

/// Send `youtubetotext.fly.dev` HTML traffic to the canonical domain.
/// Leaves `/health` and `/ready` alone (those are Resuma ops routes, not Flow).
#[middleware]
async fn redirect_legacy_fly_host(req: FlowRequest) -> Result<FlowRequest> {
    const LEGACY: &str = "youtubetotext.fly.dev";
    let host = req
        .header("x-forwarded-host")
        .or_else(|| req.header("host"))
        .unwrap_or("")
        .split(',')
        .next()
        .unwrap_or("")
        .trim()
        .split(':')
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    if host != LEGACY {
        return Ok(req);
    }
    let origin = crate::family::public_origin().trim_end_matches('/').to_string();
    if origin.is_empty() || origin.contains("fly.dev") {
        return Ok(req);
    }
    let path = if req.path.is_empty() {
        "/"
    } else {
        req.path.as_str()
    };
    let mut loc = format!("{origin}{path}");
    if !req.query.is_empty() {
        let mut ser = url::form_urlencoded::Serializer::new(String::new());
        for (k, v) in &req.query {
            ser.append_pair(k, v);
        }
        let qs = ser.finish();
        if !qs.is_empty() {
            loc.push('?');
            loc.push_str(&qs);
        }
    }
    Err(ResumaError::Redirect(loc))
}

pub fn head_extras() -> String {
    let mut out = String::new();
    if let Ok(v) = std::env::var("GSC_VERIFICATION") {
        let v = v.trim();
        if !v.is_empty()
            && v.len() < 120
            && v
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
        {
            out.push_str(&format!(
                r#"<meta name="google-site-verification" content="{v}" />"#
            ));
        }
    }
    if let Ok(id) = std::env::var("GA4_ID") {
        let id = id.trim();
        if id.starts_with("G-")
            && id.len() < 20
            && id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-')
        {
            out.push_str(&format!(
                r#"<link rel="dns-prefetch" href="https://www.googletagmanager.com" />
<script>
(function(){{
  var id={id:?};
  function load(){{
    if(window.__yttGaLoaded)return;
    window.__yttGaLoaded=1;
    var s=document.createElement('script');
    s.async=true;
    s.fetchPriority='low';
    s.src='https://www.googletagmanager.com/gtag/js?id='+id;
    s.onload=function(){{
      window.dataLayer=window.dataLayer||[];
      function gtag(){{dataLayer.push(arguments);}}
      window.gtag=gtag;
      gtag('js',new Date());
      gtag('config',id,{{send_page_view:true}});
    }};
    document.head.appendChild(s);
  }}
  function arm(){{
    var start=function(){{load();}};
    ['pointerdown','keydown','touchstart','scroll'].forEach(function(e){{
      window.addEventListener(e,start,{{once:true,passive:true}});
    }});
    if('requestIdleCallback' in window)requestIdleCallback(start,{{timeout:12000}});
    else window.addEventListener('load',function(){{setTimeout(start,8000);}},{{once:true}});
  }}
  if(document.readyState==='loading')document.addEventListener('DOMContentLoaded',arm,{{once:true}});
  else arm();
}})();
</script>"#
            ));
        }
    }
    if let Ok(domain) = std::env::var("PLAUSIBLE_DOMAIN") {
        let domain = domain.trim();
        if !domain.is_empty() && domain.len() < 80 && !domain.contains('<') {
            out.push_str(&format!(
                r#"<script defer data-domain="{domain}" src="https://plausible.io/js/script.js"></script>"#
            ));
        }
    }
    // Meta Pixel — deferred like GA4 so lab TBT stays green. Set META_PIXEL_ID (digits only).
    // When PRIVATE_PIXEL_TOKEN is also set, ViewContent is mirrored to CAPI with the same eventID.
    if let Ok(id) = std::env::var("META_PIXEL_ID") {
        let id = id.trim();
        if !id.is_empty()
            && id.len() <= 20
            && id.bytes().all(|b| b.is_ascii_digit())
        {
            out.push_str(&format!(
                r#"<link rel="dns-prefetch" href="https://connect.facebook.net" />
<link rel="dns-prefetch" href="https://www.facebook.com" />
<script>
(function(){{
  var id={id:?};
  function cookie(n){{
    try{{
      var parts=document.cookie.split(';');
      for(var i=0;i<parts.length;i++){{
        var p=parts[i].trim();
        if(p.indexOf(n+'=')===0)return decodeURIComponent(p.slice(n.length+1));
      }}
    }}catch(_){{}}
    return '';
  }}
  function eventId(){{
    try{{if(crypto.randomUUID)return crypto.randomUUID();}}catch(_){{}}
    return 'vc_'+Date.now().toString(36)+Math.random().toString(36).slice(2,10);
  }}
  function sendCapi(payload){{
    try{{
      var body=JSON.stringify(payload);
      fetch('/api/meta/view-content',{{
        method:'POST',
        headers:{{'content-type':'application/json'}},
        body:body,
        credentials:'same-origin',
        keepalive:true
      }}).catch(function(){{}});
    }}catch(_){{}}
  }}
  function trackViewContent(){{
    var v='';
    try{{v=new URLSearchParams(location.search).get('v')||'';}}catch(_){{}}
    if(!v||typeof fbq!=='function')return;
    var eid=eventId();
    try{{
      fbq('track','ViewContent',{{
        content_name:'transcript',
        content_type:'product',
        content_ids:[v]
      }},{{eventID:eid}});
    }}catch(_){{}}
    sendCapi({{
      event_id:eid,
      v:v,
      event_source_url:String(location.href||'').slice(0,2048),
      fbp:cookie('_fbp')||undefined,
      fbc:cookie('_fbc')||undefined
    }});
  }}
  window.__yttMetaViewContent=trackViewContent;
  function boot(){{
    if(window.__yttMetaPixel)return;
    window.__yttMetaPixel=1;
    !function(f,b,e,v,n,t,s){{if(f.fbq)return;n=f.fbq=function(){{n.callMethod?
    n.callMethod.apply(n,arguments):n.queue.push(arguments)}};if(!f._fbq)f._fbq=n;
    n.push=n;n.loaded=!0;n.version='2.0';n.queue=[];t=b.createElement(e);t.async=!0;
    t.fetchPriority='low';t.src=v;s=b.getElementsByTagName(e)[0];
    s.parentNode.insertBefore(t,s)}}(window,document,'script',
    'https://connect.facebook.net/en_US/fbevents.js');
    fbq('init',id);
    fbq('track','PageView');
    trackViewContent();
  }}
  function arm(){{
    var start=function(){{boot();}};
    ['pointerdown','keydown','touchstart','scroll'].forEach(function(e){{
      window.addEventListener(e,start,{{once:true,passive:true}});
    }});
    if('requestIdleCallback' in window)requestIdleCallback(start,{{timeout:12000}});
    else window.addEventListener('load',function(){{setTimeout(start,8000);}},{{once:true}});
  }}
  if(document.readyState==='loading')document.addEventListener('DOMContentLoaded',arm,{{once:true}});
  else arm();
}})();
</script>"#
            ));
        }
    }
    out
}
