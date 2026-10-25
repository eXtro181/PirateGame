//! Spielerprofile mit Name und Passwort. Passwörter werden nie gespeichert: PBKDF2-SHA256
//! leitet einen Prüfwert und einen AES-256-GCM-Schlüssel ab, der den Spielstand verschlüsselt.
//! Alles liegt im localStorage dieses Browsers (nativ: Datei neben dem Spiel).

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
    pub salt: String,
    pub verifier: String,
    pub iv: String,
    pub data: String,
    pub updated_at: f64,
    pub summary: Summary,
}

#[derive(Clone)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub salt: String,
    pub verifier: String,
    key: [u8; 32],
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

fn derive(password: &str, salt: &[u8]) -> (String, [u8; 32]) {
    let mut out = [0u8; 64];
    pbkdf2::pbkdf2_hmac::<sha2::Sha256>(password.as_bytes(), salt, ITER, &mut out);
    let verifier = B64.encode(&out[..32]);
    let mut key = [0u8; 32];
    key.copy_from_slice(&out[32..]);
    (verifier, key)
}

fn random<const N: usize>() -> [u8; N] {
    let mut b = [0u8; N];
    let _ = getrandom::fill(&mut b);
    b
}

pub fn list() -> Vec<Row> {
    let mut v: Vec<Row> = load_all().into_values().collect();
    v.sort_by(|a, b| b.updated_at.partial_cmp(&a.updated_at).unwrap_or(std::cmp::Ordering::Equal));
    v
}

pub fn create(name: &str, password: &str, save: &Save) -> Result<Profile, String> {
    let id = profile_id(name);
    if load_all().contains_key(&id) {
        return Err("Diesen Namen gibt es auf diesem Gerät schon.".into());
    }
    let salt = random::<16>();
    let (verifier, key) = derive(password, &salt);
    let p = Profile { id, name: name.trim().into(), salt: B64.encode(salt), verifier, key };
    write(&p, save)?;
    Ok(p)
}

pub fn login(name: &str, password: &str) -> Result<(Profile, Save), String> {
    let all = load_all();
    let row = all.get(&profile_id(name)).ok_or("Kein Profil mit diesem Namen.")?;
    let salt = B64.decode(&row.salt).map_err(|_| "Profil beschädigt.")?;
    let (verifier, key) = derive(password, &salt);
    if verifier != row.verifier {
        return Err("Falsches Passwort.".into());
    }
    let iv = B64.decode(&row.iv).map_err(|_| "Profil beschädigt.")?;
    let data = B64.decode(&row.data).map_err(|_| "Profil beschädigt.")?;
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|_| "Schlüsselfehler.")?;
    let plain = cipher.decrypt(Nonce::from_slice(&iv), data.as_ref()).map_err(|_| "Entschlüsseln fehlgeschlagen.")?;
    let save: Save = serde_json::from_slice(&plain).map_err(|e| format!("Spielstand unlesbar: {e}"))?;
    Ok((Profile { id: row.id.clone(), name: row.name.clone(), salt: row.salt.clone(), verifier, key }, save))
}

pub fn write(p: &Profile, save: &Save) -> Result<(), String> {
    let iv = random::<12>();
    let cipher = Aes256Gcm::new_from_slice(&p.key).map_err(|_| "Schlüsselfehler.")?;
    let json = serde_json::to_vec(save).map_err(|e| e.to_string())?;
    let data = cipher.encrypt(Nonce::from_slice(&iv), json.as_ref()).map_err(|_| "Verschlüsseln fehlgeschlagen.")?;
    let mut all = load_all();
    all.insert(
        p.id.clone(),
        Row {
            id: p.id.clone(),
            name: p.name.clone(),
            salt: p.salt.clone(),
            verifier: p.verifier.clone(),
            iv: B64.encode(iv),
            data: B64.encode(data),
            updated_at: crate::game::now_ms(),
            summary: Summary { level: save.level, play_time: save.play_time.round(), last_fire: save.last_fire.clone(), look: Some(save.look.clone()), weapon: save.weapon.clone(), deaths: save.deaths },
        },
    );
    save_all(&all)
}

pub fn remove(name: &str, password: &str) -> Result<(), String> {
    login(name, password)?;
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
