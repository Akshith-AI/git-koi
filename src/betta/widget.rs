/// JavaScript for the standalone, self-hostable website widget.
pub const WIDGET_SCRIPT: &str = include_str!("web/widget.js");

/// Generate a website widget using only local assets. Caller text is escaped.
pub fn widget_html(username: &str, pond: bool) -> String {
    let heading = username
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;");
    include_str!("web/embed.html")
        .replace("MODE", if pond { "pond" } else { "betta" })
        .replace("STILL", if pond { "pond" } else { "betta" })
        .replace("ANIMATION", if pond { "pond" } else { "betta" })
        .replace("HEADING", &heading)
}
