// Ground tiles (16×16) in the style of the reference shots: lush grass with blade clumps and tiny
// flowers, warm paths, flagstones, basalt hex pavement, organic edge overlays with grass overhang,
// tall stone-column cliffs (3 rows) with grass lips and hanging vines, stairs, and ground deco.
// Every biome has its own explicit palette: hub and start coast bright, the king lands darker.
import { Pix, BAYER } from './pix.js';
import { rgb, shade, mix } from './pal.js';
import { Noise } from '../gfx/noise.js';

const N = new Noise(91);
export const TS = 16;
export const G = { WATER: 0, SAND: 1, GRASS: 2, DIRT: 3, STONE: 4, FLOOR: 5, PLANK: 6, CLIFF: 7, HEX: 8, ASH: 9, SNOW: 10, ICE: 11, MUD: 12, BRIDGE: 13, MEADOW: 14 };
export const PRIO = { [G.SAND]: 1, [G.MUD]: 2, [G.ASH]: 3, [G.DIRT]: 4, [G.ICE]: 5, [G.GRASS]: 6, [G.MEADOW]: 7, [G.SNOW]: 7, [G.HEX]: 8, [G.STONE]: 9, [G.FLOOR]: 10 };
export const WALKABLE = new Set([G.SAND, G.GRASS, G.DIRT, G.STONE, G.FLOOR, G.PLANK, G.HEX, G.ASH, G.SNOW, G.ICE, G.MUD, G.BRIDGE, G.MEADOW]);

// palettes: 5-step ramps from dark to light, picked by hand to match the reference look
const R = (...h) => h.map(rgb);
export const BIOME = {
    coast: { grass: R('#1F4A2E', '#2E6E3A', '#4A9A44', '#6EBE52', '#9AD86A'), flowers: ['#F4F0E0', '#F2D25A', '#E890B0'], path: R('#7A5E3E', '#9A7A50', '#B89A68', '#CDB27E', '#E2CC98'), sand: R('#8A7656', '#B09A70', '#CDB88A', '#E0CCA0', '#F0E2BC'), stone: R('#4A4A52', '#6E6E74', '#9A9690', '#BDB6A8', '#DCD4C0'), cliff: R('#2A3038', '#424A54', '#5E6870', '#7E8A90', '#A2ACAE'), lip: '#3E8A3E', vine: '#2E6E3A', bright: true },
    hub: { grass: R('#1E4A30', '#2C6E3E', '#46A04A', '#6CC458', '#9CDC70'), flowers: ['#F8F4E8', '#F4E07A', '#B8E8F0'], path: R('#7E623E', '#A08052', '#C0A06C', '#D4B886', '#E8D2A2'), sand: R('#8A7656', '#B09A70', '#CDB88A', '#E0CCA0', '#F0E2BC'), stone: R('#6E6658', '#9A9080', '#C4B8A0', '#DCD2BA', '#F2EAD6'), cliff: R('#2A3038', '#444C56', '#606A72', '#808C92', '#A6B0B2'), lip: '#3E8E42', vine: '#2C6E3E', bright: true },
    wind: { grass: R('#1E3A2E', '#2A5238', '#3E7044', '#58904E', '#7AAC62'), flowers: ['#E8E4D8', '#C8D070'], path: R('#5E5244', '#7A6A56', '#968470', '#AE9C86', '#C6B6A0'), sand: R('#6E6656', '#8A806C', '#A69A84', '#BEB29A', '#D4CAB2'), stone: R('#3A4046', '#525A60', '#6E767A', '#8C9294', '#AEB2B0'), cliff: R('#22282E', '#363E46', '#4E565E', '#68727A', '#8A9298'), lip: '#3E6E44', vine: '#2A5238' },
    ice: { grass: R('#5E7480', '#82989E', '#A8BCC0', '#C8D8DA', '#E6F0F0'), flowers: ['#FFFFFF', '#B8E8F8'], path: R('#4E6070', '#687C8A', '#8698A4', '#A4B4BC', '#C2D0D6'), sand: R('#6A7A84', '#8696A0', '#A2B0B8', '#BCC8CE', '#D6DEE2'), stone: R('#3E4E5A', '#566874', '#70828C', '#8C9CA4', '#AAB8BE'), cliff: R('#1E2A36', '#2E3E4C', '#445664', '#5E727E', '#7E9098'), lip: '#C8D8DA', vine: '#7A9098' },
    earth: { grass: R('#122A1A', '#1C3E24', '#2A5A2E', '#3E7A38', '#5A9A44'), flowers: ['#E05A4A', '#E8C84A'], path: R('#3A3024', '#4E4232', '#665640', '#7E6C50', '#968462'), sand: R('#4A4434', '#5E5642', '#766C54', '#8E8466', '#A69C7C'), stone: R('#2E3430', '#424A44', '#5A625A', '#747C72', '#90988C'), cliff: R('#1A201C', '#283028', '#3A443A', '#4E5A4C', '#687462'), lip: '#2E5A2E', vine: '#1C3E24' },
    fire: { grass: R('#3A1E26', '#542A32', '#704040', '#8C5650', '#A87064'), flowers: ['#F05A4A', '#F4A060'], path: R('#5A4A4A', '#766262', '#927C78', '#AC9690', '#C4AEA6'), sand: R('#5A4A48', '#74605C', '#8E7A74', '#A8948C', '#C0ACA2'), stone: R('#2E2A30', '#45404A', '#5E5862', '#78727A', '#948E94'), cliff: R('#1E1A20', '#2E2A30', '#433E46', '#5A545C', '#767078'), lip: '#8C4A4A', vine: '#542A32' },
    end: { grass: R('#1A1016', '#2A1A22', '#3E2632', '#563444', '#704456'), flowers: ['#C2502E', '#5FF2C8'], path: R('#2A2228', '#3C3238', '#50444A', '#66585E', '#7E7076'), sand: R('#2E262A', '#40363A', '#54484C', '#6A5C60', '#807276'), stone: R('#221C22', '#342C34', '#483E48', '#5E525E', '#766874'), cliff: R('#140E14', '#221A22', '#342834', '#483848', '#5E4C5E'), lip: '#5A2A3A', vine: '#2A1A22' }
};
// water look per biome (shader uniforms)
export const WATER = {
    coast: { shallow: '#4EC4D0', deep: '#1E6E9E', glint: '#D2F6F8', foam: '#E8FAFA', soul: '#3E8EBE' },
    hub: { shallow: '#58CCD8', deep: '#2280A8', glint: '#DAF8FA', foam: '#F0FCFC', soul: '#B8FFE8' },
    wind: { shallow: '#3E8E9E', deep: '#18486A', glint: '#A8D8E0', foam: '#C8E4EA', soul: '#FFFFFF' },
    ice: { shallow: '#6EA8C0', deep: '#24506E', glint: '#E0F6FF', foam: '#F0FAFF', soul: '#B8FFE8' },
    earth: { shallow: '#2E5A4E', deep: '#12302E', glint: '#7AB0A0', foam: '#9AC0B0', soul: '#5FF2C8' },
    fire: { shallow: '#6A3A50', deep: '#2A1426', glint: '#E07A8A', foam: '#E8A0A8', soul: '#F05A4A' },
    end: { shallow: '#3A1A2A', deep: '#120810', glint: '#B04050', foam: '#C06070', soul: '#5FF2C8' },
    sea: { shallow: '#3EA6C0', deep: '#14466E', glint: '#C8EEF6', foam: '#E0F6FA', soul: '#2E6E9E' }
};

const h01 = (x, y, s = 0) => { const v = Math.sin(x * 127.1 + y * 311.7 + s * 74.7) * 43758.5453; return v - Math.floor(v); };
const clampI = (v, a, b) => Math.max(a, Math.min(b, v | 0));
function field(p, Rm, seed, scale = 0.2, amt = 1, base = 2) {
    for (let y = 0; y < TS; y++) for (let x = 0; x < TS; x++) {
        const n = N.fbm((x + seed * 16) * scale, (y + seed * 9) * scale, 3, 16 * scale, 16 * scale) * amt;
        p.px(x, y, Rm[clampI(Math.round(base + n * 2.2 + (BAYER[(y & 3) * 4 + (x & 3)] - 0.5) * 0.7), 0, 4)]);
    }
}
function grass(p, v, b, meadow = false) {
    const Gr = b.grass;
    field(p, Gr, v + 1, 0.18, 0.7, 2);
    for (let i = 0; i < 6; i++) {
        const cx = Math.floor(h01(i, v, 1) * 16), cy = Math.floor(h01(i, v, 2) * 14) + 2;
        for (let k = -2; k <= 2; k++) {
            const x = (cx + k + 16) % 16, ht = 2 + ((k + i + v) & 1) + (k === 0 ? 1 : 0);
            p.px(x, cy, Gr[1]);
            for (let j = 1; j <= ht; j++) p.px(x + (k < 0 && j > 1 ? -1 : 0) + (k > 0 && j > 2 ? 1 : 0), cy - j, j === ht ? Gr[4] : Gr[3]);
        }
    }
    const fl = meadow ? 5 : (v === 2 ? 3 : 0);
    for (let i = 0; i < fl; i++) {
        const x = Math.floor(h01(i, v, 7) * 14) + 1, y = Math.floor(h01(i, v, 8) * 14) + 1, c = rgb(b.flowers[i % b.flowers.length]);
        p.px(x, y, c); p.px(x + 1, y, shade(c, 0.85)); p.px(x, y + 1, shade(c, 0.75)); p.px(x + 1, y - 1, c);
    }
}
function path(p, v, b) {
    field(p, b.path, v + 11, 0.25, 0.6, 2);
    const Pm = b.path;
    for (let i = 0; i < 5; i++) { const x = Math.floor(h01(i, v, 3) * 15), y = Math.floor(h01(i, v, 4) * 15); p.px(x, y, Pm[1]); if (i % 2) p.px(x + 1, y, Pm[3]); }
    for (let i = 0; i < 3; i++) { const x = Math.floor(h01(i, v, 5) * 15), y = Math.floor(h01(i, v, 6) * 15); p.px(x, y, Pm[4]); }
}
function sand(p, v, b) {
    field(p, b.sand, v + 3, 0.3, 0.5, 2);
    const S = b.sand;
    for (let y = 2; y < TS; y += 5) for (let x = 0; x < TS; x++) if (Math.sin((x + v * 5) * 0.7 + y) > 0.6) p.px(x, y + Math.round(Math.sin(x * 0.5 + v)), S[1]);
    for (let i = 0; i < 3; i++) p.px(Math.floor(h01(i, v, 4) * 16), Math.floor(h01(i, v, 5) * 16), S[4]);
}
function cobble(p, v, b) {
    const S = b.stone;
    p.rect(0, 0, TS, TS, S[0]);
    const off = (v % 2) * 3;
    for (let row = 0; row < 4; row++) for (let col = -1; col < 4; col++) {
        const x0 = col * 5 + (row % 2 ? 2 : 0) + off, y0 = row * 4, w = 4, h = 3;
        const k = 1 + Math.floor(h01(row, col, v) * 2.5);
        for (let y = 0; y < h; y++) for (let x = 0; x < w; x++) {
            if ((x === 0 || x === w - 1) && (y === 0 || y === h - 1)) continue;
            p.px((x0 + x + 16) % 16, y0 + y, S[y === 0 ? Math.min(4, k + 1) : y === h - 1 ? k - 1 : k]);
        }
    }
}
function flag(p, v, b) {
    const S = b.stone;
    p.rect(0, 0, TS, TS, S[0]);
    const layouts = [[[0, 0, 9, 7], [9, 0, 7, 9], [0, 7, 5, 9], [5, 7, 4, 9], [9, 9, 7, 7]], [[0, 0, 6, 8], [6, 0, 10, 6], [6, 6, 10, 10], [0, 8, 6, 8]], [[0, 0, 16, 5], [0, 5, 8, 11], [8, 5, 8, 6], [8, 11, 8, 5]], [[0, 0, 7, 6], [7, 0, 9, 8], [0, 6, 7, 10], [7, 8, 9, 8]]][v % 4];
    layouts.forEach(([x, y, w, h], i) => {
        const k = 2 + (h01(i, v, 31) > 0.6 ? 1 : 0);
        for (let yy = y + 1; yy < y + h; yy++) for (let xx = x + 1; xx < x + w; xx++) {
            let kk = k;
            if (yy === y + 1 || xx === x + 1) kk = Math.min(4, k + 1);
            if (xx === x + w - 1 || yy === y + h - 1) kk = k - 1;
            if (h01(xx, yy, v + i) > 0.95) kk = Math.max(1, kk - 1);
            p.px(xx, yy, S[kk]);
        }
    });
}
function plank(p, v) {
    const W = R('#3A2618', '#5A3C24', '#7A5434', '#98704A', '#B48C62');
    for (let i = 0; i < 4; i++) {
        const y = i * 4;
        p.rect(0, y, 16, 3, W[2 + ((i + v) % 2)]);
        p.hline(0, 15, y, W[4]); p.hline(0, 15, y + 3, W[0]);
        const seam = Math.floor(h01(i, v, 14) * 14) + 1;
        p.px(seam, y + 1, W[1]); p.px(seam, y + 2, W[1]);
        if (h01(i, v, 15) > 0.5) p.px(seam + 2, y + 1, '#C8CCD0');
    }
}
function hex(p, v, b) {
    const S = b.stone;
    const pts = [];
    for (let j = -1; j <= 3; j++) for (let i = -1; i <= 3; i++) pts.push([i * 8 + (j & 1 ? 4 : 0), j * 7]);
    for (let y = 0; y < TS; y++) for (let x = 0; x < TS; x++) {
        let d1 = 99, d2 = 99, best = null;
        for (const q of pts) { const d = Math.hypot(x + 0.5 - q[0], (y + 0.5 - q[1]) * 1.15); if (d < d1) { d2 = d1; d1 = d; best = q; } else if (d < d2) d2 = d; }
        if (d2 - d1 < 1.1) { p.px(x, y, S[0]); if (h01(best[0] + x, best[1] + y, v) > 0.9) { p.px(x, y, '#9A2A2E'); p.glow(x, y, '#C2502E', 0.35); } }
        else { const ly = (y + 0.5 - best[1]); p.px(x, y, S[ly < -2 ? 3 : (x + 0.5 - best[0]) > 2 ? 1 : 2]); }
    }
}
function ash(p, v, b) { field(p, b.path, v + 21, 0.35, 0.8, 1); for (let i = 0; i < 2; i++) { const x = Math.floor(h01(i, v, 16) * 15), y = Math.floor(h01(i, v, 17) * 15); p.px(x, y, '#C2502E'); if (!i) p.glow(x, y, '#C2502E', 0.4); } }
function snow(p, v, b) { field(p, b.grass, v + 31, 0.22, 0.5, 3); for (let i = 0; i < 4; i++) p.px(Math.floor(h01(i, v, 18) * 16), Math.floor(h01(i, v, 19) * 16), '#FFFFFF'); }
function ice(p, v) {
    field(p, R('#5E8EA4', '#7AA8BC', '#98C2D2', '#B8DAE6', '#DAF0F8'), v + 41, 0.2, 0.5, 2);
    let x = 2 + v % 6, y = 0;
    while (y < 16) { p.px(x, y, '#4E7E94'); y++; if (h01(x, y, v) > 0.6) x += h01(y, x, v) > 0.5 ? 1 : -1; }
    p.px(10, 4, '#FFFFFF'); p.px(11, 4, '#FFFFFF');
}
function mud(p, v) { field(p, R('#1A2018', '#262E22', '#343E2C', '#465238', '#5A6846'), v + 51, 0.3, 1, 2); for (let i = 0; i < 2; i++) { const cx = 3 + h01(i, v, 22) * 10, cy = 3 + h01(i, v, 23) * 10; p.ellipse(cx, cy, 2.5, 1.4, '#16241E'); p.px(cx - 1, cy - 1, '#3E6E5E'); } }
function seabed(p, v, b) {
    const S = b.bright ? R('#3E7A78', '#4E9290', '#62A8A2', '#7ABEB4', '#96D2C6') : R('#1E2E34', '#2A3C42', '#384C52', '#486066', '#5A747A');
    field(p, S, v + 61, 0.3, 0.8, 2);
    if (v % 2 === 0) { const cx = 4 + v * 2, cy = 9; p.ellipse(cx, cy, 3, 2, S[1]); p.ellipse(cx - 0.5, cy - 0.5, 2.2, 1.3, S[3]); }
}

export const VARIANTS = 4;
const PAINT = {
    [G.GRASS]: (p, v, b) => grass(p, v, b), [G.MEADOW]: (p, v, b) => grass(p, v, b, true), [G.SAND]: sand, [G.DIRT]: path, [G.STONE]: cobble, [G.FLOOR]: flag,
    [G.PLANK]: plank, [G.BRIDGE]: plank, [G.HEX]: hex, [G.ASH]: ash, [G.SNOW]: snow, [G.ICE]: ice, [G.MUD]: mud
};

function edgeMask(dir, seed, blades) {
    const m = new Uint8Array(TS * TS);
    for (let y = 0; y < TS; y++) for (let x = 0; x < TS; x++) {
        let d;
        if (dir === 'n') d = y; else if (dir === 's') d = 15 - y; else if (dir === 'w') d = x; else if (dir === 'e') d = 15 - x;
        else { const cx = dir.includes('w') ? 0 : 15, cy = dir.includes('n') ? 0 : 15; d = Math.hypot(x - cx, y - cy) - 1; }
        const along = dir === 'n' || dir === 's' ? x : y;
        let depth = 4 + N.n2(along * 0.3 + seed * 3, seed) * 2.6;
        if (blades) depth += ((along * 7 + seed) % 3 === 0 ? 2 : 0) + ((along * 5 + seed) % 4 === 1 ? 1 : 0);
        m[y * TS + x] = d < depth ? 1 : 0;
    }
    return m;
}
export const DIRS = ['n', 'e', 's', 'w', 'ne', 'nw', 'se', 'sw'];

export function buildTiles(atlas) {
    const T = {};
    for (const [bname, b] of Object.entries(BIOME)) {
        for (const [g, paint] of Object.entries(PAINT)) for (let v = 0; v < VARIANTS; v++) {
            const p = new Pix(TS, TS); paint(p, v, b);
            T[`${g}:${bname}:${v}`] = atlas.add(`t:${g}:${bname}:${v}`, p);
        }
        for (const g of [G.GRASS, G.MEADOW, G.SAND, G.SNOW, G.DIRT, G.ASH, G.MUD, G.STONE, G.FLOOR, G.HEX, G.ICE]) {
            const base = T[`${g}:${bname}:0`], src = atlas.img;
            const grassy = g === G.GRASS || g === G.MEADOW || g === G.SNOW;
            DIRS.forEach((d, k) => {
                const m = edgeMask(d, k + g * 5, grassy);
                const p = new Pix(TS, TS);
                for (let y = 0; y < TS; y++) for (let x = 0; x < TS; x++) if (m[y * TS + x]) {
                    const i = ((base.y + y) * src.w + base.x + x) * 4;
                    p.px(x, y, [src.col[i], src.col[i + 1], src.col[i + 2]]);
                    if (src.glw[i + 3]) p.glow(x, y, [src.glw[i], src.glw[i + 1], src.glw[i + 2]]);
                }
                for (let y = 0; y < TS; y++) for (let x = 0; x < TS; x++) if (m[y * TS + x]) {
                    const open = [[1, 0], [-1, 0], [0, 1], [0, -1]].some(([dx, dy]) => { const xx = x + dx, yy = y + dy; return xx >= 0 && yy >= 0 && xx < 16 && yy < 16 && !m[yy * TS + xx]; });
                    if (open) { const c = p.get(x, y); p.px(x, y, grassy ? b.grass[0] : shade(c, 0.78)); }
                }
                T[`e:${g}:${bname}:${d}`] = atlas.add(`e:${g}:${bname}:${d}`, p);
            });
        }
        // rounded corners: a quarter disc of the lower material cut into the convex corner of a tile
        for (const g of [G.GRASS, G.MEADOW, G.SAND, G.SNOW, G.DIRT, G.ASH, G.MUD, G.STONE, G.FLOOR, G.HEX, G.ICE]) {
            const base = T[`${g}:${bname}:1`], src = atlas.img;
            for (const [k, cx, cy] of [['ne', 16, 0], ['nw', 0, 0], ['se', 16, 16], ['sw', 0, 16]]) {
                const p = new Pix(TS, TS);
                for (let y = 0; y < TS; y++) for (let x = 0; x < TS; x++) {
                    const d = Math.hypot(x + 0.5 - cx, y + 0.5 - cy);
                    const rr = 9 + N.n2(x * 0.5 + g, y * 0.5 + k.length) * 1.6 + (BAYER[(y & 3) * 4 + (x & 3)] - 0.5);
                    if (d > rr) continue;
                    const i = ((base.y + y) * src.w + base.x + x) * 4;
                    p.px(x, y, d > rr - 1 ? shade([src.col[i], src.col[i + 1], src.col[i + 2]], 0.8) : [src.col[i], src.col[i + 1], src.col[i + 2]]);
                    if (src.glw[i + 3]) p.glow(x, y, [src.glw[i], src.glw[i + 1], src.glw[i + 2]]);
                }
                T[`c:${g}:${bname}:${k}`] = atlas.add(`c:${g}:${bname}:${k}`, p);
            }
        }
        T[`cliffTop:${bname}`] = atlas.add(`ct:${bname}`, cliffTop(b, false));
        T[`cliffTopN:${bname}`] = atlas.add(`ctn:${bname}`, cliffTop(b, true));
        for (let r = 0; r < 3; r++) for (const side of ['m', 'l', 'r']) for (let v = 0; v < 2; v++) T[`cliffFace:${bname}:${r}:${side}:${v}`] = atlas.add(`cf:${bname}:${r}:${side}:${v}`, cliffFace(b, r, side, v));
        for (let r = 0; r < 3; r++) T[`stairs:${bname}:${r}`] = atlas.add(`st:${bname}:${r}`, stairs(b, r));
        for (let v = 0; v < VARIANTS; v++) { const p = new Pix(TS, TS); seabed(p, v, b); T[`seabed:${bname}:${v}`] = atlas.add(`seabed:${bname}:${v}`, p); }
    }
    { const p = new Pix(TS, TS); for (let y = 0; y < 7; y++) for (let x = 0; x < 16; x++) if (BAYER[(y & 3) * 4 + (x & 3)] < (7 - y) / 6) p.px(x, y, [0, 0, 0]); T.ao = atlas.add('ao', p); }
    return T;
}
function cliffTop(b, northEdge) {
    const p = new Pix(TS, TS);
    grass(p, 7, b);
    if (northEdge) { p.hline(0, 15, 0, b.grass[0]); p.hline(0, 15, 1, b.grass[4]); }
    return p;
}
function cliffFace(b, r, side, v) {
    const p = new Pix(TS, TS), C = b.cliff;
    const colW = [5, 6, 5];
    let x = -((v * 3) % 5), ci = 0;
    while (x < TS) {
        const w = colW[(ci + v) % 3];
        const top = r === 0 ? 2 + ((ci + v) % 2) : -1;
        for (let xx = 0; xx < w; xx++) for (let y = 0; y < TS; y++) {
            const X = x + xx; if (X < 0 || X >= TS) continue;
            if (y < top) continue;
            let k = xx === 0 ? 4 : xx === 1 ? 3 : xx === w - 1 ? 0 : 2;
            if (y === top && r === 0) k = 4;
            if ((y + ci * 7 + r * 16) % 19 === 0 && xx > 0) k = 1;
            if (r === 2 && y > 11) k = Math.max(0, k - 1);
            p.px(X, y, C[k]);
        }
        x += w; ci++;
    }
    if (r === 0) {
        const L = rgb(b.lip), D = b.grass[0];
        for (let X = 0; X < TS; X++) { p.px(X, 0, L); p.px(X, 1, h01(X, v, 3) > 0.35 ? L : D); if (h01(X, v, 4) > 0.7) p.px(X, 2, D); }
        for (let i = 0; i < 2; i++) { const vx = Math.floor(h01(i, v, 9) * 14) + 1, len = 4 + Math.floor(h01(i, v, 10) * 9); for (let y = 2; y < 2 + len; y++) p.px(vx + (y % 4 === 0 ? 1 : 0), y, y % 3 ? b.vine : L); }
    }
    if (side === 'l') for (let y = 0; y < TS; y++) p.px(0, y, C[4]);
    if (side === 'r') for (let y = 0; y < TS; y++) { p.px(15, y, C[0]); p.px(14, y, C[0]); }
    if (r === 2) for (let X = 0; X < TS; X++) if (h01(X, 9, v) > 0.55) p.px(X, 15, b.grass[1]);
    return p;
}
function stairs(b, r) {
    const p = new Pix(TS, TS), S = b.stone, C = b.cliff;
    for (let y = 0; y < TS; y++) {
        const step = Math.floor(y / 4), yy = y % 4;
        for (let x = 0; x < TS; x++) {
            const edge = x < 2 || x > 13;
            p.px(x, y, edge ? C[x < 2 ? 3 : 0] : S[yy === 0 ? 4 : yy === 3 ? 0 : 2 - ((step + r) & 1)]);
        }
    }
    if (r === 0) p.hline(0, 15, 0, b.lip);
    return p;
}

export function buildDeco(atlas) {
    const D = {};
    const add = (n, w, h, fn) => { const p = new Pix(w, h); fn(p); D[n] = atlas.add('d:' + n, p); };
    const bone = '#D8CFB8';
    add('boneshroom', 4, 4, p => { p.px(1, 0, bone); p.px(2, 0, bone); p.px(0, 1, bone); p.px(3, 1, '#B8AE98'); p.px(1, 2, '#8F8672'); p.px(1, 3, '#6F6858'); });
    add('boneshroom2', 6, 5, p => { p.rect(1, 0, 3, 1, bone); p.rect(0, 1, 5, 1, '#B8AE98'); p.px(2, 2, '#8F8672'); p.px(2, 3, '#6F6858'); p.px(5, 3, bone); p.px(5, 4, '#8F8672'); });
    add('pebbles', 6, 3, p => { p.px(0, 1, '#5A5E64'); p.px(1, 1, '#8A8E94'); p.px(3, 2, '#4A4E54'); p.px(4, 0, '#7A7E84'); p.px(5, 0, '#A8ACB0'); });
    add('shell', 4, 3, p => { p.rect(0, 1, 4, 2, '#E8C8B8'); p.px(1, 0, '#F8E8E0'); p.px(2, 0, '#F8E8E0'); p.px(1, 2, '#B08A7A'); });
    add('bones', 8, 4, p => { p.hline(1, 6, 2, bone); p.px(0, 1, bone); p.px(0, 3, bone); p.px(7, 1, bone); p.px(7, 3, bone); });
    add('skull', 6, 5, p => { p.blob(3, 2, 3, 2.3, bone, { dither: false }); p.px(2, 2, '#0B1016'); p.px(4, 2, '#0B1016'); });
    add('ember', 2, 2, p => { p.lit(0, 0, '#C2502E', 0.8); p.lit(1, 1, '#E8743A', 0.6); });
    add('crack', 10, 6, p => { p.line(0, 5, 4, 2, '#9A2A2E'); p.line(4, 2, 9, 0, '#9A2A2E'); for (const [x, y] of [[2, 3], [5, 2], [7, 1]]) p.glow(x, y, '#C2502E', 0.7); });
    add('snowtuft', 6, 3, p => { p.hline(0, 5, 2, '#B8C8CC'); p.hline(1, 4, 1, '#E6F0F0'); p.px(2, 0, '#FFFFFF'); });
    add('lily', 8, 5, p => { p.ellipse(4, 2.5, 4, 2.5, '#2E7A4A'); p.px(4, 1, '#5AAE6A'); p.px(6, 2, '#1E5A3A'); p.px(2, 2, '#F4B8D0'); p.px(3, 2, '#F8D8E8'); });
    add('reeds', 7, 10, p => { for (const [x, h] of [[1, 8], [3, 9], [5, 7]]) { p.vline(x, 9 - h, 9, '#4A6A2E'); p.px(x, 9 - h, '#C8A858'); p.px(x, 10 - h, '#8A6A3A'); } });
    add('puddle', 10, 4, p => { p.ellipse(5, 2, 5, 2, '#3A6A8A'); p.hline(3, 5, 1, '#A8D8F0'); });
    add('coralbud', 6, 5, p => { for (const [x, y] of [[1, 4], [1, 3], [1, 2], [3, 4], [3, 1], [3, 2], [3, 3], [5, 3], [5, 4]]) p.px(x, y, y % 2 ? '#7A4A7E' : '#5A2E5C'); });
    add('soulspark', 3, 3, p => { p.lit(1, 1, '#B8FFE8', 1); p.lit(0, 1, '#5FF2C8', 0.6); p.lit(2, 1, '#5FF2C8', 0.6); p.lit(1, 0, '#5FF2C8', 0.6); p.lit(1, 2, '#5FF2C8', 0.6); });
    add('rope', 12, 3, p => { for (let x = 0; x < 12; x++) p.px(x, 1 + Math.round(Math.sin(x * 0.8)), x % 2 ? '#A88A5A' : '#7A6A3A'); });
    add('plankbits', 10, 5, p => { p.rect(0, 2, 6, 2, '#7A5434'); p.hline(0, 5, 2, '#98704A'); p.rect(6, 0, 3, 2, '#5A3C24'); });
    const flowerPatch = (name, cols) => add(name, 12, 8, p => { for (let i = 0; i < 6; i++) { const x = 1 + Math.floor(h01(i, name.length + cols.length, 1) * 9), y = 1 + Math.floor(h01(i, name.length * 3, 2) * 5), c = rgb(cols[i % cols.length]); p.px(x, y + 1, '#2E6E3A'); p.px(x, y, c); p.px(x + 1, y, shade(c, 0.8)); p.px(x, y - 1, c); p.px(x - 1, y, shade(c, 0.9)); } });
    flowerPatch('flowersW', ['#F8F4E8', '#E8E4D8']); flowerPatch('flowersY', ['#F4E07A', '#F8F0B0']); flowerPatch('flowersP', ['#F4A8C8', '#F8D0E0']); flowerPatch('flowersB', ['#A8D8F8', '#D8F0FF']); flowerPatch('flowersR', ['#E05A4A', '#F08A6A']);
    add('tuft', 9, 7, p => { for (let k = 0; k < 5; k++) { const x = 1 + k * 2, ht = 3 + (k % 2) * 2; for (let j = 0; j < ht; j++) p.px(x + (j > 2 ? (k < 2 ? -1 : 1) : 0), 6 - j, j === ht - 1 ? '#9AD86A' : j > 1 ? '#6EBE52' : '#2E6E3A'); } });
    add('tuftDark', 9, 7, p => { for (let k = 0; k < 5; k++) { const x = 1 + k * 2, ht = 3 + (k % 2) * 2; for (let j = 0; j < ht; j++) p.px(x + (j > 2 ? (k < 2 ? -1 : 1) : 0), 6 - j, j === ht - 1 ? '#5A9A44' : j > 1 ? '#3E7A38' : '#1C3E24'); } });
    add('tallgrass', 12, 12, p => { for (let k = 0; k < 7; k++) { const x = 1 + k + (k > 3 ? 2 : 0), ht = 7 + (k % 3) * 2; for (let j = 0; j < ht; j++) p.px(x + Math.round(j * (k - 3) * 0.08), 11 - j, j > ht - 3 ? '#E8C858' : j > 2 ? '#7AAA44' : '#3E6A2E'); } });
    add('mushroomR', 6, 6, p => { p.ellipse(3, 2, 3, 2, '#C83A3A'); p.px(2, 1, '#F8F0E0'); p.px(4, 2, '#F8F0E0'); p.vline(3, 3, 5, '#E8DCC0'); });
    add('lanternStalk', 5, 14, p => { p.vline(2, 4, 13, '#3A1E26'); p.ellipse(2, 2.5, 2, 2.5, '#F05A4A'); p.lit(2, 2, '#F4A060', 1.4); p.lit(2, 3, '#F05A4A', 1.1); });
    return D;
}
export { mix };
