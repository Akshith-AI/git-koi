use crate::output::{GifEncodingError, encode_gif};
use crate::render::{FRAME_WIDTH_PIXELS, KoiFrame};
use std::collections::BTreeMap;

/// Encode animated pixel art with a deterministic per-frame 255-color median-cut palette.
/// The original lossless koi encoder remains unchanged. PNG/atlas output keeps full color.
pub fn encode_art_gif(frames: &[KoiFrame]) -> Result<Vec<u8>, GifEncodingError> {
    let quantized: Vec<_> = frames.iter().map(quantize).collect();
    encode_gif(&quantized)
}

fn quantize(frame: &KoiFrame) -> KoiFrame {
    let mut histogram = BTreeMap::<[u8; 3], usize>::new();
    for p in frame.as_rgba_bytes().chunks_exact(4).filter(|p| p[3] != 0) {
        *histogram.entry([p[0], p[1], p[2]]).or_default() += 1;
    }
    if histogram.len() <= 255 {
        return KoiFrame::from_rgba_pixels(
            frame
                .as_rgba_bytes()
                .chunks_exact(4)
                .map(|p| [p[0], p[1], p[2], p[3]])
                .collect(),
        );
    }
    let mut buckets = vec![
        histogram
            .iter()
            .map(|(&rgb, &count)| (rgb, count))
            .collect::<Vec<_>>(),
    ];
    while buckets.len() < 255 {
        let Some(index) = buckets
            .iter()
            .enumerate()
            .filter(|(_, b)| b.len() > 1)
            .max_by_key(|(_, b)| spread(b).1 * b.iter().map(|(_, n)| n).sum::<usize>())
            .map(|(i, _)| i)
        else {
            break;
        };
        let mut bucket = buckets.swap_remove(index);
        let channel = spread(&bucket).0;
        bucket.sort_by_key(|(rgb, _)| (rgb[channel], *rgb));
        let total: usize = bucket.iter().map(|(_, n)| n).sum();
        let mut sum = 0;
        let split = bucket
            .iter()
            .position(|(_, n)| {
                sum += n;
                sum >= total.div_ceil(2)
            })
            .unwrap_or(0)
            .saturating_add(1)
            .clamp(1, bucket.len() - 1);
        let rest = bucket.split_off(split);
        buckets.push(bucket);
        buckets.push(rest);
    }
    let mut mapping = BTreeMap::new();
    for bucket in buckets {
        let total: usize = bucket.iter().map(|(_, n)| n).sum();
        let color = [0, 1, 2].map(|axis| {
            (bucket
                .iter()
                .map(|(rgb, n)| usize::from(rgb[axis]) * n)
                .sum::<usize>()
                / total) as u8
        });
        for (rgb, _) in bucket {
            mapping.insert(rgb, color);
        }
    }
    KoiFrame::from_rgba_pixels(
        frame
            .as_rgba_bytes()
            .chunks_exact(4)
            .map(|p| {
                if p[3] == 0 {
                    [0; 4]
                } else {
                    let c = mapping[&[p[0], p[1], p[2]]];
                    [c[0], c[1], c[2], p[3]]
                }
            })
            .collect(),
    )
}

fn spread(bucket: &[([u8; 3], usize)]) -> (usize, usize) {
    (0..3)
        .map(|axis| {
            let min = bucket.iter().map(|(c, _)| c[axis]).min().unwrap_or(0);
            let max = bucket.iter().map(|(c, _)| c[axis]).max().unwrap_or(0);
            (axis, usize::from(max - min))
        })
        .max_by_key(|&(axis, range)| (range, axis))
        .unwrap()
}

/// Encode a grid of unscaled native frames. Frames must match the 256-square renderer.
pub fn encode_sprite_atlas(frames: &[KoiFrame]) -> Result<Vec<u8>, png::EncodingError> {
    let columns = 10_usize;
    let rows = frames.len().max(1).div_ceil(columns);
    let side = FRAME_WIDTH_PIXELS as usize;
    let mut pixels = vec![0; columns * side * rows * side * 4];
    for (i, frame) in frames.iter().enumerate() {
        for y in 0..side {
            let destination = ((i / columns * side + y) * columns * side + i % columns * side) * 4;
            pixels[destination..destination + side * 4]
                .copy_from_slice(&frame.as_rgba_bytes()[y * side * 4..(y + 1) * side * 4]);
        }
    }
    let mut bytes = Vec::new();
    let mut encoder = png::Encoder::new(&mut bytes, (columns * side) as u32, (rows * side) as u32);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.write_header()?.write_image_data(&pixels)?;
    Ok(bytes)
}
