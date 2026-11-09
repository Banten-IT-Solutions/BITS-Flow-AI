//! JSON-file persistence and the data model shared with the frontend.
//! ponytail: plain JSON files + std::fs instead of a database; switch to
//! SQLite only if per-record queries or locking across processes become real.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub id: String,
    pub name: String,
    pub email: String,
    pub color: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Category {
    Image,
    Video,
    Both,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Engine {
    Webview,
    Chrome,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Provider {
    pub id: String,
    pub name: String,
    pub url: String,
    pub category: Category,
    pub engine: Engine,
    pub custom: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Prompt {
    pub id: String,
    pub title: String,
    pub body: String,
    pub category: Category,
    pub pinned: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LanguageSetting {
    System,
    Id,
    En,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CloseBehavior {
    Exit,
    Hide,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub language: LanguageSetting,
    pub close_behavior: CloseBehavior,
    pub lru_size: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            language: LanguageSetting::System,
            close_behavior: CloseBehavior::Exit,
            lru_size: 3,
        }
    }
}

impl Settings {
    /// Clamp persisted values into the accepted ranges.
    pub fn normalized(mut self) -> Self {
        self.lru_size = self.lru_size.clamp(1, 10);
        self
    }
}

/// Open the GitHub releases page in the user's default browser. The URL is
/// fixed here (never passed from the webview) so the command cannot be
/// abused to open arbitrary targets.
#[tauri::command]
pub fn open_release_page() -> Result<(), String> {
    const RELEASES_URL: &str =
        "https://github.com/Banten-IT-Solutions/BITS-Flow-AI/releases/latest";
    let mut child = std::process::Command::new("xdg-open")
        .arg(RELEASES_URL)
        .spawn()
        .map_err(|e| e.to_string())?;
    // xdg-open hands off to the desktop portal and exits immediately; wait so
    // the quick exit does not leave a zombie process behind.
    child.wait().map(|_| ()).map_err(|e| e.to_string())
}

/// Write a user-picked export file. The path always comes from the native
/// save dialog, so the webview cannot target arbitrary paths by itself.
#[tauri::command]
pub fn export_file(path: String, contents: String) -> Result<(), String> {
    fs::write(&path, contents).map_err(|e| e.to_string())
}

/// Read a user-picked import file back as text.
#[tauri::command]
pub fn import_file(path: String) -> Result<String, String> {
    fs::read_to_string(&path).map_err(|e| e.to_string())
}

pub fn app_data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_local_data_dir().map_err(|e| e.to_string())?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

fn path_for(app: &AppHandle, file: &str) -> Result<PathBuf, String> {
    Ok(app_data_dir(app)?.join(file))
}

fn read_json<T: DeserializeOwned + Default>(path: &Path) -> Result<T, String> {
    if !path.exists() {
        return Ok(T::default());
    }
    let raw = fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&raw).map_err(|e| format!("{}: {e}", path.display()))
}

/// Global write lock: every JSON mutation holds it so concurrent commands
/// cannot interleave read-modify-write cycles or tmp-file renames.
static IO_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn io_guard() -> MutexGuard<'static, ()> {
    IO_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Monotonic counter for unique tmp file names (pid + seq, see `write_json`).
static TMP_SEQ: AtomicU64 = AtomicU64::new(0);

/// Atomic-ish write: uniquely-named temp file + rename so a crash never
/// truncates data. `rename` overwrites the target; stale `.tmp` leftovers
/// from a crash are harmless and never cleaned up.
fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let seq = TMP_SEQ.fetch_add(1, Ordering::Relaxed);
    let tmp = path.with_extension(format!("tmp{}.{}", std::process::id(), seq));
    let raw = serde_json::to_string_pretty(value).map_err(|e| e.to_string())?;
    fs::write(&tmp, raw).map_err(|e| e.to_string())?;
    fs::rename(&tmp, path).map_err(|e| e.to_string())
}

fn validate_url(url: &str) -> Result<(), String> {
    let parsed = tauri::Url::parse(url).map_err(|_| format!("invalid URL: {url}"))?;
    match parsed.scheme() {
        "http" | "https" => Ok(()),
        other => Err(format!("unsupported URL scheme: {other}")),
    }
}

pub fn load_settings(app: &AppHandle) -> Result<Settings, String> {
    let path = path_for(app, "settings.json")?;
    Ok(read_json::<Settings>(&path)?.normalized())
}

/// One built-in provider entry; `custom` is always false.
fn provider(id: &str, name: &str, url: &str, category: Category) -> Provider {
    Provider {
        id: id.to_string(),
        name: name.to_string(),
        url: url.to_string(),
        category,
        engine: Engine::Webview,
        custom: false,
    }
}

fn default_providers() -> Vec<Provider> {
    vec![
        provider("flow", "Flow", "https://flow.google.com", Category::Both),
        provider(
            "gemini",
            "Gemini",
            "https://gemini.google.com",
            Category::Image,
        ),
        provider(
            "leonardo",
            "Leonardo",
            "https://app.leonardo.ai",
            Category::Image,
        ),
        provider("krea", "Krea", "https://www.krea.ai", Category::Image),
        provider(
            "firefly",
            "Adobe Firefly",
            "https://firefly.adobe.com",
            Category::Both,
        ),
        provider("chatgpt", "ChatGPT", "https://chatgpt.com", Category::Image),
    ]
}

#[tauri::command]
pub fn list_accounts(app: AppHandle) -> Result<Vec<Account>, String> {
    read_json(&path_for(&app, "accounts.json")?)
}

fn validate_account(account: &Account) -> Result<(), String> {
    if account.id.trim().is_empty() {
        return Err("account id must not be empty".to_string());
    }
    if account.name.trim().is_empty() {
        return Err("account name must not be empty".to_string());
    }
    if !account.email.contains('@') {
        return Err(format!("invalid account email: {}", account.email));
    }
    Ok(())
}

#[tauri::command]
pub fn save_accounts(app: AppHandle, accounts: Vec<Account>) -> Result<(), String> {
    for account in &accounts {
        validate_account(account)?;
    }
    let _g = io_guard();
    write_json(&path_for(&app, "accounts.json")?, &accounts)
}

#[tauri::command]
pub fn list_providers(app: AppHandle) -> Result<Vec<Provider>, String> {
    let path = path_for(&app, "providers.json")?;
    if !path.exists() {
        let seeded = default_providers();
        let _g = io_guard();
        write_json(&path, &seeded)?;
        return Ok(seeded);
    }
    let saved: Vec<Provider> = read_json(&path)?;
    Ok(saved)
}

fn validate_provider(provider: &Provider) -> Result<(), String> {
    if provider.id.trim().is_empty() {
        return Err("provider id must not be empty".to_string());
    }
    if provider.name.trim().is_empty() {
        return Err("provider name must not be empty".to_string());
    }
    validate_url(&provider.url)
}

#[tauri::command]
pub fn save_providers(app: AppHandle, providers: Vec<Provider>) -> Result<(), String> {
    for provider in &providers {
        validate_provider(provider)?;
    }
    let _g = io_guard();
    write_json(&path_for(&app, "providers.json")?, &providers)
}

#[tauri::command]
pub fn list_prompts(app: AppHandle) -> Result<Vec<Prompt>, String> {
    read_json(&path_for(&app, "prompts.json")?)
}

fn validate_prompt(prompt: &Prompt) -> Result<(), String> {
    if prompt.id.trim().is_empty() {
        return Err("prompt id must not be empty".to_string());
    }
    if prompt.title.trim().is_empty() || prompt.body.trim().is_empty() {
        return Err("prompt title and body must not be empty".to_string());
    }
    Ok(())
}

#[tauri::command]
pub fn save_prompts(app: AppHandle, prompts: Vec<Prompt>) -> Result<(), String> {
    for prompt in &prompts {
        validate_prompt(prompt)?;
    }
    let _g = io_guard();
    write_json(&path_for(&app, "prompts.json")?, &prompts)
}

#[tauri::command]
pub fn get_settings(app: AppHandle) -> Result<Settings, String> {
    load_settings(&app)
}

#[tauri::command]
pub fn save_settings(app: AppHandle, settings: Settings) -> Result<Settings, String> {
    let settings = settings.normalized();
    let _g = io_guard();
    write_json(&path_for(&app, "settings.json")?, &settings)?;
    Ok(settings)
}

/// One "persona opened provider" activity row. `ts` is unix millis.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentEntry {
    pub provider_id: String,
    pub persona_id: String,
    pub persona_name: String,
    pub provider_name: String,
    pub ts: u64,
}

/// Payload for `log_activity`; `ts` is stamped by the backend, never trusted
/// from the client.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentInput {
    pub provider_id: String,
    pub persona_id: String,
    pub persona_name: String,
    pub provider_name: String,
}

pub const RECENT_CAP: usize = 50;

/// Pure cap: returns the newest-first list trimmed to the newest `RECENT_CAP`
/// entries. Sorts defensively so an out-of-order file still keeps the newest.
fn cap_recent(mut entries: Vec<RecentEntry>) -> Vec<RecentEntry> {
    entries.sort_by_key(|e| std::cmp::Reverse(e.ts));
    entries.truncate(RECENT_CAP);
    entries
}

fn unix_millis() -> Result<u64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_recent(app: AppHandle) -> Result<Vec<RecentEntry>, String> {
    let entries: Vec<RecentEntry> = read_json(&path_for(&app, "recent.json")?)?;
    Ok(cap_recent(entries))
}

#[tauri::command]
pub fn log_activity(app: AppHandle, input: RecentInput) -> Result<Vec<RecentEntry>, String> {
    if input.provider_id.trim().is_empty() || input.provider_name.trim().is_empty() {
        return Err("provider id and name must not be empty".to_string());
    }
    if input.persona_name.trim().is_empty() {
        return Err("persona name must not be empty".to_string());
    }
    if !crate::persona::is_valid_persona_id(&input.persona_id) {
        return Err(format!("invalid persona id: {:?}", input.persona_id));
    }
    let _g = io_guard();
    let mut recent: Vec<RecentEntry> = read_json(&path_for(&app, "recent.json")?)?;
    recent.insert(
        0,
        RecentEntry {
            provider_id: input.provider_id,
            persona_id: input.persona_id,
            persona_name: input.persona_name,
            provider_name: input.provider_name,
            ts: unix_millis()?,
        },
    );
    let recent = cap_recent(recent);
    write_json(&path_for(&app, "recent.json")?, &recent)?;
    Ok(recent)
}

/// Delete a persona's isolated data directory. Refuses while its window is
/// open so a live WebKitGTK session cannot resurrect or race the deletion.
#[tauri::command]
pub fn clear_persona_data(app: AppHandle, persona_id: String) -> Result<(), String> {
    if !crate::persona::is_valid_persona_id(&persona_id) {
        return Err(format!("invalid persona id: {persona_id:?}"));
    }
    let label = format!("persona-{persona_id}");
    if app.get_webview_window(&label).is_some() {
        return Err("error.close_first".to_string());
    }
    let _g = io_guard();
    let dir = app_data_dir(&app)?.join("profiles").join(&persona_id);
    if !dir.is_dir() {
        return Err("error.no_data_dir".to_string());
    }
    fs::remove_dir_all(&dir).map_err(|e| e.to_string())
}

/// Result of clearing every persona data directory at once.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClearAllResult {
    pub cleared: Vec<String>,
    pub skipped: Vec<String>,
    pub failed: Vec<String>,
}

#[tauri::command]
pub fn clear_all_persona_data(app: AppHandle) -> Result<ClearAllResult, String> {
    let _g = io_guard();
    let profiles = app_data_dir(&app)?.join("profiles");
    let mut result = ClearAllResult {
        cleared: Vec::new(),
        skipped: Vec::new(),
        failed: Vec::new(),
    };
    if !profiles.is_dir() {
        return Ok(result);
    }
    let entries = fs::read_dir(&profiles).map_err(|e| e.to_string())?;
    for entry in entries {
        let path = entry.map_err(|e| e.to_string())?.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if !crate::persona::is_valid_persona_id(name) || !path.is_dir() {
            continue;
        }
        if app.get_webview_window(&format!("persona-{name}")).is_some() {
            result.skipped.push(name.to_string());
            continue;
        }
        // Collect per-dir failures instead of aborting the whole sweep.
        match fs::remove_dir_all(&path) {
            Ok(()) => result.cleared.push(name.to_string()),
            Err(_) => result.failed.push(name.to_string()),
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(ts: u64) -> RecentEntry {
        RecentEntry {
            provider_id: "p".to_string(),
            persona_id: "a".to_string(),
            persona_name: "A".to_string(),
            provider_name: "P".to_string(),
            ts,
        }
    }

    #[test]
    fn caps_to_the_newest_entries() {
        let out = cap_recent((0..60).map(entry).collect());
        assert_eq!(out.len(), RECENT_CAP);
        assert_eq!(out[0].ts, 59);
        assert_eq!(out[RECENT_CAP - 1].ts, 10);
    }

    #[test]
    fn keeps_everything_under_cap() {
        let out = cap_recent(vec![entry(1), entry(2), entry(3)]);
        assert_eq!(out.len(), 3);
        assert_eq!(out[0].ts, 3);
    }

    #[test]
    fn newest_first_regardless_of_input_order() {
        let out = cap_recent(vec![entry(5), entry(9), entry(7)]);
        let ts: Vec<u64> = out.iter().map(|e| e.ts).collect();
        assert_eq!(ts, vec![9, 7, 5]);
    }

    #[test]
    fn old_theme_setting_is_ignored_and_other_values_load() {
        let settings: Settings = serde_json::from_str(
            r#"{"language":"en","theme":"light","closeBehavior":"hide","lruSize":5}"#,
        )
        .unwrap();
        assert_eq!(settings.language, LanguageSetting::En);
        assert_eq!(settings.close_behavior, CloseBehavior::Hide);
        assert_eq!(settings.lru_size, 5);
    }

    #[test]
    fn empty_input_stays_empty() {
        assert!(cap_recent(Vec::new()).is_empty());
    }
}
