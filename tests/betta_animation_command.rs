use ltkoi::leetcode::LeetCodeSnapshot;
use std::{fs, io::Cursor, process::Command};

#[test]
fn both_commands_export_looping_gifs_and_complete_widget_bundles() {
    let root = std::env::temp_dir().join(format!("ltkoi-animation-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    let profile = root.join("leetcode.json");
    fs::write(
        &profile,
        include_str!("fixtures/leetcode/betta-snapshot.json"),
    )
    .unwrap();
    let github = root.join("github.json");
    fs::write(&github, include_str!("fixtures/account-with-history.json")).unwrap();
    for mode in ["betta", "pond"] {
        let destination = root.join(mode);
        let mut command = Command::new(env!("CARGO_BIN_EXE_ltkoi"));
        command.arg(format!("animate-{mode}"));
        if mode == "pond" {
            command.arg(&github);
        }
        let output = command
            .arg(&profile)
            .arg("--output-dir")
            .arg(&destination)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let gif_bytes = fs::read(destination.join(format!("{mode}.gif"))).unwrap();
        let mut decoder = gif::DecodeOptions::new();
        decoder.set_color_output(gif::ColorOutput::RGBA);
        let mut reader = decoder.read_info(Cursor::new(&gif_bytes)).unwrap();
        assert_eq!((reader.width(), reader.height()), (512, 512));
        assert_eq!(reader.repeat(), gif::Repeat::Infinite);
        let mut count = 0;
        let mut duration = 0;
        while let Some(frame) = reader.read_next_frame().unwrap() {
            count += 1;
            duration += u32::from(frame.delay);
        }
        assert_eq!(count, 60);
        assert_eq!(duration, 400);
        for name in [
            "fin-detail.gif",
            "sprites.png",
            "embed.html",
            "preview.html",
            "widget.js",
            "animation.json",
        ] {
            assert!(destination.join(name).exists());
        }
        let meta: serde_json::Value =
            serde_json::from_slice(&fs::read(destination.join("animation.json")).unwrap()).unwrap();
        assert_eq!(meta["flare_levels"], 5);
        assert_eq!(meta["mode"], mode);
        let mut invalid: LeetCodeSnapshot =
            serde_json::from_slice(&fs::read(&profile).unwrap()).unwrap();
        invalid.schema_version = 9;
        fs::write(&profile, serde_json::to_vec(&invalid).unwrap()).unwrap();
        let mut command = Command::new(env!("CARGO_BIN_EXE_ltkoi"));
        command.arg(format!("animate-{mode}"));
        if mode == "pond" {
            command.arg(&github);
        }
        assert_eq!(
            command
                .arg(&profile)
                .arg("--output-dir")
                .arg(&destination)
                .output()
                .unwrap()
                .status
                .code(),
            Some(1)
        );
        assert_eq!(
            fs::read(destination.join(format!("{mode}.gif"))).unwrap(),
            gif_bytes
        );
        fs::write(
            &profile,
            include_str!("fixtures/leetcode/betta-snapshot.json"),
        )
        .unwrap();
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn animation_usage_is_validated_before_any_rendering() {
    for args in [
        vec!["animate-betta"],
        vec!["animate-pond", "x", "--output-dir", "y"],
        vec!["animate-betta", "x", "--bad", "y"],
    ] {
        assert_eq!(
            Command::new(env!("CARGO_BIN_EXE_ltkoi"))
                .args(args)
                .output()
                .unwrap()
                .status
                .code(),
            Some(2)
        );
    }
    for command in ["animate-betta", "animate-pond"] {
        assert!(
            Command::new(env!("CARGO_BIN_EXE_ltkoi"))
                .args([command, "--help"])
                .output()
                .unwrap()
                .status
                .success()
        );
    }
}
