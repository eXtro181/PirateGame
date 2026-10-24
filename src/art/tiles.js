// Ground tiles (16×16) for every biome, edge overlays for organic transitions (autotiling),
// basalt-column cliffs with grass lips, and ground decoration.
import { Pix, BAYER } from './pix.js';
import { P, rgb, ramp, shade, mix } from './pal.js';
import { Noise } from '../gfx/noise.js';

const N = new Noise(91);
export const TS = 16;

// ground materials; prio decides who overlaps whom at borders
export const G = {
    WATER: 0, SAND: 1, GRASS: 2, DIRT: 3, STONE: 4, FLOOR: 5, PLANK: 6, CLIFF: 7, HEX: 8, ASH: 9, SNOW: 10, ICE: 11, MUD: 12, BRIDGE: 13
};
export const PRIO = { [G.SAND]: 1, [G.MUD]: 2, [G.ASH]: 3, [G.DIRT]: 4, [G.ICE]: 5, [G.GRASS]: 6, [G.SNOW]: 7, [G.HEX]: 8, [G.STONE]: 9, [G.FLOOR]: 10 };
export const WALKABLE = new Set([G.SAND, G.GRASS, G.DIRT, G.STONE, G.FLOOR, G.PLANK, G.HEX, G.ASH, G.SNOW, G.ICE, G.MUD, G.BRIDGE]);

// biome tints for grass/cliff tops
export const BIOME = {
    coast: { grass: P.swamp, grassLight: P.pale, grassDark: P.mossDark, cliff: P.basalt, lip: P.pale },
    hub: { grass: '#2F4436', grassLight: '#4E6A52', grassDark: '#1A2820', cliff: P.basalt, lip: '#4E6A52' },
    fire: { grass: '#3E2E2C', grassLight: '#5A4440', grassDark: '#241A1A', cliff: '#2A2426', lip: P.ember },
    ice: { grass: P.snow, grassLight: '#E2ECEE', grassDark: '#8EA6B0', cliff: '#3E4E5A', lip: '#E2ECEE' },
    wind: { grass: '#3A4640', grassLight: '#566458', grassDark: '#222A26', cliff: '#41464C', lip: '#566458' },
    earth: { grass: '#2E3E2A', grassLight: '#4A5E3A', grassDark: '#1A2418', cliff: '#33382E', lip: '#4A5E3A' },
    end: { grass: '#2A1E26', grassLight: '#4A2E3A', grassDark: '#160E14', cliff: '#231C22', lip: P.blood }
};

const h01 = (x, y, s = 0) => { const v = Math.sin(x * 127.1 + y * 311.7 + s * 74.7) * 43758.5453; return v - Math.floor(v); };

function noiseFill(p, base, light, dark, seed, scale = 0.25, amt = 1) {
    const RB = ramp(base);
    for (let y = 0; y < TS; y++) for (let x = 0; x < TS; x++) {
        const n = N.fbm((x + seed * 16) * scale, (y + seed * 7) * scale, 3, 0) * amt;
        let v = 2 + n * 2.2 + (BAYER[(y & 3) * 4 + (x & 3)] - 0.5) * 0.8;
        p.px(x, y, RB[Math.max(0, Math.min(4, Math.round(v)))]);
    }
    void light; void dark;
}

// ---------------------------------------------------------------- base tiles
function grass(p, v, b) {
    noiseFill(p, b.grass, b.grassLight, b.grassDark, v, 0.22, 0.8);
    const L = rgb(b.grassLight), D = rgb(b.grassDark);
    for (let i = 0; i < 9; i++) { // blades: 1×2 highlight over a dark root
        const x = Math.floor(h01(i, v, 1) * 15), y = Math.floor(h01(i, v, 2) * 14) + 1;
        p.px(x, y + 1, D); p.px(x, y, L);
        if (h01(i, v, 3) > 0.6) p.px(x + 1, y - 1, L);
    }
}
function sand(p, v, wet = false) {
    noiseFill(p, wet ? P.sandDark : P.sand, null, null, v + 3, 0.3, 0.6);
    const R = ramp(wet ? P.sandDark : P.sand);
    for (let y = 2; y < TS; y += 5) for (let x = 0; x < TS; x++) if (Math.sin((x + v * 5) * 0.7 + y) > 0.55) p.px(x, y + Math.round(Math.sin(x * 0.5 + v)), R[1]);
    if (!wet) for (let i = 0; i < 3; i++) p.px(Math.floor(h01(i, v, 4) * 16), Math.floor(h01(i, v, 5) * 16), R[4]);
}
function dirt(p, v) { // wet, glossy path
    noiseFill(p, P.tarwood, null, null, v + 9, 0.3, 1);
    const R = ramp(P.tarwood);
    for (let i = 0; i < 4; i++) { const x = Math.floor(h01(i, v, 6) * 14), y = Math.floor(h01(i, v, 7) * 14); p.px(x, y, R[0]); p.px(x + 1, y, R[1]); }
    for (let i = 0; i < 2; i++) { const x = Math.floor(h01(i, v, 8) * 15), y = Math.floor(h01(i, v, 9) * 15); p.px(x, y, rgb(P.oldwood)); } // wet glints
    if (v === 3) { // small puddle
        const cx = 4 + (v * 5) % 8, cy = 5 + (v * 3) % 7;
        p.ellipse(cx, cy, 3, 1.6, P.waterMid); p.px(cx - 1, cy - 1, P.glint);
    }
}
function cobble(p, v, base = P.slate) {
    p.rect(0, 0, TS, TS, shade(rgb(base), 0.55));
    const R = ramp(base);
    const stones = [[1, 1, 6, 4], [8, 0, 7, 5], [0, 6, 5, 4], [6, 6, 5, 4], [12, 6, 4, 4], [1, 11, 6, 4], [8, 11, 7, 4]];
    stones.forEach(([x, y, w, h], i) => {
        const k = 1 + Math.floor(h01(i, v, 11) * 3);
        p.rect(x, y, w, h, R[k]);
        p.hline(x, x + w - 1, y, R[Math.min(4, k + 1)]);
        p.hline(x, x + w - 1, y + h - 1, R[Math.max(0, k - 1)]);
        if (h01(i, v, 12) > 0.7) p.px(x + 1, y + 1, P.mistBlue); // wet glint
    });
}
function floor(p, v, base = P.basalt) { // irregular flagstones, mossy joints
    const R = ramp(base, 0.7);
    p.rect(0, 0, TS, TS, R[0]);
    const layouts = [[[0, 0, 9, 7], [9, 0, 7, 9], [0, 7, 5, 9], [5, 7, 4, 9], [9, 9, 7, 7]], [[0, 0, 6, 8], [6, 0, 10, 6], [6, 6, 10, 10], [0, 8, 6, 8]], [[0, 0, 16, 5], [0, 5, 8, 11], [8, 5, 8, 6], [8, 11, 8, 5]], [[0, 0, 7, 6], [7, 0, 9, 8], [0, 6, 7, 10], [7, 8, 9, 8]]][v % 4];
    layouts.forEach(([x, y, w, h], i) => {
        const k = 1 + Math.floor(h01(i, v, 31) * 2.2);
        for (let yy = y + 1; yy < y + h; yy++) for (let xx = x + 1; xx < x + w; xx++) {
            let kk = k;
            if (yy === y + 1) kk = Math.min(4, k + 1);
            if (xx === x + w - 1 || yy === y + h - 1) kk = Math.max(1, k - 1);
            if (h01(xx, yy, v + i) > 0.94) kk = Math.max(0, kk - 1);
            p.px(xx, yy, R[kk]);
        }
    });
    for (let y = 0; y < TS; y++) for (let x = 0; x < TS; x++) { const c = p.get(x, y); if (c[0] === R[0][0] && c[1] === R[0][1] && h01(x, y, v) > 0.55) p.px(x, y, P.mossDark); }
}
function plank(p, v) {
    const R = ramp(P.tarwood);
    for (let i = 0; i < 4; i++) {
        const y = i * 4;
        p.rect(0, y, 16, 3, R[2 + (h01(i, v, 13) > 0.5 ? 0 : -1) + 1 - 1]);
        p.hline(0, 15, y, R[3]);
        p.hline(0, 15, y + 3, R[0]);
        const seam = Math.floor(h01(i, v, 14) * 14) + 1;
        p.px(seam, y + 1, R[0]); p.px(seam, y + 2, R[0]);
        if (h01(i, v, 15) > 0.5) p.px(seam + 2, y + 1, P.mist); // nail
    }
}
function hex(p, v) { // basalt hexagon tops with blood-red veins (a few glowing)
    const R = ramp('#2E2A2E');
    const pts = [];
    for (let j = -1; j <= 3; j++) for (let i = -1; i <= 3; i++) pts.push([i * 8 + (j % 2 ? 4 : 0) + (j < 0 ? 4 : 0), j * 7]);
    for (let y = 0; y < TS; y++) for (let x = 0; x < TS; x++) {
        let d1 = 99, d2 = 99, best = null;
        for (const q of pts) { const d = Math.hypot(x + 0.5 - q[0], (y + 0.5 - q[1]) * 1.15); if (d < d1) { d2 = d1; d1 = d; best = q; } else if (d < d2) d2 = d; }
        if (d2 - d1 < 1.1) { p.px(x, y, '#3A1418'); if (h01(best[0] + x, best[1] + y, v) > 0.88) { p.px(x, y, P.hexVein); p.glow(x, y, P.ember, 0.35); } }
        else { const ly = (y + 0.5 - best[1]); p.px(x, y, R[ly < -2 ? 3 : (x + 0.5 - best[0]) > 2 ? 1 : 2]); }
    }
}
function ash(p, v) {
    noiseFill(p, P.ash, null, null, v + 21, 0.35, 0.8);
    for (let i = 0; i < 3; i++) { const x = Math.floor(h01(i, v, 16) * 15), y = Math.floor(h01(i, v, 17) * 15); p.px(x, y, P.ember); if (i === 0) p.glow(x, y, P.ember, 0.4); }
}
function snow(p, v) {
    noiseFill(p, P.snow, null, null, v + 31, 0.25, 0.6);
    for (let i = 0; i < 4; i++) p.px(Math.floor(h01(i, v, 18) * 16), Math.floor(h01(i, v, 19) * 16), '#FFFFFF');
    for (let i = 0; i < 2; i++) p.px(Math.floor(h01(i, v, 20) * 16), Math.floor(h01(i, v, 21) * 16), P.iceDark);
}
function ice(p, v) {
    noiseFill(p, P.ice, null, null, v + 41, 0.2, 0.5);
    const D = rgb(P.iceDark);
    let x = 2 + v % 6, y = 0;
    while (y < 16) { p.px(x, y, D); y++; if (h01(x, y, v) > 0.6) x += h01(y, x, v) > 0.5 ? 1 : -1; }
    p.px(10, 4, '#FFFFFF'); p.px(11, 4, '#FFFFFF'); p.px(5, 11, '#E8F4F8');
}
function mud(p, v) {
    noiseFill(p, '#2E3226', null, null, v + 51, 0.3, 1);
    for (let i = 0; i < 2; i++) { const cx = 3 + h01(i, v, 22) * 10, cy = 3 + h01(i, v, 23) * 10; p.ellipse(cx, cy, 2.5, 1.4, '#1A2A2A'); p.px(cx - 1, cy - 1, P.kelp); }
}
function seabed(p, v) { noiseFill(p, '#2E3A44', null, null, v + 61, 0.3, 0.8); for (let i = 0; i < 2; i++) p.px(Math.floor(h01(i, v, 24) * 16), Math.floor(h01(i, v, 25) * 16), P.mistBlue); }

export const VARIANTS = 4;
const PAINT = {
    [G.GRASS]: grass, [G.SAND]: (p, v) => sand(p, v), [G.DIRT]: dirt, [G.STONE]: (p, v) => cobble(p, v), [G.FLOOR]: (p, v) => floor(p, v),
    [G.PLANK]: plank, [G.BRIDGE]: plank, [G.HEX]: hex, [G.ASH]: ash, [G.SNOW]: snow, [G.ICE]: ice, [G.MUD]: mud
};

// edge overlay masks: a noisy fringe 3–5 px deep along one side, or a corner
function edgeMask(dir, seed) {
    const m = new Uint8Array(TS * TS);
    for (let y = 0; y < TS; y++) for (let x = 0; x < TS; x++) {
        let d;
        if (dir === 'n') d = y; else if (dir === 's') d = 15 - y; else if (dir === 'w') d = x; else if (dir === 'e') d = 15 - x;
        else {
            const cx = dir.includes('w') ? 0 : 15, cy = dir.includes('n') ? 0 : 15;
            d = Math.hypot(x - cx, y - cy) - 1;
        }
        const along = dir === 'n' || dir === 's' ? x : y;
        const depth = 4.5 + N.n2(along * 0.35 + seed * 3, seed) * 3.5 + (BAYER[(y & 3) * 4 + (x & 3)] - 0.5) * 1.6;
        m[y * TS + x] = d < depth ? 1 : 0;
    }
    return m;
}
export const DIRS = ['n', 'e', 's', 'w', 'ne', 'nw', 'se', 'sw'];

// biome-specific base tile cache keys: `${g}:${biome}:${v}` ; overlays: `e:${g}:${biome}:${dir}`
export function buildTiles(atlas) {
    const T = {};
    for (const [bname, b] of Object.entries(BIOME)) {
        for (const [g, paint] of Object.entries(PAINT)) {
            if (+g !== G.GRASS && bname !== 'coast' && ![G.STONE, G.FLOOR].includes(+g)) {
                // only grass, cobble and floor differ per biome; others are shared
                for (let v = 0; v < VARIANTS; v++) T[`${g}:${bname}:${v}`] = T[`${g}:coast:${v}`];
                continue;
            }
            for (let v = 0; v < VARIANTS; v++) {
                const p = new Pix(TS, TS);
                if (+g === G.GRASS) paint(p, v, b);
                else if (+g === G.STONE) cobble(p, v, bname === 'ice' ? '#5A6A74' : bname === 'fire' ? '#3A3036' : P.slate);
                else if (+g === G.FLOOR) floor(p, v, bname === 'ice' ? '#55646E' : bname === 'end' ? '#2E2228' : bname === 'fire' ? '#352E30' : P.basalt);
                else paint(p, v, b);
                T[`${g}:${bname}:${v}`] = atlas.add(`t:${g}:${bname}:${v}`, p);
            }
        }
        // transition overlays (sand-ish and grass are the ones that spread)
        for (const g of [G.GRASS, G.SAND, G.SNOW, G.DIRT, G.ASH, G.MUD, G.STONE, G.FLOOR, G.HEX, G.ICE]) {
            if (g !== G.GRASS && bname !== 'coast' && ![G.STONE, G.FLOOR].includes(g)) { for (const d of DIRS) T[`e:${g}:${bname}:${d}`] = T[`e:${g}:coast:${d}`]; continue; }
            const base = T[`${g}:${bname}:0`];
            const src = atlas.img;
            DIRS.forEach((d, k) => {
                const m = edgeMask(d, k + g * 5);
                const p = new Pix(TS, TS);
                for (let y = 0; y < TS; y++) for (let x = 0; x < TS; x++) if (m[y * TS + x]) {
                    const i = ((base.y + y) * src.w + base.x + x) * 4;
                    p.px(x, y, [src.col[i], src.col[i + 1], src.col[i + 2]]);
                    if (src.glw[i + 3]) p.glow(x, y, [src.glw[i], src.glw[i + 1], src.glw[i + 2]]);
                }
                // darker rim on the fringe for depth
                for (let y = 0; y < TS; y++) for (let x = 0; x < TS; x++) if (m[y * TS + x]) {
                    const nb = [[1, 0], [-1, 0], [0, 1], [0, -1]].some(([dx, dy]) => { const xx = x + dx, yy = y + dy; return xx >= 0 && yy >= 0 && xx < 16 && yy < 16 && !m[yy * TS + xx]; });
                    if (nb) { const c = p.get(x, y); p.px(x, y, shade(c, 0.78)); }
                }
                T[`e:${g}:${bname}:${d}`] = atlas.add(`e:${g}:${bname}:${d}`, p);
            });
        }
        // cliff: top surface, top lip and face rows
        T[`cliffTop:${bname}`] = atlas.add(`ct:${bname}`, cliffTop(b, false));
        T[`cliffTopN:${bname}`] = atlas.add(`ctn:${bname}`, cliffTop(b, true));
        for (let r = 0; r < 3; r++) for (const side of ['m', 'l', 'r']) T[`cliffFace:${bname}:${r}:${side}`] = atlas.add(`cf:${bname}:${r}:${side}`, cliffFace(b, r, side));
    }
    for (let v = 0; v < VARIANTS; v++) { const p = new Pix(TS, TS); seabed(p, v); T[`seabed:${v}`] = atlas.add(`seabed:${v}`, p); }
    { const p = new Pix(TS, TS); p.rect(0, 0, 16, 4, [0, 0, 0]); for (let y = 4; y < 8; y++) for (let x = 0; x < 16; x++) if (BAYER[(y & 3) * 4 + (x & 3)] < (8 - y) / 4) p.px(x, y, [0, 0, 0]); T.ao = atlas.add('ao', p); }
    return T;
}

function cliffTop(b, northEdge) {
    const p = new Pix(TS, TS);
    grass(p, 7, { grass: b.grass, grassLight: b.grassLight, grassDark: b.grassDark });
    if (northEdge) { p.hline(0, 15, 0, b.grassDark); p.hline(0, 15, 1, b.lip); }
    return p;
}
// basalt columns: vertical hexagonal columns, light from the top-left; r = row inside the face (0 top)
function cliffFace(b, r, side) {
    const p = new Pix(TS, TS);
    const R = ramp(b.cliff);
    for (let x = 0; x < TS; x++) {
        const col = Math.floor((x + 2) / 4), cx = (x + 2) % 4;
        const k = cx === 0 ? 0 : cx === 1 ? 3 : cx === 2 ? 2 : 1;
        for (let y = 0; y < TS; y++) {
            let kk = k;
            if ((y + col * 5 + r * 16) % 23 === 0) kk = 0; // column joints
            if (r === 0 && y < 3) kk = Math.min(4, kk + 1);
            p.px(x, y, R[kk]);
        }
    }
    if (r === 0) { // grass lip hanging over the edge
        const L = rgb(b.lip), D = rgb(b.grassDark);
        for (let x = 0; x < TS; x++) { p.px(x, 0, L); if (h01(x, r, 3) > 0.45) p.px(x, 1, L); if (h01(x, r, 4) > 0.75) p.px(x, 2, D); }
    }
    if (side === 'l') for (let y = 0; y < TS; y++) { p.px(0, y, R[4]); p.px(1, y, R[3]); }
    if (side === 'r') for (let y = 0; y < TS; y++) { p.px(15, y, R[0]); p.px(14, y, R[0]); }
    if (r === 2) for (let x = 0; x < TS; x++) if (h01(x, 9, 2) > 0.5) p.px(x, 15, P.mossDark);
    return p;
}

// ---------------------------------------------------------------- ground decoration (no collision)
export function buildDeco(atlas) {
    const D = {};
    const add = (n, w, h, fn) => { const p = new Pix(w, h); fn(p); D[n] = atlas.add('d:' + n, p); };
    add('boneshroom', 4, 4, p => { p.px(1, 0, P.bone); p.px(2, 0, P.bone); p.px(0, 1, P.bone); p.px(3, 1, '#B8AE98'); p.px(1, 2, '#8F8672'); p.px(1, 3, '#6F6858'); });
    add('boneshroom2', 6, 5, p => { p.rect(1, 0, 3, 1, P.bone); p.rect(0, 1, 5, 1, '#B8AE98'); p.px(2, 2, '#8F8672'); p.px(2, 3, '#6F6858'); p.px(5, 3, P.bone); p.px(5, 4, '#8F8672'); });
    add('pebbles', 6, 3, p => { p.px(0, 1, '#5A5E64'); p.px(1, 1, '#7A7E84'); p.px(3, 2, '#4A4E54'); p.px(4, 0, '#6A6E74'); p.px(5, 0, '#8A8E94'); });
    add('shell', 4, 3, p => { p.rect(0, 1, 4, 2, '#B8A898'); p.px(1, 0, '#D8CFB8'); p.px(2, 0, '#D8CFB8'); p.px(1, 2, '#8A7A6A'); });
    add('bones', 8, 4, p => { p.hline(1, 6, 2, P.bone); p.px(0, 1, P.bone); p.px(0, 3, P.bone); p.px(7, 1, P.bone); p.px(7, 3, P.bone); p.px(3, 1, '#B8AE98'); });
    add('skull', 6, 5, p => { p.blob(3, 2, 3, 2.3, P.bone, { dither: false }); p.px(2, 2, P.abyss); p.px(4, 2, P.abyss); p.hline(2, 4, 4, '#8F8672'); });
    add('ember', 2, 2, p => { p.lit(0, 0, P.ember, 0.8); p.lit(1, 1, P.lava, 0.6); });
    add('crack', 10, 6, p => { p.line(0, 5, 4, 2, P.hexVein); p.line(4, 2, 9, 0, P.hexVein); for (const [x, y] of [[2, 3], [5, 2], [7, 1]]) p.glow(x, y, P.ember, 0.7); });
    add('snowtuft', 6, 3, p => { p.hline(0, 5, 2, '#B8C8CC'); p.hline(1, 4, 1, P.snow); p.px(2, 0, '#FFFFFF'); });
    add('lily', 7, 4, p => { p.ellipse(3.5, 2, 3.5, 2, P.kelp); p.px(3, 1, P.leafLight); p.px(5, 2, '#1A3A3A'); });
    add('reeds', 7, 9, p => { for (const [x, h] of [[1, 7], [3, 8], [5, 6]]) { p.vline(x, 8 - h, 8, '#3A4A2E'); p.px(x, 8 - h, '#6A5A3A'); } });
    add('puddle', 10, 4, p => { p.ellipse(5, 2, 5, 2, P.waterMid); p.hline(3, 5, 1, P.glint); });
    add('coralbud', 6, 5, p => { p.px(1, 4, P.coral); p.px(1, 3, P.violet); p.px(1, 2, P.violet); p.px(3, 4, P.coral); p.px(3, 1, P.violet); p.px(3, 2, P.violet); p.px(3, 3, P.coral); p.px(5, 3, P.violet); p.px(5, 4, P.coral); });
    add('soulspark', 3, 3, p => { p.lit(1, 1, P.soulLight, 1); p.lit(0, 1, P.soul, 0.6); p.lit(2, 1, P.soul, 0.6); p.lit(1, 0, P.soul, 0.6); p.lit(1, 2, P.soul, 0.6); });
    add('rope', 12, 3, p => { for (let x = 0; x < 12; x++) p.px(x, 1 + Math.round(Math.sin(x * 0.8)), x % 2 ? '#8A7A5A' : '#6A5A3A'); });
    add('plankbits', 10, 5, p => { p.rect(0, 2, 6, 2, P.tarwood); p.hline(0, 5, 2, P.oldwood); p.rect(6, 0, 3, 2, P.deadwood); });
    return D;
}
export { mix };
