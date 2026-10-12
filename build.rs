//! Embeds the app icon (and version info) into the Windows `.exe`.
//!
//! No-op on other platforms. Requires `rc.exe` (Windows SDK) or `windres`
//! (MinGW) when targeting Windows; the build fails with an explicit message
//! if the resource compiler is missing, so a missing icon never goes
//! unnoticed.

fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let mut res = winres::WindowsResource::new();
    res.set_icon("assets/favicon.ico");
    res.set("ProductName", "Anime Player");
    res.set(
        "FileDescription",
        "Anime Player — video player with Anime4K upscaling",
    );
    res.set("CompanyName", "avencores");
    if let Err(err) = res.compile() {
        panic!("winres: failed to embed the app icon ({err:#}). Install the Windows SDK (rc.exe) or MinGW (windres).");
    }
}
