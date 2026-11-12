//! Isolated persona windows: one WebKitGTK data dir per account, one visible
//! persona at a time, with a naive LRU cap on open persona windows.
//! Each persona window hosts a small toolbar webview (provider + account
//! switcher, `toolbar.html`) above the provider page webview, so switching
//! never touches the manager window.
//! ponytail: no proxy-per-profile, no cookie import/export, eviction is
//! open-order based (hidden windows only); revisit when reuse stats matter.

use std::collections::HashMap;
use std::sync::Mutex;

use serde::Serialize;
use tauri::window::WindowBuilder;
use tauri::{
    AppHandle, LogicalPosition, LogicalSize, Manager, State, WebviewBuilder, WebviewUrl, Window,
};

use crate::commands;
use crate::paste_fallback;

/// Toolbar strip height in logical pixels — one standard toolbar row.
const BAR_HEIGHT: f64 = 36.0;

/// Chrome UA so providers that gate on WebView/Headless UA behave like a
/// normal desktop Chrome.
pub const CHROME_USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";

#[derive(Default)]
pub struct PersonaState {
    /// Persona window labels in open order, oldest first.
    pub order: Mutex<Vec<String>>,
    /// Persona label -> provider URL its page webview currently shows.
    pub urls: Mutex<HashMap<String, String>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenPersonaResult {
    pub label: String,
}

fn persona_label(persona_id: &str) -> String {
    format!("persona-{persona_id}")
}

pub(crate) fn is_valid_persona_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 128
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// Pin the GTK layout of a persona window. tauri's `add_child` on Linux
/// packs child webviews into the window's vertical `GtkBox` with
/// `expand=true`, ignoring the bounds passed to `add_child` (wry only
/// honours bounds inside a `GtkFixed`). So instead of sizing webviews
/// manually, pin the bar's height and let the page expand into the rest —
/// GTK then re-lays out on every window resize for free.
#[cfg(target_os = "linux")]
fn pin_gtk_layout(window: &Window, bar_height: f64) {
    use gtk::prelude::*;

    let win = window.clone();
    // GTK calls are main-thread only; add_child already finished there, so
    // both child webviews exist by the time this closure runs.
    let _ = window.run_on_main_thread(move || {
        let Ok(vbox) = win.default_vbox() else {
            return;
        };
        let children = vbox.children();
        let count = children.len();
        if count < 2 {
            return;
        }
        let bar = &children[count - 2];
        let page = &children[count - 1];
        bar.set_size_request(-1, bar_height as i32);
        vbox.set_child_packing(bar, false, true, 0, gtk::PackType::Start);
        vbox.set_child_packing(page, true, true, 0, gtk::PackType::Start);
        vbox.queue_resize();
    });
}

/// Pure LRU plan: which labels to close so the open count fits `cap`
/// (one slot is reserved for `current`). Oldest first.
fn evict_plan(order: &[String], current: &str, cap: u32) -> Vec<String> {
    if cap == 0 {
        return Vec::new();
    }
    let others: Vec<&String> = order.iter().filter(|l| *l != current).collect();
    let allowed = cap.saturating_sub(1) as usize;
    if others.len() <= allowed {
        return Vec::new();
    }
    others[..others.len() - allowed]
        .iter()
        .map(|l| (*l).clone())
        .collect()
}

#[tauri::command]
pub fn open_persona(
    app: AppHandle,
    state: State<'_, PersonaState>,
    persona_id: String,
    url: String,
    engine: String,
) -> Result<OpenPersonaResult, String> {
    if !is_valid_persona_id(&persona_id) {
        return Err(format!("invalid persona id: {persona_id:?}"));
    }
    let parsed = tauri::Url::parse(&url).map_err(|_| format!("invalid URL: {url}"))?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err(format!("unsupported URL scheme: {}", parsed.scheme()));
    }
    let label = persona_label(&persona_id);
    let cap = commands::load_settings(&app)?.lru_size;

    match app.get_window(&label) {
        // Existing persona: reveal; only navigate when the provider actually
        // changes so a plain reveal never reloads (and loses) page state.
        Some(window) => {
            let page_label = format!("{label}-page");
            let page = app
                .get_webview(&page_label)
                .ok_or_else(|| format!("persona webview missing: {page_label}"))?;
            if page.url().ok().as_ref() != Some(&parsed) {
                page.navigate(parsed.clone()).map_err(|e| e.to_string())?;
            }
            window.show().map_err(|e| e.to_string())?;
            window.set_focus().map_err(|e| e.to_string())?;
            state
                .urls
                .lock()
                .map_err(|_| "persona url lock poisoned".to_string())?
                .insert(label.clone(), url.clone());
        }
        None => {
            let profile_dir = commands::app_data_dir(&app)?
                .join("profiles")
                .join(&persona_id);
            let window = WindowBuilder::new(&app, &label)
                .title(&persona_id)
                .inner_size(1120.0, 760.0)
                .visible(false)
                .build()
                .map_err(|e| e.to_string())?;
            window
                .add_child(
                    WebviewBuilder::new(
                        format!("{label}-bar"),
                        WebviewUrl::App("toolbar.html".into()),
                    )
                    .initialization_script(format!("window.__PERSONA_ID__ = {persona_id:?};")),
                    LogicalPosition::new(0.0, 0.0),
                    LogicalSize::new(1120.0, BAR_HEIGHT),
                )
                .map_err(|e| e.to_string())?;
            let mut page = WebviewBuilder::new(
                format!("{label}-page"),
                WebviewUrl::External(parsed.clone()),
            )
            .initialization_script(paste_fallback::PASTE_INJECT_SCRIPT)
            // OAuth sign-in (Google, etc.) opens a popup via window.open.
            // Without a handler wry never wires WebKitGTK's create signal,
            // window.open returns null and providers show "Unable to sign
            // in". Allow opens a related-view window sharing the persona
            // session (cookies) and keeping window.opener alive.
            .on_new_window(|url, _features| {
                if matches!(url.scheme(), "http" | "https" | "about") {
                    tauri::webview::NewWindowResponse::Allow
                } else {
                    tauri::webview::NewWindowResponse::Deny
                }
            })
            .data_directory(profile_dir);
            if engine == "chrome" {
                page = page.user_agent(CHROME_USER_AGENT);
            }
            let page_webview = window
                .add_child(
                    page,
                    LogicalPosition::new(0.0, BAR_HEIGHT),
                    LogicalSize::new(1120.0, 760.0 - BAR_HEIGHT),
                )
                .map_err(|e| e.to_string())?;
            paste_fallback::install_paste_hook(&page_webview);
            // tauri packs both children into a plain GtkBox (expand=true), so
            // the bounds above are ignored on Linux. Pin the real layout.
            #[cfg(target_os = "linux")]
            pin_gtk_layout(&window, BAR_HEIGHT);
            window.show().map_err(|e| e.to_string())?;
            window.set_focus().map_err(|e| e.to_string())?;
            state
                .urls
                .lock()
                .map_err(|_| "persona url lock poisoned".to_string())?
                .insert(label.clone(), url.clone());
        }
    }

    // Exactly one visible persona at a time.
    for (other_label, window) in app.windows() {
        if other_label.starts_with("persona-") && other_label != label {
            let _ = window.hide();
        }
    }

    let mut order = state
        .order
        .lock()
        .map_err(|_| "persona order lock poisoned".to_string())?;
    order.retain(|l| l != &label);
    order.push(label.clone());

    // Evict the oldest hidden personas over the cap. Visible windows are kept
    // (they are re-hidden above only when they are not the current persona).
    let mut closed: Vec<String> = Vec::new();
    for victim in evict_plan(&order, &label, cap) {
        if let Some(window) = app.get_window(&victim) {
            if window.is_visible().unwrap_or(true) {
                continue;
            }
            let _ = window.close();
        }
        closed.push(victim);
    }
    order.retain(|l| !closed.contains(l));
    drop(order);

    Ok(OpenPersonaResult { label })
}

/// Navigate an open persona's page webview to another provider. The profile
/// (cookies) stays with the account, so this is a same-profile switch.
#[tauri::command]
pub fn switch_persona_provider(
    app: AppHandle,
    state: State<'_, PersonaState>,
    persona_id: String,
    url: String,
) -> Result<(), String> {
    if !is_valid_persona_id(&persona_id) {
        return Err(format!("invalid persona id: {persona_id:?}"));
    }
    let parsed = tauri::Url::parse(&url).map_err(|_| format!("invalid URL: {url}"))?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err(format!("unsupported URL scheme: {}", parsed.scheme()));
    }
    let label = persona_label(&persona_id);
    let page = app
        .get_webview(&format!("{label}-page"))
        .ok_or_else(|| "persona window is not open".to_string())?;
    page.navigate(parsed).map_err(|e| e.to_string())?;
    state
        .urls
        .lock()
        .map_err(|_| "persona url lock poisoned".to_string())?
        .insert(label, url);
    Ok(())
}

/// Current provider URL of a persona window (null when not open). The
/// toolbar uses it to preselect the provider switcher.
#[tauri::command]
pub fn persona_state(
    state: State<'_, PersonaState>,
    persona_id: String,
) -> Result<Option<String>, String> {
    if !is_valid_persona_id(&persona_id) {
        return Err(format!("invalid persona id: {persona_id:?}"));
    }
    let urls = state
        .urls
        .lock()
        .map_err(|_| "persona url lock poisoned".to_string())?;
    Ok(urls.get(&persona_label(&persona_id)).cloned())
}

/// Labels of every currently-open persona window (`persona-*`), for the
/// Home "open" badge. Sorted for stable UI ordering.
#[tauri::command]
pub fn list_open_personas(app: AppHandle) -> Result<Vec<String>, String> {
    let mut labels: Vec<String> = app
        .windows()
        .into_keys()
        .filter(|label| label.starts_with("persona-"))
        .collect();
    labels.sort();
    Ok(labels)
}

#[cfg(test)]
mod tests {
    use super::{evict_plan, is_valid_persona_id};

    fn order(labels: &[&str]) -> Vec<String> {
        labels.iter().map(|l| l.to_string()).collect()
    }

    #[test]
    fn evicts_oldest_hidden_over_cap() {
        let order = order(&["a", "b", "c", "d", "current"]);
        assert_eq!(evict_plan(&order, "current", 3), vec!["a", "b"]);
    }

    #[test]
    fn keeps_everything_under_cap() {
        let order = order(&["a", "b", "current"]);
        assert!(evict_plan(&order, "current", 3).is_empty());
    }

    #[test]
    fn never_evicts_current() {
        let order = order(&["a", "b", "c"]);
        let plan = evict_plan(&order, "a", 1);
        assert!(!plan.contains(&"a".to_string()));
        assert_eq!(plan, vec!["b", "c"]);
    }

    #[test]
    fn unknown_labels_are_planned_for_removal() {
        let order = order(&["ghost", "b", "current"]);
        assert_eq!(evict_plan(&order, "current", 2), vec!["ghost"]);
    }

    #[test]
    fn valid_persona_ids_only() {
        assert!(is_valid_persona_id("nurul-imam-studio"));
        assert!(!is_valid_persona_id(""));
        assert!(!is_valid_persona_id("../etc"));
        assert!(!is_valid_persona_id("spaced id"));
    }
}
