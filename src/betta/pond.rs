use super::BETTA_FRAMES;
use crate::render::{FRAME_WIDTH_PIXELS, KoiFrame};
use std::f64::consts::TAU;

/// Composite fish in separate swimming lanes, on transparent surroundings.
/// Each supplied frame retains its own progression, palette and swimming behavior.
/// Frames wrap along a shared four-second gentle lateral course without crossing lanes.
pub fn compose_pond(betta: &KoiFrame, koi: &KoiFrame, frame: u32) -> KoiFrame {
    let phase = f64::from(frame % BETTA_FRAMES) / f64::from(BETTA_FRAMES) * TAU;
    let mut pixels = vec![[0; 4]; (FRAME_WIDTH_PIXELS * FRAME_WIDTH_PIXELS) as usize];
    // Fixed full-frame bounds have a gap even at maximum growth and movement.
    paste(
        &mut pixels,
        betta,
        [
            128. + 48. * (phase + 0.16 * (phase * 2.).sin()).sin(),
            64. + 5. * phase.cos(),
        ],
        false,
    );
    paste(
        &mut pixels,
        koi,
        [128. - 48. * phase.sin(), 192. - 5. * phase.cos()],
        true,
    );
    KoiFrame::from_rgba_pixels(pixels)
}

fn paste(destination: &mut [[u8; 4]], frame: &KoiFrame, center: [f64; 2], mirror: bool) {
    let side = FRAME_WIDTH_PIXELS as usize;
    // 112 native pixels wide; each full frame occupies its own 112-pixel lane.
    for y in 0..112_usize {
        for x in 0..112_usize {
            let source_x = if mirror { 111 - x } else { x } * side / 112;
            let source_y = y * side / 112;
            let source = (source_y * side + source_x) * 4;
            let pixel = &frame.as_rgba_bytes()[source..source + 4];
            let dx = center[0].round() as i32 + x as i32 - 56;
            let dy = center[1].round() as i32 + y as i32 - 56;
            if pixel[3] == 255 && (0..side as i32).contains(&dx) && (0..side as i32).contains(&dy) {
                destination[dy as usize * side + dx as usize].copy_from_slice(pixel);
            }
        }
    }
}
