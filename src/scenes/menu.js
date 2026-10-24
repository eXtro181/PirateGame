// Animated main menu (480×270, ten parallax layers) with the menu list:
// Neues Spiel · Fortsetzen · Optionen · Credits · Beenden.
import { MENU_W, MENU_H, LAYER_W, HORIZON } from '../art/menu.js';
import { rgb } from '../art/pal.js';
import { NOISE } from '../gfx/noise.js';

const c3 = (h, a = 1) => { const v = rgb(h); return [v[0] / 255, v[1] / 255, v[2] / 255, a]; };
const SKY = ['#0B1016', '#0F1820', '#121E29', '#172634', '#1D2F3F', '#24394A'].map(h => c3(h));
const GOLD = c3('#E8A443'), BONE = c3('#D8CFB8', 0.7), BONE_FULL = c3('#D8CFB8');
export const MENU_ITEMS = ['Neues Spiel', 'Fortsetzen', 'Optionen', 'Credits', 'Beenden'];

export class MenuScene {
    constructor(app) {
        this.app = app; this.mode = 'menu'; this.A = app.art.menu; this.U = app.art.ui;
        this.t = 0; this.sel = 0; this.slide = MENU_ITEMS.map(() => 0);
        this.pressed = false; // "Drücke eine Taste"
        this.flashT = 0; this.nextFlash = 6;
        this.stars = Array.from({ length: 70 }, (_, i) => ({ x: (i * 97.3) % LAYER_W, y: (i * 53.7) % (HORIZON - 70), p: (i * 0.37) % 6.28, b: i % 5 === 0 }));
        this.rain = Array.from({ length: 120 }, (_, i) => ({ x: Math.random() * MENU_W, y: Math.random() * MENU_H, v: 220 + Math.random() * 120 }));
        this.runes = [0, 1, 2];
        this.enabled = true;
    }
    enter() { this.enabled = true; this.app.audio.setMood('title'); }
    update(dt, input) {
        this.t += dt;
        // lightning
        this.nextFlash -= dt;
        if (this.nextFlash <= 0) { this.flashT = 2; this.nextFlash = 7 + Math.random() * 10; this.app.audio.thunder?.(); }
        for (const d of this.rain) { d.y += d.v * dt; d.x -= d.v * dt * 0.36; if (d.y > MENU_H) { d.y = -6; d.x = Math.random() * (MENU_W + 60); } }
        for (let i = 0; i < MENU_ITEMS.length; i++) this.slide[i] += ((i === this.sel ? 4 : 0) - this.slide[i]) * Math.min(1, dt / 0.12 * 2.2);
        if (!this.enabled || this.app.ui.busy()) return;
        if (!this.pressed) { if (input.anyKey()) { this.pressed = true; this.app.audio.unlock(); this.app.audio.sfx('click'); } return; }
        if (input.pressed('up')) { this.sel = (this.sel + MENU_ITEMS.length - 1) % MENU_ITEMS.length; this.app.audio.sfx('click'); }
        if (input.pressed('down')) { this.sel = (this.sel + 1) % MENU_ITEMS.length; this.app.audio.sfx('click'); }
        if (input.pressed('interact') || input.pressed('confirm')) this.choose(this.sel);
    }
    choose(i) { this.sel = i; this.app.audio.sfx('kindle'); this.app.ui.menuAction(MENU_ITEMS[i]); }
    // pointer: map canvas px → menu px
    pointer(mx, my, click) {
        if (!this.pressed) { if (click) { this.pressed = true; this.app.audio.unlock(); } return; }
        for (let i = 0; i < MENU_ITEMS.length; i++) {
            const y = this.listY + i * 16;
            if (mx > 20 && mx < 150 && my > y - 3 && my < y + 11) { if (this.sel !== i) { this.sel = i; this.app.audio.sfx('click'); } if (click) this.choose(i); }
        }
    }
    frame() {
        const A = this.A, t = this.t;
        const drift = Math.sin(t * Math.PI * 2 / 30); // ±1 over 30 s
        const off = s => Math.round(-16 + drift * 15 * s); // layer x offset (max ±12 px at layer 8)
        const self = this;
        return {
            cam: [0, 0], ambient: [1, 1, 1], lights: [],
            drawScene(r) {
                const W = r.atlasW;
                r.spriteBegin('alpha');
                const white = self.app.art.atlas.white;
                // 0: sky in 6 hard steps
                for (let i = 0; i < 6; i++) r.rect(white.tex, 0, Math.floor(i * HORIZON / 6), MENU_W, Math.ceil(HORIZON / 6) + 1, SKY[i]);
                // 1: stars + moon with halo rings
                for (const s of self.stars) { const tw = 0.5 + 0.5 * Math.sin(t * 1.3 + s.p * 3); if (tw < 0.25) continue; r.rect(white.tex, s.x + off(0.02), s.y, 1, 1, [0.8, 0.86, 0.88, 0.4 + tw * 0.6]); }
                const mx = Math.round(MENU_W * 0.22) + off(0.02) + 16, my = Math.round(MENU_H * 0.18);
                r.sprite(A.halo3, mx - 46, my - 46, { color: [0.6, 0.7, 0.72, 0.05] });
                r.sprite(A.halo2, mx - 32, my - 32, { color: [0.6, 0.7, 0.72, 0.1] });
                r.sprite(A.halo1, mx - 22, my - 22, { color: [0.65, 0.75, 0.76, 0.2] });
                r.sprite(A.moon, mx - 15, my - 15);
                // 2 + 3: clouds (wrap)
                const cl = (rect, edge, y, speed, sc) => {
                    const x = -((t * speed) % 960) + off(sc);
                    for (const xx of [x, x + 960]) { r.sprite(rect, xx, y); if (self.flashT > 0) r.sprite(edge, xx, y, { color: [0.72, 1, 0.91, 1] }); }
                };
                cl(A.cloudsFar, A.cloudsFarEdge, 4, 2, 0.05);
                cl(A.cloudsNear, A.cloudsNearEdge, 26, 5, 0.1);
                // 4: mountains + horizon sea, ghost ship drifting right → left in 90 s at 60 %
                const my4 = HORIZON - A.mountainsBase;
                r.sprite(A.mountains, off(0.15), my4);
                const gs = ((t / 90) % 1);
                r.sprite(A.ghostShip, Math.round(MENU_W + 20 - gs * (MENU_W + 80)) + off(0.15), HORIZON - 26, { color: [1, 1, 1, 0.6] });
                // 5: soul-tree island
                const ix = Math.round(MENU_W * 0.63 - A.islandW / 2) + off(0.25), iy = HORIZON - A.islandSea + 6;
                r.sprite(A.island, ix, iy);
                self.islandPos = [ix, iy];
                // 6: plain with fortress and rivers
                const py = HORIZON - 30;
                r.sprite(A.plain, off(0.4), py);
                self.plainPos = [off(0.4), py];
                r.end();
                // 7: fog band, sine-distorted rows, 30–50 % alpha
                r.spriteBegin('alpha');
                const fy = HORIZON + 14, fa = 0.3 + 0.2 * (0.5 + 0.5 * Math.sin(t * 0.4));
                const fogRect = A.fog;
                for (let row = 0; row < A.fogH; row += 2) {
                    const dx = Math.round(Math.sin(t * 0.6 + row * 0.25) * 3);
                    const sub = { x: fogRect.x, y: fogRect.y + row, w: fogRect.w, h: 2, tex: fogRect.tex };
                    r.sprite(sub, off(0.5) + dx, fy + row, { color: [0.56, 0.64, 0.66, fa] });
                }
                // 8: foreground silhouettes (sway)
                r.sprite(A.fore, off(0.8), MENU_H - A.foreH, { sway: 1, wx: 0 });
                // 9: rain at 20°
                for (const d of self.rain) r.sprite(A.rain, d.x, d.y, { color: [0.55, 0.65, 0.72, 0.5] });
                r.end();
            },
            drawGlow(r) {
                r.spriteBegin('alpha');
                const black = [0, 0, 0, 1];
                r.sprite(A.moon, Math.round(MENU_W * 0.22) + off(0.02) + 1, Math.round(MENU_H * 0.18) - 15, { tex: A.moon.gtex, color: [0.6, 0.6, 0.6, 1] });
                if (self.islandPos) {
                    const [ix, iy] = self.islandPos;
                    const k = 0.7 + 0.3 * Math.sin(t * Math.PI * 2 / 4);
                    r.sprite(A.island, ix, iy, { tex: A.island.gtex, color: [k, k, k, 1] });
                    // three rune rings, one turn per 12 s
                    const cx = ix + A.islandW / 2, cy = iy + 40;
                    const white = self.app.art.atlas.white;
                    for (let ring = 0; ring < 3; ring++) {
                        const rx = 48 + ring * 14, ry = 12 + ring * 4, n = 18 + ring * 6;
                        for (let i = 0; i < n; i++) {
                            const a = i / n * Math.PI * 2 + (t / 12) * Math.PI * 2 * (ring % 2 ? -1 : 1);
                            if (i % 3 === 2) continue;
                            r.rect(white.tex, Math.round(cx + Math.cos(a) * rx), Math.round(cy + Math.sin(a) * ry), i % 3 ? 1 : 2, 1, [0.37 * k, 0.95 * k, 0.78 * k, 1]);
                        }
                    }
                }
                if (self.plainPos) {
                    const [px, py] = self.plainPos;
                    r.sprite(A.plain, px, py, { tex: A.plain.gtex });
                    const white = self.app.art.atlas.white;
                    A.windows.forEach(([x, y], i) => { const f = 0.5 + 0.5 * NOISE.n1(t * 3 + i * 7); if (f > 0.25) r.rect(white.tex, px + x, py + y, 1, 1, [1.4 * f, 0.8 * f, 0.3 * f, 1]); });
                    // lighthouse beam: a rotating cone of dithered light, period 8 s
                    const [lx, ly] = A.lighthouse;
                    const ang = (t / 8) * Math.PI * 2;
                    const dir = Math.cos(ang);
                    const vis = Math.abs(Math.sin(ang)) * 0.5 + 0.5;
                    for (let d = 3; d < 140; d += 1) {
                        const w = d * 0.12;
                        for (let k2 = -w; k2 <= w; k2 += 1) {
                            if (((d + Math.round(k2)) & 1) && d > 40) continue;
                            const x = px + lx + dir * d, y = py + ly + k2 * 0.6 - d * 0.04;
                            const a = (1 - d / 140) * 0.55 * vis;
                            r.rect(white.tex, Math.round(x), Math.round(y), 1, 1, [1.0 * a, 0.8 * a, 0.45 * a, 1]);
                        }
                    }
                }
                void black;
                r.end();
            },
            drawUI(r) { self.drawUI(r); }
        };
    }
    drawUI(r) {
        const t = this.t, F = this.app.font, U = this.U, white = this.app.art.atlas.white;
        r.spriteBegin('alpha');
        if (this.flashT > 0) { r.rect(white.tex, 0, 0, MENU_W, MENU_H, [0.72, 1, 0.9, 0.08]); this.flashT--; }
        // logo with emblem, top-left
        const lx = Math.round(MENU_W * 0.06), ly = Math.round(MENU_H * 0.1);
        r.sprite(U.emblem, lx, ly - 4);
        r.sprite(this.app.art.logo, lx + 38, ly + 2);
        F.draw(r, 'Die See nimmt. Die See gibt zurück.', lx + 40, ly + 26, [0.56, 0.64, 0.66, 0.9]);
        const ver = this.app.version;
        F.draw(r, ver, MENU_W - 6, MENU_H - 12, [0.85, 0.81, 0.72, 0.5], { align: 'right' });
        if (!this.pressed) {
            const a = 0.45 + 0.55 * (0.5 + 0.5 * Math.sin(t * Math.PI * 2 / 1.5));
            F.draw(r, 'Drücke eine Taste', MENU_W / 2, MENU_H - 40, [0.85, 0.81, 0.72, a], { align: 'center' });
            r.end();
            return;
        }
        // list on a parchment band
        const y0 = this.listY = Math.round(MENU_H * 0.42);
        r.rect(white.tex, 0, y0 - 8, 168, MENU_ITEMS.length * 16 + 10, [0.043, 0.063, 0.086, 0.55]);
        r.rect(white.tex, 0, y0 - 9, 168, 1, [0.91, 0.64, 0.26, 0.35]);
        r.rect(white.tex, 0, y0 + MENU_ITEMS.length * 16 + 2, 168, 1, [0.91, 0.64, 0.26, 0.35]);
        MENU_ITEMS.forEach((it, i) => {
            const y = y0 + i * 16, s = Math.round(this.slide[i]);
            const disabled = it === 'Fortsetzen' && !this.app.hasProfiles;
            if (i === this.sel) r.sprite(U.compass[Math.floor(t * 8) % 4], 20 + s - 12, y - 1);
            F.draw(r, it, 24 + s, y, i === this.sel ? GOLD : disabled ? [0.85, 0.81, 0.72, 0.35] : BONE);
        });
        F.draw(r, '↑↓ wählen · E / Enter bestätigen', 24, y0 + MENU_ITEMS.length * 16 + 8, [0.56, 0.64, 0.66, 0.6]);
        void BONE_FULL;
        r.end();
    }
}
