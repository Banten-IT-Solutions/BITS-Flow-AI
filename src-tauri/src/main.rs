// Prevents an additional console window on Windows in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // WebKitGTK GPU init (DMABUF renderer / hardware compositing) crashes the
    // windowing event loop on several Linux stacks (NVIDIA, Wayland): the
    // window blinks once and dies. Force software rendering paths unless the
    // user explicitly overrides. Must run before any webview is created.
    // ponytail: remove when webkit2gtk ships a durable GPU init fix.
    // SAFETY: `set_var` is unsafe in edition 2024 because env access is not
    // thread-safe; here it runs in `main` before any thread is spawned.
    unsafe {
        if std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none() {
            std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        }
        if std::env::var_os("WEBKIT_DISABLE_COMPOSITING_MODE").is_none() {
            std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
        }
    }
    bits_flow_ai_lib::run()
}
