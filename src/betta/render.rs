use super::BettaDevelopment;
use crate::render::{FRAME_WIDTH_PIXELS, KoiFrame};

type Point = [f64; 2];
#[derive(Clone, Copy)]
pub(super) enum Region {
    Tail,
    Dorsal,
    Anal,
    Ventral,
    Body,
    Pectoral,
    Gill,
}

/// Draw a right-facing male Halfmoon betta in a fully spread presentation pose.
/// The caudal fin is a 180-degree fan. All growth stages retain the same anatomy.
/// Output is a transparent 256-square native frame, compatible with existing encoders.
pub fn render_betta(development: &BettaDevelopment) -> KoiFrame {
    render_pose(development, None)
}

/// Render a swimming pose with fins and gill covers driven by the flare state.
pub fn render_betta_frame(development: &BettaDevelopment, pose: super::BettaPose) -> KoiFrame {
    render_pose(development, Some(pose))
}

fn render_pose(development: &BettaDevelopment, pose: Option<super::BettaPose>) -> KoiFrame {
    let scale = 0.64 + 0.36 * development.growth;
    let dorsal = outline(
        [119., 111.],
        &[
            [[110., 90.], [102., 56.], [115., 44.]],
            [[136., 35.], [166., 47.], [185., 75.]],
            [[194., 91.], [195., 101.], [202., 110.]],
        ],
    );
    let anal = outline(
        [111., 130.],
        &[
            [[108., 157.], [113., 190.], [126., 202.]],
            [[143., 211.], [163., 208.], [176., 196.]],
            [[188., 180.], [191., 159.], [198., 144.]],
            [[177., 131.], [137., 129.], [111., 130.]],
        ],
    );
    let body = outline(
        [109., 114.],
        &[
            [[134., 110.], [140., 90.], [170., 94.]],
            [[194., 95.], [207., 105.], [219., 111.]],
            [[226., 112.], [230., 119.], [224., 125.]],
            [[215., 131.], [209., 143.], [189., 147.]],
            [[153., 154.], [138., 138.], [109., 137.]],
        ],
    );
    let ventral = outline(
        [195., 142.],
        &[
            [[192., 160.], [177., 192.], [165., 205.]],
            [[174., 183.], [183., 157.], [189., 143.]],
        ],
    );
    let ventral_back = outline(
        [184., 143.],
        &[
            [[181., 164.], [162., 190.], [153., 199.]],
            [[164., 179.], [173., 157.], [179., 143.]],
        ],
    );
    let pectoral = outline(
        [184., 122.],
        &[
            [[176., 128.], [164., 143.], [168., 155.]],
            [[181., 155.], [191., 138.], [184., 122.]],
        ],
    );
    let mut pixels = vec![[0; 4]; (FRAME_WIDTH_PIXELS * FRAME_WIDTH_PIXELS) as usize];
    for y in 0..FRAME_WIDTH_PIXELS {
        for x in 0..FRAME_WIDTH_PIXELS {
            let mut point = if let Some(pose) = pose {
                let x = f64::from(x) + 0.5 - 128. - pose.dx;
                let y = f64::from(y) + 0.5 - 128. - pose.dy;
                let (sine, cosine) = pose.pitch.sin_cos();
                [
                    (cosine * x + sine * y) / pose.projection / (scale * pose.scene_scale) + 128.,
                    (-sine * x + cosine * y) / (scale * pose.scene_scale) + 128.,
                ]
            } else {
                [
                    (f64::from(x) + 0.5 - 128.) / scale + 128.,
                    (f64::from(y) + 0.5 - 128.) / scale + 128.,
                ]
            };
            if let Some(pose) = pose {
                let lag = ((174. - point[0]) / 140.).clamp(0., 1.);
                let stroke = (pose.phase * 4. - lag * 2.8).sin();
                point[1] -= stroke * lag.powi(2) * 7. * pose.energy;
            }
            let [px, py] = point;
            let tail_point = fin_sample(point, Region::Tail, pose);
            let angle = (tail_point[1] - 126.).atan2(tail_point[0] - 113.);
            let radius = (tail_point[0] - 113.).hypot(tail_point[1] - 126.);
            let edge = 86. + 0.9 * (angle * 36.).cos();
            let mut region = if px <= 113. && radius < edge {
                Some(Region::Tail)
            } else {
                None
            };
            let mut pigment_point = tail_point;
            let mut pectoral_underlay = None;
            for (shape, part) in [
                (&dorsal, Region::Dorsal),
                (&anal, Region::Anal),
                (&ventral_back, Region::Ventral),
                (&ventral, Region::Ventral),
                (&body, Region::Body),
                (&pectoral, Region::Pectoral),
            ] {
                let sample = fin_sample(point, part, pose);
                if contains(shape, sample) {
                    if matches!(part, Region::Pectoral) {
                        pectoral_underlay = region
                            .map(|under| super::marble::color(development, pigment_point, under));
                    }
                    region = Some(part);
                    pigment_point = sample;
                }
            }
            if let Some(pose) = pose {
                let beard = [
                    [192., 132.],
                    [201., 141.],
                    [187., 142. + 16. * pose.flare],
                    [182., 142.],
                ];
                if pose.flare > 0.05 && contains(&beard, point) {
                    region = Some(Region::Gill);
                }
            }
            if let Some(region) = region {
                let mut color = super::marble::color(development, pigment_point, region);
                if matches!(region, Region::Pectoral)
                    && let Some(underlay) = pectoral_underlay
                {
                    // Simulate a thin translucent membrane with opaque RGB.
                    for axis in 0..3 {
                        color[axis] =
                            ((u16::from(color[axis]) + 3 * u16::from(underlay[axis])) / 4) as u8;
                    }
                }
                // A dark operculum arc and small upturned mouth distinguish the head.
                let gill = ((px - 194.) / 10.).powi(2) + ((py - 122.) / 21.).powi(2);
                if matches!(region, Region::Body) && px < 195. && (0.88..1.08).contains(&gill) {
                    color = [49, 49, 71, 255];
                }
                if matches!(region, Region::Body)
                    && px > 221.
                    && (py - (119. + (px - 222.) * 0.25)).abs() < 0.65
                {
                    color = [38, 28, 43, 255];
                }
                let eye = (px - 213.).hypot(py - 113.);
                if matches!(region, Region::Body) && eye < 3.5 {
                    color = if eye > 2.6 {
                        [181, 150, 118, 255]
                    } else {
                        [14, 21, 33, 255]
                    };
                }
                if (px - 212.3).hypot(py - 112.) < 0.85 {
                    color = [251, 251, 244, 255];
                }
                pixels[(y * FRAME_WIDTH_PIXELS + x) as usize] = color;
            }
        }
    }
    KoiFrame::from_rgba_pixels(pixels)
}

// Inverse, root-anchored deformations let each fin move independently while its
// pigment and rays remain attached to its membrane. Motion grows toward the tips.
fn fin_sample([x, y]: Point, region: Region, pose: Option<super::BettaPose>) -> Point {
    let Some(p) = pose else {
        return [x, y];
    };
    let spread = 0.72 + 0.28 * p.flare;
    let strength = 0.55 + 0.45 * p.energy;
    let beat = p.phase * 3.;
    match region {
        Region::Tail => {
            let trail = ((113. - x) / 86.).clamp(0., 1.);
            let pulse = 1. + 0.10 * (beat - 0.9).sin() * strength;
            [
                113. + (x - 113.) / spread - 4. * (beat - trail * 2.4).sin() * trail * strength,
                126. + (y - 126.) / (spread * pulse)
                    - 14. * (beat - trail * 2.4).sin() * trail.powf(1.3) * strength,
            ]
        }
        Region::Dorsal => {
            let tip = ((106. - y) / 65.).clamp(0., 1.);
            [
                x - 10. * (beat - tip * 2.).sin() * tip * strength,
                106. + (y - 106.) / (spread * (1. + 0.11 * (beat - tip + 0.7).sin() * strength)),
            ]
        }
        Region::Anal => {
            let tip = ((y - 140.) / 72.).clamp(0., 1.);
            [
                x - 11. * (beat - tip * 2.2 + 1.6).sin() * tip * strength,
                140. + (y - 140.) / (spread * (1. + 0.12 * (beat - tip + 2.1).sin() * strength)),
            ]
        }
        Region::Ventral => {
            let tip = ((y - 143.) / 62.).clamp(0., 1.);
            [
                x - 8. * (beat - tip * 2.8 + 2.4).sin() * tip * strength,
                143. + (y - 143.) / spread,
            ]
        }
        Region::Pectoral => {
            let flap = 0.72 + 0.28 * (p.phase * 6.).cos();
            [
                184. + (x - 184.) / flap,
                122. + (y - 122.) / (0.9 + 0.1 * (p.phase * 6. + 0.8).sin()),
            ]
        }
        Region::Body | Region::Gill => [x, y],
    }
}

fn outline(start: Point, curves: &[[Point; 3]]) -> Vec<Point> {
    let mut points = vec![start];
    let mut from = start;
    for &[a, b, end] in curves {
        for step in 1..=20 {
            let t = f64::from(step) / 20.;
            let u = 1. - t;
            points.push([0, 1].map(|axis| {
                u * u * u * from[axis]
                    + 3. * u * u * t * a[axis]
                    + 3. * u * t * t * b[axis]
                    + t * t * t * end[axis]
            }));
        }
        from = end;
    }
    points
}

fn contains(polygon: &[Point], [x, y]: Point) -> bool {
    let mut inside = false;
    let mut previous = polygon[polygon.len() - 1];
    for &point in polygon {
        if (point[1] > y) != (previous[1] > y)
            && x < (previous[0] - point[0]) * (y - point[1]) / (previous[1] - point[1]) + point[0]
        {
            inside = !inside;
        }
        previous = point;
    }
    inside
}
