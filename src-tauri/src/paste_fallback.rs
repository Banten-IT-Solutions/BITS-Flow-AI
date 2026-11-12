//! Clipboard image paste fallback for persona page webviews.
//!
//! WebKitGTK does not expose native clipboard images as files in the JS
//! `paste` event (`clipboardData.files/items` stay empty), so provider
//! editors (e.g. Gemini) see an empty paste. This module registers an
//! inlined plugin that serves the GTK clipboard image as base64 PNG, and
//! an initialization script that synthesizes a real `paste` event with a
//! `File` when the native event carries none. The synthetic event is a
//! plain `Event` with `clipboardData` shadowed via defineProperty (WebKit's
//! ClipboardEvent constructor drops init clipboardData), and when the page
//! ignores it (`defaultPrevented` false) the fallback drives the page's own
//! `input[type=file]` so the upload still goes through the site pipeline.
//! ponytail: upgrade path from `src/lib/clipboard.ts` (cross-platform
//! would need tauri-plugin-clipboard-manager; we reuse gtk instead).
//! Trust boundary: remote pages granted this permission could read
//! clipboard image bytes at any time — acceptable only because users
//! explicitly open those providers in persona windows.
//! ponytail: main-frame only (cross-origin iframe editors not covered);
//! non-Linux returns None (native paste expected to work there).

/// Priority order of clipboard image MIME targets to accept.
const IMAGE_TARGETS: [&str; 4] = ["image/png", "image/jpeg", "image/bmp", "image/tiff"];

/// Pick the first clipboard image target worth fetching, by priority.
fn pick_image_target<'a, T: AsRef<str>>(names: &'a [T]) -> Option<&'a str> {
    IMAGE_TARGETS
        .iter()
        .find(|t| names.iter().any(|n| n.as_ref() == **t))
        .copied()
}

pub const PASTE_FALLBACK_SCRIPT: &str = r#"
(() => {
  if (window.__BITS_PASTE__) return;
  window.__BITS_PASTE__ = true;
  const invoke = window.__TAURI_INTERNALS__ && window.__TAURI_INTERNALS__.invoke;
  if (!invoke) return;
  const hasFile = (items) => {
    for (const it of items) if (it.kind === 'file') return true;
    return false;
  };
  // Copy every string payload so pure-text pastes survive the intercept.
  const harvest = (dt) => {
    const out = [];
    for (const t of dt.types) {
      if (t === 'Files') continue;
      const d = dt.getData(t);
      if (d) out.push([t, d]);
    }
    return out;
  };
  document.addEventListener('paste', (ev) => {
    if (!ev.isTrusted) return;
    const dt = ev.clipboardData;
    if (!dt) return;
    if (hasFile(dt.items)) return; // native image paste already works
    const el = document.activeElement;
    // Plain inputs/textareas can only take text; keep their native paste.
    if (el && (el.tagName === 'INPUT' || el.tagName === 'TEXTAREA')) return;
    const strings = harvest(dt);
    // Must cancel both: without preventDefault WebKitGTK still runs the
    // default paste and dumps raw html/img into the editor.
    ev.preventDefault();
    ev.stopImmediatePropagation();
    invoke('plugin:paste-fallback|read_clipboard_image')
      .then((b64) => {
        const out = new DataTransfer();
        if (b64) {
          const bin = atob(b64);
          const bytes = new Uint8Array(bin.length);
          for (let i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i);
          out.items.add(new File([bytes], 'clipboard-image.png', { type: 'image/png' }));
          try { out.setData('Files', ''); } catch {} // Chrome-style pseudo type
        }
        for (const [t, d] of strings) {
          try { out.setData(t, d); } catch {}
        }
        // WebKit's ClipboardEvent constructor ignores clipboardData in the
        // init dict (added only in Safari 17.4), so build a plain event and
        // shadow the getter — handlers reading ev.clipboardData get ours.
        const synth = new Event('paste', { bubbles: true, cancelable: true });
        Object.defineProperty(synth, 'clipboardData', { value: out });
        (document.activeElement || document.body).dispatchEvent(synth);
        // Editors that ignore untrusted pastes leave defaultPrevented false;
        // drive their own file input so the image still reaches the real
        // upload pipeline instead of dying silently.
        if (b64 && !synth.defaultPrevented) {
          const inputs = document.querySelectorAll('input[type=file]');
          const input =
            [...inputs].find((i) => !i.accept || /image/i.test(i.accept)) || inputs[0];
          if (input) {
            try {
              input.files = out.files;
              input.dispatchEvent(new Event('change', { bubbles: true }));
            } catch {}
          }
        }
      })
      .catch(() => {});
  }, true);
})();
"#;

/// Return the GTK clipboard image as base64 PNG, or None when the
/// clipboard holds no image. Runs on the command thread; GTK calls are
/// hopped to the main thread.
#[tauri::command]
pub fn read_clipboard_image(window: tauri::WebviewWindow) -> Result<Option<String>, String> {
    #[cfg(not(target_os = "linux"))]
    {
        let _ = &window;
        Ok(None)
    }

    #[cfg(target_os = "linux")]
    {
        use std::sync::mpsc;

        let (tx, rx) = mpsc::channel();
        window
            .run_on_main_thread(move || {
                let bytes = read_clipboard_image_gtk();
                let _ = tx.send(bytes);
            })
            .map_err(|e| e.to_string())?;
        let bytes = rx
            .recv_timeout(std::time::Duration::from_millis(1500))
            .map_err(|_| "clipboard read timed out".to_string())?;
        Ok(bytes.map(|b| {
            use base64::Engine as _;
            base64::engine::general_purpose::STANDARD.encode(b)
        }))
    }
}

/// GTK clipboard read; main thread only.
#[cfg(target_os = "linux")]
fn read_clipboard_image_gtk() -> Option<Vec<u8>> {
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
