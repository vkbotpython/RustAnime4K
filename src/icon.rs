//! App icon bytes, shared by the window icon (`main.rs`) and the About
//! dialog (`about.rs`). A single `include_bytes!` static instead of two
//! identical ones saves ~295KB in the binary (LTO does not merge duplicate
//! statics across modules).
pub static ICON_PNG: &[u8] = include_bytes!("../assets/images/anime4k-player-icon.png");
