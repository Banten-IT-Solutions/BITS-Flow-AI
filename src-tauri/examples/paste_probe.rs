//! Headless smoke test for the clipboard image paste chain (Linux only).
//!
//! Reproduces the real pipeline without the app: GTK clipboard image ->
//! Ctrl+V key hook -> stop WebKit's paste -> eval the inject script ->
//! page records what its paste handler saw.
//!
//! Run under Xvfb and drive with xdotool:
//!   Xvfb :105 &
//!   DISPLAY=:105 cargo run --example paste_probe &
//!   DISPLAY=:105 xdotool search --name paste-probe windowfocus key ctrl+v
//! Expected stdout: TITLE:PASTE:1:untrusted (or INPUT:1 if the page's paste
//! handler does not preventDefault and the file-input tier takes over).

use bits_flow_ai_lib::paste_fallback::{PASTE_INJECT_SCRIPT, read_clipboard_image_gtk};
use gtk::prelude::*;
use webkit2gtk::{
    LoadEvent, UserContentInjectedFrames, UserContentManager, UserContentManagerExt, UserScript,
    UserScriptInjectionTime, WebView, WebViewExt,
};

/// Tier-1 page: paste handler accepts the synthetic event.
const PAGE: &str = r#"<!doctype html><html><body>
<div id="e" contenteditable="true">type here</div>
<input id="f" type="file" style="display:none">
<script>
document.addEventListener('paste', function (ev) {
  var files = ev.clipboardData ? ev.clipboardData.files.length : -1;
  document.title = 'PASTE:' + files + ':' + (ev.isTrusted ? 'trusted' : 'untrusted');
  if (files > 0) ev.preventDefault();
});
document.getElementById('f').addEventListener('change', function () {
  document.title = 'INPUT:' + document.getElementById('f').files.length;
});
</script></body></html>"#;

/// Tier-2 page: no paste handler (like an editor that ignores untrusted
/// pastes), so the file-input tier must take over.
const PAGE_INPUT: &str = r#"<!doctype html><html><body>
<div id="e" contenteditable="true">type here</div>
<input id="f" type="file" style="display:none">
<script>
document.getElementById('f').addEventListener('change', function () {
  document.title = 'INPUT:' + document.getElementById('f').files.length;
});
</script></body></html>"#;

fn main() {
    gtk::init().expect("gtk init");

    let window = gtk::ApplicationWindow::builder()
        .title("paste-probe")
        .default_width(900)
        .default_height(700)
        .build();

    let manager = UserContentManager::new();
    manager.add_script(&UserScript::new(
        PASTE_INJECT_SCRIPT,
        UserContentInjectedFrames::TopFrame,
        UserScriptInjectionTime::Start,
        &[],
        &[],
    ));

    let view = WebView::with_user_content_manager(&manager);
    let page = if std::env::var("PROBE_TIER").as_deref() == Ok("input") {
        PAGE_INPUT
    } else {
        PAGE
    };
    view.load_html(page, None);
    window.add(&view);
    window.show_all();
    view.grab_focus();

    view.connect_load_changed(|_, event| {
        if event == LoadEvent::Finished {
            // 4x4 red image on the clipboard, owned by this process.
            let pixbuf =
                gtk::gdk_pixbuf::Pixbuf::new(gtk::gdk_pixbuf::Colorspace::Rgb, false, 8, 4, 4)
                    .expect("pixbuf");
            pixbuf.fill(0xff0000ff);
            let clipboard = gtk::Clipboard::get(&gtk::gdk::SELECTION_CLIPBOARD);
            clipboard.set_image(&pixbuf);
            println!("READY");
        }
    });

    view.connect_notify_local(Some("title"), |view, _| {
        println!("TITLE:{}", view.title().unwrap_or_default());
        std::io::Write::flush(&mut std::io::stdout()).ok();
    });

    let eval_view = view.clone();
    view.connect_key_press_event(move |view, event| {
        let state = event.state();
        let keyval = event.keyval();
        let paste = state.contains(gtk::gdk::ModifierType::CONTROL_MASK)
            && (keyval == gtk::gdk::keys::constants::v || keyval == gtk::gdk::keys::constants::V);
        if paste {
            if let Some(png) = read_clipboard_image_gtk() {
                gtk::glib::signal::signal_stop_emission_by_name(view, "key-press-event");
                use base64::Engine as _;
                let b64 = base64::engine::general_purpose::STANDARD.encode(png);
                eval_view.evaluate_javascript(
                    &format!(
                        "window.__BITS_PASTE_INJECT__ && window.__BITS_PASTE_INJECT__({b64:?})"
                    ),
                    None,
                    None,
                    None::<&gtk::gio::Cancellable>,
                    |_| {},
                );
                return gtk::glib::Propagation::Stop;
            }
        }
        gtk::glib::Propagation::Proceed
    });

    gtk::main();
}
