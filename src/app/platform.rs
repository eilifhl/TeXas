use std::{env, path::Path};

pub fn open_path_with_default_app(path: &Path) -> std::io::Result<()> {
    #[cfg(target_os = "linux")]
    {
        let mut command = std::process::Command::new("xdg-open");
        command.arg(path);

        if env::var_os("WAYLAND_DISPLAY").is_none() {
            if let Some(wayland_display) = discover_wayland_display() {
                command.env("WAYLAND_DISPLAY", wayland_display);
            }
        }

        command.spawn().map(|_| ())
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(path)
            .spawn()
            .map(|_| ())
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", ""])
            .arg(path)
            .spawn()
            .map(|_| ())
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "opening PDFs is not supported on this platform",
        ))
    }
}

#[cfg(target_os = "linux")]
fn discover_wayland_display() -> Option<String> {
    use std::{env, fs};

    let runtime_dir = env::var_os("XDG_RUNTIME_DIR")?;
    let mut candidates = fs::read_dir(runtime_dir)
        .ok()?
        .filter_map(Result::ok)
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| name.starts_with("wayland-"))
        .collect::<Vec<_>>();

    candidates.sort();
    candidates.into_iter().next()
}
