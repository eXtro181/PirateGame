// World rendering: autotiled ground with organic edge overlays, basalt cliffs, the water shader,
// ground deco, multiplied soft shadows, y-sorted props and entities, overhead crowns that fade
// when the player walks beneath, glow pass with occlusion, fog layers, baked + dynamic lights.
import { G, TS, PRIO, BIOME } from '../art/tiles.js';
import { MW, MH, BIOMES, ZONES } from './world.js';
import { Pix, BAYER } from '../art/pix.js';
import { f3, P } from '../art/pal.js';
import { Noise, NOISE } from '../gfx/noise.js';

const hash = (x, y) => { let h = (x * 374761393 + y * 668265263) | 0; h = (h ^ (h >>> 13)) * 1274126177; return (h ^ (h >>> 16)) >>> 0; };
const DIR4 = [['n', 0, -1], ['e', 1, 0], ['s', 0, 1], ['w', -1, 0]];
const DIAG = [['ne', 1, -1, 'n', 'e'], ['nw', -1, -1, 'n', 'w'], ['se', 1, 1, 's', 'e'], ['sw', -1, 1, 's', 'w']];
const FOG = { coast: [[0.56, 0.64, 0.66], 0.18], hub: [[0.5, 0.7, 0.68], 0.22], fire: [[0.5, 0.2, 0.3], 0.2], ice: [[0.8, 0.88, 0.92], 0.24], wind: [[0.5, 0.56, 0.62], 0.26], earth: [[0.36, 0.46, 0.36], 0.3], end: [[0.5, 0.15, 0.22], 0.24] };

export class WorldView {
    constructor(r, world, art) {
        this.r = r; this.world = world; this.art = art; // art: { tiles, deco, props, ui }
        this.time = 0;
        // per tile: base tile rect + overlay list (computed once)
        this.cache = new Array(MW * MH);
        for (let ty = 0; ty < MH; ty++) for (let tx = 0; tx < MW; tx++) this.cache[ty * MW + tx] = this.tileLayers(tx, ty);
        // water mask: r = water, g = distance to the shore (0..8 tiles)
        const data = new Uint8Array(MW * MH * 4);
        for (let i = 0; i < MW * MH; i++) {
            const w = world.ground[i] === G.WATER;
            data[i * 4] = w ? 255 : 0;
            data[i * 4 + 1] = w ? Math.min(255, Math.max(0, (world.shoreDist[i] - 0.5) / 8 * 255)) : 0;
            data[i * 4 + 3] = 255;
        }
        this.mask = r.texture(null, { w: MW, h: MH, linear: true });
        const gl = r.gl; gl.bindTexture(gl.TEXTURE_2D, this.mask); gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA8, MW, MH, 0, gl.RGBA, gl.UNSIGNED_BYTE, data);
        this.water = { mask: this.mask, mapW: MW, mapH: MH, colors: { shallow: f3('#22508A'), deep: f3('#0E2140'), glint: f3('#4F86C0'), foam: f3('#A9C8E6'), soul: f3('#5FF2C8') } };
        this.bake();
        this.fog = this.makeFog();
        // spatial buckets for props (8×8 tiles)
        this.buckets = new Map();
        for (const p of world.props) this.bucketAdd(p);
    }
    bucketAdd(p) { const k = Math.floor(p.y / 128) * 1000 + Math.floor(p.x / 128); if (!this.buckets.has(k)) this.buckets.set(k, []); this.buckets.get(k).push(p); }
    bucketRemove(p) { const k = Math.floor(p.y / 128) * 1000 + Math.floor(p.x / 128); const a = this.buckets.get(k); if (a) { const i = a.indexOf(p); if (i >= 0) a.splice(i, 1); } }
    bake() {
        const st = this.world.lightsStatic.filter(l => !l.flicker && !l.pulse);
        // soft ambient soul glow from soul stones etc. is part of the static set
        this.lightmap = this.r.bakeLightmap(MW * TS, MH * TS, st, 4);
    }
    makeFog() {
        const n = new Noise(77), S = 128;
        const layers = [];
        for (let L = 0; L < 2; L++) {
            const p = new Pix(S, S);
            for (let y = 0; y < S; y++) for (let x = 0; x < S; x++) {
                const v = n.fbm(x / 32 + L * 5, y / 32, 3, 4) * 0.5 + 0.5 - 0.35;
                const q = Math.floor(Math.max(0, v) * 6) / 4; // quantized to 4 steps
                if (q <= 0 || BAYER[(y & 3) * 4 + (x & 3)] >= q) continue;
                p.px(x, y, [255, 255, 255]);
            }
            const tex = this.r.texture(p.canvas('col'), { repeat: true });
            layers.push({ tex, x: 0, y: 0, w: S, h: S });
        }
        return layers;
    }
    // ---------------------------------------------------------------- tiles
    tileLayers(tx, ty) {
        const W = this.world, i = ty * MW + tx;
        const g = W.ground[i], bn = BIOMES[W.biome[i]] || 'coast';
        const v = hash(tx, ty) % 4;
        const T = this.art.tiles;
        if (W.cliff[i]) {
            const f = W.face[i];
            const cl = (x, y) => W.inb(x, y) && W.cliff[y * MW + x];
            if (f >= 0) { const side = !cl(tx - 1, ty) ? 'l' : !cl(tx + 1, ty) ? 'r' : 'm'; return { base: T[`cliffFace:${bn}:${f}:${side}`], over: [], face: true }; }
            return { base: T[cl(tx, ty - 1) ? `cliffTop:${bn}` : `cliffTopN:${bn}`], over: [], top: true };
        }
        const base = g === G.WATER ? T[`seabed:${v}`] : T[`${g}:${bn}:${v}`] || T[`${G.GRASS}:${bn}:${v}`];
        const over = [];
        if (g !== G.BRIDGE && g !== G.PLANK) {
            const my = PRIO[g] || 0;
            const gat = (x, y) => (W.inb(x, y) && !W.cliff[y * MW + x]) ? W.ground[y * MW + x] : g;
            const cand = new Map();
            for (const [d, dx, dy] of DIR4) { const ng = gat(tx + dx, ty + dy); if ((PRIO[ng] || 0) > my && T[`e:${ng}:${bn}:${d}`]) { if (!cand.has(ng)) cand.set(ng, []); cand.get(ng).push(d); } }
            for (const [d, dx, dy, a, b] of DIAG) {
                const ng = gat(tx + dx, ty + dy);
                if ((PRIO[ng] || 0) <= my) continue;
                const has = cand.get(ng) || [];
                if (has.includes(a) || has.includes(b)) continue;
                if (!cand.has(ng)) cand.set(ng, []); cand.get(ng).push(d);
            }
            [...cand.keys()].sort((a, b) => (PRIO[a] || 0) - (PRIO[b] || 0)).forEach(ng => cand.get(ng).forEach(d => over.push(T[`e:${ng}:${bn}:${d}`])));
        }
        // ambient occlusion under cliff faces
        const ao = ty > 0 && W.cliff[(ty - 1) * MW + tx] && !W.cliff[i];
        return { base, over, ao };
    }
    eachTile(cam, fn) {
        const r = this.r;
        const x0 = Math.max(0, Math.floor(cam[0] / TS)), y0 = Math.max(0, Math.floor(cam[1] / TS));
        const x1 = Math.min(MW - 1, Math.floor((cam[0] + r.W) / TS)), y1 = Math.min(MH - 1, Math.floor((cam[1] + r.H) / TS));
        for (let ty = y0; ty <= y1; ty++) for (let tx = x0; tx <= x1; tx++) fn(tx, ty, this.cache[ty * MW + tx], tx * TS - cam[0], ty * TS - cam[1]);
        return [x0, y0, x1, y1];
    }
    // ---------------------------------------------------------------- scene
    // ents: [{ y, draw(r,cam), drawGlow(r,cam), shadow:[rect, x, y] }]
    drawScene(r, cam, ents, focus, hooks = {}) {
        this.time += 0; // advanced by the game
        const A = this.art;
        r.spriteBegin('alpha');
        // out-of-map: open sea seabed
        const [x0, y0, x1, y1] = this.eachTile(cam, (tx, ty, c, sx, sy) => {
            if (c.base) r.sprite(c.base, sx, sy);
            for (const o of c.over) r.sprite(o, sx, sy);
            if (c.ao) r.sprite(A.tiles.ao, sx, sy, { color: [1, 1, 1, 0.45] });
        });
        r.end();
        r.drawWater(this.water, cam);
        r.spriteBegin('alpha');
        // deco
        for (const d of this.decoInView(x0, y0, x1, y1)) { const rect = A.deco[d.k]; if (rect) r.sprite(rect, d.x - cam[0], d.y - cam[1]); }
        r.end();
        // shadows (multiplied)
        const props = this.propsInView(cam);
        r.spriteBegin('mul');
        const shade = [0.58, 0.6, 0.72, 1];
        for (const p of props) { const sh = p.def.shadow; if (!sh || !sh[0]) continue; const rect = this.shadowRect(sh[0]); r.sprite(rect, p.x - rect.w / 2 - cam[0] + (p.def.shadowDx || 0), p.y - rect.h / 2 - cam[1], { color: shade }); }
        for (const e of ents) if (e.shadow && !e.hidden) { const rect = this.shadowRect(e.shadow); r.sprite(rect, e.x - rect.w / 2 - cam[0], (e.shadowY ?? e.y) - rect.h / 2 - cam[1], { color: shade }); }
        r.end();
        hooks.under?.(r);
        // y-sorted entities
        const list = props.map(p => ({ y: p.y + (p.def.sortDy || 0), p })).concat(ents.filter(e => !e.hidden).map(e => ({ y: e.sortY ?? e.y, e })));
        list.sort((a, b) => a.y - b.y);
        this.sorted = list;
        r.spriteBegin('alpha');
        for (const it of list) { if (it.p) this.drawProp(r, it.p, cam, focus); else it.e.draw(r, cam); }
        // overhead: crowns and roofs
        for (const p of props) {
            if (!p.def.overheadRect) continue;
            const ox = p.x - (p.flip ? p.def.w - p.def.ax : p.def.ax) - cam[0], oy = p.y - p.def.ay - cam[1];
            let a = p.def.overheadAlpha ?? 1;
            if (focus && focus.x > p.x - p.def.w / 2 - 4 && focus.x < p.x + p.def.w / 2 + 4 && focus.y > p.y - p.def.h && focus.y < p.y + 6) a = Math.min(a, 0.35);
            r.sprite(p.def.overheadRect, ox, oy, { flip: p.flip, color: [1, 1, 1, a], sway: p.def.sway || 0, wx: p.x });
        }
        if (this.world.soulCrown && this.inView(this.world.soulCrown, cam, 200)) { const c = A.props.soulcrown; r.sprite(c.frames[0], this.world.soulCrown.x - c.ax - cam[0], this.world.soulCrown.y - c.ay + 30 - cam[1], { color: [1, 1, 1, 0.92] }); }
        r.end();
    }
    drawFog(r, cam, zoneKey, k = 1) {
        const [col, a] = FOG[ZONES[zoneKey]?.biome] || FOG.coast;
        r.spriteBegin('alpha');
        this.fog.forEach((f, i) => {
            const sp = i ? 6 : 3.5, par = i ? 1.15 : 1.05;
            const ox = ((cam[0] * par + this.time * sp) % 128 + 128) % 128, oy = ((cam[1] * par + this.time * sp * 0.4) % 128 + 128) % 128;
            const tex = f.tex;
            r.setTex(tex);
            // one big quad with repeating uv
            r.batch.quad(0, 0, r.W, r.H, ox / 128, oy / 128, (ox + r.W) / 128, (oy + r.H) / 128, col[0], col[1], col[2], a * k * (i ? 0.7 : 1));
        });
        r.end();
    }
    shadowRect(rx) { const U = this.art.ui; return rx <= 6 ? U.shadowS : rx <= 10 ? U.shadowM : rx <= 18 ? U.shadowL : rx <= 34 ? U.shadowXL : U.shadowXXL; }
    inView(pt, cam, m = 64) { return pt.x > cam[0] - m && pt.x < cam[0] + this.r.W + m && pt.y > cam[1] - m && pt.y < cam[1] + this.r.H + m; }
    propsInView(cam) {
        const out = [];
        const bx0 = Math.floor((cam[0] - 96) / 128), bx1 = Math.floor((cam[0] + this.r.W + 96) / 128);
        const by0 = Math.floor((cam[1] - 32) / 128), by1 = Math.floor((cam[1] + this.r.H + 160) / 128);
        for (let by = by0; by <= by1; by++) for (let bx = bx0; bx <= bx1; bx++) { const a = this.buckets.get(by * 1000 + bx); if (a) for (const p of a) if (!p.hidden) out.push(p); }
        return out;
    }
    decoInView(x0, y0, x1, y1) {
        if (!this.decoGrid) { this.decoGrid = new Map(); for (const d of this.world.deco) { const k = Math.floor(d.y / 128) * 1000 + Math.floor(d.x / 128); if (!this.decoGrid.has(k)) this.decoGrid.set(k, []); this.decoGrid.get(k).push(d); } }
        const out = [];
        for (let by = Math.floor(y0 / 8); by <= Math.floor(y1 / 8); by++) for (let bx = Math.floor(x0 / 8); bx <= Math.floor(x1 / 8); bx++) { const a = this.decoGrid.get(by * 1000 + bx); if (a) out.push(...a); }
        return out;
    }
    frameOf(p) {
        const d = p.def;
        if (d.frames.length > 1) return d.frames[Math.floor(this.time * (d.anim === 'waterfall' ? 12 : 8)) % d.frames.length];
        return d.frames[0];
    }
    drawProp(r, p, cam, focus, glow = false) {
        const d = p.def, rect = this.frameOf(p);
        const x = p.x - (p.flip ? d.w - d.ax : d.ax) - cam[0], y = p.y - d.ay - cam[1];
        if (glow) {
            if (rect.glow) r.sprite(rect, x, y, { tex: rect.gtex, flip: p.flip, sway: d.sway || 0, wx: p.x });
            else r.sprite(rect, x, y, { flip: p.flip, color: [0, 0, 0, 1], sway: d.sway || 0, wx: p.x });
        } else {
            let col;
            if (d.anim === 'fog') col = [0.8, 0.9, 0.9, p.open === false ? 0.8 : 0.42 + Math.sin(this.time * 2) * 0.08];
            r.sprite(rect, x, y, { flip: p.flip, sway: d.sway || 0, wx: p.x, outline: focus && focus.hover === p, color: col });
        }
        // attached animations
        const A = this.art.props, t = this.time;
        const fl = (k, dx, dy, speed = 10) => { const s = A[k]; const fr = s.frames[Math.floor(t * speed + p.x * 0.1) % s.frames.length]; const gx = p.x - s.ax + dx * (p.flip ? -1 : 1) - cam[0], gy = p.y - s.ay + dy - cam[1]; if (glow) r.sprite(fr, gx, gy, { tex: fr.gtex }); else r.sprite(fr, gx, gy); };
        if (d.anim === 'soulfire' && p.lit !== false) fl('soulflame', 0, -1);
        if (d.anim === 'fire') fl('fireflame', 0, -17);
        if (p.kind === 'forge') { fl('forgeflame', -8, -9, 12); fl('forgeflame', -2, -9, 9); }
        if (p.flag) { const s = A.flagcloth; const fr = s.frames[Math.floor(t * 6) % 4]; if (!glow) r.sprite(fr, p.x - d.ax + 3 - cam[0], p.y - d.ay + 1 - cam[1]); }
    }
    drawGlow(r, cam, ents) {
        r.spriteBegin('alpha');
        this.eachTile(cam, (tx, ty, c, sx, sy) => {
            if (c.base?.glow) r.sprite(c.base, sx, sy, { tex: c.base.gtex });
            for (const o of c.over) if (o.glow) r.sprite(o, sx, sy, { tex: o.gtex });
        });
        for (const d of this.decoInView(Math.floor(cam[0] / TS), Math.floor(cam[1] / TS), Math.floor((cam[0] + r.W) / TS), Math.floor((cam[1] + r.H) / TS))) { const rect = this.art.deco[d.k]; if (rect?.glow) r.sprite(rect, d.x - cam[0], d.y - cam[1], { tex: rect.gtex }); }
        for (const it of this.sorted || []) { if (it.p) this.drawProp(r, it.p, cam, null, true); else it.e.drawGlow?.(r, cam); }
        for (const p of this.propsInView(cam)) if (p.def.overheadRect?.glow) { const ox = p.x - (p.flip ? p.def.w - p.def.ax : p.def.ax) - cam[0], oy = p.y - p.def.ay - cam[1]; r.sprite(p.def.overheadRect, ox, oy, { tex: p.def.overheadRect.gtex, flip: p.flip, sway: p.def.sway || 0, wx: p.x }); }
        if (this.world.soulCrown && this.inView(this.world.soulCrown, cam, 200)) { const c = this.art.props.soulcrown; const k = 0.75 + 0.25 * Math.sin(this.time * Math.PI * 2 / 4); r.sprite(c.frames[0], this.world.soulCrown.x - c.ax - cam[0], this.world.soulCrown.y - c.ay + 30 - cam[1], { tex: c.frames[0].gtex, color: [k, k, k, 1] }); }
        r.end();
    }
    // dynamic lights in view (flickering / pulsing ones; static ones are baked)
    lights(cam, out) {
        const t = this.time;
        for (const L of this.world.lightsStatic) {
            if (!L.flicker && !L.pulse) continue;
            if (L.prop?.lit === false) continue;
            if (L.x < cam[0] - L.r || L.x > cam[0] + this.r.W + L.r || L.y < cam[1] - L.r || L.y > cam[1] + this.r.H + L.r) continue;
            let k = 1;
            if (L.flicker) k *= 1 + NOISE.n1(t * 6 + L.x * 0.01) * L.flicker * 1.5;
            if (L.pulse) k *= 0.7 + 0.3 * Math.sin(t * Math.PI * 2 / L.pulse);
            out.push({ x: L.x, y: L.y, r: L.r * (0.96 + 0.04 * k), col: L.col, i: L.i * k });
        }
        return out;
    }
}
export { P };
