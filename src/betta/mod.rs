//! Deterministic male Halfmoon betta development and still-image rendering.
mod development;
pub use development::{BettaDevelopment, derive_betta_development};

mod render;
pub use render::render_betta;
