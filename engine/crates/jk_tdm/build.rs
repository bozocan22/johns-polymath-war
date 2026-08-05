// Embeds the app icon as a PE resource on Windows, so the exe itself
// carries the icon - Explorer, Alt-Tab, the taskbar, and any shortcut
// that doesn't override IconLocation all pick it up automatically.
// A missing/failed embed is a cosmetic loss, never a build blocker: it
// warns and lets the build continue rather than panicking.
fn main() {
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("../../assets/branding/app_icon.ico");
        res.set("ProductName", "John Kingdom Game");
        res.set("FileDescription", "John Kingdom Game");
        if let Err(e) = res.compile() {
            println!("cargo:warning=failed to embed app icon: {e}");
        }
    }
}
