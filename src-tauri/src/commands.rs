//! JSON-file persistence and the data model shared with the frontend.
//! ponytail: plain JSON files + std::fs instead of a database; switch to
//! SQLite only if per-record queries or locking across processes become real.

use std::fs;
use std::path::{Path, PathBuf};

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
pub enum ThemeSetting {
    System,
    Light,
    Dark,
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
    pub theme: ThemeSetting,
    pub close_behavior: CloseBehavior,
    pub lru_size: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            language: LanguageSetting::System,
            theme: ThemeSetting::System,
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

/// Atomic-ish write: temp file + rename so a crash never truncates data.
fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let tmp = path.with_extension("tmp");
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

fn default_providers() -> Vec<Provider> {
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
    vec![
        provider(
            "flow",
            "Flow",
            "https://labs.google/fx/tools/flow",
            Category::Both,
        ),
        provider(
            "gemini",
            "Gemini",
            "https://gemini.google.com",
            Category::Image,
        ),
        provider("suno", "Suno", "https://suno.com", Category::Both),
        provider(
            "pixverse",
            "PixVerse",
            "https://app.pixverse.ai",
            Category::Video,
        ),
        provider(
            "leonardo",
            "Leonardo",
            "https://app.leonardo.ai",
            Category::Image,
        ),
        provider("krea", "Krea", "https://www.krea.ai", Category::Image),
        provider(
            "imagineart",
            "ImagineArt",
            "https://imagine.art",
            Category::Both,
        ),
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
    write_json(&path_for(&app, "accounts.json")?, &accounts)
}

#[tauri::command]
pub fn list_providers(app: AppHandle) -> Result<Vec<Provider>, String> {
    let path = path_for(&app, "providers.json")?;
    if !path.exists() {
        let seeded = default_providers();
        write_json(&path, &seeded)?;
        return Ok(seeded);
    }
    read_json(&path)
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
    write_json(&path_for(&app, "prompts.json")?, &prompts)
}

#[tauri::command]
pub fn get_settings(app: AppHandle) -> Result<Settings, String> {
    load_settings(&app)
}

#[tauri::command]
pub fn save_settings(app: AppHandle, settings: Settings) -> Result<Settings, String> {
    let settings = settings.normalized();
    write_json(&path_for(&app, "settings.json")?, &settings)?;
    Ok(settings)
}
