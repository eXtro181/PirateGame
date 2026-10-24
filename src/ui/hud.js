// In-game HUD, built for a 1920×1080 reference and scaled with the canvas height:
// life and spirit orbs (animated liquid), lower bar with stamina, pearl progress, skill slots and
// belt 1–4, a live 3D portrait on the left, controls legend, minimap with area name, target bar.
import { Entity, Color, Vec3, Vec4, TONEMAP_ACES } from 'playcanvas';
import { levelCost } from '../game/data.js';
import { T } from '../world/defs.js';
import { NOISE } from '../gfx/noise.js';
import { QUALITY } from '../core/stage.js';

export const h = (tag, attrs = {}, ...kids) => {
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


// ------------------------------------------------------------------ liquid orb
class Orb {
    constructor(kind) {
        this.kind = kind; // 'life' | 'spirit'
        this.c = h('canvas', { width: 220, height: 220, class: 'orb-canvas' });
        this.g = this.c.getContext('2d');
        this.level = 1; this.shown = 1; this.pulse = 0; this.t = Math.random() * 10;
        // two noise layers for the swirling liquid, generated once
        this.noise = [0, 1].map(k => {
            const c = document.createElement('canvas'); c.width = c.height = 128;
            const g = c.getContext('2d'), id = g.createImageData(128, 128);
            for (let y = 0; y < 128; y++) for (let x = 0; x < 128; x++) {
                const n = NOISE.fbm(x / 128 * 4 + k * 7, y / 128 * 4, 4, 4) * 0.5 + 0.5, i = (y * 128 + x) * 4;
                id.data[i] = id.data[i + 1] = id.data[i + 2] = 255; id.data[i + 3] = n * 255;
            }
            g.putImageData(id, 0, 0);
            return c;
        });
    }
    set(v, hurt) { if (v < this.level - 0.001 && hurt) this.pulse = 1; this.level = Math.max(0, Math.min(1, v)); }
    draw(dt, poison = false) {
        this.t += dt;
        this.shown += (this.level - this.shown) * Math.min(1, dt * 6);
        this.pulse = Math.max(0, this.pulse - dt * 2.5);
        const g = this.g, S = 220, R = 100, cx = 110, cy = 110;
        g.clearRect(0, 0, S, S);
        g.save();
        g.beginPath(); g.arc(cx, cy, R, 0, Math.PI * 2); g.clip();
        g.fillStyle = '#050405'; g.fillRect(0, 0, S, S);
        const life = this.kind === 'life';
        const base = poison ? ['#1e4a14', '#5aa040'] : life ? ['#3a0606', '#c0201a'] : ['#06103a', '#2a5ad8'];
        // liquid surface: sin(x * f + time) * amp
        const top = cy + R - this.shown * R * 2;
        g.beginPath(); g.moveTo(0, S);
        for (let x = 0; x <= S; x += 4) g.lineTo(x, top + Math.sin(x * 0.05 + this.t * 2.2) * 3.5 + Math.sin(x * 0.11 - this.t * 3.1) * 1.8);
        g.lineTo(S, S); g.closePath();
        const grd = g.createLinearGradient(0, top, 0, S);
        grd.addColorStop(0, base[1]); grd.addColorStop(1, base[0]);
        g.fillStyle = grd; g.fill();
        // two noise textures moving against each other: swirling liquid
        g.save(); g.clip();
        g.globalCompositeOperation = 'overlay';
        for (let k = 0; k < 2; k++) {
            const off = (this.t * (k ? -14 : 9)) % 128, y = (this.t * (k ? 6 : -4)) % 128;
            g.globalAlpha = 0.55;
            for (let i = -1; i <= 2; i++) for (let j = -1; j <= 2; j++) g.drawImage(this.noise[k], i * 128 + off, j * 128 + y, 128, 128);
        }
        g.restore();
        g.globalAlpha = 1; g.globalCompositeOperation = 'source-over';
        if (this.pulse > 0) { g.fillStyle = `rgba(255,220,200,${this.pulse * 0.35})`; g.fillRect(0, 0, S, S); }
        // inner shadow
        const sh = g.createRadialGradient(cx, cy, R * 0.55, cx, cy, R);
        sh.addColorStop(0, 'rgba(0,0,0,0)'); sh.addColorStop(1, 'rgba(0,0,0,0.75)');
        g.fillStyle = sh; g.fillRect(0, 0, S, S);
        // glass highlight
        const hl = g.createRadialGradient(cx - 30, cy - 42, 2, cx - 30, cy - 42, 52);
        hl.addColorStop(0, 'rgba(255,255,255,0.42)'); hl.addColorStop(1, 'rgba(255,255,255,0)');
        g.fillStyle = hl; g.beginPath(); g.ellipse(cx - 26, cy - 40, 46, 30, -0.5, 0, Math.PI * 2); g.fill();
        g.restore();
    }
}

// ------------------------------------------------------------------ HUD
export class HUD {
    constructor(ui) {
        this.ui = ui;
        this.lifeOrb = new Orb('life'); this.spiritOrb = new Orb('spirit');
        this.lifeTxt = h('div', { class: 'orb-txt' }); this.spiritTxt = h('div', { class: 'orb-txt' });
        this.stFill = h('i'); this.stLag = h('i', { class: 'lag' });
        this.hullFill = h('i');
        this.xpSegs = h('div', { class: 'xp-segs' }, Array.from({ length: 10 }, () => h('i')));
        this.pearlN = h('span', { class: 'pearl-n' }, '0');
        this.belt = Array.from({ length: 4 }, (_, i) => h('div', { class: 'slot belt' }, h('span', { class: 'flask' }), h('kbd', {}, String(i + 1))));
        this.skillL = h('div', { class: 'slot skill' }, h('span', { class: 'ico sword' }), h('kbd', {}, 'J'));
        this.skillR = h('div', { class: 'slot skill' }, h('span', { class: 'ico bolt' }), h('i', { class: 'cd' }), h('kbd', {}, 'C'));
        this.offEl = h('div', { class: 'offhand-n' });
        this.bottom = h('div', { class: 'bottombar' },
            h('div', { class: 'orb life' }, this.lifeOrb.c, h('div', { class: 'orb-ring' }), this.lifeTxt, h('label', {}, 'Leben')),
            h('div', { class: 'midbar' },
                h('div', { class: 'bar-row' }, h('div', { class: 'stam' }, this.stLag, this.stFill), h('div', { class: 'hullbar hidden' }, this.hullFill)),
                h('div', { class: 'slots' }, this.skillL, h('div', { class: 'belt-row' }, this.belt), this.skillR),
                h('div', { class: 'xp-row' }, this.xpSegs, h('div', { class: 'pearls' }, h('i', { class: 'pearl-ico' }), this.pearlN)),
                this.offEl),
            h('div', { class: 'orb spirit' }, this.spiritOrb.c, h('div', { class: 'orb-ring' }), this.spiritTxt, h('label', {}, 'Geist')));
        this.gainEl = h('div', { class: 'gain' });
        this.promptEl = h('div', { class: 'prompt hidden' });
        this.toastEl = h('div', { class: 'toast' });
        this.hintEl = h('div', { class: 'hint' });
        this.bannerEl = h('div', { class: 'banner' });
        this.saveEl = h('div', { class: 'saveico' }, 'GESPEICHERT');
        this.lockEl = h('div', { class: 'lockmark hidden' });
        // ship panel (at sea)
        this.compassWind = h('i'); this.compassShip = h('b');
        this.sailsEl = h('div', { class: 'sails' });
        this.reloadL = h('i'); this.reloadR = h('i');
        this.ship = h('div', { class: 'shippanel hidden' },
            h('div', { class: 'compass', title: 'Türkis: Wind · Gold: Kurs' }, this.compassWind, this.compassShip),
            h('div', {}, h('div', {}, 'Segel'), this.sailsEl),
            h('div', {}, h('div', {}, 'Kanonen L / R'), h('div', { class: 'reload' }, h('div', {}, this.reloadL), h('div', {}, this.reloadR))));
        // target bar at the top edge
        this.targetName = h('div', { class: 'tname' }); this.targetFill = h('i');
        this.target = h('div', { class: 'target hidden' }, this.targetName, h('div', { class: 'tbar' }, this.targetFill));
        // portrait frame (the picture itself is a second camera rendering into this rectangle)
        this.portraitName = h('div', { class: 'pname' }); this.portraitLvl = h('div', { class: 'plvl' });
        this.portrait = h('div', { class: 'portrait' }, h('div', { class: 'pframe' }), h('div', { class: 'pbar' }, h('i')), this.portraitName, this.portraitLvl);
        // minimap
        this.mapC = h('canvas', { width: 240, height: 240 });
        this.areaEl = h('div', { class: 'area' });
        this.minimap = h('div', { class: 'minimap' }, this.areaEl, this.mapC);
        this.legendEl = h('div', { class: 'legend' });
        try { this.legendOn = localStorage.getItem('sw-legend') !== 'off'; } catch { this.legendOn = true; }
        this.ebars = new Map();
        this.el = h('div', { id: 'hud', class: 'hidden' }, this.portrait, this.legendEl, this.minimap, this.target, this.bottom, this.ship, this.gainEl,
            this.promptEl, this.toastEl, this.hintEl, this.bannerEl, this.saveEl, this.lockEl);
        this.mapT = 0;
        this.lastT = performance.now();
    }

    bindGame(game) {
        this.game = game;
        // live portrait: a second camera looking at the player's face, drawn into the top-left corner
        const cam = new Entity('portrait-cam');
        cam.addComponent('camera', { fov: 26, nearClip: 0.1, farClip: 20, clearColor: new Color(0.01, 0.012, 0.016), priority: 10, toneMapping: TONEMAP_ACES });
        game.app.root.addChild(cam);
        cam.enabled = false;
        this.pcam = cam;
        // mouse hover over enemies → name and life bar at the top
        const canvas = game.app.graphicsDevice.canvas;
        canvas.addEventListener('mousemove', (e) => { const r = canvas.getBoundingClientRect(); this.mouse = { x: e.clientX - r.left, y: e.clientY - r.top }; });
        canvas.addEventListener('mouseleave', () => { this.mouse = null; });
    }

    prompt(label) {
        if (label === this.lastPrompt) return;
        this.lastPrompt = label;
        if (!label) { this.promptEl.classList.add('hidden'); return; }
        this.promptEl.replaceChildren(h('kbd', {}, this.game?.stage.device.touch ? '●' : 'E'), label);
        this.promptEl.classList.remove('hidden');
    }

    // controls legend on the left (keyboard / gamepad; touch shows labelled buttons instead)
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
            const K = (k, p) => (pad ? p : k);
            const rows = !this.legendOn ? [[K('H', '—'), 'Steuerung einblenden']] : sea ? [
                [K('W / S', 'Stick ↑↓'), 'Segel setzen / reffen'], [K('A / D', 'Stick ←→'), 'Steuern'],
                [K('J', 'R1'), 'Breitseite links'], [K('K', 'B'), 'Breitseite rechts'],
                [K('E', 'X'), 'An Land gehen / Rasten', 'interact'], [K('F', '↑'), 'Shanty bestellen'],
                [K('M', 'Select'), 'Karte'], [K('Esc', 'Start'), 'Menü'], [K('H', ''), 'Ausblenden']
            ] : [
                [K('WASD', 'Stick'), 'Laufen'], [K('J', 'R1'), 'Angriff'], [K('L', 'R2'), 'Schwerer Angriff'],
                [K('Leertaste', 'A'), 'Ausweichrolle'], [K('Q', 'L1'), 'Parieren'],
                [K('C', 'Y'), 'Kettenblitz'],
                ...(g.save.off !== 'none' ? [[K('K', 'B'), 'Zweithand']] : []),
                [K('1–4 / R', '↓'), 'Rum trinken'], [K('Tab', 'R3'), 'Gegner fixieren'],
                [K('E', 'X'), 'Interagieren / An Bord', 'interact'], [K('M', 'Select'), 'Karte'],
                [K('Esc', 'Start'), 'Menü'], [K('H', ''), 'Ausblenden']
            ];
            this.legendEl.classList.toggle('mini', !this.legendOn);
            this.legendEl.replaceChildren(...rows.filter(r => r[0] !== '').map(([k, label, id]) => h('div', { class: 'row', 'data-id': id || '' }, h('kbd', {}, k), h('span', {}, label))));
        }
        this.legendEl.querySelector('[data-id="interact"]')?.classList.toggle('on', !!this.lastPrompt);
    }

    // world-space mouse hover: nearest living enemy to the ray's ground hit
    hovered(g) {
        if (!this.mouse) return null;
        const cam = g.stage.camera.camera;
        const cr = g.app.graphicsDevice.clientRect;
        const a = cam.screenToWorld(this.mouse.x * cr.width / g.stage.cssSize[0], this.mouse.y * cr.height / g.stage.cssSize[1], 1);
        const b = cam.screenToWorld(this.mouse.x * cr.width / g.stage.cssSize[0], this.mouse.y * cr.height / g.stage.cssSize[1], 60);
        let best = null, bd = 1.1;
        for (const e of g.enemies) {
            if (!e.alive) continue;
            const y = (e.sea ? 0 : Math.max(0, g.gy(e.x, e.z))) + 0.8;
            const t = (y - a.y) / (b.y - a.y);
            const px = a.x + (b.x - a.x) * t, pz = a.z + (b.z - a.z) * t;
            const d = Math.hypot(px - e.x, pz - e.z);
            if (d < bd + (e.sea ? 1 : 0)) { bd = d; best = e; }
        }
        return best;
    }

    updatePortrait(g) {
        const show = g.mode === 'land' && QUALITY[g.stage.quality].post && !this.ui.modalOpen() && g.dying === undefined && !g.stage.portrait;
        this.pcam.enabled = show;
        this.portrait.classList.toggle('nocam', !show);
        if (!show) return;
        const p = g.player, y = g.gy(p.x, p.z);
        const fr = this.portrait.querySelector('.pframe').getBoundingClientRect();
        const cv = g.app.graphicsDevice.canvas.getBoundingClientRect();
        if (!fr.width || fr.top < cv.top - 1 || fr.left < cv.left - 1) { this.pcam.enabled = false; return; }
        this.pcam.camera.rect = new Vec4((fr.left - cv.left) / cv.width, 1 - (fr.bottom - cv.top) / cv.height, fr.width / cv.width, fr.height / cv.height);
        this.pcam.camera.scissorRect = this.pcam.camera.rect;
        this.pcam.camera.aspectRatioMode = 1;
        this.pcam.camera.aspectRatio = fr.width / fr.height;
        // head height follows the animation (rolls, sitting)
        const head = p.fig.joints.head.e.getPosition();
        const fx = p.fx, fz = p.fz;
        this.pcam.setPosition(head.x + fx * 1.5 + fz * 0.45, head.y + 0.25, head.z + fz * 1.5 - fx * 0.45);
        this.pcam.lookAt(new Vec3(head.x, head.y + 0.05, head.z));
        void y;
    }

    drawMinimap(g, dt) {
        const m = g.map, c = this.mapC, x = c.getContext('2d');
        this.mapT -= dt;
        // the coast lines are re-traced when more of the map is explored
        if (!this.lines || this.mapT <= 0) {
            this.mapT = 1.5;
            const L = this.lines || (this.lines = document.createElement('canvas'));
            L.width = m.w * 6; L.height = m.h * 6;
            const lg = L.getContext('2d');
            lg.clearRect(0, 0, L.width, L.height);
            const seen = (i, j) => g.explored[j * m.w + i];
            for (let j = 0; j < m.h; j++) for (let i = 0; i < m.w; i++) {
                if (!seen(i, j)) continue;
                const t = m.get(i, j), land = m.isLand(i, j);
                if (t === T.ROCK) { lg.fillStyle = 'rgba(150,140,125,0.35)'; lg.fillRect(i * 6, j * 6, 6, 6); }
                if (t === T.STONE) { lg.fillStyle = 'rgba(200,190,160,0.25)'; lg.fillRect(i * 6, j * 6, 6, 6); }
                if (t === T.DIRT) { lg.fillStyle = 'rgba(150,110,70,0.25)'; lg.fillRect(i * 6, j * 6, 6, 6); }
                if (t === T.DOCK) { lg.fillStyle = 'rgba(190,150,100,0.6)'; lg.fillRect(i * 6, j * 6, 6, 6); }
                lg.strokeStyle = 'rgba(228,220,198,0.85)'; lg.lineWidth = 1.6;
                const edge = (a, b, c2, d) => { lg.beginPath(); lg.moveTo(a, b); lg.lineTo(c2, d); lg.stroke(); };
                if (land) {
                    if (!m.isLand(i, j - 1)) edge(i * 6, j * 6, i * 6 + 6, j * 6);
                    if (!m.isLand(i, j + 1)) edge(i * 6, j * 6 + 6, i * 6 + 6, j * 6 + 6);
                    if (!m.isLand(i - 1, j)) edge(i * 6, j * 6, i * 6, j * 6 + 6);
                    if (!m.isLand(i + 1, j)) edge(i * 6 + 6, j * 6, i * 6 + 6, j * 6 + 6);
                }
            }
        }
        const S = c.width, px = g.mode === 'sea' ? g.ship.x : g.player.x, pz = g.mode === 'sea' ? g.ship.z : g.player.z;
        x.clearRect(0, 0, S, S);
        x.save();
        x.beginPath(); x.arc(S / 2, S / 2, S / 2 - 2, 0, Math.PI * 2); x.clip();
        x.fillStyle = 'rgba(4,6,8,0.35)'; x.fillRect(0, 0, S, S);
        x.translate(S / 2, S / 2);
        x.rotate(Math.PI / 4); // screen-up is world north-west (camera turned 45°)
        const z = 0.75;
        x.drawImage(this.lines, -px * 6 * z, -pz * 6 * z, m.w * 6 * z, m.h * 6 * z);
        const dot = (wx, wz, col, r) => { x.fillStyle = col; x.beginPath(); x.arc((wx - px) * 6 * z, (wz - pz) * 6 * z, r, 0, Math.PI * 2); x.fill(); };
        for (const f of m.fires) if (g.explored[Math.floor(f.z) * m.w + Math.floor(f.x)]) dot(f.x, f.z, g.save.litFires.includes(f.id) ? (f.kind === 'sea' ? '#ff9a3a' : '#5ff2c8') : '#777', 4);
        if (g.save.stain) dot(g.save.stain.x, g.save.stain.z, '#9fe8ff', 3.5);
        if (g.mode === 'land' && g.ship.state === 'surfaced') dot(g.ship.x, g.ship.z, '#c9a060', 3.5);
        for (const e of g.enemies) if (e.alive && Math.hypot(e.x - px, e.z - pz) < 14) dot(e.x, e.z, 'rgba(200,60,50,0.8)', 2);
        x.restore();
        // player arrow (fixed in the centre, rotated with the facing)
        const fx = g.mode === 'sea' ? g.ship.fwd.x : g.player.fx, fz = g.mode === 'sea' ? g.ship.fwd.z : g.player.fz;
        const a = Math.atan2(fz, fx) + Math.PI / 4;
        x.save(); x.translate(S / 2, S / 2); x.rotate(a);
        x.fillStyle = '#fff'; x.beginPath(); x.moveTo(7, 0); x.lineTo(-5, 4.5); x.lineTo(-3, 0); x.lineTo(-5, -4.5); x.closePath(); x.fill();
        x.restore();
        // area name
        let area = 'Schwarze See';
        if (g.mode === 'land') { let bd = 1e9; for (const f of m.fires) if (f.kind === 'land') { const d = Math.hypot(f.x - px, f.z - pz); if (d < bd) { bd = d; area = d < 14 ? f.name : 'Smaragdküste'; } } }
        if (area !== this.lastArea) { this.lastArea = area; this.areaEl.textContent = area; }
    }

    update() {
        const g = this.game;
        if (!g || !g.save || !g.player) return;
        const now = performance.now(), dt = Math.min(0.1, (now - this.lastT) / 1000); this.lastT = now;
        const p = g.player, d = g.derived, s = g.save;
        const sea = g.mode === 'sea';
        // scale the HUD with the canvas (reference: 1080 lines)
        const scale = Math.max(0.55, Math.min(1.6, g.stage.cssSize[1] / 1080));
        if (scale !== this.scale) { this.scale = scale; this.el.style.setProperty('--hud', String(scale)); }
        this.legend(g);
        if (this.touch) {
            const show = g.stage.device.touch && !this.ui.modalOpen() && g.dying === undefined;
            this.touch.setVisible(show);
            if (show) this.touch.setMode(g.mode, s.off);
        }
        // orbs
        this.lifeOrb.set(p.hp / d.maxHp, true); this.lifeOrb.draw(dt);
        this.spiritOrb.set(p.mp / d.maxMp, false); this.spiritOrb.draw(dt);
        this.lifeTxt.textContent = `${Math.ceil(p.hp)} / ${d.maxHp}`;
        this.spiritTxt.textContent = `${Math.floor(p.mp)} / ${d.maxMp}`;
        // stamina with lag / hull at sea
        const sf = p.st / d.maxSt;
        this.stFill.style.transform = `scaleX(${Math.max(0, sf)})`;
        this.stLagV = this.stLagV > sf ? Math.max(sf, this.stLagV - dt * 0.6) : sf;
        this.stLag.style.transform = `scaleX(${Math.max(0, this.stLagV)})`;
        this.bottom.querySelector('.stam').classList.toggle('hidden', sea);
        this.bottom.querySelector('.hullbar').classList.toggle('hidden', !sea);
        if (sea) this.hullFill.style.transform = `scaleX(${g.ship.hull / g.ship.maxHull})`;
        // pearls towards the next level, ten segments
        const cost = levelCost(s.level), prog = Math.min(1, s.pearls / cost);
        [...this.xpSegs.children].forEach((seg, i) => { const f = Math.max(0, Math.min(1, prog * 10 - i)); seg.style.setProperty('--f', String(f)); });
        this.pearlN.textContent = String(s.pearls);
        this.xpSegs.title = `${s.pearls} / ${cost} Perlen bis Stufe ${s.level + 1}`;
        // belt: one rum flask per slot (more than four: the last slot shows the rest)
        this.belt.forEach((b, i) => { const have = p.flasks > i; b.classList.toggle('empty', !have); b.querySelector('.flask').textContent = i === 3 && p.flasks > 4 ? `+${p.flasks - 3}` : ''; });
        this.skillR.classList.toggle('empty', p.mp < 28);
        this.offEl.textContent = s.off !== 'none' ? `${s.off === 'pistol' ? 'Pulver' : 'Gezeiten'}: ${p.ammo}  (K)` : '';
        // ship panel
        this.ship.classList.toggle('hidden', !sea);
        if (sea) {
            this.compassWind.style.transform = `rotate(${g.wind.angle + Math.PI / 4}rad)`;
            this.compassShip.style.transform = `rotate(${g.ship.heading + Math.PI / 4}rad)`;
            const sails = g.ship.sail;
            if (this.lastSail !== sails) { this.lastSail = sails; this.sailsEl.replaceChildren(...[0, 1].map(i => h('span', { class: i < sails ? 'on' : '' }))); }
            this.reloadL.style.transform = `scaleX(${1 - g.ship.reload[0] / 2.6})`;
            this.reloadR.style.transform = `scaleX(${1 - g.ship.reload[1] / 2.6})`;
        }
        // portrait
        this.portraitName.textContent = s.name;
        this.portraitLvl.textContent = `Stufe ${s.level}`;
        this.portrait.querySelector('.pbar i').style.transform = `scaleX(${Math.max(0, p.hp / d.maxHp)})`;
        this.updatePortrait(g);
        this.drawMinimap(g, dt);
        // target: hovered > locked > last hit
        const hov = this.hovered(g);
        let tgt = hov || g.lockTarget;
        if (!tgt) for (const e of g.enemies) if (e.alive && e.showBarUntil > g.time && (!tgt || e.lastDmgT > tgt.lastDmgT)) tgt = e;
        this.target.classList.toggle('hidden', !tgt);
        if (tgt) {
            this.targetName.textContent = tgt.displayName || 'Feind';
            this.targetFill.style.transform = `scaleX(${Math.max(0, tgt.hp / tgt.maxHp)})`;
        }
        g.hovered = hov;
        // lock marker
        const lt = g.lockTarget;
        if (lt) { const sp = g.toScreen(lt.x, lt.z, lt.sea ? 0.6 : 1.0); this.lockEl.style.left = `${sp.x}px`; this.lockEl.style.top = `${sp.y}px`; this.lockEl.classList.remove('hidden'); }
        else this.lockEl.classList.add('hidden');
        // small floating bars above damaged enemies
        for (const e of g.enemies) {
            let b = this.ebars.get(e);
            const show = e.alive && e.showBarUntil > g.time && e.hp > 0 && e !== tgt;
            if (!show) { if (b) b.el.classList.add('hidden'); continue; }
            if (!b) { b = { el: h('div', { class: 'ebar' }, h('i'), h('b')) }; this.el.append(b.el); this.ebars.set(e, b); }
            const sp = g.toScreen(e.x, e.z, e.sea ? 1.2 : 2.1);
            b.el.classList.remove('hidden');
            b.el.style.left = `${sp.x}px`; b.el.style.top = `${sp.y}px`;
            b.el.querySelector('i').style.transform = `scaleX(${Math.max(0, e.hp / e.maxHp)})`;
            b.el.querySelector('b').textContent = g.time - e.lastDmgT < 1.5 ? String(e.lastDmg) : '';
        }
    }
}
