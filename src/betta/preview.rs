use crate::leetcode::LeetCodeSnapshot;

/// Build a standalone still-image preview with a metric legend and escaped text.
/// The page has no scripts, network requests, or hover interaction in stage 3.
pub fn betta_preview(snapshot: &LeetCodeSnapshot) -> String {
    let username = escape(&snapshot.username);
    format!(
        r#"<!doctype html>
<html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>{username} · LTKoi betta</title><style>
body{{margin:0;background:#101a25;color:#edf1f2;font:16px/1.6 system-ui,sans-serif}}
main{{max-width:960px;margin:auto;padding:32px}}h1{{font-size:28px;margin-bottom:4px}}
.panels{{display:flex;flex-wrap:wrap;gap:16px}}figure{{margin:0;border-radius:16px;padding:16px;flex:1;min-width:240px}}
img{{display:block;width:100%;max-width:512px;image-rendering:pixelated}}.light{{background:#f5f1e7;color:#263341}}.dark{{background:#07121c}}
code,a{{color:#9dd9dc}}small{{color:#bccbd3}}
</style></head><body><main><small>LTKOI / LEETCODE BETTA</small>
<h1>{username}</h1><p>Male Halfmoon · Koi marble · Still portrait</p>
<div class="panels"><figure class="dark"><img src="betta.png" alt="Koi-marble Halfmoon male betta on dark background"><figcaption>Dark background</figcaption></figure>
<figure class="light"><img src="betta.png" alt="The same betta on light background"><figcaption>Light background</figcaption></figure></div>
<p><strong>{total} unique solves</strong> · {easy} Easy / {medium} Medium / {hard} Hard</p>
<p>Growth follows unique solves. Easy favors ivory and gold, Medium crimson and tangerine, Hard indigo.
Topic proportions warp the marble patches; these are overlapping tag counts, not a disjoint breakdown.</p>
<p>Blank-topic or restricted-topic accounts keep identity-based marbling. Calendar availability does not shrink the fish.</p>
<p><a href="betta.svg">SVG portrait</a> · <a href="traits.json">Visual traits</a></p>
<small>Observed {date} UTC. Still renderer only; animation, shared pond and hover flaring are planned.
Koi pipeline adapted from Anthony Yalong's git-koi (MIT); betta geometry and marbling added by LTKoi.</small>
</main></body></html>"#,
        total = snapshot.solved.all,
        easy = snapshot.solved.easy,
        medium = snapshot.solved.medium,
        hard = snapshot.solved.hard,
        date = snapshot.observed_on
    )
}

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
