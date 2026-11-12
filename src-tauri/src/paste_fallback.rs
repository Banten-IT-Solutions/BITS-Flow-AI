//! Clipboard image paste fallback for persona page webviews.
//!
//! WebKitGTK does not expose native clipboard images as files in the JS
//! `paste` event (`clipboardData.files/items` stay empty), so provider
//! editors (e.g. Gemini) see an empty paste.
//!
//! The JS->Rust direction is a dead end here: Tauri's IPC on Linux is a
//! `fetch` to `http://ipc.localhost`, which provider CSPs (`connect-src`)
//! block, and a sync command would run inline on the main thread anyway.
//! So the flow is inverted and never uses IPC:
//!
//! 1. `install_paste_hook` connects GTK `key-press-event` on the page
//!    webview; on Ctrl+V it reads the GTK clipboard image (same toolkit as
//!    the webview, works on X11 and Wayland) and stops the emission so
//!    WebKitGTK cannot dump the image inline into the editor.
//! 2. The image is base64-encoded and `eval`-ed into the page as
//!    `window.__BITS_PASTE_INJECT__(b64)` — native evaluation is not
//!    subject to the page CSP.
//! 3. The injected function hands the image to the editor in three tiers:
//!    a synthetic paste event carrying a real `File`, then the page's own
//!    `input[type=file]` (+ change event), then a synthetic `drop`.
//!
//! ponytail: covers Ctrl+V in the main frame only; context-menu paste and
//! cross-origin iframe editors are out of scope. Non-Linux returns early
//! (native paste expected to work there).

/// Init script: defines the injector called by the Rust paste hook.
/// Built as plain `Event`s with the payload shadowed via `defineProperty`
/// because WebKit's `ClipboardEvent`/`DragEvent` constructors drop
/// `clipboardData`/`dataTransfer` from their init dicts (only Safari 17.4+).
pub const PASTE_INJECT_SCRIPT: &str = r#"
(() => {
  if (window.__BITS_PASTE_INJECT__) return;
  const fileInputs = () => {
    const out = [];
    const walk = (root) => {
      for (const el of root.querySelectorAll('*')) {
        if (el.tagName === 'INPUT' && el.type === 'file') out.push(el);
        if (el.shadowRoot) walk(el.shadowRoot);
        if (el.tagName === 'IFRAME') {
          try {
            if (el.contentDocument) walk(el.contentDocument);
          } catch (e) {}
        }
      }
    };
    walk(document);
    return out;
  };
  window.__BITS_PASTE_INJECT__ = (b64) => {
    try {
      const bin = atob(b64);
      const bytes = new Uint8Array(bin.length);
      for (let i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i);
      const dt = new DataTransfer();
      dt.items.add(new File([bytes], 'clipboard-image.png', { type: 'image/png' }));
      try { dt.setData('Files', ''); } catch (e) {} // Chrome-style pseudo type
      const target = document.activeElement || document.body;
      const paste = new Event('paste', { bubbles: true, cancelable: true });
      Object.defineProperty(paste, 'clipboardData', { value: dt });
      target.dispatchEvent(paste);
      if (paste.defaultPrevented) return 'paste';
      const inputs = fileInputs();
      const input = inputs.find((i) => !i.accept || /image/i.test(i.accept)) || inputs[0];
      if (input) {
        try {
          input.files = dt.files;
          input.dispatchEvent(new Event('change', { bubbles: true }));
          return 'file-input';
        } catch (e) {}
      }
      const drop = new Event('drop', { bubbles: true, cancelable: true });
      Object.defineProperty(drop, 'dataTransfer', { value: dt });
      target.dispatchEvent(drop);
      return drop.defaultPrevented ? 'drop' : 'none';
    } catch (e) {
      return 'error';
    }
  };
})();
"#;

/// Priority order of clipboard image MIME targets to accept.
const IMAGE_TARGETS: [&str; 4] = ["image/png", "image/jpeg", "image/bmp", "image/tiff"];

/// Pick the first clipboard image target worth fetching, by priority.
fn pick_image_target<'a, T: AsRef<str>>(names: &'a [T]) -> Option<&'a str> {
    IMAGE_TARGETS
        .iter()
        .find(|t| names.iter().any(|n| n.as_ref() == **t))
        .copied()
}

/// Hook Ctrl+V on the page webview: read the clipboard image and hand it to
/// the page injector instead of letting WebKitGTK paste it inline.
#[cfg(target_os = "linux")]
pub fn install_paste_hook(webview: &tauri::Webview) {
    let dispatcher = webview.clone();
    let _ = webview.with_webview(move |platform| {
        use gtk::glib::signal::signal_stop_emission_by_name;
        use gtk::prelude::*;

        let webview = dispatcher.clone();
        let view = platform.inner();
        view.connect_key_press_event(move |view, event| {
            if !is_paste_shortcut(event) {
                return gtk::glib::Propagation::Proceed;
            }
            let Some(png) = read_clipboard_image_gtk() else {
                return gtk::glib::Propagation::Proceed;
            };
            // WebKitGTK would insert the image inline into the editor;
            // stop its paste and inject our file-carrying event instead.
            signal_stop_emission_by_name(view, "key-press-event");
            use base64::Engine as _;
            let b64 = base64::engine::general_purpose::STANDARD.encode(png);
            let _ = webview.eval(format!(
                "window.__BITS_PASTE_INJECT__ && window.__BITS_PASTE_INJECT__({b64:?})"
            ));
            gtk::glib::Propagation::Stop
        });
    });
}

#[cfg(not(target_os = "linux"))]
pub fn install_paste_hook(_webview: &tauri::Webview) {}

/// Ctrl+V without Shift (Ctrl+Shift+V is paste-plain and stays native).
#[cfg(target_os = "linux")]
fn is_paste_shortcut(event: &gtk::gdk::EventKey) -> bool {
    use gtk::gdk;
    let state = event.state();
    let keyval = event.keyval();
    state.contains(gdk::ModifierType::CONTROL_MASK)
        && !state.contains(gdk::ModifierType::SHIFT_MASK)
        && (keyval == gdk::keys::constants::v || keyval == gdk::keys::constants::V)
}

/// GTK clipboard read; main thread only.
#[cfg(target_os = "linux")]
pub fn read_clipboard_image_gtk() -> Option<Vec<u8>> {
    let clipboard = gtk::Clipboard::get(&gtk::gdk::SELECTION_CLIPBOARD);
    let targets = clipboard.wait_for_targets()?;
    let names: Vec<String> = targets.iter().map(|a| a.name().to_string()).collect();
    let target = pick_image_target(&names)?;
    let data = clipboard.wait_for_contents(&gtk::gdk::Atom::intern(target))?;
    let pixbuf = data.pixbuf()?;
    pixbuf.save_to_bufferv("png", &[]).ok()
}

#[cfg(test)]
mod tests {
    use super::pick_image_target;

    #[test]
    fn prefers_png_then_jpeg_then_none() {
        assert_eq!(
            pick_image_target(&["image/jpeg", "image/png"]),
            Some("image/png")
        );
        assert_eq!(
            pick_image_target(&["image/jpeg", "image/tiff"]),
            Some("image/jpeg")
        );
        assert_eq!(pick_image_target(&["text/plain"]), None);
    }
}
