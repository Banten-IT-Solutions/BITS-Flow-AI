fn main() {
    tauri_build::try_build(tauri_build::Attributes::new().plugin(
        "paste-fallback",
        tauri_build::InlinedPlugin::new().commands(&["read_clipboard_image"]),
    ))
    .expect("failed to run tauri-build");
}
