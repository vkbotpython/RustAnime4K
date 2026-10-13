//! Anime4K shader presets.
//!
//! Shader file names are resolved against `assets/shaders/` at runtime;
//! `preset_paths()` joins them with the OS-specific list separator
//! (`;` on Windows, `:` elsewhere).

use std::collections::HashMap;
use std::path::Path;

pub const QUALITIES: &[&str] = &["S", "M", "L", "VL", "UL"];
pub const MODES: &[&str] = &["A", "B", "C", "A+A", "B+B", "C+A"];

/// Quality used for the second (perceptual) pass, mirrors `sec_qualities`.
pub fn secondary_quality(quality: &str) -> Option<&'static str> {
    match quality {
        "S" => Some("S"),
        "M" => Some("S"),
        "L" => Some("M"),
        "VL" => Some("M"),
        "UL" => Some("L"),
        _ => None,
    }
}

fn std(preset: &[&str]) -> Vec<String> {
    preset.iter().map(|s| s.to_string()).collect()
}

/// The 9 standard HQ presets.
/// (The presets menu builds these dynamically via [`create_preset`].)
#[allow(dead_code)]
pub fn standard_presets() -> HashMap<&'static str, Vec<String>> {
    HashMap::from([
        (
            "A (Fast)",
            std(&[
                "Anime4K_Clamp_Highlights.glsl",
                "Anime4K_Restore_CNN_M.glsl",
                "Anime4K_Upscale_CNN_x2_M.glsl",
                "Anime4K_AutoDownscalePre_x2.glsl",
                "Anime4K_AutoDownscalePre_x4.glsl",
                "Anime4K_Upscale_CNN_x2_S.glsl",
            ]),
        ),
        (
            "B (Fast)",
            std(&[
                "Anime4K_Clamp_Highlights.glsl",
                "Anime4K_Restore_CNN_Soft_M.glsl",
                "Anime4K_Upscale_CNN_x2_M.glsl",
                "Anime4K_AutoDownscalePre_x2.glsl",
                "Anime4K_AutoDownscalePre_x4.glsl",
                "Anime4K_Upscale_CNN_x2_S.glsl",
            ]),
        ),
        (
            "C (Fast)",
            std(&[
                "Anime4K_Clamp_Highlights.glsl",
                "Anime4K_Upscale_Denoise_CNN_x2_M.glsl",
                "Anime4K_AutoDownscalePre_x2.glsl",
                "Anime4K_AutoDownscalePre_x4.glsl",
                "Anime4K_Upscale_CNN_x2_S.glsl",
            ]),
        ),
        (
            "A (HQ)",
            std(&[
                "Anime4K_Clamp_Highlights.glsl",
                "Anime4K_Restore_CNN_VL.glsl",
                "Anime4K_Upscale_CNN_x2_VL.glsl",
                "Anime4K_AutoDownscalePre_x2.glsl",
                "Anime4K_AutoDownscalePre_x4.glsl",
                "Anime4K_Upscale_CNN_x2_M.glsl",
            ]),
        ),
        (
            "B (HQ)",
            std(&[
                "Anime4K_Clamp_Highlights.glsl",
                "Anime4K_Restore_CNN_Soft_VL.glsl",
                "Anime4K_Upscale_CNN_x2_VL.glsl",
                "Anime4K_AutoDownscalePre_x2.glsl",
                "Anime4K_AutoDownscalePre_x4.glsl",
                "Anime4K_Upscale_CNN_x2_M.glsl",
            ]),
        ),
        (
            "C (HQ)",
            std(&[
                "Anime4K_Clamp_Highlights.glsl",
                "Anime4K_Upscale_Denoise_CNN_x2_VL.glsl",
                "Anime4K_AutoDownscalePre_x2.glsl",
                "Anime4K_AutoDownscalePre_x4.glsl",
                "Anime4K_Upscale_CNN_x2_M.glsl",
            ]),
        ),
        (
            "A+A (HQ)",
            std(&[
                "Anime4K_Clamp_Highlights.glsl",
                "Anime4K_Restore_CNN_VL.glsl",
                "Anime4K_Upscale_CNN_x2_VL.glsl",
                "Anime4K_Restore_CNN_M.glsl",
                "Anime4K_AutoDownscalePre_x2.glsl",
                "Anime4K_AutoDownscalePre_x4.glsl",
                "Anime4K_Upscale_CNN_x2_M.glsl",
            ]),
        ),
        (
            "B+B (HQ)",
            std(&[
                "Anime4K_Clamp_Highlights.glsl",
                "Anime4K_Restore_CNN_Soft_VL.glsl",
                "Anime4K_Upscale_CNN_x2_VL.glsl",
                "Anime4K_AutoDownscalePre_x2.glsl",
                "Anime4K_AutoDownscalePre_x4.glsl",
                "Anime4K_Restore_CNN_Soft_M.glsl",
                "Anime4K_Upscale_CNN_x2_M.glsl",
            ]),
        ),
        (
            "C+A (HQ)",
            std(&[
                "Anime4K_Clamp_Highlights.glsl",
                "Anime4K_Upscale_Denoise_CNN_x2_VL.glsl",
                "Anime4K_AutoDownscalePre_x2.glsl",
                "Anime4K_AutoDownscalePre_x4.glsl",
                "Anime4K_Restore_CNN_M.glsl",
                "Anime4K_Upscale_CNN_x2_M.glsl",
            ]),
        ),
    ])
}

/// The 8 ultra-HQ presets.
pub fn ultra_hq_presets() -> HashMap<&'static str, Vec<String>> {
    HashMap::from([
        (
            "A",
            std(&[
                "Anime4K_Clamp_Highlights.glsl",
                "Anime4K_Restore_GAN_UUL.glsl",
                "Anime4K_Upscale_CNN_x2_UL.glsl",
                "Anime4K_AutoDownscalePre_x2.glsl",
                "Anime4K_AutoDownscalePre_x4.glsl",
                "Anime4K_Upscale_CNN_x2_UL.glsl",
            ]),
        ),
        (
            "B",
            std(&[
                "Anime4K_Clamp_Highlights.glsl",
                "Anime4K_Restore_CNN_Soft_UL.glsl",
                "Anime4K_Upscale_CNN_x2_UL.glsl",
                "Anime4K_AutoDownscalePre_x2.glsl",
                "Anime4K_AutoDownscalePre_x4.glsl",
                "Anime4K_Upscale_CNN_x2_UL.glsl",
            ]),
        ),
        (
            "C",
            std(&[
                "Anime4K_Clamp_Highlights.glsl",
                "Anime4K_Upscale_Denoise_CNN_x2_UL.glsl",
                "Anime4K_AutoDownscalePre_x2.glsl",
                "Anime4K_AutoDownscalePre_x4.glsl",
                "Anime4K_Upscale_CNN_x2_UL.glsl",
            ]),
        ),
        (
            "A+A",
            std(&[
                "Anime4K_Clamp_Highlights.glsl",
                "Anime4K_Restore_GAN_UUL.glsl",
                "Anime4K_Upscale_CNN_x2_UL.glsl",
                "Anime4K_Restore_GAN_UUL.glsl",
                "Anime4K_AutoDownscalePre_x2.glsl",
                "Anime4K_AutoDownscalePre_x4.glsl",
                "Anime4K_Upscale_CNN_x2_UL.glsl",
            ]),
        ),
        (
            "B+B",
            std(&[
                "Anime4K_Clamp_Highlights.glsl",
                "Anime4K_Restore_CNN_Soft_UL.glsl",
                "Anime4K_Upscale_CNN_x2_UL.glsl",
                "Anime4K_AutoDownscalePre_x2.glsl",
                "Anime4K_AutoDownscalePre_x4.glsl",
                "Anime4K_Restore_CNN_Soft_UL.glsl",
                "Anime4K_Upscale_CNN_x2_UL.glsl",
            ]),
        ),
        (
            "C+A",
            std(&[
                "Anime4K_Clamp_Highlights.glsl",
                "Anime4K_Upscale_Denoise_CNN_x2_UL.glsl",
                "Anime4K_AutoDownscalePre_x2.glsl",
                "Anime4K_AutoDownscalePre_x4.glsl",
                "Anime4K_Restore_GAN_UUL.glsl",
                "Anime4K_Upscale_CNN_x2_UL.glsl",
            ]),
        ),
        (
            "A (Death)",
            std(&[
                "Anime4K_Clamp_Highlights.glsl",
                "Anime4K_Restore_GAN_UUL.glsl",
                "Anime4K_Upscale_GAN_x4_UUL.glsl",
                "Anime4K_Restore_CNN_Soft_M.glsl",
                "Anime4K_Upscale_CNN_x2_M.glsl",
            ]),
        ),
        (
            "C+A (Death)",
            std(&[
                "Anime4K_Clamp_Highlights.glsl",
                "Anime4K_Upscale_Denoise_CNN_x2_UL.glsl",
                "Anime4K_AutoDownscalePre_x2.glsl",
                "Anime4K_AutoDownscalePre_x4.glsl",
                "Anime4K_Restore_GAN_UUL.glsl",
                "Anime4K_Upscale_GAN_x4_UUL.glsl",
                "Anime4K_Restore_CNN_Soft_M.glsl",
            ]),
        ),
    ])
}

/// Builds a preset for a quality/mode pair.
pub fn create_preset(quality: &str, mode: &str) -> anyhow::Result<Vec<String>> {
    let sec =
        secondary_quality(quality).ok_or_else(|| anyhow::anyhow!("unknown quality: {quality}"))?;
    let preset = match mode {
        "A" => vec![
            "Anime4K_Clamp_Highlights.glsl".to_string(),
            format!("Anime4K_Restore_CNN_{quality}.glsl"),
            format!("Anime4K_Upscale_CNN_x2_{quality}.glsl"),
            "Anime4K_AutoDownscalePre_x2.glsl".to_string(),
            "Anime4K_AutoDownscalePre_x4.glsl".to_string(),
            format!("Anime4K_Upscale_CNN_x2_{sec}.glsl"),
        ],
        "B" => vec![
            "Anime4K_Clamp_Highlights.glsl".to_string(),
            format!("Anime4K_Restore_CNN_Soft_{quality}.glsl"),
            format!("Anime4K_Upscale_CNN_x2_{quality}.glsl"),
            "Anime4K_AutoDownscalePre_x2.glsl".to_string(),
            "Anime4K_AutoDownscalePre_x4.glsl".to_string(),
            format!("Anime4K_Upscale_CNN_x2_{sec}.glsl"),
        ],
        "C" => vec![
            "Anime4K_Clamp_Highlights.glsl".to_string(),
            format!("Anime4K_Upscale_Denoise_CNN_x2_{quality}.glsl"),
            "Anime4K_AutoDownscalePre_x2.glsl".to_string(),
            "Anime4K_AutoDownscalePre_x4.glsl".to_string(),
            format!("Anime4K_Upscale_CNN_x2_{sec}.glsl"),
        ],
        "A+A" => vec![
            "Anime4K_Clamp_Highlights.glsl".to_string(),
            format!("Anime4K_Restore_CNN_{quality}.glsl"),
            format!("Anime4K_Upscale_CNN_x2_{quality}.glsl"),
            format!("Anime4K_Restore_CNN_{sec}.glsl"),
            "Anime4K_AutoDownscalePre_x2.glsl".to_string(),
            "Anime4K_AutoDownscalePre_x4.glsl".to_string(),
            format!("Anime4K_Upscale_CNN_x2_{sec}.glsl"),
        ],
        "B+B" => vec![
            "Anime4K_Clamp_Highlights.glsl".to_string(),
            format!("Anime4K_Restore_CNN_Soft_{quality}.glsl"),
            format!("Anime4K_Upscale_CNN_x2_{quality}.glsl"),
            "Anime4K_AutoDownscalePre_x2.glsl".to_string(),
            "Anime4K_AutoDownscalePre_x4.glsl".to_string(),
            format!("Anime4K_Restore_CNN_Soft_{sec}.glsl"),
            format!("Anime4K_Upscale_CNN_x2_{sec}.glsl"),
        ],
        "C+A" => vec![
            "Anime4K_Clamp_Highlights.glsl".to_string(),
            format!("Anime4K_Upscale_Denoise_CNN_x2_{quality}.glsl"),
            "Anime4K_AutoDownscalePre_x2.glsl".to_string(),
            "Anime4K_AutoDownscalePre_x4.glsl".to_string(),
            format!("Anime4K_Restore_CNN_{sec}.glsl"),
            format!("Anime4K_Upscale_CNN_x2_{sec}.glsl"),
        ],
        _ => anyhow::bail!("unknown mode: {mode}"),
    };
    Ok(preset)
}

/// Joins a preset into the `glsl-shaders` mpv option value.
pub fn preset_to_glsl_shaders(shaders_dir: &Path, preset: &[String]) -> String {
    let sep = if cfg!(windows) { ";" } else { ":" };
    preset
        .iter()
        .map(|s| shaders_dir.join(s).to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join(sep)
}

/// Builds an mpv `glsl-shaders=` config line for Android.
pub fn android_config_line(preset: &[String], path: &str) -> String {
    let base = path.strip_suffix('/').unwrap_or(path);
    let joined = preset
        .iter()
        .map(|s| format!("{base}/{s}"))
        .collect::<Vec<_>>()
        .join(":");
    format!("glsl-shaders={joined}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_quality_mode_combos_build() {
        for q in QUALITIES {
            for m in MODES {
                let preset = create_preset(q, m).unwrap();
                assert!(!preset.is_empty(), "{q}/{m}");
                assert_eq!(preset[0], "Anime4K_Clamp_Highlights.glsl");
            }
        }
    }

    #[test]
    fn secondary_quality_map() {
        assert_eq!(secondary_quality("S"), Some("S"));
        assert_eq!(secondary_quality("M"), Some("S"));
        assert_eq!(secondary_quality("L"), Some("M"));
        assert_eq!(secondary_quality("VL"), Some("M"));
        assert_eq!(secondary_quality("UL"), Some("L"));
        assert_eq!(secondary_quality("XXL"), None);
    }

    #[test]
    fn unknown_inputs_error() {
        assert!(create_preset("XXL", "A").is_err());
        assert!(create_preset("M", "Z").is_err());
    }

    #[test]
    fn preset_to_string_uses_os_separator() {
        let dir = Path::new("/shaders");
        let s = preset_to_glsl_shaders(dir, &["a.glsl".to_string(), "b.glsl".to_string()]);
        #[cfg(windows)]
        assert!(s.contains(';'));
        #[cfg(not(windows))]
        assert_eq!(s, "/shaders/a.glsl:/shaders/b.glsl");
    }

    #[test]
    fn android_config_trims_trailing_slash() {
        let preset = vec!["x.glsl".to_string()];
        assert_eq!(
            android_config_line(&preset, "/storage/mpv/shaders/"),
            "glsl-shaders=/storage/mpv/shaders/x.glsl"
        );
    }

    #[test]
    fn standard_and_ultra_counts() {
        assert_eq!(standard_presets().len(), 9);
        assert_eq!(ultra_hq_presets().len(), 8);
    }
}
