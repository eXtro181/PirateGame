// DOM user interface: title & profiles, character creator, HUD, menus, death screen, map.
import { DEFAULT_LOOK, SKIN, HAIR_COLORS, CLOTH, SCARF, HAIR_STYLES, BEARDS, HATS, BODIES, FW, FH } from '../art/character.js';
import { paintCharacterSheetHi, saberFireHi, flameSheetHi } from '../art/hires.js';
import { ORIGINS, GIFTS, STAT_NAMES, WEAPONS, OFFHAND, levelCost, newSave } from '../game/data.js';
import { T } from '../art/tiles.js';
import { TouchControls } from './touch.js';

const h = (tag, attrs = {}, ...kids) => {
    const el = document.createElement(tag);
    for (const [k, v] of Object.entries(attrs)) {
        if (k === 'class') el.className = v;
        else if (k.startsWith('on')) el.addEventListener(k.slice(2), v);
        else if (k === 'html') el.innerHTML = v;
        else if (v !== false && v != null) el.setAttribute(k, v);
    }
    for (const k of kids.flat()) if (k != null) el.append(k.nodeType ? k : document.createTextNode(k));
    return el;
};
const TOUCH_KEYS = [['Linke Hälfte', 'Daumen auflegen und ziehen: laufen · auf See steuern'], ['Hieb / Schwer', 'Leichter und schwerer Angriff'], ['Rolle', 'Ausweichen mit Unverwundbarkeit'], ['Parieren', 'Im richtigen Moment für einen Gegenschlag'], ['Rum / Zielen', 'Heilen · Gegner fixieren'], ['Hinweisfeld', 'Antippen zum Interagieren, Rasten, An Bord gehen'], ['Auf See', 'Breitseiten links/rechts · Segel + / −']];
const fmtTime = s => { s = Math.round(s); const hh = Math.floor(s / 3600), mm = Math.floor(s / 60) % 60; return hh ? `${hh} Std ${mm} Min` : `${mm} Min`; };

function portrait(look) {
    const c = document.createElement('canvas');
    c.width = FW * 4; c.height = FH * 4;
    try {
        const { canvas, frames } = paintCharacterSheetHi(look);
        const r = frames['stand-down'][0];
        c.getContext('2d').drawImage(canvas, r.x * 4, r.y * 4, r.w * 4, r.h * 4, 0, 0, r.w * 4, r.h * 4);
    } catch (e) { console.warn(e); }
    return c;
}

export class UI {
    constructor(root, { store, audio }) {
        this.root = root; this.store = store; this.audio = audio;
        this.modal = null;
        this.hud = new HUD(this);
        this.deathEl = h('div', { class: 'death' }, h('h1', {}, 'DU BIST ERTRUNKEN'));
        this.fadeEl = h('div', { class: 'fade-black' });
        root.append(this.hud.el, this.deathEl, this.fadeEl);
        document.addEventListener('pointerdown', () => this.audio.unlock(), { once: false });
        document.addEventListener('keydown', () => this.audio.unlock());
    }
    bind(game) {
        this.game = game; this.hud.game = game;
        this.hud.touch = new TouchControls(game.input, this.hud.el);
        this.hud.promptEl.addEventListener('pointerdown', (e) => { e.preventDefault(); game.input.vPress('interact'); setTimeout(() => game.input.vRelease('interact'), 80); });
    }
    modalOpen() { return !!this.modal; }
    closeModal() { if (this.modal) { this.modal.remove(); this.modal = null; } }
    openModal(el) { this.closeModal(); this.modal = el; this.root.append(el); const f = el.querySelector('button'); f?.focus(); }

    // --------------------------------------------------------------- title + profiles
    async title(onStart) {
        this.onStart = onStart;
        this.hud.el.classList.add('hidden');
        this.screen?.remove();
        const profiles = await this.store.list().catch(() => []);
        let selected = null;
        const err = h('div', { class: 'error' });
        const pass = h('input', { class: 'text', type: 'password', placeholder: 'Passwort', autocomplete: 'current-password' });
        const loginBox = h('div', { class: 'login-box hidden' });
        const who = h('div', { class: 'pname' });
        const go = async () => {
            err.textContent = '';
            if (!selected) return;
            try {
                go.btn.disabled = true; go.btn.textContent = 'Prüfe …';
                const { profile, save } = await this.store.login(selected.name, pass.value);
                this.screen.remove();
                onStart(profile, save);
            } catch (e) { err.textContent = e.message || 'Anmeldung fehlgeschlagen.'; }
            finally { go.btn.disabled = false; go.btn.textContent = 'Weiterspielen'; }
        };
        go.btn = h('button', { class: 'btn primary', onclick: go }, 'Weiterspielen');
        pass.addEventListener('keydown', e => { if (e.key === 'Enter') go(); });
        loginBox.append(who, h('div', { class: 'rule' }), h('div', { class: 'row' }, pass, go.btn,
            h('button', { class: 'btn ghost', onclick: async () => { try { await this.store.exportFile(selected.name); } catch (e) { err.textContent = e.message; } } }, 'Exportieren'),
            h('button', { class: 'btn ghost', onclick: async (ev) => {
                const b = ev.currentTarget;
                if (!b.dataset.armed) { b.dataset.armed = '1'; b.textContent = 'Wirklich löschen?'; err.textContent = 'Löschen kann nicht rückgängig gemacht werden. Passwort eingeben und erneut klicken.'; return; }
                try { await this.store.remove(selected.name, pass.value); this.title(onStart); } catch (e) { err.textContent = e.message; }
            } }, 'Löschen')), err);
        const list = h('div', { class: 'profiles' }, profiles.map(p => {
            const card = h('button', { class: 'profile', onclick: () => {
                selected = p;
                list.querySelectorAll('.profile').forEach(x => x.classList.remove('active'));
                card.classList.add('active');
                who.textContent = `${p.name} – Passwort eingeben`;
                loginBox.classList.remove('hidden');
                pass.value = ''; err.textContent = '';
                pass.focus();
            } }, p.summary?.look ? portrait(p.summary.look) : null,
            h('div', {}, h('div', { class: 'pname' }, p.name), h('div', { class: 'pmeta' }, `Stufe ${p.summary?.level ?? '?'} · ${fmtTime(p.summary?.playTime || 0)} · ${p.summary?.deaths || 0}× ertrunken`)));
            return card;
        }));
        const importErr = h('div', { class: 'error', style: 'text-align:center' });
        const fileIn = h('input', { type: 'file', accept: '.json,application/json', class: 'hidden', onchange: async () => {
            try { await this.store.importFile(fileIn.files[0]); this.title(onStart); } catch (e) { importErr.textContent = e.message; }
        } });
        this.screen = h('div', { class: 'screen' }, h('div', { class: 'screen-inner' },
            h('h1', { class: 'logo' }, 'SCHWARZWASSER'),
            h('p', { class: 'tagline' }, 'Die See nimmt. Die See gibt zurück. Manchmal.'),
            profiles.length ? h('div', {}, h('div', { class: 'panel' }, h('h3', {}, 'Ertränkte auf diesem Gerät'), list, loginBox)) : h('p', { class: 'note', style: 'text-align:center' }, 'Noch kein Profil auf diesem Gerät.'),
            h('div', { class: 'rule' }),
            h('div', { class: 'row center' },
                h('button', { class: 'btn primary', onclick: () => { this.audio.unlock(); this.creator(); } }, 'Neuer Ertränkter'),
                h('button', { class: 'btn ghost', onclick: () => fileIn.click() }, 'Spielstand importieren'), fileIn),
            importErr,
            h('p', { class: 'note', style: 'text-align:center;margin-top:18px' }, 'Spielstände liegen verschlüsselt in diesem Browser. Mit „Exportieren“ sicherst du sie als Datei.')));
        this.root.append(this.screen);
        this.audio.setMood('title');
    }

    // --------------------------------------------------------------- character creator
    creator() {
        this.screen?.remove();
        const look = { ...DEFAULT_LOOK };
        let origin = 'matrose', gift = 'rum';
        const err = h('div', { class: 'error' });
        const name = h('input', { class: 'text', maxlength: 16, placeholder: 'Finn Rabenfeder', value: 'Finn Rabenfeder' });
        const pw1 = h('input', { class: 'text', type: 'password', placeholder: 'mind. 4 Zeichen', autocomplete: 'new-password' });
        const pw2 = h('input', { class: 'text', type: 'password', placeholder: 'wiederholen', autocomplete: 'new-password' });
        const canvas = h('canvas', { width: 384, height: 400 });
        const statsEl = h('div', { class: 'stats' });
        const giftDesc = h('div', { class: 'note' });

        const optDefs = [
            ['Statur', 'body', BODIES.length, i => BODIES[i]],
            ['Hautton', 'skin', SKIN.length, i => swatch(SKIN[i], `Ton ${i + 1}`)],
            ['Frisur', 'hair', HAIR_STYLES.length, i => HAIR_STYLES[i]],
            ['Haarfarbe', 'hairColor', HAIR_COLORS.length, i => swatch(HAIR_COLORS[i], `Farbe ${i + 1}`)],
            ['Bart', 'beard', BEARDS.length, i => BEARDS[i]],
            ['Kopf', 'hat', HATS.length, i => HATS[i]],
            ['Mantel', 'coat', CLOTH.length, i => swatch(CLOTH[i], `Farbe ${i + 1}`)],
            ['Akzent', 'accent', CLOTH.length, i => swatch(CLOTH[i], `Farbe ${i + 1}`)],
            ['Halstuch', 'scarf', SCARF.length, i => swatch(SCARF[i], `Farbe ${i + 1}`)],
            ['Augenklappe', 'patch', 2, i => (i ? 'Ja' : 'Nein')],
            ['Narbe', 'scar', 2, i => (i ? 'Ja' : 'Nein')]
        ];
        function swatch(c, label) { const s = h('span', {}, h('i', { class: 'swatch', style: `background:${c}` }), label); return s; }
        let sheet = null;
        const repaint = () => { sheet = paintCharacterSheetHi(look); sheet.canvas = sheet.pix.toCanvas(); };
        const opts = h('div', { class: 'opts' }, optDefs.map(([label, key, n, fmt]) => {
            const val = h('span', { class: 'val' });
            const show = () => { val.replaceChildren(); const v = typeof look[key] === 'boolean' ? (look[key] ? 1 : 0) : look[key]; const f = fmt(v); val.append(f.nodeType ? f : document.createTextNode(f)); };
            const step = d => {
                this.audio.sfx('click');
                if (typeof look[key] === 'boolean') look[key] = !look[key];
                else look[key] = (look[key] + d + n) % n;
                show(); repaint();
            };
            show();
            return h('div', { class: 'opt' }, h('span', { class: 'name' }, label), h('div', { class: 'ctl' }, h('button', { onclick: () => step(-1), 'aria-label': `${label} zurück` }, '‹'), val, h('button', { onclick: () => step(1), 'aria-label': `${label} weiter` }, '›')));
        }));
        const originBtns = ORIGINS.map(o => h('button', { class: 'origin' + (o.id === origin ? ' active' : ''), onclick: () => {
            origin = o.id; originBtns.forEach(b => b.classList.toggle('active', b.dataset.id === o.id)); showStats(); this.audio.sfx('click');
        }, 'data-id': o.id }, h('b', {}, o.name), h('span', {}, o.desc)));
        const showStats = () => {
            const o = ORIGINS.find(x => x.id === origin);
            statsEl.replaceChildren(
                ...['Stufe', o.level].map(t => h('span', {}, String(t))),
                ...Object.entries(o.stats).flatMap(([k, v]) => [h('span', {}, STAT_NAMES[k]), h('span', {}, String(v))]),
                h('span', {}, 'Waffe'), h('span', {}, WEAPONS[o.weapon].name),
                h('span', {}, 'Zweithand'), h('span', {}, o.off === 'none' ? '–' : OFFHAND[o.off].name));
        };
        const giftSel = h('select', { class: 'text', onchange: () => { gift = giftSel.value; giftDesc.textContent = GIFTS.find(g => g.id === gift).desc; } }, GIFTS.map(g => h('option', { value: g.id }, g.name)));
        giftDesc.textContent = GIFTS[0].desc;
        showStats(); repaint();

        // preview: the figure next to a burning saber, lit by its flicker
        const ctx = canvas.getContext('2d');
        const fireP = saberFireHi(); const fire = fireP.toCanvas();
        const flames = flameSheetHi('teal', 14, 22);
        const flameCanvas = flames.pix.toCanvas();
        let t0 = performance.now();
        const draw = () => {
            if (!canvas.isConnected) return;
            const t = (performance.now() - t0) / 1000;
            ctx.setTransform(4, 0, 0, 4, 0, 0);
            ctx.imageSmoothingEnabled = false;
            ctx.fillStyle = '#0e1210'; ctx.fillRect(0, 0, 96, 100);
            for (let i = 0; i < 90; i++) { const x = (i * 37) % 96, y = (i * 53) % 100; ctx.fillStyle = i % 3 ? '#16201a' : '#1c2a20'; ctx.fillRect(x, y, 1, 1); }

            const fr = flames.frames.burn[Math.floor(t * 10) % flames.frames.burn.length];
            ctx.drawImage(flameCanvas, fr.x * 4, fr.y * 4, fr.w * 4, fr.h * 4, 8, 36, fr.w, fr.h);
            ctx.drawImage(fire, 6, 50, fireP.w / 4, fireP.h / 4);
            const walk = sheet.frames['walk-down'], stand = sheet.frames['stand-down'];
            const phase = Math.floor(t / 2.5) % 4;
            const anims = [stand, walk, sheet.frames['stand-side'], sheet.frames['stand-up']];
            const fl = anims[phase];
            const r = fl[Math.floor(t * 8) % fl.length];
            ctx.save();
            ctx.translate(52, 44);
            ctx.scale(2, 2);
            ctx.drawImage(sheet.canvas, r.x * 4, r.y * 4, r.w * 4, r.h * 4, 0, 0, r.w, r.h);
            ctx.restore();
            // light: darken everything, then add the teal glow of the fire
            const flick = 0.85 + Math.sin(t * 13) * 0.06 + Math.sin(t * 7.1) * 0.05;
            ctx.globalCompositeOperation = 'multiply';
            const g = ctx.createRadialGradient(15, 52, 4, 15, 52, 90 * flick);
            g.addColorStop(0, '#e8fff8'); g.addColorStop(0.55, '#7fc0b4'); g.addColorStop(1, '#1a2628');
            ctx.fillStyle = g; ctx.fillRect(0, 0, 96, 100);
            ctx.globalCompositeOperation = 'lighter';
            const g2 = ctx.createRadialGradient(15, 48, 1, 15, 48, 26 * flick);
            g2.addColorStop(0, 'rgba(95,242,200,0.55)'); g2.addColorStop(1, 'rgba(95,242,200,0)');
            ctx.fillStyle = g2; ctx.fillRect(0, 0, 96, 100);
            ctx.globalCompositeOperation = 'source-over';
            requestAnimationFrame(draw);
        };
        requestAnimationFrame(draw);

        const create = async () => {
            err.textContent = '';
            const n = name.value.trim();
            if (n.length < 2) { err.textContent = 'Der Name braucht mindestens 2 Zeichen.'; return; }
            if (pw1.value.length < 4) { err.textContent = 'Das Passwort braucht mindestens 4 Zeichen.'; return; }
            if (pw1.value !== pw2.value) { err.textContent = 'Die Passwörter stimmen nicht überein.'; return; }
            try {
                createBtn.disabled = true; createBtn.textContent = 'Die Flut nimmt dich …';
                const save = newSave(n, { ...look }, origin, gift);
                const profile = await this.store.create(n, pw1.value, save);
                this.screen.remove();
                this.onStart(profile, save);
            } catch (e) { err.textContent = e.message || 'Fehler beim Anlegen.'; createBtn.disabled = false; createBtn.textContent = 'In die Tiefe'; }
        };
        const createBtn = h('button', { class: 'btn primary', onclick: create }, 'In die Tiefe');
        this.screen = h('div', { class: 'screen' }, h('div', { class: 'screen-inner' },
            h('h1', { class: 'logo', style: 'font-size:clamp(26px,4vw,44px)' }, 'Neuer Ertränkter'),
            h('p', { class: 'tagline' }, 'Wer warst du, bevor die See dich nahm?'),
            h('div', { class: 'creator' },
                h('div', { style: 'display:flex;flex-direction:column;gap:14px' },
                    h('div', { class: 'panel' }, h('h3', {}, 'Name und Passwort'),
                        h('div', { class: 'row' }, h('label', { class: 'field', style: 'flex:1 1 180px' }, 'Charaktername', name)),
                        h('div', { class: 'row', style: 'margin-top:8px' }, h('label', { class: 'field', style: 'flex:1 1 140px' }, 'Passwort', pw1), h('label', { class: 'field', style: 'flex:1 1 140px' }, 'Wiederholen', pw2)),
                        h('p', { class: 'note' }, 'Das Passwort schützt deinen Spielstand auf diesem Gerät. Nutze keins, das du anderswo verwendest.')),
                    h('div', { class: 'panel' }, h('h3', {}, 'Herkunft'), h('div', { class: 'origins' }, originBtns)),
                    h('div', { class: 'panel' }, h('h3', {}, 'Aussehen'), opts)),
                h('div', { class: 'preview-wrap' },
                    h('div', { class: 'preview' }, canvas),
                    h('div', { class: 'panel' }, h('h3', {}, 'Werte'), statsEl),
                    h('div', { class: 'panel' }, h('h3', {}, 'Startgeschenk'), giftSel, giftDesc),
                    err,
                    h('div', { class: 'row' }, createBtn, h('button', { class: 'btn ghost', onclick: () => this.title(this.onStart) }, 'Zurück'))))));
        this.root.append(this.screen);
        name.focus();
    }

    // --------------------------------------------------------------- in-game messages
    showHud() { this.hud.el.classList.remove('hidden'); }
    toast(text) {
        const el = this.hud.toastEl;
        el.textContent = text; el.style.opacity = 1;
        clearTimeout(this.toastT); this.toastT = setTimeout(() => (el.style.opacity = 0), 2200);
    }
    hint(text) {
        const el = this.hud.hintEl;
        el.textContent = text; el.style.opacity = 1;
        clearTimeout(this.hintT); this.hintT = setTimeout(() => (el.style.opacity = 0), Math.max(5000, text.length * 70));
    }
    banner(title, sub, kind = '', dur = 3) {
        const el = this.hud.bannerEl;
        el.className = 'banner ' + kind;
        el.replaceChildren(h('h2', {}, title), sub ? h('p', {}, sub) : null);
        el.style.opacity = 1;
        clearTimeout(this.bannerT); this.bannerT = setTimeout(() => (el.style.opacity = 0), dur * 1000);
    }
    saveIcon() { const el = this.hud.saveEl; el.style.opacity = 1; clearTimeout(this.saveT); this.saveT = setTimeout(() => (el.style.opacity = 0), 1800); }
    pearlGain(n) {
        const el = this.hud.gainEl;
        this.gainSum = (this.gainOn ? this.gainSum : 0) + n; this.gainOn = true;
        el.textContent = `+${this.gainSum}`; el.style.opacity = 1;
        clearTimeout(this.gainT); this.gainT = setTimeout(() => { el.style.opacity = 0; this.gainOn = false; }, 1600);
        this.audio.sfx('pearls');
    }
    floatText(text, x, z, cls) {
        const p = this.game.toScreen(x, z, 1.9);
        const el = h('div', { class: `float ${cls}`, style: `left:${p.x}px;top:${p.y}px` }, text);
        this.hud.el.append(el);
        requestAnimationFrame(() => { el.style.transform = 'translate(-50%,-160%)'; el.style.opacity = 0; });
        setTimeout(() => el.remove(), 1100);
    }
    death() { this.deathEl.classList.add('show'); }
    hideDeath() { this.deathEl.classList.remove('show'); }

    // --------------------------------------------------------------- fire / rest menu
    restMenu(fire) {
        const g = this.game;
        const s = g.save;
        const main = () => {
            const lit = g.map.fires.filter(f => s.litFires.includes(f.id) && f.id !== fire.id);
            this.openModal(h('div', { class: 'modal' }, h('div', { class: 'menu' },
                h('h2', {}, fire.name), h('div', { class: 'sub' }, 'Die Flammen wärmen nicht. Aber sie erinnern.'),
                h('div', { class: 'items' },
                    h('button', { class: 'btn', onclick: () => levelMenu() }, `Stufe aufsteigen  (Stufe ${s.level})`),
                    h('button', { class: 'btn', disabled: lit.length ? false : 'disabled', onclick: () => travelMenu(lit) }, lit.length ? 'Reisen' : 'Reisen (kein anderes Feuer entzündet)'),
                    h('button', { class: 'btn primary', onclick: () => { this.closeModal(); g.leaveFire(); } }, 'Aufstehen')))));
        };
        const levelMenu = () => {
            const render = () => {
                const cost = levelCost(s.level);
                const rows = Object.entries(STAT_NAMES).flatMap(([k, n]) => [
                    h('span', {}, n), h('span', {}, String(s.stats[k])),
                    h('button', { class: 'btn', disabled: s.pearls < cost ? 'disabled' : false, onclick: () => { if (g.levelUp(k)) render(); } }, '+1')]);
                const d = g.derived;
                this.openModal(h('div', { class: 'modal' }, h('div', { class: 'menu' },
                    h('h2', {}, 'Stufe aufsteigen'),
                    h('div', { class: 'sub' }, `Stufe ${s.level} · Kosten ${cost} Perlen · Du hast ${s.pearls}`),
                    h('div', { class: 'lvl' }, rows),
                    h('div', { class: 'rule' }),
                    h('div', { class: 'stats' }, h('span', {}, 'Leben'), h('span', {}, String(d.maxHp)), h('span', {}, 'Ausdauer'), h('span', {}, String(d.maxSt)), h('span', {}, 'Angriff'), h('span', {}, String(d.dmg))),
                    h('div', { class: 'rule' }),
                    h('button', { class: 'btn', onclick: main }, 'Zurück'))));
            };
            render();
        };
        const travelMenu = (lit) => {
            this.openModal(h('div', { class: 'modal' }, h('div', { class: 'menu' },
                h('h2', {}, 'Reisen'), h('div', { class: 'sub' }, 'Wohin trägt dich die Strömung?'),
                h('div', { class: 'items' }, lit.map(f => h('button', { class: 'btn', onclick: () => {
                    this.closeModal();
                    this.fadeEl.classList.add('on');
                    setTimeout(() => { g.travel(f); this.fadeEl.classList.remove('on'); setTimeout(() => this.restMenu(f), 500); }, 700);
                } }, f.name)), h('button', { class: 'btn ghost', onclick: main }, 'Zurück')))));
        };
        main();
    }

    // --------------------------------------------------------------- pause + map
    pauseMenu() {
        const g = this.game;
        const qualityLabel = q => `Grafik: ${{ ultra: '4K (Ultra)', high: 'Bildschirm (Hoch)', low: 'Niedrig' }[q]}`;
        const keys = [['WASD / Pfeile', 'Laufen · Schiff steuern (A/D) · Segel (W/S)'], ['J', 'Angriff · Breitseite links'], ['L / Shift+J', 'Schwerer Angriff'], ['K', 'Zweithand · Breitseite rechts'], ['Leertaste', 'Ausweichrolle'], ['Q', 'Parieren'], ['R', 'Rumflasche trinken'], ['Tab', 'Gegner fixieren'], ['E', 'Interagieren · Rasten · An Bord'], ['F', 'Shanty bestellen (auf See)'], ['M', 'Karte'], ['Esc', 'Menü']];
        const main = () => this.openModal(h('div', { class: 'modal' }, h('div', { class: 'menu' },
            h('h2', {}, g.save.name), h('div', { class: 'sub' }, `Stufe ${g.save.level} · ${fmtTime(g.save.playTime)} · ${g.save.deaths || 0}× ertrunken`),
            h('div', { class: 'items' },
                h('button', { class: 'btn primary', onclick: () => this.closeModal() }, 'Fortsetzen'),
                h('button', { class: 'btn', onclick: () => this.openModal(h('div', { class: 'modal' }, h('div', { class: 'menu' }, h('h2', {}, 'Steuerung'),
                    g.stage.device.touch ? h('div', { class: 'keys' }, TOUCH_KEYS.flatMap(([k, v]) => [h('kbd', {}, k), h('span', {}, v)])) : null,
                    g.stage.device.touch ? h('div', { class: 'rule' }) : null,
                    h('div', { class: 'keys' }, keys.flatMap(([k, v]) => [h('kbd', {}, k), h('span', {}, v)])),
                    h('p', { class: 'note' }, 'Gamepad: Stick bewegen · A Rolle · R1 Angriff · R2 schwer · L1 Parieren · X Interagieren · Steuerkreuz unten Rum.'),
                    h('button', { class: 'btn', onclick: main }, 'Zurück')))) }, 'Steuerung'),
                h('button', { class: 'btn', onclick: (e) => { this.audio.setMuted(!this.audio.muted); e.target.textContent = this.audio.muted ? 'Ton an' : 'Ton aus'; } }, this.audio.muted ? 'Ton an' : 'Ton aus'),
                h('button', { class: 'btn', onclick: (e) => {
                    const order = ['ultra', 'high', 'low'];
                    g.stage.setQuality(order[(order.indexOf(g.stage.quality) + 1) % 3]);
                    e.target.textContent = qualityLabel(g.stage.quality);
                } }, qualityLabel(g.stage.quality)),
                h('button', { class: 'btn', onclick: (e) => {
                    g.stage.setTouch(!g.stage.device.touch);
                    e.target.textContent = `Steuerung: ${g.stage.device.touch ? 'Touch' : 'Tastatur / Gamepad'}`;
                } }, `Steuerung: ${g.stage.device.touch ? 'Touch' : 'Tastatur / Gamepad'}`),
                h('button', { class: 'btn', onclick: () => this.store.exportFile(g.save.name).catch(e => this.toast(e.message)) }, 'Spielstand exportieren'),
                h('button', { class: 'btn ghost', onclick: () => { this.closeModal(); g.logout(); } }, 'Abmelden')),
            h('p', { class: 'note', style: 'margin-top:12px' }, 'Gespeichert wird automatisch, wenn du an einem Säbel- oder Wrackfeuer rastest.'))));
        main();
    }
    toggleMap() {
        if (this.modal?.classList.contains('mapwrap')) { this.closeModal(); return; }
        const g = this.game, m = g.map;
        const c = h('canvas', { width: m.w * 4, height: m.h * 4 });
        const ctx = c.getContext('2d');
        const col = { [T.WATER]: '#0b1f27', [T.SAND]: '#8a7650', [T.GRASS]: '#2f4d2c', [T.DARK]: '#22382a', [T.DIRT]: '#5a4230', [T.STONE]: '#5d5a62', [T.ROCK]: '#3a3740', [T.DOCK]: '#6a4a30' };
        for (let y = 0; y < m.h; y++) for (let x = 0; x < m.w; x++) {
            const seen = g.explored[y * m.w + x];
            ctx.fillStyle = seen ? col[m.get(x, y)] : '#020304';
            ctx.fillRect(x * 4, y * 4, 4, 4);
        }
        for (const w of m.whirlpools) if (g.explored[Math.floor(w.z) * m.w + Math.floor(w.x)]) { ctx.strokeStyle = '#5f8c98'; ctx.beginPath(); ctx.arc(w.x * 4, w.z * 4, w.r * 2, 0, 7); ctx.stroke(); }
        for (const f of m.fires) {
            if (!g.explored[Math.floor(f.z) * m.w + Math.floor(f.x)]) continue;
            ctx.fillStyle = g.save.litFires.includes(f.id) ? (f.kind === 'sea' ? '#ff9a3a' : '#5ff2c8') : '#555';
            ctx.fillRect(f.x * 4 - 3, f.z * 4 - 3, 6, 6);
        }
        if (g.save.stain) { ctx.fillStyle = '#9fe8ff'; ctx.beginPath(); ctx.arc(g.save.stain.x * 4, g.save.stain.z * 4, 3, 0, 7); ctx.fill(); }
        ctx.fillStyle = '#fff';
        const px = g.mode === 'sea' ? g.ship.x : g.player.x, pz = g.mode === 'sea' ? g.ship.z : g.player.z;
        ctx.fillRect(px * 4 - 2, pz * 4 - 2, 5, 5);
        const wrap = h('div', { class: 'mapwrap', onclick: () => this.closeModal() }, c, h('div', { class: 'note' }, 'Türkis: Säbelfeuer · Orange: Wrackfeuer · Weiß: du · Hellblau: deine Perlen · M schließen'));
        this.openModal(wrap);
    }
}

// ---------------------------------------------------------------------------- HUD
class HUD {
    constructor(ui) {
        this.ui = ui;
        const bar = (cls) => { const b = h('div', { class: `bar ${cls}` }, h('i', { class: 'lag' }), h('i', { class: 'fill' })); return b; };
        this.hp = bar('hp'); this.st = bar('st'); this.hull = bar('hull');
        this.flaskIco = h('div', { class: 'flask-ico' });
        this.flaskN = h('span', {});
        this.offEl = h('span', { class: 'offhand' });
        this.pearlN = h('span', {}, '0');
        this.gainEl = h('div', { class: 'gain' });
        this.promptEl = h('div', { class: 'prompt hidden' });
        this.toastEl = h('div', { class: 'toast' });
        this.hintEl = h('div', { class: 'hint' });
        this.bannerEl = h('div', { class: 'banner' });
        this.saveEl = h('div', { class: 'saveico' }, 'GESPEICHERT');
        this.lockEl = h('div', { class: 'lockmark hidden' });
        this.compassWind = h('i'); this.compassShip = h('b');
        this.sailsEl = h('div', { class: 'sails' });
        this.reloadL = h('i'); this.reloadR = h('i');
        this.ship = h('div', { class: 'shippanel hidden' },
            h('div', { class: 'compass', title: 'Türkis: Wind · Gold: Kurs' }, this.compassWind, this.compassShip),
            h('div', {}, h('div', {}, 'Segel'), this.sailsEl),
            h('div', {}, h('div', {}, 'Kanonen L / R'), h('div', { class: 'reload' }, h('div', {}, this.reloadL), h('div', {}, this.reloadR))));
        this.ebars = new Map();
        this.legendEl = h('div', { class: 'legend' });
        try { this.legendOn = localStorage.getItem('sw-legend') !== 'off'; } catch { this.legendOn = true; }
        this.el = h('div', { id: 'hud', class: 'hidden' }, this.legendEl,
            h('div', { class: 'bars' }, this.hp, this.st, this.hull, h('div', { class: 'flasks' }, this.flaskIco, this.flaskN, this.offEl)),
            this.ship,
            h('div', { class: 'pearls' }, h('div', { class: 'pearl-ico' }), this.pearlN), this.gainEl,
            this.promptEl, this.toastEl, this.hintEl, this.bannerEl, this.saveEl, this.lockEl);
        this.lagHp = 1;
    }
    // controls legend on the left edge (keyboard / gamepad; touch shows labelled buttons instead)
    legend(g) {
        if (g.input.pressed('legend')) {
            this.legendOn = !this.legendOn;
            try { localStorage.setItem('sw-legend', this.legendOn ? 'on' : 'off'); } catch { /* ignore */ }
        }
        const sea = g.mode === 'sea', pad = g.input.usingPad;
        const show = !g.stage.device.touch && g.dying === undefined;
        const key = `${show}:${this.legendOn}:${sea}:${pad}:${g.save.off}`;
        if (key !== this.legendKey) {
            this.legendKey = key;
            this.legendEl.classList.toggle('hidden', !show);
            const K = (k, p) => pad ? p : k;
            const rows = !this.legendOn ? [[K('H', '—'), 'Steuerung einblenden']] : sea ? [
                [K('W / S', 'Stick ↑↓'), 'Segel setzen / reffen'],
                [K('A / D', 'Stick ←→'), 'Steuern'],
                [K('J', 'R1'), 'Breitseite links'],
                [K('K', 'B'), 'Breitseite rechts'],
                [K('E', 'X'), 'An Land gehen / Rasten', 'interact'],
                [K('F', '↑'), 'Shanty bestellen'],
                [K('M', 'Select'), 'Karte'],
                [K('Esc', 'Start'), 'Menü'],
                [K('H', ''), 'Ausblenden']
            ] : [
                [K('WASD', 'Stick'), 'Laufen'],
                [K('J', 'R1'), 'Angriff'],
                [K('L', 'R2'), 'Schwerer Angriff'],
                [K('Leertaste', 'A'), 'Ausweichrolle'],
                [K('Q', 'L1'), 'Parieren'],
                ...(g.save.off !== 'none' ? [[K('K', 'B'), 'Zweithand']] : []),
                [K('R', '↓'), 'Rum trinken'],
                [K('Tab', 'R3'), 'Gegner fixieren'],
                [K('E', 'X'), 'Interagieren / An Bord', 'interact'],
                [K('M', 'Select'), 'Karte'],
                [K('Esc', 'Start'), 'Menü'],
                [K('H', ''), 'Ausblenden']
            ];
            this.legendEl.classList.toggle('mini', !this.legendOn);
            this.legendEl.replaceChildren(...rows.filter(r => r[0] !== '').map(([k, label, id]) => h('div', { class: 'row', 'data-id': id || '' }, h('kbd', {}, k), h('span', {}, label))));
        }
        const act = !!this.lastPrompt;
        this.legendEl.querySelector('[data-id="interact"]')?.classList.toggle('on', act);
    }
    prompt(label) {
        if (label === this.lastPrompt) return;
        this.lastPrompt = label;
        if (!label) { this.promptEl.classList.add('hidden'); return; }
        this.promptEl.replaceChildren(h('kbd', {}, this.game?.stage.device.touch ? '●' : 'E'), label);
        this.promptEl.classList.remove('hidden');
    }
    setBar(b, frac, widthPx) {
        b.style.width = `${widthPx}px`;
        b.querySelector('.fill').style.transform = `scaleX(${Math.max(0, frac)})`;
    }
    update() {
        const g = this.game;
        if (!g || !g.save || !g.player) return;
        const p = g.player, d = g.derived, s = g.save;
        const sea = g.mode === 'sea';
        this.legend(g);
        if (this.touch) {
            const show = g.stage.device.touch && !this.ui.modalOpen() && g.dying === undefined;
            this.touch.setVisible(show);
            if (show) this.touch.setMode(g.mode, s.off);
        }
        const bw = Math.min(2.4, (window.innerWidth - (g.stage.device.touch ? 150 : 60)) / Math.max(d.maxHp, d.maxSt));
        this.setBar(this.hp, p.hp / d.maxHp, d.maxHp * bw);
        const hf = p.hp / d.maxHp;
        this.lagHp = this.lagHp > hf ? Math.max(hf, this.lagHp - 0.006) : hf;
        this.hp.querySelector('.lag').style.transform = `scaleX(${this.lagHp})`;
        this.setBar(this.st, p.st / d.maxSt, d.maxSt * bw);
        this.hull.classList.toggle('hidden', !sea);
        this.st.classList.toggle('hidden', sea);
        this.ship.classList.toggle('hidden', !sea);
        if (sea) {
            this.setBar(this.hull, g.ship.hull / g.ship.maxHull, 220);
            this.compassWind.style.transform = `rotate(${g.wind.angle}rad)`;
            this.compassShip.style.transform = `rotate(${g.ship.heading}rad)`;
            const sails = g.ship.sail;
            if (this.lastSail !== sails) { this.lastSail = sails; this.sailsEl.replaceChildren(...[0, 1].map(i => h('span', { class: i < sails ? 'on' : '' }))); }
            this.reloadL.style.transform = `scaleX(${1 - g.ship.reload[0] / 2.6})`;
            this.reloadR.style.transform = `scaleX(${1 - g.ship.reload[1] / 2.6})`;
        }
        this.flaskN.textContent = String(p.flasks);
        this.flaskIco.classList.toggle('empty', p.flasks <= 0);
        this.offEl.textContent = s.off !== 'none' ? ` · ${s.off === 'pistol' ? 'Pulver' : 'Gezeiten'} ${p.ammo}` : '';
        this.pearlN.textContent = String(s.pearls);
        // lock marker
        const lt = g.lockTarget;
        if (lt) { const sp = g.toScreen(lt.x, lt.z, lt.sea ? 0.6 : 0.9); this.lockEl.style.left = `${sp.x}px`; this.lockEl.style.top = `${sp.y}px`; this.lockEl.classList.remove('hidden'); }
        else this.lockEl.classList.add('hidden');
        // enemy health bars
        for (const e of g.enemies) {
            let b = this.ebars.get(e);
            const show = e.alive && e.showBarUntil > g.time && (e.hp > 0);
            if (!show) { if (b) b.el.classList.add('hidden'); continue; }
            if (!b) { b = { el: h('div', { class: 'ebar' }, h('i'), h('b')) }; this.el.append(b.el); this.ebars.set(e, b); }
            const sp = (e.sea ? g.toScreen(e.x, e.z - 1.3, 0.3) : g.toScreen(e.x, e.z, 2.05));
            b.el.classList.remove('hidden');
            b.el.style.left = `${sp.x}px`; b.el.style.top = `${sp.y}px`;
            b.el.querySelector('i').style.transform = `scaleX(${Math.max(0, e.hp / e.maxHp)})`;
            b.el.querySelector('b').textContent = g.time - e.lastDmgT < 1.5 ? String(e.lastDmg) : '';
        }
    }
}
