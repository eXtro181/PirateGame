// DOM panels over the canvas: character creator (with a live pixel preview), login with
// name + password, options and credits. Everything else is drawn in the pixel canvas.
import { LOOK_OPTS, DEFAULT_LOOK, COAT, buildHumanSheet } from '../art/chars.js';
import { Atlas } from '../art/pix.js';
import { ORIGINS, GIFTS, STAT_NAMES, WEAPONS, OFFHAND, newSave } from '../game/data.js';

export function h(tag, attrs = {}, ...kids) {
    const el = document.createElement(tag);
    for (const [k, v] of Object.entries(attrs || {})) {
        if (k.startsWith('on')) el.addEventListener(k.slice(2), v);
        else if (k === 'class') el.className = v;
        else if (v !== undefined && v !== null && v !== false) el.setAttribute(k, v === true ? '' : v);
    }
    for (const k of kids.flat()) if (k !== null && k !== undefined) el.append(k.nodeType ? k : document.createTextNode(String(k)));
    return el;
}
const fmtTime = s => { s = Math.round(s); const hh = Math.floor(s / 3600), mm = Math.floor(s / 60) % 60; return hh ? `${hh} Std ${mm} Min` : `${mm} Min`; };

// small animated pixel preview of a look (walk cycle, turning)
class Preview {
    constructor(scale = 5) {
        this.scale = scale;
        this.canvas = h('canvas', { class: 'preview', width: 32 * scale, height: 36 * scale });
        this.t = 0;
        const loop = () => { if (!this.canvas.isConnected) { this.raf = null; return; } this.t += 1 / 60; this.draw(); this.raf = requestAnimationFrame(loop); };
        this.start = () => { if (!this.raf) this.raf = requestAnimationFrame(loop); };
    }
    set(look, weapon) {
        const a = new Atlas(512, 256);
        this.sheet = buildHumanSheet(a, 'pv', { ...look, weapon }, { anims: { idle: 2, walk: 6 } });
        this.img = a.img.canvas('col');
        this.start();
    }
    draw() {
        if (!this.sheet) return;
        const g = this.canvas.getContext('2d'); g.imageSmoothingEnabled = false;
        g.clearRect(0, 0, this.canvas.width, this.canvas.height);
        const dirs = ['down', 'side', 'up', 'side'];
        const di = Math.floor(this.t / 2.5) % 4, dir = dirs[di];
        const fr = this.sheet.dirs[dir].walk[Math.floor(this.t * 8) % 6];
        g.fillStyle = 'rgba(0,0,0,0.35)'; g.beginPath(); g.ellipse(16 * this.scale, 33 * this.scale, 7 * this.scale, 2 * this.scale, 0, 0, Math.PI * 2); g.fill();
        g.save();
        if (di === 3) { g.translate(this.canvas.width, 0); g.scale(-1, 1); }
        g.drawImage(this.img, fr.x, fr.y, fr.w, fr.h, 0, 2 * this.scale, fr.w * this.scale, fr.h * this.scale);
        g.restore();
    }
}

export class UI {
    constructor(root, app) {
        this.root = root; this.app = app; this.panel = null;
        document.addEventListener('pointerdown', () => app.audio.unlock());
        document.addEventListener('keydown', () => app.audio.unlock());
    }
    busy() { return !!this.panel; }
    close() { this.panel?.remove(); this.panel = null; this.app.input.enabled = true; }
    open(el) { this.close(); this.panel = el; this.root.append(el); setTimeout(() => el.querySelector('input, button')?.focus(), 30); }
    panelWrap(title, ...kids) {
        return h('div', { class: 'screen' }, h('div', { class: 'panel-box' }, h('div', { class: 'ptitle' }, title), ...kids));
    }
    menuAction(item) {
        if (item === 'Neues Spiel') this.creator();
        else if (item === 'Fortsetzen') this.login();
        else if (item === 'Optionen') this.options();
        else if (item === 'Credits') this.credits();
        else if (item === 'Beenden') this.quit();
    }
    // ---------------------------------------------------------------- creator
    creator() {
        const look = { ...DEFAULT_LOOK };
        let origin = 'matrose', gift = 'rum';
        const pv = new Preview(5);
        const err = h('div', { class: 'error' });
        const name = h('input', { class: 'text', maxlength: 16, value: 'Finn Rabenfeder', autocomplete: 'off' });
        const pw1 = h('input', { class: 'text', type: 'password', placeholder: 'mind. 4 Zeichen', autocomplete: 'new-password' });
        const pw2 = h('input', { class: 'text', type: 'password', placeholder: 'wiederholen', autocomplete: 'new-password' });
        const repaint = () => { const o = ORIGINS.find(x => x.id === origin); pv.set(look, o.weapon); };
        const opts = h('div', { class: 'opts' }, LOOK_OPTS.map(([label, key, names]) => {
            const val = h('span', { class: 'val' });
            const show = () => { val.textContent = names[look[key]]; };
            const step = d => { this.app.audio.sfx('click'); look[key] = (look[key] + d + names.length) % names.length; show(); repaint(); };
            show();
            return h('div', { class: 'opt' }, h('span', { class: 'oname' }, label), h('div', { class: 'ctl' }, h('button', { onclick: () => step(-1), 'aria-label': label + ' zurück' }, '‹'), val, h('button', { onclick: () => step(1), 'aria-label': label + ' weiter' }, '›')));
        }));
        const statsEl = h('div', { class: 'stats' });
        const originBtns = ORIGINS.map(o => h('button', { class: 'origin' + (o.id === origin ? ' active' : ''), 'data-id': o.id, onclick: () => { origin = o.id; originBtns.forEach(b => b.classList.toggle('active', b.dataset.id === o.id)); showStats(); repaint(); this.app.audio.sfx('click'); } }, h('b', {}, o.name), h('span', {}, o.desc)));
        const showStats = () => {
            const o = ORIGINS.find(x => x.id === origin);
            statsEl.replaceChildren(h('span', {}, 'Stufe'), h('span', {}, String(o.level)),
                ...Object.entries(o.stats).flatMap(([k, v]) => [h('span', {}, STAT_NAMES[k]), h('span', {}, String(v))]),
                h('span', {}, 'Waffe'), h('span', {}, WEAPONS[o.weapon].name), h('span', {}, 'Zweithand'), h('span', {}, o.off === 'none' ? '–' : OFFHAND[o.off].name));
        };
        const giftDesc = h('div', { class: 'note' }, GIFTS[0].desc);
        const giftSel = h('select', { class: 'text', onchange: () => { gift = giftSel.value; giftDesc.textContent = GIFTS.find(g => g.id === gift).desc; } }, GIFTS.map(g => h('option', { value: g.id }, g.name)));
        showStats(); repaint();
        const create = async () => {
            err.textContent = '';
            const n = name.value.trim();
            if (n.length < 2) { err.textContent = 'Der Name braucht mindestens 2 Zeichen.'; return; }
            if (pw1.value.length < 4) { err.textContent = 'Das Passwort braucht mindestens 4 Zeichen.'; return; }
            if (pw1.value !== pw2.value) { err.textContent = 'Die Passwörter stimmen nicht überein.'; return; }
            try {
                btn.disabled = true; btn.textContent = 'Die Flut nimmt dich …';
                const save = newSave(n, { ...look }, origin, gift);
                const profile = await this.app.store.create(n, pw1.value, save);
                this.close(); this.app.startGame(profile, save);
            } catch (e) { err.textContent = e.message || 'Fehler beim Anlegen.'; btn.disabled = false; btn.textContent = 'In die Tiefe'; }
        };
        const btn = h('button', { class: 'btn primary', onclick: create }, 'In die Tiefe');
        const el = this.panelWrap('Neuer Ertränkter',
            h('div', { class: 'creator' },
                h('div', { class: 'col' },
                    h('div', { class: 'card preview-card' }, pv.canvas),
                    h('div', { class: 'card' }, h('h3', {}, 'Aussehen'), opts)),
                h('div', { class: 'col' },
                    h('div', { class: 'card' }, h('h3', {}, 'Name und Passwort'),
                        h('label', { class: 'field' }, 'Name', name),
                        h('div', { class: 'row' }, h('label', { class: 'field' }, 'Passwort', pw1), h('label', { class: 'field' }, 'Wiederholen', pw2)),
                        h('p', { class: 'note' }, 'Das Passwort verschlüsselt deinen Spielstand in diesem Browser. Nimm keins, das du anderswo nutzt.')),
                    h('div', { class: 'card' }, h('h3', {}, 'Herkunft'), h('div', { class: 'origins' }, originBtns), statsEl),
                    h('div', { class: 'card' }, h('h3', {}, 'Startgeschenk'), giftSel, giftDesc),
                    err, h('div', { class: 'row' }, btn, h('button', { class: 'btn ghost', onclick: () => this.close() }, 'Zurück')))));
        this.open(el);
        name.addEventListener('keydown', e => e.stopPropagation());
    }
    // ---------------------------------------------------------------- login
    async login() {
        const profiles = await this.app.store.list().catch(() => []);
        const err = h('div', { class: 'error' });
        const pass = h('input', { class: 'text', type: 'password', placeholder: 'Passwort', autocomplete: 'current-password' });
        let selected = null;
        const box = h('div', { class: 'login-box hidden' });
        const who = h('div', { class: 'pname' });
        const go = async () => {
            if (!selected) return;
            err.textContent = '';
            try { goBtn.disabled = true; goBtn.textContent = 'Prüfe …'; const { profile, save } = await this.app.store.login(selected.name, pass.value); this.close(); this.app.startGame(profile, save); }
            catch (e) { err.textContent = e.message || 'Anmeldung fehlgeschlagen.'; }
            finally { goBtn.disabled = false; goBtn.textContent = 'Weiterspielen'; }
        };
        const goBtn = h('button', { class: 'btn primary', onclick: go }, 'Weiterspielen');
        pass.addEventListener('keydown', e => { e.stopPropagation(); if (e.key === 'Enter') go(); });
        const delBtn = h('button', { class: 'btn ghost', onclick: async () => {
            if (!delBtn.dataset.armed) { delBtn.dataset.armed = '1'; delBtn.textContent = 'Wirklich löschen?'; err.textContent = 'Löschen kann nicht rückgängig gemacht werden. Passwort eingeben und erneut klicken.'; return; }
            try { await this.app.store.remove(selected.name, pass.value); this.login(); this.app.refreshProfiles(); } catch (e) { err.textContent = e.message; }
        } }, 'Löschen');
        box.append(who, h('div', { class: 'row' }, pass, goBtn, h('button', { class: 'btn ghost', onclick: async () => { try { await this.app.store.exportFile(selected.name); } catch (e) { err.textContent = e.message; } } }, 'Exportieren'), delBtn), err);
        const list = h('div', { class: 'profiles' }, profiles.map(p => {
            const card = h('button', { class: 'profile', onclick: () => { selected = p; list.querySelectorAll('.profile').forEach(x => x.classList.remove('active')); card.classList.add('active'); who.textContent = `${p.name} – Passwort eingeben`; box.classList.remove('hidden'); pass.value = ''; err.textContent = ''; delete delBtn.dataset.armed; delBtn.textContent = 'Löschen'; pass.focus(); } },
                this.mini(p.summary?.look, p.summary?.weapon), h('div', {}, h('div', { class: 'pname' }, p.name), h('div', { class: 'pmeta' }, `Stufe ${p.summary?.level ?? '?'} · ${fmtTime(p.summary?.playTime || 0)} · ${p.summary?.deaths || 0}× ertrunken`)));
            return card;
        }));
        const fileIn = h('input', { type: 'file', accept: '.json,application/json', class: 'hidden', onchange: async () => { try { await this.app.store.importFile(fileIn.files[0]); this.login(); this.app.refreshProfiles(); } catch (e) { err.textContent = e.message; } } });
        this.open(this.panelWrap('Fortsetzen',
            profiles.length ? list : h('p', { class: 'note' }, 'Noch kein Ertränkter auf diesem Gerät.'), box,
            h('div', { class: 'row', style: 'margin-top:14px' }, h('button', { class: 'btn ghost', onclick: () => fileIn.click() }, 'Spielstand importieren'), fileIn, h('button', { class: 'btn ghost', onclick: () => this.close() }, 'Zurück')),
            h('p', { class: 'note' }, 'Spielstände liegen verschlüsselt in diesem Browser. Mit „Exportieren“ sicherst du sie als Datei.')));
    }
    mini(look, weapon) {
        const c = h('canvas', { width: 32, height: 34, class: 'mini' });
        try {
            const a = new Atlas(256, 64);
            const s = buildHumanSheet(a, 'm', { ...DEFAULT_LOOK, ...(look || {}), weapon: weapon || 'cutlass' }, { anims: { idle: 1 } });
            const fr = s.dirs.down.idle[0];
            c.getContext('2d').drawImage(a.img.canvas('col'), fr.x, fr.y, fr.w, fr.h, 0, 1, 32, 32);
        } catch { /* old save without a look */ }
        return c;
    }
    // ---------------------------------------------------------------- options
    options() {
        const o = this.app.options;
        const row = (label, ctl) => h('div', { class: 'opt' }, h('span', { class: 'oname' }, label), ctl);
        const sel = (key, vals) => { const s = h('select', { class: 'text', onchange: () => { o[key] = isNaN(+s.value) ? s.value : +s.value; this.app.applyOptions(); } }, vals.map(([v, n]) => h('option', { value: v, selected: String(o[key]) === String(v) }, n))); return s; };
        const range = (key, min, max, step) => { const r = h('input', { type: 'range', min, max, step, value: o[key], oninput: () => { o[key] = +r.value; this.app.applyOptions(); } }); return r; };
        this.open(this.panelWrap('Optionen',
            h('div', { class: 'card' },
                row('Farbstimmung', sel('look', [['bright', 'Leuchtend (wie die Beispiele)'], ['dark', 'Düster (Spezifikation)']])),
                row('Effekte', sel('fx', [['high', 'Bloom, Körnung, Nebel'], ['low', 'Sparsam (schwache Geräte)']])),
                row('Helligkeit', range('exposure', 0.7, 1.5, 0.05)),
                row('Lautstärke', range('volume', 0, 1, 0.05)),
                row('Bildschirmzittern', sel('shake', [[1, 'An'], [0, 'Aus']])),
                row('Touch-Steuerung', sel('touch', [['auto', 'Automatisch'], ['on', 'Immer'], ['off', 'Nie']]))),
            h('div', { class: 'row' }, h('button', { class: 'btn primary', onclick: () => { this.app.saveOptions(); this.close(); } }, 'Fertig'))));
    }
    credits() {
        this.open(this.panelWrap('Credits',
            h('div', { class: 'card credits' },
                h('p', {}, h('b', {}, 'Schwarzwasser')), h('p', {}, 'Ein Soulslike über Ertrunkene, Geisterkönige und das Ende der Welt.'),
                h('p', {}, 'Alle Grafiken, Shantys und Klänge werden beim Start aus Code erzeugt: Pixel für Pixel, Note für Note.'),
                h('p', {}, 'Idee und Spielleitung: Dennis · Umsetzung mit Claude'),
                h('p', { class: 'note' }, 'Shantys: Salz auf der Haut · Hol die Leine · Lied der Laternen · Glutwind · Eisige Leine · Rabensturm · Tiefer Grund · Weltschlund')),
            h('div', { class: 'row' }, h('button', { class: 'btn primary', onclick: () => this.close() }, 'Zurück'))));
    }
    quit() {
        this.open(this.panelWrap('Beenden', h('p', { class: 'note', style: 'font-size:18px' }, 'Die See wartet. Schließe den Tab, wenn du gehen willst. Dein Spielstand ist an jedem Feuer gesichert.'), h('div', { class: 'row' }, h('button', { class: 'btn primary', onclick: () => this.close() }, 'Bleiben'))));
    }
}
