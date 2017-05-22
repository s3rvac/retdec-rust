//! Internal utilities.

/// Returns the name of the current platform (e.g. `"Linux"`).
///
/// When the name cannot be detected, it returns `"Unknown"`.
pub fn current_platform_name() -> &'static str {
    if cfg!(target_os = "linux") {
        "Linux"
    } else if cfg!(target_os = "windows") {
        "Windows"
    } else if cfg!(target_os = "macos") {
        "macOS"
    } else if cfg!(target_os = "ios") {
        "iOS"
    } else if cfg!(target_os = "android") {
        "Android"
    } else if cfg!(target_os = "freebsd") {
        "FreeBSD"
    } else if cfg!(target_os = "netbsd") {
        "NetBSD"
    } else if cfg!(target_os = "openbsd") {
        "OpenBSD"
    } else {
        "Unknown"
    }
}
