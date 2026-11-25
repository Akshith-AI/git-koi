use chrono::{Days, NaiveDate};
use git_koi::betta::*;
use git_koi::leetcode::{ActivityCalendar, Availability, LeetCodeSnapshot};
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
        assert!((first.dx - last.dx).abs() < 0.4 && (first.dy - last.dy).abs() < 0.5);
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
    let account: git_koi::history::AccountSnapshot =
        serde_json::from_str(include_str!("fixtures/account-with-history.json")).unwrap();
    let koi = git_koi::render::render_koi(
        &git_koi::koi::derive_koi_development(&account).unwrap(),
        git_koi::coat::KoiBody::Asagi,
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
