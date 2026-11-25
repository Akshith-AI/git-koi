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
            [[111., 90.], [117., 56.], [135., 45.]],
            [[150., 34.], [175., 47.], [185., 75.]],
            [[194., 91.], [195., 101.], [202., 110.]],
        ],
    );
    let anal = outline(
        [111., 130.],
        &[
            [[110., 157.], [113., 193.], [132., 211.]],
            [[146., 220.], [179., 191.], [198., 144.]],
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
        [197., 142.],
        &[
            [[194., 160.], [178., 191.], [165., 205.]],
            [[173., 182.], [179., 158.], [185., 143.]],
        ],
    );
    let ventral_back = outline(
        [184., 143.],
        &[
            [[182., 163.], [164., 191.], [153., 199.]],
            [[162., 176.], [169., 156.], [176., 142.]],
        ],
    );
    let pectoral = outline(
        [184., 122.],
        &[
            [[174., 126.], [160., 147.], [168., 163.]],
            [[184., 161.], [194., 140.], [184., 122.]],
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
            let spread = pose.map_or(1., |p| 0.72 + 0.28 * p.flare);
            let flutter = pose.map_or(1., |p| {
                1. + (0.03 * (p.phase * 4. - px * 0.045).sin()
                    + 0.012 * (p.phase * 8. - px * 0.06).sin())
                    * p.energy
            });
            let fin_point = [px, 126. + (py - 126.) / (spread * flutter)];
            let tail_point = [113. + (px - 113.) / spread, fin_point[1]];
            let angle = (tail_point[1] - 126.).atan2(tail_point[0] - 113.);
            let radius = (tail_point[0] - 113.).hypot(tail_point[1] - 126.);
            let edge = 86. + 0.9 * (angle * 36.).cos();
            let mut region = if px <= 113. && radius < edge {
                Some(Region::Tail)
            } else {
                None
            };
            let mut pigment_point = tail_point;
            for (shape, part) in [
                (&dorsal, Region::Dorsal),
                (&anal, Region::Anal),
                (&ventral_back, Region::Ventral),
                (&ventral, Region::Ventral),
                (&body, Region::Body),
                (&pectoral, Region::Pectoral),
            ] {
                let sample = if matches!(part, Region::Body) {
                    point
                } else {
                    fin_point
                };
                if contains(shape, sample) {
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
