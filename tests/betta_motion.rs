use chrono::{Days, NaiveDate};
use ltkoi::betta::*;
use ltkoi::leetcode::{ActivityCalendar, Availability, LeetCodeSnapshot};
use std::collections::BTreeMap;

fn snapshot() -> LeetCodeSnapshot {
    serde_json::from_str(include_str!("fixtures/leetcode/betta-snapshot.json")).unwrap()
}
#[test]
fn poses_wrap_and_opening_is_bounded() {
    for activity in [
        BettaActivity::Active,
        BettaActivity::Quiet,
        BettaActivity::Unknown,
    ] {
        assert_eq!(
            betta_pose(0, activity, 0.),
            betta_pose(BETTA_FRAMES, activity, 0.)
        );
        let first = betta_pose(0, activity, 0.);
        let last = betta_pose(BETTA_FRAMES - 1, activity, 0.);
        assert!((first.dx - last.dx).abs() < 7. && (first.dy - last.dy).abs() < 0.6);
    }
    assert_eq!(betta_pose(0, BettaActivity::Unknown, f64::NAN).flare, 0.);
    assert_eq!(betta_pose(0, BettaActivity::Unknown, 99.).flare, 1.);
}
#[test]
fn absent_or_year_incomplete_activity_remains_unknown() {
    let mut profile = snapshot();
    assert_eq!(betta_activity(&profile), BettaActivity::Unknown);
    profile.calendar = Availability::Available {
        data: ActivityCalendar {
            year: 2026,
            reported_streak: 999,
            total_active_days: 0,
            submissions: BTreeMap::new(),
        },
    };
    assert_eq!(betta_activity(&profile), BettaActivity::Quiet);
    let Availability::Available { data } = &mut profile.calendar else {
        panic!()
    };
    data.submissions
        .insert(profile.observed_on - Days::new(2), 1);
    assert_eq!(betta_activity(&profile), BettaActivity::Active);
    profile.observed_on = NaiveDate::from_ymd_opt(2026, 1, 2).unwrap();
    assert_eq!(betta_activity(&profile), BettaActivity::Unknown);
}
#[test]
fn all_flare_levels_animate_without_clipping_and_opening_adds_fin_area() {
    let traits = derive_betta_development(&snapshot()).unwrap();
    let mut areas = Vec::new();
    for level in 0..FLARE_LEVELS {
        let mut area = 0;
        for frame in [0, 15, 30, 45] {
            let image = render_betta_frame(
                &traits,
                betta_pose(frame, BettaActivity::Active, f64::from(level) / 4.),
            );
            for (i, p) in image.as_rgba_bytes().chunks_exact(4).enumerate() {
                if p[3] > 0 {
                    assert!(i % 256 > 3 && i % 256 < 252 && i / 256 > 3 && i / 256 < 252);
                    area += 1;
                }
            }
        }
        areas.push(area);
    }
    assert!(areas.windows(2).all(|pair| pair[1] > pair[0]));
    let a = render_betta_frame(&traits, betta_pose(0, BettaActivity::Unknown, 0.));
    let b = render_betta_frame(&traits, betta_pose(15, BettaActivity::Unknown, 0.));
    assert_ne!(a, b);
    assert_eq!(
        a,
        render_betta_frame(
            &traits,
            betta_pose(BETTA_FRAMES, BettaActivity::Unknown, 0.)
        )
    );
    assert!(encode_art_gif(&[a, b]).unwrap().starts_with(b"GIF89a"));
}
#[test]
fn pond_lanes_are_distinct_and_scene_wraps() {
    let profile = snapshot();
    let traits = derive_betta_development(&profile).unwrap();
    let betta = render_betta(&traits);
    let account: ltkoi::history::AccountSnapshot =
        serde_json::from_str(include_str!("fixtures/account-with-history.json")).unwrap();
    let koi = ltkoi::render::render_koi(
        &ltkoi::koi::derive_koi_development(&account).unwrap(),
        ltkoi::coat::KoiBody::Asagi,
    );
    let scene = compose_pond(&betta, &koi, 0);
    assert_eq!(scene, compose_pond(&betta, &koi, BETTA_FRAMES));
    let mut upper = 0;
    let mut lower = 0;
    for (i, p) in scene.as_rgba_bytes().chunks_exact(4).enumerate() {
        if p[3] > 0 {
            if i / 256 < 127 {
                upper += 1;
            } else {
                lower += 1;
            }
        }
    }
    assert!(upper > 500 && lower > 10);
    assert!(encode_art_gif(&[scene]).is_ok());
}

#[test]
fn atlas_slots_preserve_unquantized_pixels_and_widget_escapes_last() {
    use std::io::Cursor;
    let traits = derive_betta_development(&snapshot()).unwrap();
    let first = render_betta_frame(&traits, betta_pose(0, BettaActivity::Unknown, 0.));
    let second = render_betta_frame(&traits, betta_pose(0, BettaActivity::Unknown, 1.));
    let atlas = encode_sprite_atlas(&[first, second]).unwrap();
    let mut reader = png::Decoder::new(Cursor::new(atlas)).read_info().unwrap();
    let mut bytes = vec![0; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut bytes).unwrap();
    assert_eq!((info.width, info.height), (2560, 256));
    let open = render_betta_frame(&traits, betta_pose(0, BettaActivity::Unknown, 1.));
    for y in 0..256_usize {
        assert_eq!(
            &bytes[(y * 2560 + 256) * 4..(y * 2560 + 512) * 4],
            &open.as_rgba_bytes()[y * 256 * 4..(y + 1) * 256 * 4]
        );
    }
    let html = widget_html("MODE_STILL_ANIMATION_HEADING", true);
    assert!(html.contains("<h1>MODE_STILL_ANIMATION_HEADING</h1>"));
    assert!(widget_html("<script>", false).contains("&lt;script&gt;"));
    assert!(html.contains("data-mode=\"pond\""));
}

#[test]
fn maximum_size_active_flare_and_pond_keep_safe_margins() {
    let mut profile = snapshot();
    profile.solved.all = 3000;
    profile.solved.easy = 600;
    profile.solved.medium = 1400;
    profile.solved.hard = 1000;
    let traits = derive_betta_development(&profile).unwrap();
    for frame in (0..BETTA_FRAMES).step_by(5) {
        let fish = render_betta_frame(&traits, betta_pose(frame, BettaActivity::Active, 1.));
        for (i, p) in fish.as_rgba_bytes().chunks_exact(4).enumerate() {
            if p[3] > 0 {
                assert!(i % 256 > 4 && i % 256 < 251 && i / 256 > 4 && i / 256 < 251);
            }
        }
        let scene = compose_pond(&fish, &fish, frame);
        for (i, p) in scene.as_rgba_bytes().chunks_exact(4).enumerate() {
            if p[3] > 0 {
                assert!(i % 256 > 2 && i % 256 < 253 && i / 256 > 2 && i / 256 < 253);
            }
        }
    }
}

#[test]
fn swimming_travels_across_the_scene_and_faces_its_direction() {
    let traits = derive_betta_development(&snapshot()).unwrap();
    let mut centers = Vec::new();
    for frame in (0..BETTA_FRAMES).step_by(5) {
        let pose = betta_pose(frame, BettaActivity::Unknown, 0.);
        let image = render_betta_frame(&traits, pose);
        let mut sum = [0_usize; 2];
        let mut count = 0;
        for (i, pixel) in image.as_rgba_bytes().chunks_exact(4).enumerate() {
            if pixel[3] > 0 {
                sum[0] += i % 256;
                sum[1] += i / 256;
                count += 1;
            }
        }
        assert!(
            count > 500,
            "fish disappeared while turning at frame {frame}"
        );
        centers.push([sum[0] as f64 / count as f64, sum[1] as f64 / count as f64]);
        let next = betta_pose(frame + 1, BettaActivity::Unknown, 0.);
        if (next.dx - pose.dx).abs() > 0.5 {
            assert_eq!(
                pose.projection.is_sign_positive(),
                (next.dx - pose.dx).is_sign_positive()
            );
        }
    }
    let range = |axis| {
        centers
            .iter()
            .map(|c| c[axis])
            .fold(f64::NEG_INFINITY, f64::max)
            - centers
                .iter()
                .map(|c| c[axis])
                .fold(f64::INFINITY, f64::min)
    };
    assert!(range(0) > 65., "fish is still hovering instead of swimming");
    assert!(range(1) > 15.);
}

#[test]
fn koi_marbling_contains_visible_gold_and_tangerine_patches() {
    let image = render_betta(&derive_betta_development(&snapshot()).unwrap());
    let mut gold = 0;
    let mut orange = 0;
    for p in image
        .as_rgba_bytes()
        .chunks_exact(4)
        .filter(|p| p[3] == 255)
    {
        if p[0] > 160 && p[1] > 125 && p[2] < 110 && f64::from(p[1]) / f64::from(p[0]) > 0.65 {
            gold += 1;
        }
        if p[0] > 180 && (70..155).contains(&p[1]) && p[2] < 70 {
            orange += 1;
        }
    }
    assert!(gold > 50, "yellow/gold patches are missing: {gold}");
    assert!(orange > 50, "orange patches are missing: {orange}");
}

#[test]
fn membranes_move_visibly_with_the_camera_and_head_held_still() {
    let mut profile = snapshot();
    profile.solved.all = 3000;
    profile.solved.easy = 600;
    profile.solved.medium = 1400;
    profile.solved.hard = 1000;
    let traits = derive_betta_development(&profile).unwrap();
    let fixed = |frame| {
        let mut pose = betta_pose(frame, BettaActivity::Unknown, 0.);
        pose.dx = 0.;
        pose.dy = 0.;
        pose.projection = 1.;
        pose.pitch = 0.;
        pose.scene_scale = 1.;
        render_betta_frame(&traits, pose)
    };
    let a = fixed(0);
    let b = fixed(10);
    let alpha_changes = |bounds: [usize; 4]| {
        let mut count = 0;
        for y in bounds[1]..bounds[3] {
            for x in bounds[0]..bounds[2] {
                let i = (y * 256 + x) * 4;
                if a.as_rgba_bytes()[i + 3] != b.as_rgba_bytes()[i + 3] {
                    count += 1;
                }
            }
        }
        count
    };
    assert!(
        alpha_changes([15, 40, 105, 220]) > 150,
        "tail outline barely moves"
    );
    assert!(
        alpha_changes([105, 25, 195, 95]) > 50,
        "dorsal outline barely moves"
    );
    assert!(
        alpha_changes([105, 155, 195, 235]) > 80,
        "anal outline barely moves"
    );
    let mut pectoral_changes = 0;
    for y in 130..163 {
        for x in 163..190 {
            let i = (y * 256 + x) * 4;
            if a.as_rgba_bytes()[i..i + 4] != b.as_rgba_bytes()[i..i + 4] {
                pectoral_changes += 1;
            }
        }
    }
    // Sample another phase for the faster pectoral beat (frame 10 is a full beat).
    let c = fixed(5);
    for y in 130..163 {
        for x in 163..190 {
            let i = (y * 256 + x) * 4;
            if a.as_rgba_bytes()[i..i + 4] != c.as_rgba_bytes()[i..i + 4] {
                pectoral_changes += 1;
            }
        }
    }
    assert!(pectoral_changes > 100, "pectoral fin does not flap");
    for y in 109..126 {
        for x in 207..223 {
            let i = (y * 256 + x) * 4;
            assert_eq!(
                &a.as_rgba_bytes()[i..i + 4],
                &b.as_rgba_bytes()[i..i + 4],
                "head moves during the fixed-camera check"
            );
        }
    }
}

#[test]
fn relaxed_glides_keep_the_fish_upright_and_never_squash_its_profile() {
    let first = betta_pose(0, BettaActivity::Unknown, 0.);
    let mut directions = Vec::new();
    for frame in 0..BETTA_FRAMES {
        let pose = betta_pose(frame, BettaActivity::Unknown, 0.);
        assert_eq!(
            pose.projection.abs(),
            1.,
            "the fish is being flattened during a turn"
        );
        assert_eq!(pose.pitch, 0., "the whole fish is rotating");
        assert!(pose.dy.abs() <= 10.);
        assert_eq!(pose.scene_scale, first.scene_scale);
        directions.push(pose.projection);
    }
    assert!(directions.contains(&1.) && directions.contains(&-1.));
    let reversals = directions
        .iter()
        .zip(directions.iter().cycle().skip(1))
        .filter(|(a, b)| a != b)
        .count();
    assert_eq!(
        reversals, 2,
        "direction changes should happen only at the two course ends"
    );
    assert_eq!(first, betta_pose(BETTA_FRAMES, BettaActivity::Unknown, 0.));
}
