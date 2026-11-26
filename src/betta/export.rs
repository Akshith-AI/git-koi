use super::{
    BETTA_FRAMES, FLARE_LEVELS, WIDGET_SCRIPT, betta_activity, betta_pose, compose_pond,
    derive_betta_development, encode_art_gif, encode_sprite_atlas, render_betta,
    render_betta_frame, widget_html,
};
use crate::behavior::select_behavior;
use crate::coat::KoiBody;
use crate::history::AccountSnapshot;
use crate::koi::derive_koi_development;
use crate::leetcode::LeetCodeSnapshot;
use crate::motion::frame_count;
use crate::output::{encode_png, encode_svg};
use crate::render::render_swim_frame;

/// Generate a complete portable widget and GIF bundle in memory before writing files.
/// With a GitHub snapshot, render a shared pond; otherwise render the betta alone.
/// Koi behavior is selected independently and retimed to the four-second scene loop.
pub fn animation_files(
    profile: &LeetCodeSnapshot,
    github: Option<&AccountSnapshot>,
) -> Result<Vec<(String, Vec<u8>)>, String> {
    let development = derive_betta_development(profile).map_err(|e| e.to_string())?;
    let activity = betta_activity(profile);
    let koi = github
        .map(|account| {
            let development = derive_koi_development(account).map_err(|e| e.to_string())?;
            let behavior = select_behavior(&development);
            Ok::<_, String>(
                (0..BETTA_FRAMES)
                    .map(|i| {
                        render_swim_frame(
                            &development,
                            KoiBody::Asagi,
                            behavior,
                            i * frame_count(behavior) / BETTA_FRAMES,
                        )
                    })
                    .collect::<Vec<_>>(),
            )
        })
        .transpose()?;
    let mut frames = Vec::with_capacity((BETTA_FRAMES * FLARE_LEVELS) as usize);
    for level in 0..FLARE_LEVELS {
        for i in 0..BETTA_FRAMES {
            let mut pose = betta_pose(i, activity, f64::from(level) / f64::from(FLARE_LEVELS - 1));
            if koi.is_some() {
                // The compositor owns the pond course; preserve a full-size local fish.
                pose.dx = 0.;
                pose.dy = 0.;
                pose.scene_scale = 1.;
            }
            let betta = render_betta_frame(&development, pose);
            frames.push(if let Some(koi) = &koi {
                compose_pond(&betta, &koi[i as usize], i)
            } else {
                betta
            });
        }
    }
    let still = render_betta(&development);
    let still = if let Some(koi) = &koi {
        compose_pond(&still, &koi[0], 0)
    } else {
        still
    };
    let stem = if github.is_some() { "pond" } else { "betta" };
    let page = widget_html(&profile.username, github.is_some());
    // A fixed-camera detail loop makes membrane motion inspectable without turns.
    let detail_frames: Vec<_> = (0..BETTA_FRAMES)
        .map(|i| {
            let mut pose = betta_pose(i, activity, 0.);
            pose.dx = 0.;
            pose.dy = 0.;
            pose.projection = 1.;
            pose.pitch = 0.;
            pose.scene_scale = 1.;
            render_betta_frame(&development, pose)
        })
        .collect();
    let manifest = serde_json::json!({
        "schema_version":1,"mode":stem,"username":profile.username,
        "frames":BETTA_FRAMES,"fps":15,"flare_levels":FLARE_LEVELS,
        "native_size":256,"atlas_columns":10,
        "activity":format!("{activity:?}").to_ascii_lowercase(),
        "github_observed_on":github.and_then(|g| g.contribution_history.as_ref().map(|h| h.observed_on)),
        "leetcode_observed_on":profile.observed_on,
    });
    Ok(vec![
        (
            "fin-detail.gif".into(),
            encode_art_gif(&detail_frames).map_err(|e| e.to_string())?,
        ),
        (
            format!("{stem}.gif"),
            encode_art_gif(&frames[..BETTA_FRAMES as usize]).map_err(|e| e.to_string())?,
        ),
        (
            format!("{stem}.png"),
            encode_png(&still).map_err(|e| e.to_string())?,
        ),
        (format!("{stem}.svg"), encode_svg(&still).into_bytes()),
        (
            "sprites.png".into(),
            encode_sprite_atlas(&frames).map_err(|e| e.to_string())?,
        ),
        ("embed.html".into(), page.as_bytes().to_vec()),
        ("preview.html".into(), page.into_bytes()),
        ("widget.js".into(), WIDGET_SCRIPT.as_bytes().to_vec()),
        (
            "animation.json".into(),
            serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?,
        ),
    ])
}
