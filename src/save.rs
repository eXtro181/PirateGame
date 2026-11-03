//! Spielerprofile: nur ein Name, der Spielstand liegt als JSON im localStorage dieses Browsers
//! (nativ: Datei neben dem Spiel). Ältere Profile waren mit einem Passwort verschlüsselt
//! (PBKDF2-SHA256 + AES-256-GCM); sie werden beim ersten Laden einmal mit dem alten Passwort
//! entschlüsselt und danach ohne Passwort gespeichert.

use crate::game::data::{LookIdx, Save};
use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const KEY: &str = "schwarzwasser-profiles-v2";
const ITER: u32 = 150_000;

#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Summary {
    pub level: i32,
    pub play_time: f64,
    pub last_fire: String,
    pub look: Option<LookIdx>,
    pub weapon: String,
    pub deaths: i32,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Row {
    pub id: String,
    pub name: String,
    /// Spielstand als JSON (Profile ohne Passwort)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plain: Option<String>,
    /// alte, verschlüsselte Profile: Salz, Prüfwert, IV und verschlüsselter Spielstand
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub salt: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub verifier: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub iv: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub data: String,
    pub updated_at: f64,
    pub summary: Summary,
}
impl Row {
    /// Altes, noch verschlüsseltes Profil (braucht einmal das alte Passwort)
    pub fn locked(&self) -> bool {
        self.plain.is_none() && !self.data.is_empty()
    }
}

#[derive(Clone)]
pub struct Profile {
    pub id: String,
    pub name: String,
}

/// Warum ein Profil nicht geladen werden konnte
pub enum LoginErr {
    /// altes verschlüsseltes Profil: einmal das alte Passwort eingeben
    NeedsPassword,
    Msg(String),
}

pub fn profile_id(name: &str) -> String {
    name.trim().to_lowercase()
}

fn store_get() -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::window()?.local_storage().ok()??.get_item(KEY).ok()?
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::fs::read_to_string(format!("{KEY}.json")).ok()
    }
}
fn store_set(v: &str) -> Result<(), String> {
    #[cfg(target_arch = "wasm32")]
    {
        let s = web_sys::window().and_then(|w| w.local_storage().ok().flatten()).ok_or("Kein Speicher im Browser verfügbar.")?;
        s.set_item(KEY, v).map_err(|_| "Speichern fehlgeschlagen (Speicher voll oder privat).".to_string())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::fs::write(format!("{KEY}.json"), v).map_err(|e| e.to_string())
    }
}

fn load_all() -> BTreeMap<String, Row> {
    store_get().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
}
fn save_all(m: &BTreeMap<String, Row>) -> Result<(), String> {
    store_set(&serde_json::to_string(m).map_err(|e| e.to_string())?)
}

/// Nur noch zum Entschlüsseln alter Profile.
fn derive(password: &str, salt: &[u8]) -> (String, [u8; 32]) {
    let mut out = [0u8; 64];
    pbkdf2::pbkdf2_hmac::<sha2::Sha256>(password.as_bytes(), salt, ITER, &mut out);
    let verifier = B64.encode(&out[..32]);
    let mut key = [0u8; 32];
    key.copy_from_slice(&out[32..]);
    (verifier, key)
}

pub fn list() -> Vec<Row> {
    let mut v: Vec<Row> = load_all().into_values().collect();
    v.sort_by(|a, b| b.updated_at.partial_cmp(&a.updated_at).unwrap_or(std::cmp::Ordering::Equal));
    v
}

pub fn create(name: &str, save: &Save) -> Result<Profile, String> {
    let id = profile_id(name);
    if load_all().contains_key(&id) {
        return Err("Diesen Namen gibt es auf diesem Gerät schon.".into());
    }
    let p = Profile { id, name: name.trim().into() };
    write(&p, save)?;
    Ok(p)
}

/// Profil laden. Alte verschlüsselte Profile brauchen einmal `old_password`; danach werden sie
/// ohne Passwort gespeichert.
pub fn login(name: &str, old_password: Option<&str>) -> Result<(Profile, Save), LoginErr> {
    let all = load_all();
    let row = all.get(&profile_id(name)).ok_or(LoginErr::Msg("Kein Profil mit diesem Namen.".into()))?;
    let p = Profile { id: row.id.clone(), name: row.name.clone() };
    if let Some(j) = &row.plain {
        let save: Save = serde_json::from_str(j).map_err(|e| LoginErr::Msg(format!("Spielstand unlesbar: {e}")))?;
        return Ok((p, save));
    }
    let Some(pw) = old_password.filter(|s| !s.is_empty()) else { return Err(LoginErr::NeedsPassword) };
    let bad = |m: &str| LoginErr::Msg(m.into());
    let salt = B64.decode(&row.salt).map_err(|_| bad("Profil beschädigt."))?;
    let (verifier, key) = derive(pw, &salt);
    if verifier != row.verifier {
        return Err(bad("Falsches Passwort."));
    }
    let iv = B64.decode(&row.iv).map_err(|_| bad("Profil beschädigt."))?;
    let data = B64.decode(&row.data).map_err(|_| bad("Profil beschädigt."))?;
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|_| bad("Schlüsselfehler."))?;
    let plain = cipher.decrypt(Nonce::from_slice(&iv), data.as_ref()).map_err(|_| bad("Entschlüsseln fehlgeschlagen."))?;
    let save: Save = serde_json::from_slice(&plain).map_err(|e| LoginErr::Msg(format!("Spielstand unlesbar: {e}")))?;
    // ab jetzt ohne Passwort
    write(&p, &save).map_err(LoginErr::Msg)?;
    Ok((p, save))
}

pub fn write(p: &Profile, save: &Save) -> Result<(), String> {
    let json = serde_json::to_string(save).map_err(|e| e.to_string())?;
    let mut all = load_all();
    all.insert(
        p.id.clone(),
        Row {
            id: p.id.clone(),
            name: p.name.clone(),
            plain: Some(json),
            salt: String::new(),
            verifier: String::new(),
            iv: String::new(),
            data: String::new(),
            updated_at: crate::game::now_ms(),
            summary: Summary { level: save.level, play_time: save.play_time.round(), last_fire: save.last_fire.clone(), look: Some(save.look.clone()), weapon: save.weapon.clone(), deaths: save.deaths },
        },
    );
    save_all(&all)
}

pub fn remove(name: &str) -> Result<(), String> {
    let mut all = load_all();
    all.remove(&profile_id(name));
    save_all(&all)
}

// ------------------------------------------------------------------ Optionen
pub fn load_options() -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::window()?.local_storage().ok()??.get_item("schwarzwasser-options").ok()?
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::fs::read_to_string("schwarzwasser-options.json").ok()
    }
}
pub fn store_options(s: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(st) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
            let _ = st.set_item("schwarzwasser-options", s);
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = std::fs::write("schwarzwasser-options.json", s);
    }
}
