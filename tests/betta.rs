use ltkoi::betta::{betta_preview, derive_betta_development, render_betta};
use ltkoi::leetcode::{Availability, LeetCodeSnapshot, SolvedCounts};
use std::{fs, process::Command};

fn snapshot() -> LeetCodeSnapshot {
    serde_json::from_str(include_str!("fixtures/leetcode/betta-snapshot.json")).unwrap()
}
fn image(snapshot: &LeetCodeSnapshot) -> Vec<u8> {
    render_betta(&derive_betta_development(snapshot).unwrap())
        .as_rgba_bytes()
        .to_vec()
}
fn change_solves(snapshot: &mut LeetCodeSnapshot, counts: [u64; 3]) {
    snapshot.solved = SolvedCounts {
        all: counts.iter().sum(),
        easy: counts[0],
        medium: counts[1],
        hard: counts[2],
    };
    snapshot.topics = Availability::Available { data: vec![] };
}

#[test]
fn stable_identity_survives_case_changes_and_growth() {
    let original = snapshot();
    let development = derive_betta_development(&original).unwrap();
    let mut changed = original.clone();
    changed.username = changed.username.to_uppercase();
    assert_eq!(derive_betta_development(&changed).unwrap(), development);
    change_solves(&mut changed, [1000, 1000, 1000]);
    assert_eq!(
        derive_betta_development(&changed).unwrap().identity_seed,
        development.identity_seed
    );
    changed.username = "DifferentCoder".into();
    assert_ne!(
        derive_betta_development(&changed).unwrap().identity_seed,
        development.identity_seed
    );
}

#[test]
fn growth_is_monotonic_bounded_and_saturates_at_realistic_solve_counts() {
    let mut snapshot = snapshot();
    let mut previous = -1.;
    for total in [0, 1, 10, 100, 500, 1500, 3000, 100_000] {
        change_solves(&mut snapshot, [total, 0, 0]);
        let growth = derive_betta_development(&snapshot).unwrap().growth;
        assert!((0.0..=1.0).contains(&growth));
        assert!(growth >= previous);
        previous = growth;
    }
    assert_eq!(previous, 1.);
}

#[test]
fn future_schema_bad_counts_and_conflicting_topics_are_rejected() {
    let mut profile = snapshot();
    profile.schema_version = 2;
    assert!(derive_betta_development(&profile).is_err());
    profile = snapshot();
    profile.solved.all += 1;
    assert!(derive_betta_development(&profile).is_err());
    profile = snapshot();
    profile.solved.easy = u64::MAX;
    assert!(derive_betta_development(&profile).is_err());
    profile = snapshot();
    let Availability::Available { data } = &mut profile.topics else {
        panic!()
    };
    let mut duplicate = data[0].clone();
    duplicate.solved += 1;
    data.push(duplicate);
    assert!(derive_betta_development(&profile).is_err());
}

#[test]
fn all_growth_stages_keep_fins_inside_transparent_canvas() {
    let mut profile = snapshot();
    let mut previous_area = 0;
    for total in [0, 1, 100, 709, 3000, 100_000] {
        change_solves(&mut profile, [total, 0, 0]);
        let pixels = image(&profile);
        assert_eq!(pixels.len(), 256 * 256 * 4);
        let mut area = 0;
        for (i, pixel) in pixels.chunks_exact(4).enumerate() {
            assert!(pixel[3] == 0 || pixel[3] == 255);
            if pixel[3] != 0 {
                let (x, y) = (i % 256, i / 256);
                assert!(x > 8 && x < 247 && y > 8 && y < 247);
                area += 1;
            }
        }
        assert!(area > 8000 && area >= previous_area);
        previous_area = area;
    }
}

#[test]
fn drawings_are_reproducible_and_topic_order_does_not_change_marbling() {
    let original = snapshot();
    assert_eq!(image(&original), image(&original));
    let mut reordered = original.clone();
    let Availability::Available { data } = &mut reordered.topics else {
        panic!()
    };
    data.reverse();
    assert_eq!(image(&original), image(&reordered));
    reordered.topics = Availability::Unavailable {
        reason: "restricted".into(),
    };
    assert_ne!(image(&original), image(&reordered));
    assert_eq!(
        derive_betta_development(&original).unwrap().growth,
        derive_betta_development(&reordered).unwrap().growth
    );
}

#[test]
fn difficulty_changes_palette_without_changing_silhouette() {
    let mut easy = snapshot();
    change_solves(&mut easy, [709, 0, 0]);
    let mut medium = easy.clone();
    change_solves(&mut medium, [0, 709, 0]);
    let a = image(&easy);
    let b = image(&medium);
    let alpha = |pixels: &[u8]| pixels.chunks_exact(4).map(|p| p[3]).collect::<Vec<_>>();
    assert_eq!(alpha(&a), alpha(&b));
    let red_count = |pixels: &[u8]| {
        pixels
            .chunks_exact(4)
            .filter(|p| p[3] == 255 && u16::from(p[0]) > u16::from(p[2]) * 2)
            .count()
    };
    assert!(red_count(&b) > red_count(&a) * 2);
}

#[test]
fn unavailable_activity_does_not_shrink_or_recolor_the_fish() {
    let original = snapshot();
    let mut changed = original.clone();
    changed.calendar = Availability::Unavailable {
        reason: "network failure".into(),
    };
    changed.recent_accepted = Availability::Unavailable {
        reason: "restricted".into(),
    };
    assert_eq!(image(&original), image(&changed));
}

#[test]
fn preview_escapes_untrusted_text_and_does_not_claim_interactivity() {
    let mut profile = snapshot();
    profile.username = "<script>\"&".into();
    let html = betta_preview(&profile);
    assert!(!html.contains("<script>"));
    assert!(html.contains("&lt;script&gt;&quot;&amp;"));
    assert!(html.contains("Still portrait"));
    assert!(html.contains("Use animate-betta or animate-pond"));
}

#[test]
fn command_exports_png_svg_traits_and_preview_and_preserves_files_on_bad_input() {
    let directory = std::env::temp_dir().join(format!("ltkoi-betta-{}", std::process::id()));
    let _ = fs::remove_dir_all(&directory);
    fs::create_dir_all(&directory).unwrap();
    let source = directory.join("profile.json");
    fs::write(&source, serde_json::to_vec(&snapshot()).unwrap()).unwrap();
    let output = directory.join("portrait");
    let run = || {
        Command::new(env!("CARGO_BIN_EXE_ltkoi"))
            .args([
                "render-betta",
                source.to_str().unwrap(),
                "--output-dir",
                output.to_str().unwrap(),
            ])
            .output()
            .unwrap()
    };
    let result = run();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let png = fs::read(output.join("betta.png")).unwrap();
    assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"));
    assert!(
        fs::read_to_string(output.join("betta.svg"))
            .unwrap()
            .contains("width=\"512\"")
    );
    assert!(
        fs::read_to_string(output.join("preview.html"))
            .unwrap()
            .contains("betta.png")
    );
    let traits: serde_json::Value =
        serde_json::from_slice(&fs::read(output.join("traits.json")).unwrap()).unwrap();
    assert_eq!(traits["solved"], 709);
    let mut invalid = snapshot();
    invalid.schema_version = 999;
    fs::write(&source, serde_json::to_vec(&invalid).unwrap()).unwrap();
    assert_eq!(run().status.code(), Some(1));
    assert_eq!(fs::read(output.join("betta.png")).unwrap(), png);
    fs::remove_dir_all(directory).unwrap();
}
