// Local player profiles: name + password. Passwords are never stored; PBKDF2 derives a verifier
// and an AES-GCM key that encrypts the save game. Everything lives in this browser's IndexedDB.

const DB = 'schwarzwasser', STORE = 'profiles', ITER = 150000;
const enc = new TextEncoder(), dec = new TextDecoder();

const b64 = buf => btoa(String.fromCharCode(...new Uint8Array(buf)));
const unb64 = s => Uint8Array.from(atob(s), c => c.charCodeAt(0));

function openDb() {
    return new Promise((res, rej) => {
        const r = indexedDB.open(DB, 1);
        r.onupgradeneeded = () => r.result.createObjectStore(STORE, { keyPath: 'id' });
        r.onsuccess = () => res(r.result);
        r.onerror = () => rej(r.error);
    });
}
async function tx(mode, fn) {
    const db = await openDb();
    return new Promise((res, rej) => {
        const t = db.transaction(STORE, mode);
        const out = fn(t.objectStore(STORE));
        t.oncomplete = () => res(out instanceof IDBRequest ? out.result : out);
        t.onerror = () => rej(t.error);
    });
}

async function derive(password, salt) {
    const base = await crypto.subtle.importKey('raw', enc.encode(password), 'PBKDF2', false, ['deriveBits']);
    const bits = new Uint8Array(await crypto.subtle.deriveBits({ name: 'PBKDF2', hash: 'SHA-256', salt, iterations: ITER }, base, 512));
    const verifier = b64(bits.slice(0, 32));
    const key = await crypto.subtle.importKey('raw', bits.slice(32), 'AES-GCM', false, ['encrypt', 'decrypt']);
    return { verifier, key };
}

export const profileId = name => name.trim().toLowerCase();

export class ProfileStore {
    async list() {
        const rows = await tx('readonly', s => s.getAll());
        return (rows || []).map(r => ({ id: r.id, name: r.name, summary: r.summary || {}, updatedAt: r.updatedAt })).sort((a, b) => (b.updatedAt || 0) - (a.updatedAt || 0));
    }
    async exists(name) { return !!(await tx('readonly', s => s.get(profileId(name)))); }
    async create(name, password, save) {
        if (await this.exists(name)) throw new Error('Diesen Namen gibt es auf diesem Gerät schon.');
        const salt = crypto.getRandomValues(new Uint8Array(16));
        const { verifier, key } = await derive(password, salt);
        const profile = { id: profileId(name), name: name.trim(), salt: b64(salt), verifier, key };
        await this.save(profile, save);
        return profile;
    }
    async login(name, password) {
        const row = await tx('readonly', s => s.get(profileId(name)));
        if (!row) throw new Error('Kein Profil mit diesem Namen.');
        const { verifier, key } = await derive(password, unb64(row.salt));
        if (verifier !== row.verifier) throw new Error('Falsches Passwort.');
        const iv = unb64(row.iv);
        const plain = await crypto.subtle.decrypt({ name: 'AES-GCM', iv }, key, unb64(row.data));
        const save = JSON.parse(dec.decode(plain));
        return { profile: { id: row.id, name: row.name, salt: row.salt, verifier, key }, save };
    }
    async save(profile, save) {
        const iv = crypto.getRandomValues(new Uint8Array(12));
        const data = await crypto.subtle.encrypt({ name: 'AES-GCM', iv }, profile.key, enc.encode(JSON.stringify(save)));
        const row = {
            id: profile.id, name: profile.name, salt: profile.salt, verifier: profile.verifier,
            iv: b64(iv), data: b64(data), updatedAt: Date.now(),
            summary: { level: save.level, playTime: Math.round(save.playTime), lastFire: save.lastFire, origin: save.origin, look: save.look, deaths: save.deaths || 0 }
        };
        await tx('readwrite', s => s.put(row));
    }
    async remove(name, password) {
        await this.login(name, password); // proves ownership
        await tx('readwrite', s => s.delete(profileId(name)));
    }
    async exportFile(name) {
        const row = await tx('readonly', s => s.get(profileId(name)));
        if (!row) throw new Error('Kein Profil.');
        const blob = new Blob([JSON.stringify({ format: 'schwarzwasser-save', v: 1, row })], { type: 'application/json' });
        const a = document.createElement('a');
        a.href = URL.createObjectURL(blob);
        a.download = `schwarzwasser-${row.id}.json`;
        a.click();
        setTimeout(() => URL.revokeObjectURL(a.href), 2000);
    }
    async importFile(file) {
        const json = JSON.parse(await file.text());
        if (json.format !== 'schwarzwasser-save' || !json.row?.id) throw new Error('Keine gültige Spielstand-Datei.');
        if (await this.exists(json.row.name)) throw new Error(`Das Profil „${json.row.name}“ gibt es hier schon.`);
        await tx('readwrite', s => s.put(json.row));
        return json.row.name;
    }
}
