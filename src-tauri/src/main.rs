// Prevents an additional console window on Windows in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::os::unix::fs::MetadataExt;
use std::os::unix::process::CommandExt;
use std::process::Command;

/// Effective uid of this process, read from /proc (no libc dependency).
fn effective_uid() -> Option<u32> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    let line = status.lines().find(|l| l.starts_with("Uid:"))?;
    // Uid: real effective saved fs
    line.split_whitespace().nth(2)?.parse().ok()
}

/// Owner username for a uid, from /etc/passwd.
fn username_for_uid(uid: u32) -> Option<String> {
    let passwd = std::fs::read_to_string("/etc/passwd").ok()?;
    passwd
        .lines()
        .find(|l| l.split(':').nth(2) == Some(&uid.to_string()))
        .and_then(|l| l.split(':').next().map(str::to_string))
}

/// First non-root process connected to `display`, by scanning /proc
/// environ files. Its uid owns the desktop session; returns the uid plus
/// the raw NUL-separated environment. Scanning (not just Xorg) is needed
/// because Xorg often runs as root (lightdm) with no usable session env.
fn session_proc_for_display(display: &str) -> Option<(u32, Vec<u8>)> {
    let mut dirs: Vec<_> = std::fs::read_dir("/proc")
        .ok()?
        .filter_map(Result::ok)
        .filter(|e| {
            e.file_name()
                .to_str()
                .is_some_and(|n| n.chars().all(|c| c.is_ascii_digit()))
        })
        .collect();
    dirs.sort_by_key(|e| {
        e.file_name()
            .to_str()
            .and_then(|n| n.parse::<u32>().ok())
            .unwrap_or(0)
    });
    for entry in dirs {
        let pid_dir = entry.path();
        let Ok(meta) = std::fs::metadata(&pid_dir) else {
            continue;
        };
        if meta.uid() == 0 {
            continue;
        }
        let Ok(env) = std::fs::read(pid_dir.join("environ")) else {
            continue;
        };
        if environ_value(&env, "DISPLAY") == Some(display)
            && environ_value(&env, "XAUTHORITY").is_some()
        {
            return Some((meta.uid(), env));
        }
    }
    None
}

fn environ_value<'a>(env: &'a [u8], key: &str) -> Option<&'a str> {
    let prefix = format!("{key}=");
    env.split(|b| *b == 0)
        .filter_map(|e| std::str::from_utf8(e).ok())
        .find_map(|s| s.strip_prefix(&prefix))
}

/// When started as root against an existing X session (e.g. `DISPLAY=:0
/// bits-flow-ai` over SSH), hand the process over to the user owning the X
/// session: GTK/WebKit as root breaks portal activation (~25s startup freeze)
/// and running desktop apps as root is unsafe. `ssh -X` forwarding is skipped:
/// root's forwarded cookie would be lost by switching users.
/// ponytail: revisit if GTK ships a startup that does not auto-start portals.
fn hand_over_to_session_user() {
    if effective_uid() != Some(0) {
        return;
    }
    let display = match std::env::var("DISPLAY") {
        Ok(d) if !d.is_empty() => d,
        _ => return,
    };
    if display.starts_with("localhost:") {
        return;
    }
    let Some((uid, env)) = session_proc_for_display(&display) else {
        return;
    };
    let Some(user) = username_for_uid(uid) else {
        return;
    };
    let exe = match std::env::current_exe() {
        Ok(e) => e,
        Err(_) => return,
    };
    let run_dir = format!("/run/user/{uid}");
    if !std::path::Path::new(&run_dir).exists() {
        return;
    }
    let _ = Command::new("runuser")
        .arg("-u")
        .arg(&user)
        .arg("--")
        .arg(exe)
        .args(std::env::args_os().skip(1))
        .env("DISPLAY", &display)
        .env("XDG_RUNTIME_DIR", &run_dir)
        .env(
            "DBUS_SESSION_BUS_ADDRESS",
            format!("unix:path={run_dir}/bus"),
        )
        .envs(
            environ_value(&env, "XAUTHORITY")
                .map(|x| [("XAUTHORITY", x.to_string())])
                .unwrap_or_default(),
        )
        .exec();
    // exec() only returns on failure; fall through and run as-is.
}

fn main() {
    hand_over_to_session_user();
    // WebKitGPU notes (AMD Radeon + webkit2gtk 2.52, Mint 22.3, verified by
    // pixel-diff sampling on the affected machine, Nov 2026):
    // - forcing WEBKIT_DISABLE_COMPOSITING_MODE makes the whole window
    //   flicker (light<->dark full-frame repaints) on composited desktops;
    // - leaving the DMABUF renderer enabled flickers even harder.
    // So force ONLY the DMABUF renderer off, and honor explicit overrides.
    // ponytail: remove when webkit2gtk ships a durable GPU init fix.
    // SAFETY: `set_var` is unsafe in edition 2024 because env access is not
    // thread-safe; here it runs in `main` before any thread is spawned.
    unsafe {
        if std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none() {
            std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        }
    }
    bits_flow_ai_lib::run()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_user_resolves() {
        assert_eq!(username_for_uid(0).as_deref(), Some("root"));
    }

    #[test]
    fn environ_lookup_finds_exact_key_only() {
        let env = b"DISPLAY=:0\0XAUTHORITY=/home/u/.Xauthority\0XDISPLAYX=oops\0";
        assert_eq!(environ_value(env, "DISPLAY"), Some(":0"));
        assert_eq!(
            environ_value(env, "XAUTHORITY"),
            Some("/home/u/.Xauthority")
        );
        assert_eq!(environ_value(env, "XDISPLAY"), None);
        assert_eq!(environ_value(env, "MISSING"), None);
    }

    #[test]
    fn display_scan_without_session_finds_nothing() {
        // No X session on the test machine uses this display; must not match
        // partial values (":99x") or processes lacking XAUTHORITY.
        assert_eq!(session_proc_for_display(":99x"), None);
    }

    #[test]
    fn non_root_uid_short_circuits_handover() {
        // Can't force euid in a unit test; pin the parsing instead.
        let status = std::fs::read_to_string("/proc/self/status").unwrap();
        let line = status.lines().find(|l| l.starts_with("Uid:")).unwrap();
        let euid: u32 = line.split_whitespace().nth(2).unwrap().parse().unwrap();
        let _ = effective_uid();
        assert_eq!(effective_uid(), Some(euid));
    }
}
