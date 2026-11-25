use super::BettaDevelopment;
use super::development::stable_seed;
use super::render::Region;

pub(super) fn color(development: &BettaDevelopment, [x, y]: [f64; 2], region: Region) -> [u8; 4] {
    let seed = development.identity_seed;
    let mut warp_x = 0.;
    let mut warp_y = 0.;
    for (slug, weight) in &development.topic_weights {
        let tag = stable_seed(slug);
        let angle = (tag % 6283) as f64 / 1000.;
        let frequency = 0.035 + (tag % 7) as f64 * 0.006;
        let wave =
            (x * angle.cos() * frequency + y * angle.sin() * frequency + (tag % 97) as f64).sin();
        warp_x += weight * wave * 19. * angle.cos();
        warp_y += weight * wave * 19. * angle.sin();
    }
    let px = x + warp_x;
    let py = y + warp_y;
    let field = 0.72 * noise(px * 0.045, py * 0.045, seed)
        + 0.21 * noise(px * 0.103, py * 0.103, seed ^ 0x1234)
        + 0.07 * noise(px * 0.24, py * 0.24, seed ^ 0x5678);
    let pigment = ((field - 0.22) / 0.56).clamp(0., 1.);
    let shares = if development.solved == 0 {
        [0.45, 0.40, 0.15]
    } else {
        development.difficulty_shares
    };
    let white = 0.20 + 0.55 * shares[0];
    let red = 0.20 + 0.55 * shares[1];
    let mut base = if pigment < white {
        [233., 229., 215.]
    } else if pigment < white + red {
        [205., 48., 45.]
    } else {
        [40., 65., 102.]
    };
    // Warm patches share the coherent marble field: broad gold and tangerine
    // islands rather than isolated bright pixels. Difficulty still sets the base mix.
    let warmth = noise(px * 0.038 + 11., py * 0.038 - 7., seed ^ 0xa19f);
    if pigment >= white && pigment < white + red && warmth > 0.46 {
        base = if warmth > 0.66 {
            [239., 187., 63.]
        } else {
            [233., 119., 36.]
        };
    } else if pigment < white && warmth > 0.67 {
        base = [238., 198., 82.];
    }
    // Fine dark islands break large patches without using uncorrelated pixel noise.
    if noise(px * 0.14, py * 0.14, seed ^ 0x8877) > 0.85 {
        base = [24., 32., 53.];
    }
    let angle = (y - 126.).atan2(x - 113.);
    let radius = (x - 113.).hypot(y - 126.);
    let mut shade = match region {
        Region::Gill => {
            base = [44., 28., 49.];
            1.
        }
        Region::Body => {
            let belly = ((y - 118.) / 31.).abs().min(1.);
            let scales = ((x + ((y / 3.).floor() % 2.) * 2.) * 1.55).cos() * (y * 2.1).cos();
            1.04 - 0.23 * belly + 0.018 * scales
        }
        Region::Tail => {
            let fold = (angle * 36. + radius * 0.015).cos();
            if radius > 78. && pigment < white + 0.18 {
                base = [208., 223., 233.];
            }
            0.86 + 0.10 * fold.abs() + 0.05 * (radius / 86.)
        }
        Region::Dorsal => 0.88 + 0.12 * ((x - 143.).atan2(110. - y) * 26.).cos().abs(),
        Region::Anal => 0.86 + 0.14 * ((x - 150.).atan2(y - 130.) * 29.).cos().abs(),
        Region::Ventral => {
            base = [218., 222., 228.];
            0.80 + 0.20 * (x * 1.3).cos().abs()
        }
        Region::Pectoral => {
            base = [174., 191., 207.];
            0.80 + 0.20 * ((x - 184.).atan2(y - 122.) * 25.).cos().abs()
        }
    };
    if !matches!(region, Region::Body) && (x * 0.7 + y * 0.35).sin() > 0.96 {
        shade += 0.05;
    }
    [
        (base[0] * shade).clamp(0., 255.) as u8,
        (base[1] * shade).clamp(0., 255.) as u8,
        (base[2] * shade).clamp(0., 255.) as u8,
        255,
    ]
}

fn noise(x: f64, y: f64, seed: u64) -> f64 {
    let ix = x.floor() as i64;
    let iy = y.floor() as i64;
    let smooth = |v: f64| v * v * (3. - 2. * v);
    let tx = smooth(x - x.floor());
    let ty = smooth(y - y.floor());
    let random = |dx: i64, dy: i64| {
        let mut hash = seed
            ^ (ix + dx).cast_unsigned().wrapping_mul(0x9e3779b97f4a7c15)
            ^ (iy + dy).cast_unsigned().wrapping_mul(0xbf58476d1ce4e5b9);
        hash = (hash ^ (hash >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        hash = (hash ^ (hash >> 27)).wrapping_mul(0x94d049bb133111eb);
        (hash ^ (hash >> 31)) as f64 / u64::MAX as f64
    };
    let top = random(0, 0) * (1. - tx) + random(1, 0) * tx;
    let bottom = random(0, 1) * (1. - tx) + random(1, 1) * tx;
    top * (1. - ty) + bottom * ty
}
