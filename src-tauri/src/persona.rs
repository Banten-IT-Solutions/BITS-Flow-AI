//! Isolated persona windows: one WebKitGTK data dir per account, one visible
//! persona at a time, with a naive LRU cap on open persona windows.
//! ponytail: no proxy-per-profile, no cookie import/export, eviction is
//! open-order based (hidden windows only); revisit when reuse stats matter.

use std::sync::Mutex;

use serde::Serialize;
use tauri::{AppHandle, Manager, State, WebviewUrl, WebviewWindowBuilder};

use crate::commands;

/// Chrome UA so providers that gate on WebView/Headless UA behave like a
/// normal desktop Chrome.
pub const CHROME_USER_AGENT: &str =
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";

#[derive(Default)]
pub struct PersonaState {
    /// Persona window labels in open order, oldest first.
    pub order: Mutex<Vec<String>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenPersonaResult {
    pub label: String,
    pub open_order: Vec<String>,
}

fn persona_label(persona_id: &str) -> String {
    format!("persona-{persona_id}")
}

fn is_valid_persona_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 128
        && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
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
    others[..others.len() - allowed].iter().map(|l| (*l).clone()).collect()
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

    if let Some(existing) = app.get_webview_window(&label) {
        existing.navigate(parsed).map_err(|e| e.to_string())?;
        existing.show().map_err(|e| e.to_string())?;
        existing.set_focus().map_err(|e| e.to_string())?;
    } else {
        let profile_dir = commands::app_data_dir(&app)?
            .join("profiles")
            .join(&persona_id);
        let mut builder = WebviewWindowBuilder::new(&app, &label, WebviewUrl::External(parsed))
            .title(&label)
            .visible(false)
            .data_directory(profile_dir);
        if engine == "chrome" {
            builder = builder.user_agent(CHROME_USER_AGENT);
        }
        let window = builder.build().map_err(|e| e.to_string())?;
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
    }

    // Exactly one visible persona at a time.
    for (other_label, window) in app.webview_windows() {
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
        if let Some(window) = app.get_webview_window(&victim) {
            if window.is_visible().unwrap_or(true) {
                continue;
            }
            let _ = window.close();
        }
        closed.push(victim);
    }
    order.retain(|l| !closed.contains(l));
    let snapshot = order.clone();
    drop(order);

    Ok(OpenPersonaResult {
        label,
        open_order: snapshot,
    })
}

#[cfg(test)]
mod tests {
    use super::evict_plan;

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
}
