//! Deterministic male Halfmoon betta development and still-image rendering.
mod development;
pub use development::{BettaDevelopment, derive_betta_development};

mod render;
pub use render::{render_betta, render_betta_frame};

mod marble;

mod preview;
pub use preview::betta_preview;

mod motion;
pub use motion::{
    BETTA_FRAMES, BettaActivity, BettaPose, FLARE_LEVELS, betta_activity, betta_pose,
};
mod animation;
pub use animation::{encode_art_gif, encode_sprite_atlas};
mod pond;
pub use pond::compose_pond;
mod widget;
pub use widget::{WIDGET_SCRIPT, widget_html};
mod export;
pub use export::animation_files;
