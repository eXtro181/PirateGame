// Effects in world pixels: particles (blood, sparks, embers, soul light, snow, rain…), slash arcs,
// chain-lightning bolts, floating damage numbers, telegraph markers for enemy attacks, flashes.
import { NOISE } from '../gfx/noise.js';

const C = {
    blood: [0.48, 0.1, 0.13, 1], spark: [1, 0.85, 0.5, 1], ember: [1, 0.42, 0.15, 1], soul: [0.37, 0.95, 0.78, 1], soulLight: [0.72, 1, 0.91, 1],
    bone: [0.85, 0.81, 0.72, 1], splash: [0.66, 0.78, 0.9, 1], dust: [0.45, 0.42, 0.36, 1], smoke: [0.25, 0.27, 0.3, 1], ice: [0.75, 0.92, 1, 1],
    snow: [0.85, 0.9, 0.92, 1], rain: [0.55, 0.65, 0.72, 0.55], leaf: [0.3, 0.42, 0.3, 1], spore: [0.6, 0.85, 0.45, 1], wind: [0.85, 0.92, 1, 0.6], heal: [0.9, 0.75, 0.35, 1], earth: [0.42, 0.36, 0.26, 1]
};
export { C as FXCOL };

export class FX {
    constructor(app) {
        this.app = app;
        this.parts = []; this.slashes = []; this.bolts = []; this.nums = []; this.marks = []; this.rings = [];
        this.ambT = 0;
    }
    clear() { this.parts.length = 0; this.slashes.length = 0; this.bolts.length = 0; this.nums.length = 0; this.marks.length = 0; this.rings.length = 0; }
    p(x, y, o) { if (this.parts.length > 900) this.parts.shift(); this.parts.push({ x, y, z: o.z ?? 0, vx: o.vx || 0, vy: o.vy || 0, vz: o.vz || 0, g: o.g ?? 0, life: o.life ?? 0.6, max: o.life ?? 0.6, col: o.col || C.spark, glow: o.glow || 0, size: o.size || 1, drag: o.drag ?? 0, rain: o.rain, ground: o.ground ?? false, fade: o.fade ?? true, wobble: o.wobble || 0, ph: Math.random() * 6 }); }
    burst(kind, x, y, n = 8, o = {}) {
        for (let i = 0; i < n; i++) {
            const a = (o.dir ?? Math.random() * Math.PI * 2) + (o.dir !== undefined ? (Math.random() - 0.5) * (o.spread ?? 1.2) : 0);
            const sp = (o.speed ?? 60) * (0.4 + Math.random() * 0.8);
            const base = { vx: Math.cos(a) * sp, vy: Math.sin(a) * sp * 0.6, z: o.z ?? 8, vz: (o.up ?? 40) * (0.5 + Math.random()), g: o.g ?? 160, life: (o.life ?? 0.6) * (0.6 + Math.random() * 0.8), col: o.col || C[kind] || C.spark, glow: o.glow ?? (['spark', 'ember', 'soul', 'soulLight', 'ice', 'heal'].includes(kind) ? 1.2 : 0), ground: o.ground ?? (kind === 'blood' || kind === 'bone'), drag: o.drag ?? 1 };
            this.p(x + (Math.random() - 0.5) * (o.jitter ?? 4), y + (Math.random() - 0.5) * (o.jitter ?? 4), base);
        }
    }
    rise(kind, x, y, n = 6, o = {}) { for (let i = 0; i < n; i++) this.p(x + (Math.random() - 0.5) * (o.w ?? 12), y + (Math.random() - 0.5) * 4, { z: Math.random() * 6, vz: (o.speed ?? 22) * (0.6 + Math.random() * 0.8), vx: (Math.random() - 0.5) * 8, g: 0, life: o.life ?? 1.2, col: o.col || C[kind], glow: o.glow ?? 1.2, wobble: 6 }); }
    slash(x, y, ang, r = 18, col = [0.9, 0.95, 1, 1], width = 1.4, dur = 0.14) { this.slashes.push({ x, y, ang, r, col, width, t: 0, dur }); }
    bolt(pts, col = [0.75, 1, 0.95, 1], dur = 0.25) {
        // jagged segments between the points
        const segs = [];
        for (let i = 0; i < pts.length - 1; i++) {
            const [x0, y0] = pts[i], [x1, y1] = pts[i + 1];
            const n = Math.max(3, Math.round(Math.hypot(x1 - x0, y1 - y0) / 10));
            let px = x0, py = y0;
            for (let k = 1; k <= n; k++) { const t = k / n; const nx = x0 + (x1 - x0) * t + (k < n ? (Math.random() - 0.5) * 10 : 0), ny = y0 + (y1 - y0) * t + (k < n ? (Math.random() - 0.5) * 10 : 0); segs.push([px, py, nx, ny]); px = nx; py = ny; }
        }
        this.bolts.push({ segs, col, t: 0, dur });
    }
    num(x, y, v, col = [0.95, 0.9, 0.8, 1]) { this.nums.push({ x, y, v: String(v), col, t: 0 }); }
    // telegraph markers: { kind:'circle'|'cone'|'line', x, y, r, ang, w, t, dur, col }
    mark(m) { this.marks.push({ t: 0, col: [0.9, 0.2, 0.18, 0.4], ...m }); }
    ring(x, y, r0, r1, dur, col) { this.rings.push({ x, y, r0, r1, t: 0, dur, col }); }

    // ambient particles around the camera per biome
    ambient(dt, cam, W, H, biome, sailing) {
        this.ambT += dt;
        const rate = { hub: 14, fire: 26, ice: 30, wind: 50, earth: 10, end: 24, coast: 4 }[biome] || 0;
        let n = rate * dt + Math.random();
        while (n-- > 1) {
            const x = cam[0] + Math.random() * W, y = cam[1] + Math.random() * H;
            switch (biome) {
                case 'hub': this.p(x, y, { z: 4 + Math.random() * 20, vx: (Math.random() - 0.5) * 6, vz: (Math.random() - 0.5) * 4, life: 4, col: C.soul, glow: 1.6, wobble: 10 }); break; // fireflies
                case 'fire': this.p(x, y - 40, { z: 30, vx: 10 + Math.random() * 10, vy: 6, vz: -10, life: 4, col: Math.random() < 0.3 ? C.ember : [0.35, 0.32, 0.32, 1], glow: Math.random() < 0.3 ? 1 : 0, wobble: 6 }); break; // ash
                case 'ice': this.p(x, y - 60, { z: 60, vx: 6, vy: 4, vz: -16, life: 4, col: C.snow, wobble: 5 }); break;
                case 'wind': this.p(x + 30, y - 80, { z: 80, vx: -40, vy: 30, vz: -120, life: 0.8, col: C.rain, size: 2, rain: true }); break;
                case 'earth': this.p(x, y, { z: 6 + Math.random() * 10, vz: 3, life: 3, col: C.spore, glow: 0.8, wobble: 8 }); break;
                case 'end': this.p(x, y + 20, { z: 0, vz: 18, vx: 4, life: 3, col: Math.random() < 0.5 ? C.ember : [0.6, 0.1, 0.15, 1], glow: 1.2, wobble: 4 }); break;
                case 'coast': if (Math.random() < 0.5) this.p(x, y, { z: 8 + Math.random() * 12, vx: (Math.random() - 0.5) * 4, life: 4, col: C.soul, glow: 1.2, wobble: 10 }); break;
            }
        }
        void sailing;
    }
    update(dt) {
        for (const p of this.parts) {
            p.life -= dt;
            const k = Math.max(0, 1 - p.drag * dt);
            p.vx *= k; p.vy *= k;
            p.x += p.vx * dt + (p.wobble ? Math.sin(this.ambT * 2 + p.ph) * p.wobble * dt : 0);
            p.y += p.vy * dt;
            p.vz -= p.g * dt; p.z += p.vz * dt;
            if (p.z < 0) { p.z = 0; if (p.ground) { p.vx = p.vy = p.vz = 0; p.g = 0; } else if (p.g) { p.vz = -p.vz * 0.3; p.vx *= 0.5; p.vy *= 0.5; } }
        }
        this.parts = this.parts.filter(p => p.life > 0);
        for (const arr of [this.slashes, this.bolts, this.nums, this.marks, this.rings]) for (const s of arr) s.t += dt;
        this.slashes = this.slashes.filter(s => s.t < s.dur);
        this.bolts = this.bolts.filter(s => s.t < s.dur);
        this.nums = this.nums.filter(s => s.t < 0.9);
        this.marks = this.marks.filter(s => s.t < s.dur);
        this.rings = this.rings.filter(s => s.t < s.dur);
    }
    // ---------------------------------------------------------------- drawing
    px(r, x, y, c, w = 1, h = 1) { r.rect(this.white.tex, Math.round(x), Math.round(y), w, h, c); }
    // telegraphs on the ground (under entities)
    drawUnder(r, cam) {
        this.white = this.app.art.atlas.white;
        r.spriteBegin('alpha');
        for (const m of this.marks) {
            const k = Math.min(1, m.t / Math.max(0.01, m.dur));
            const a = m.col[3] * (0.5 + 0.5 * Math.sin(m.t * 18) * 0.3);
            const col = [m.col[0], m.col[1], m.col[2], a];
            if (m.kind === 'circle') {
                const R = m.r, step = 1;
                for (let y = -R * 0.6; y <= R * 0.6; y += step) {
                    const w = Math.sqrt(Math.max(0, 1 - (y / (R * 0.6)) ** 2)) * R;
                    if (((Math.round(y) + Math.round(m.t * 20)) & 1) === 0) this.px(r, m.x - w - cam[0], m.y + y - cam[1], col, Math.round(w * 2), 1);
                    else { this.px(r, m.x - w - cam[0], m.y + y - cam[1], col); this.px(r, m.x + w - cam[0], m.y + y - cam[1], col); }
                }
                // fill ring shows the time left
                const rr = R * k;
                for (let a2 = 0; a2 < Math.PI * 2; a2 += 0.08) this.px(r, m.x + Math.cos(a2) * rr - cam[0], m.y + Math.sin(a2) * rr * 0.6 - cam[1], [col[0], col[1], col[2], 0.8]);
            } else if (m.kind === 'cone') {
                for (let d = 6; d < m.r; d += 2) for (let s = -m.w; s <= m.w; s += 0.12) {
                    const a2 = m.ang + s;
                    if ((d / 2 + Math.round(s * 10)) % 2) continue;
                    this.px(r, m.x + Math.cos(a2) * d - cam[0], m.y + Math.sin(a2) * d * 0.7 - cam[1], d < m.r * k ? [col[0], col[1], col[2], 0.55] : col);
                }
            } else if (m.kind === 'line') {
                const len = m.r, ca = Math.cos(m.ang), sa = Math.sin(m.ang);
                for (let d = 0; d < len; d += 2) for (let s = -m.w; s <= m.w; s += 2) this.px(r, m.x + ca * d - sa * s - cam[0], m.y + sa * d * 0.85 + ca * s - cam[1], d < len * k ? [col[0], col[1], col[2], 0.55] : col);
            }
        }
        r.end();
    }
    drawParticles(r, cam, glowPass = false) {
        this.white = this.app.art.atlas.white;
        r.spriteBegin('alpha');
        for (const p of this.parts) {
            if (glowPass && !p.glow) continue;
            const a = p.fade ? Math.min(1, p.life / p.max * 2) : 1;
            const c = glowPass ? [p.col[0] * p.glow * a, p.col[1] * p.glow * a, p.col[2] * p.glow * a, 1] : [p.col[0], p.col[1], p.col[2], (p.col[3] ?? 1) * a];
            if (p.rain) { const x = p.x - cam[0], y = p.y - p.z - cam[1]; this.px(r, x, y, c); this.px(r, x - 1, y + 1, c); this.px(r, x - 1, y + 2, c); continue; }
            this.px(r, p.x - cam[0], p.y - p.z - cam[1], c, p.size, p.size);
        }
        // slash arcs
        for (const s of this.slashes) {
            const k = s.t / s.dur, n = 14;
            for (let i = 0; i < n; i++) {
                const t = i / (n - 1);
                if (t > k * 1.6) continue;
                const a2 = s.ang - 1.2 + t * 2.4;
                const rr = s.r * (0.85 + 0.15 * Math.sin(t * Math.PI));
                const x = s.x + Math.cos(a2) * rr - cam[0], y = s.y + Math.sin(a2) * rr * 0.75 - cam[1];
                const fade = 1 - k;
                const c = glowPass ? [s.col[0] * 0.8 * fade, s.col[1] * 0.8 * fade, s.col[2] * 0.8 * fade, 1] : [s.col[0], s.col[1], s.col[2], fade];
                this.px(r, x, y, c, 2, 1);
                if (s.width > 1 && t > 0.2 && t < 0.8) this.px(r, x - Math.cos(a2) * 2, y - Math.sin(a2) * 1.5, c);
            }
        }
        // lightning
        for (const b of this.bolts) {
            const fade = 1 - b.t / b.dur;
            const c = glowPass ? [b.col[0] * 2 * fade, b.col[1] * 2 * fade, b.col[2] * 2 * fade, 1] : [1, 1, 1, fade];
            for (const [x0, y0, x1, y1] of b.segs) {
                const n = Math.max(1, Math.round(Math.hypot(x1 - x0, y1 - y0)));
                for (let i = 0; i <= n; i++) this.px(r, x0 + (x1 - x0) * i / n - cam[0], y0 + (y1 - y0) * i / n - cam[1], c);
            }
        }
        // shock rings
        for (const g of this.rings) {
            const k = g.t / g.dur, rr = g.r0 + (g.r1 - g.r0) * k;
            const c = glowPass ? [g.col[0] * (1 - k) * 1.5, g.col[1] * (1 - k) * 1.5, g.col[2] * (1 - k) * 1.5, 1] : [g.col[0], g.col[1], g.col[2], (1 - k) * 0.8];
            const n = Math.max(12, Math.round(rr * 1.2));
            for (let i = 0; i < n; i++) { const a2 = i / n * Math.PI * 2; this.px(r, g.x + Math.cos(a2) * rr - cam[0], g.y + Math.sin(a2) * rr * 0.6 - cam[1], c); }
        }
        r.end();
    }
    drawNumbers(r, cam, font) {
        r.spriteBegin('alpha');
        for (const n of this.nums) {
            const k = n.t / 0.9;
            font.draw(r, n.v, n.x - cam[0], n.y - cam[1] - 14 - k * 14, [n.col[0], n.col[1], n.col[2], 1 - k * k], { align: 'center' });
        }
        r.end();
    }
    lights(out) {
        for (const b of this.bolts) for (let i = 0; i < b.segs.length; i += 4) out.push({ x: b.segs[i][0], y: b.segs[i][1], r: 50, col: [0.6, 1, 0.95], i: 1.5 * (1 - b.t / b.dur) });
        return out;
    }
}
export { NOISE };
