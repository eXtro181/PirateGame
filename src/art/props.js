// Prop sprites (y-sorted entities): vegetation, basalt arches, soul tree, forge, tavern, wrecks,
// houses, statues, saber fire, chests, lanterns … Each prop: frames, anchor (feet), shadow, lights.
import { Pix, BAYER } from './pix.js';
import { P, rgb, ramp, shade } from './pal.js';

const h01 = (x, y, s = 0) => { const v = Math.sin(x * 127.1 + y * 311.7 + s * 74.7) * 43758.5453; return v - Math.floor(v); };
const DEF = {};
function def(name, w, h, paint, meta = {}) { DEF[name] = { w, h, paint, ...meta }; }

// ---------------------------------------------------------------- trees and plants
function trunk(p, x0, y0, x1, y1, w, hex) {
    const R = ramp(hex);
    const steps = Math.max(Math.abs(x1 - x0), Math.abs(y1 - y0));
    for (let i = 0; i <= steps; i++) {
        const t = i / steps, x = x0 + (x1 - x0) * t, y = y0 + (y1 - y0) * t, ww = w * (1 - t * 0.35);
        for (let k = -ww / 2; k <= ww / 2; k++) p.px(x + k, y, R[k < -ww / 4 ? 3 : k > ww / 4 ? 0 : 1]);
    }
}
function crown(p, cx, cy, rx, ry, hex, seed, n = 7) { // blob clusters, light from the top-left
    for (let i = 0; i < n; i++) {
        const a = h01(i, seed) * Math.PI * 2, d = h01(i, seed, 2) * 0.55;
        p.blob(cx + Math.cos(a) * rx * d, cy + Math.sin(a) * ry * d, rx * (0.45 + h01(i, seed, 3) * 0.25), ry * (0.45 + h01(i, seed, 4) * 0.25), hex);
    }
}
// cursed willow (coral violet), crown is overhead
def('willow', 40, 52, (p) => {
    trunk(p, 20, 51, 18, 22, 6, P.deadwood); trunk(p, 18, 30, 10, 20, 3, P.deadwood); trunk(p, 19, 28, 28, 18, 3, P.deadwood);
    crown(p, 20, 16, 18, 13, P.coral, 3, 9);
    for (let i = 0; i < 9; i++) { const x = 4 + i * 4 + Math.floor(h01(i, 9) * 3); const len = 10 + Math.floor(h01(i, 8) * 14); for (let y = 18; y < 18 + len; y++) p.px(x + Math.round(Math.sin(y * 0.4) * 0.6), y, y % 3 ? P.violet : P.coral); }
}, { ax: 20, ay: 50, shadow: [12, 4], r: 0.4, sway: 1, overheadFrom: 32 });
// kelp palm (petrol)
def('kelppalm', 36, 50, (p) => {
    for (let y = 18; y < 50; y++) { const x = 18 + Math.round(Math.sin(y * 0.15) * 2); p.px(x - 1, y, P.deadwood); p.px(x, y, y % 4 === 0 ? P.tarwood : '#3A2A22'); p.px(x + 1, y, P.deadwood); }
    const R = ramp(P.petrol);
    for (let i = 0; i < 7; i++) {
        const a = -Math.PI + i * Math.PI / 6 + 0.1, len = 14 + h01(i, 4) * 4;
        for (let t = 0; t < len; t++) { const x = 18 + Math.cos(a) * t, y = 17 + Math.sin(a) * t * 0.6 + t * t * 0.03; p.px(x, y, R[t < 4 ? 3 : 2]); p.px(x, y + 1, R[0]); if (t % 3 === 0) p.px(x + Math.sin(a), y + 2, R[1]); }
    }
}, { ax: 18, ay: 49, shadow: [10, 3], r: 0.3, sway: 1.5, overheadFrom: 30 });
// ember palm (rust red, fire region)
def('emberpalm', 36, 50, (p) => {
    for (let y = 18; y < 50; y++) { const x = 18 + Math.round(Math.sin(y * 0.15) * 2); p.px(x - 1, y, P.abyss); p.px(x, y, '#3A2420'); p.px(x + 1, y, P.abyss); }
    const R = ramp(P.rust);
    for (let i = 0; i < 7; i++) {
        const a = -Math.PI + i * Math.PI / 6 + 0.1, len = 13 + h01(i, 5) * 4;
        for (let t = 0; t < len; t++) { const x = 18 + Math.cos(a) * t, y = 17 + Math.sin(a) * t * 0.6 + t * t * 0.03; p.px(x, y, R[2]); p.px(x, y + 1, R[0]); if (t > len - 3) p.lit(x, y, P.ember, 0.5); }
    }
}, { ax: 18, ay: 49, shadow: [10, 3], r: 0.3, sway: 1.5, overheadFrom: 30, light: { dy: -30, r: 30, col: [1, 0.4, 0.15], i: 0.5 } });
// dead mangrove (swamp, roots arched)
def('mangrove', 44, 46, (p) => {
    for (const [x0, x1] of [[6, 18], [12, 20], [38, 25], [32, 24], [22, 22]]) { for (let t = 0; t <= 10; t++) { const x = x0 + (x1 - x0) * t / 10, y = 45 - Math.sin(t / 10 * Math.PI) * 6 - t * 1.2; p.px(x, y, P.deadwood); p.px(x, y - 1, P.tarwood); } }
    trunk(p, 22, 34, 22, 14, 5, P.deadwood);
    for (let i = 0; i < 6; i++) { const a = -Math.PI * 0.9 + i * 0.35; trunk(p, 22, 16, 22 + Math.cos(a) * 16, 16 + Math.sin(a) * 12, 2, P.deadwood); }
    crown(p, 22, 10, 16, 8, P.leaf, 7, 6);
    for (let i = 0; i < 6; i++) { const x = 8 + i * 6; for (let y = 12; y < 22 + i % 3 * 3; y++) p.px(x, y, '#3A4A2E'); }
}, { ax: 22, ay: 44, shadow: [14, 4], r: 0.45, sway: 0.8, overheadFrom: 26 });
// bush (blob cluster)
for (let v = 0; v < 3; v++) def(`bush${v}`, 20, 14, (p) => { crown(p, 10, 8, 9, 5.5, [P.leaf, '#2E3E36', P.kelp][v], 11 + v, 5); }, { ax: 10, ay: 13, shadow: [8, 2], r: 0.3, sway: 0.5 });
// dead tree
def('deadtree', 30, 40, (p) => { trunk(p, 15, 39, 15, 12, 4, P.deadwood); trunk(p, 15, 20, 5, 8, 2, P.deadwood); trunk(p, 15, 18, 26, 6, 2, P.deadwood); trunk(p, 15, 14, 14, 2, 2, P.deadwood); }, { ax: 15, ay: 38, shadow: [8, 3], r: 0.3 });
// lantern mushroom (fire / end)
def('lanternshroom', 14, 16, (p) => {
    p.vline(7, 8, 15, P.bone); p.vline(6, 9, 15, '#B8AE98');
    p.ellipse(7, 6, 6, 4, P.ember);
    for (let x = 2; x < 13; x++) for (let y = 3; y < 8; y++) if (p.a(x, y)) { p.glow(x, y, y < 5 ? P.lantern : P.ember, 0.8); if (y < 5) p.px(x, y, P.lantern); }
}, { ax: 7, ay: 15, shadow: [5, 2], light: { dy: -7, r: 26, col: [1, 0.55, 0.2], i: 0.7, flicker: 0.2 } });
// ice crystal
def('icecrystal', 16, 22, (p) => {
    const R = ramp(P.ice);
    for (const [x, h, w] of [[8, 20, 3], [4, 12, 2], [12, 14, 2]]) for (let y = 0; y < h; y++) { const ww = Math.max(0, Math.round(w * (1 - (y < 3 ? (3 - y) / 3 : 0)))); for (let k = -ww; k <= ww; k++) p.px(x + k, 21 - h + y, R[k < 0 ? 4 : k > 0 ? 1 : 3]); }
    for (const [x, y] of [[8, 4], [4, 12], [12, 10]]) p.lit(x, y, P.soulLight, 0.7);
}, { ax: 8, ay: 21, shadow: [6, 2], r: 0.3, light: { dy: -10, r: 22, col: [0.4, 0.9, 1], i: 0.45 } });
// coral growth (end / earth)
def('coral', 18, 16, (p) => {
    for (const [x, h] of [[3, 10], [7, 14], [11, 9], [14, 12]]) for (let y = 0; y < h; y++) { p.px(x + Math.round(Math.sin(y * 0.6) * 1), 15 - y, y % 3 ? P.violet : P.coral); if (y === h - 1) p.lit(x, 15 - y, '#C88AD0', 0.5); }
}, { ax: 9, ay: 15, shadow: [7, 2], r: 0.25 });

// ---------------------------------------------------------------- rocks
for (let v = 0; v < 3; v++) def(`rock${v}`, 18 + v * 6, 14 + v * 4, (p) => {
    const w = 18 + v * 6, h = 14 + v * 4;
    p.blob(w / 2, h / 2 + 1, w / 2 - 1, h / 2 - 1, P.basalt);
    for (let x = 2; x < w - 2; x++) if (p.a(x, 2) && h01(x, v) > 0.5) p.px(x, 2, P.pale);
}, { ax: 9 + v * 3, ay: 13 + v * 4, shadow: [8 + v * 3, 3], r: 0.45 + v * 0.2 });

// ---------------------------------------------------------------- hub: soul tree, arches, statues
def('soultree', 96, 120, (p) => {
    // black, gnarled trunk of petrified driftwood
    const R = ramp('#1C1C22');
    for (let y = 50; y < 120; y++) {
        const w = 12 + Math.max(0, (y - 95)) * 0.9 + Math.sin(y * 0.3) * 1.5;
        for (let k = -w; k <= w; k++) { const x = 48 + k + Math.sin(y * 0.12) * 3; p.px(x, y, R[k < -w * 0.4 ? 3 : k > w * 0.5 ? 0 : (((k + y) % 7 === 0) ? 0 : 1)]); }
    }
    for (const [a, l] of [[-2.4, 30], [-0.7, 30], [-1.6, 34], [-2.9, 24], [-0.2, 24]]) trunk(p, 48, 60, 48 + Math.cos(a) * l, 60 + Math.sin(a) * l * 0.9, 5, '#1C1C22');
    // roots
    for (const [dx] of [[-26], [-14], [14], [28]]) for (let t = 0; t <= 12; t++) p.px(48 + dx * t / 12, 118 - Math.sin(t / 12 * Math.PI) * 4, R[0]);
    // nets, kelp and chains with soul lanterns hanging from the branches
    for (let i = 0; i < 10; i++) {
        const x = 16 + i * 7 + Math.floor(h01(i, 4) * 4), y0 = 40 + Math.floor(h01(i, 5) * 14), len = 12 + Math.floor(h01(i, 6) * 18);
        for (let y = y0; y < y0 + len; y++) p.px(x + Math.round(Math.sin(y * 0.3 + i)), y, i % 3 === 0 ? '#6A6A72' : P.kelp);
        if (i % 2 === 0) { const ly = y0 + len; p.rect(x - 1, ly, 3, 4, P.deadwood); p.lit(x, ly + 1, P.soul, 1.2); p.lit(x, ly + 2, P.soulLight, 1.4); }
    }
}, { ax: 48, ay: 118, shadow: [30, 7], r: 1.8, overheadFrom: 70 });
// crown as a pale glowing mist ball (separate overhead sprite)
def('soulcrown', 128, 72, (p) => {
    for (let y = 0; y < 72; y++) for (let x = 0; x < 128; x++) {
        const dx = (x - 64) / 62, dy = (y - 38) / 34, d = dx * dx + dy * dy;
        const n = Math.sin(x * 0.21 + y * 0.13) * 0.12 + Math.sin(x * 0.07 - y * 0.19) * 0.12;
        if (d + n > 1) continue;
        const k = d + n;
        if (BAYER[(y & 3) * 4 + (x & 3)] > 1.2 - k) continue; // dithered fringe
        const c = k < 0.35 ? P.soulLight : k < 0.7 ? P.mist : P.mistBlue;
        p.px(x, y, c, 200);
        if (k < 0.55) p.glow(x, y, P.soul, (0.55 - k) * 1.2);
    }
}, { ax: 64, ay: 72, overhead: true, light: { dy: -36, r: 120, col: [0.35, 0.95, 0.8], i: 0.9 } });
def('arch', 48, 44, (p) => {
    const R = ramp(P.basalt);
    for (const x0 of [2, 38]) { p.rect(x0, 10, 8, 34, R[2]); p.vline(x0, 10, 43, R[3]); p.vline(x0 + 7, 10, 43, R[0]); for (let y = 14; y < 44; y += 7) p.hline(x0, x0 + 7, y, R[1]); }
    for (let x = 2; x < 46; x++) { const y = 10 - Math.round(Math.sin((x - 2) / 44 * Math.PI) * 8); for (let k = 0; k < 6; k++) p.px(x, y + k, R[k === 0 ? 3 : k === 5 ? 0 : 2]); }
    for (let x = 4; x < 44; x += 3) if (h01(x, 1) > 0.5) p.px(x, 10 - Math.round(Math.sin((x - 2) / 44 * Math.PI) * 8) - 1, P.pale);
    // soul stone in the keystone
    p.rect(21, 1, 6, 8, P.abyss);
    for (let y = 2; y < 8; y++) for (let x = 22; x < 26; x++) p.lit(x, y, y < 4 ? P.soulLight : P.soul, 1.1);
}, { ax: 24, ay: 43, shadow: [22, 3], box: [3, 0.6], light: { dy: -38, r: 34, col: [0.35, 0.95, 0.8], i: 0.7, pulse: 3 } });
def('guardian', 24, 44, (p) => { // hooded drowned statue with a lantern
    const R = ramp('#4A5056');
    p.rect(4, 38, 16, 6, R[1]); p.hline(4, 19, 38, R[3]);
    for (let y = 8; y < 38; y++) { const w = 4 + Math.min(4, (y - 8) / 4); for (let k = -w; k <= w; k++) p.px(12 + k, y, R[k < -w / 2 ? 3 : k > w / 2 ? 0 : 2]); }
    p.blob(12, 9, 5, 6, '#4A5056'); p.rect(10, 9, 5, 4, P.abyss);
    p.lit(11, 11, P.soul, 1); p.lit(13, 11, P.soul, 1);
    p.vline(19, 16, 30, R[1]); p.rect(17, 30, 5, 6, P.deadwood); for (let y = 31; y < 35; y++) for (let x = 18; x < 21; x++) p.lit(x, y, y < 33 ? P.soulLight : P.soul, 1.3);
}, { ax: 12, ay: 43, shadow: [9, 3], r: 0.45, light: { dx: 7, dy: -11, r: 30, col: [0.35, 0.95, 0.8], i: 0.8, flicker: 0.15 } });
def('oarfence', 32, 18, (p) => {
    for (let i = 0; i < 4; i++) { const x = 3 + i * 8; p.vline(x, 2, 17, P.tarwood); p.vline(x + 1, 2, 17, P.deadwood); p.rect(x - 1, 0, 4, 4, P.oldwood); }
    for (let x = 0; x < 32; x++) { p.px(x, 8 + Math.round(Math.sin(x * 0.4)), '#8A7A5A'); p.px(x, 13 + Math.round(Math.sin(x * 0.4 + 1)), '#6A5A3A'); }
}, { ax: 16, ay: 17, shadow: [14, 2], box: [2, 0.3] });
def('basin', 26, 12, (p) => { p.ellipse(13, 6, 13, 6, P.basalt); p.ellipse(13, 5, 10, 4, P.waterMid); for (const [x, y] of [[9, 4], [16, 5]]) p.lit(x, y, P.soul, 1); }, { ax: 13, ay: 11, shadow: [12, 3], r: 0.6 });

// ---------------------------------------------------------------- camp / town
def('forge', 52, 44, (p) => {
    const R = ramp(P.basalt);
    p.rect(4, 12, 28, 30, R[1]); for (let y = 12; y < 42; y += 5) p.hline(4, 31, y, R[0]); p.vline(4, 12, 41, R[3]);
    p.rect(10, 2, 10, 10, R[2]); p.rect(12, 0, 6, 3, R[0]); // chimney
    p.rect(10, 24, 16, 12, P.abyss);
    for (let y = 26; y < 36; y++) for (let x = 11; x < 25; x++) { const c = y > 32 ? P.lava : y > 29 ? P.ember : P.blood; p.lit(x, y, c, y > 30 ? 1.4 : 0.8); }
    // anvil + cannon rack
    p.rect(36, 32, 10, 3, '#3A3E44'); p.rect(38, 35, 6, 6, '#2A2E34'); p.hline(36, 45, 32, '#6A7078');
    for (let i = 0; i < 3; i++) { const y = 18 + i * 5; p.rect(34, y, 16, 3, '#2A2E34'); p.hline(34, 49, y, '#5A6068'); p.px(49, y + 1, P.abyss); }
}, { ax: 26, ay: 43, shadow: [24, 4], box: [2.6, 1.2], light: { dx: -8, dy: -14, r: 50, col: [1, 0.45, 0.15], i: 1.1, flicker: 0.3 } });
def('tavern', 88, 60, (p) => { // tavern built from an upturned hull
    const R = ramp(P.tarwood);
    for (let y = 0; y < 44; y++) { const w = 42 - Math.max(0, 14 - y) * 1.8; for (let k = -w; k <= w; k++) p.px(44 + k, y + 4, R[(y % 5 === 0) ? 0 : (k < -w * 0.6 ? 3 : 2)]); }
    p.rect(4, 44, 80, 16, R[1]); for (let x = 4; x < 84; x += 6) p.vline(x, 44, 59, R[0]);
    p.rect(38, 46, 12, 14, P.abyss); p.rect(40, 48, 8, 12, '#2A1A12');
    for (const x of [16, 62]) { p.rect(x, 48, 8, 6, P.abyss); for (let y = 49; y < 53; y++) for (let k = x + 1; k < x + 7; k++) p.lit(k, y, P.lantern, 0.9); }
    p.hline(30, 58, 42, '#6A5A3A'); // sign rope
    p.rect(36, 36, 16, 6, P.deadwood); p.hline(38, 49, 38, P.bone);
}, { ax: 44, ay: 59, shadow: [40, 5], box: [5.2, 1.6], light: { dy: -10, r: 44, col: [1, 0.65, 0.3], i: 0.8, flicker: 0.12 } });
def('barreltable', 22, 18, (p) => {
    p.rect(3, 4, 16, 12, P.tarwood); for (const y of [6, 11]) p.hline(3, 18, y, '#3A3E44'); p.vline(3, 4, 15, P.oldwood);
    p.ellipse(11, 4, 10, 3, P.oldwood); p.rect(7, 1, 3, 3, P.bone); p.rect(13, 2, 2, 2, '#4A6A3A');
}, { ax: 11, ay: 17, shadow: [9, 2], r: 0.4 });
def('barrel', 12, 14, (p) => { p.rect(1, 2, 10, 12, P.tarwood); p.vline(1, 2, 13, P.oldwood); for (const y of [4, 10]) p.hline(1, 10, y, '#3A3E44'); p.ellipse(6, 2, 5, 2, P.oldwood); p.px(6, 2, P.deadwood); }, { ax: 6, ay: 13, shadow: [5, 2], r: 0.3 });
def('crate', 14, 14, (p) => { p.rect(1, 1, 12, 12, P.tarwood); p.rect(1, 1, 12, 2, P.oldwood); p.line(2, 3, 12, 12, P.deadwood); p.rect(1, 1, 1, 12, P.oldwood); p.rect(12, 1, 1, 12, P.deadwood); p.px(10, 5, '#4A5A5A'); }, { ax: 7, ay: 13, shadow: [6, 2], r: 0.35 });
def('flag', 16, 34, (p) => { p.vline(2, 0, 33, P.deadwood); }, { ax: 2, ay: 33, shadow: [3, 1], r: 0.15, anim: 'flag' });
def('gallows', 30, 40, (p) => { p.vline(6, 4, 39, P.tarwood); p.vline(7, 4, 39, P.deadwood); p.hline(6, 26, 4, P.tarwood); p.hline(6, 26, 5, P.deadwood); p.line(7, 12, 14, 5, P.tarwood); p.vline(23, 6, 16, '#8A7A5A'); p.ellipse(23, 18, 2, 2, '#8A7A5A'); }, { ax: 6, ay: 39, shadow: [8, 2], r: 0.25 });
def('maptable', 24, 18, (p) => { p.rect(2, 4, 20, 8, P.tarwood); p.rect(3, 5, 18, 6, '#B8A888'); p.line(5, 6, 18, 9, P.blood); p.px(8, 8, P.abyss); for (const x of [3, 20]) p.vline(x, 12, 17, P.deadwood); }, { ax: 12, ay: 17, shadow: [10, 2], r: 0.4 });
def('cannon', 24, 14, (p) => { p.rect(2, 4, 18, 6, '#2A2E34'); p.hline(2, 19, 4, '#6A7078'); p.ellipse(20, 7, 3, 3, '#1A1E24'); p.ellipse(6, 11, 3, 3, P.tarwood); p.ellipse(14, 11, 3, 3, P.tarwood); }, { ax: 12, ay: 13, shadow: [10, 2], r: 0.4 });
def('lamppost', 10, 30, (p) => { p.vline(5, 8, 29, P.deadwood); p.hline(3, 7, 29, P.tarwood); p.rect(2, 2, 7, 7, P.abyss); for (let y = 3; y < 8; y++) for (let x = 3; x < 8; x++) p.lit(x, y, y < 5 ? '#FFD08A' : P.lantern, 1.2); p.rect(3, 0, 5, 2, P.tarwood); }, { ax: 5, ay: 29, shadow: [3, 1], r: 0.15, light: { dy: -24, r: 46, col: [1, 0.66, 0.3], i: 1, flicker: 0.15 } });
// half-timbered harbour house with a slate fish-scale roof (roof is overhead)
for (let v = 0; v < 2; v++) def(`house${v}`, 64, 72, (p) => {
    const W = ramp(P.bone), T = ramp(P.tarwood), S = ramp(P.slate);
    p.rect(4, 32, 56, 40, W[1]);
    for (const x of [4, 22, 42, 59]) p.rect(x, 32, 2, 40, T[1]);
    for (const y of [32, 50, 70]) p.rect(4, y, 56, 2, T[1]);
    p.line(6, 34, 20, 48, T[1]); p.line(44, 34, 58, 48, T[1]);
    for (const [x, y] of [[10, 54], [46, 54], [28, 36]]) { // arched windows with warm light
        p.rect(x, y, 8, 9, T[0]); for (let j = 1; j < 8; j++) for (let i = 1; i < 7; i++) if (!(j === 1 && (i === 1 || i === 6))) p.lit(x + i, y + j, j < 4 ? '#FFD08A' : P.lantern, 0.9);
        p.hline(x, x + 7, y + 9, T[2]); p.px(x + 2, y + 10, '#8A8A7A'); p.px(x + 5, y + 10, P.bone); // herb box with fish bones
    }
    if (v === 1) { p.rect(27, 56, 10, 16, T[0]); p.rect(28, 57, 8, 15, P.deadwood); p.px(34, 64, P.lantern); }
    // roof: fish scales 4×3
    for (let y = 0; y < 34; y++) { const inset = Math.max(0, 14 - y) * 1.6; for (let x = Math.round(inset); x < 64 - Math.round(inset); x++) { const row = Math.floor(y / 3), sx = (x + (row % 2) * 2) % 4; p.px(x, y, (y % 3 === 2 || sx === 0) ? S[0] : S[2 + (sx === 1 ? 1 : 0)]); } }
    p.hline(0, 63, 33, S[0]);
    p.rect(44, 0, 6, 10, '#3A2E2A');
}, { ax: 32, ay: 71, shadow: [30, 4], box: [3.8, 1.4], light: { dy: -14, r: 38, col: [1, 0.66, 0.3], i: 0.7 }, overheadFrom: 34 });

// ---------------------------------------------------------------- ships, wrecks, figureheads
function hull(p, x0, y0, w, h, hex, broken = false) {
    const R = ramp(hex);
    for (let y = 0; y < h; y++) {
        const t = y / h, inset = Math.round(Math.pow(t, 2) * w * 0.18);
        for (let x = inset; x < w - inset; x++) {
            if (broken && x > w * 0.62 && y < h * 0.5 && h01(x, y) > 0.25) continue;
            p.px(x0 + x, y0 + y, R[y % 4 === 0 ? 0 : (x < w * 0.15 ? 3 : 2)]);
        }
    }
    for (let x = 0; x < w; x++) p.px(x0 + x, y0, R[4]);
}
def('wreck', 100, 56, (p) => { // grave of a ship: broken hull, bare mast, missing figurehead at the bow (left)
    hull(p, 8, 22, 88, 30, P.tarwood, true);
    p.rect(2, 26, 10, 4, P.deadwood); // stump of the bowsprit
    p.vline(40, 0, 24, P.deadwood); p.vline(41, 0, 24, P.tarwood); p.hline(30, 52, 6, P.deadwood);
    for (let y = 7; y < 20; y++) p.px(31 + Math.round(Math.sin(y) * 1.5), y, y % 2 ? P.bone : '#A89F88'); // torn sail rag
    for (let i = 0; i < 5; i++) p.px(20 + i * 12, 34 + (i % 2) * 6, P.abyss); // holes
}, { ax: 50, ay: 52, shadow: [46, 5], box: [6, 1.8], water: true });
def('figureheadSlot', 20, 30, (p) => { p.rect(6, 22, 8, 8, P.deadwood); p.hline(6, 13, 22, P.oldwood); p.rect(9, 10, 2, 12, P.deadwood); }, { ax: 10, ay: 29, shadow: [6, 2], r: 0.3 });
const FIGUREHEAD_COL = { fire: P.ember, ice: P.ice, wind: P.mist, earth: P.pale };
for (const [el, col] of Object.entries(FIGUREHEAD_COL)) def(`figurehead_${el}`, 20, 34, (p) => {
    p.rect(6, 26, 8, 8, P.deadwood);
    p.blob(10, 9, 5, 6, col); p.blob(10, 19, 5, 8, col); // carved maiden / king bust
    p.px(9, 8, P.abyss); p.px(11, 8, P.abyss);
    for (let y = 2; y < 26; y++) for (let x = 4; x < 16; x++) if (p.a(x, y) && h01(x, y) > 0.82) p.glow(x, y, col, 0.9);
    p.lit(9, 8, P.soul, 1.2); p.lit(11, 8, P.soul, 1.2);
}, { ax: 10, ay: 33, shadow: [6, 2], r: 0.3, light: { dy: -16, r: 34, col: el === 'fire' ? [1, 0.45, 0.2] : el === 'ice' ? [0.5, 0.85, 1] : el === 'wind' ? [0.8, 0.9, 1] : [0.5, 0.9, 0.4], i: 0.9, pulse: 2 } });
def('sunkenfigurehead', 30, 22, (p) => { p.blob(15, 12, 13, 8, '#6A6A60'); p.px(10, 9, P.abyss); p.px(14, 9, P.abyss); p.rect(20, 10, 9, 3, P.deadwood); for (let x = 3; x < 28; x++) p.px(x, 19, P.foam); }, { ax: 15, ay: 20, shadow: [0, 0], r: 0.5 });

// ---------------------------------------------------------------- bonfires, chests, misc
def('saberfire', 22, 34, (p) => {
    for (let i = 0; i < 7; i++) { const a = i / 7 * Math.PI * 2; p.blob(11 + Math.cos(a) * 8, 30 + Math.sin(a) * 3, 2.5, 2, P.basalt); }
    p.ellipse(11, 30, 6, 2, P.abyss);
    for (let y = 6; y < 30; y++) { const x = 11 + Math.round((30 - y) * 0.08); p.px(x, y, '#C8CCD2'); p.px(x + 1, y, '#8A9098'); if (y > 26) p.px(x - 1, y, '#5A6068'); }
    p.hline(8, 15, 7, P.gold); p.px(15, 6, P.gold); p.px(16, 5, P.gold);
    p.rect(11, 2, 2, 5, P.tarwood);
    p.blob(12, 1.5, 2.8, 2.3, P.bone, { dither: false }); p.lit(11, 1, P.soul, 1.4); p.lit(13, 1, P.soul, 1.4);
}, { ax: 11, ay: 31, shadow: [9, 3], r: 0.45, anim: 'soulfire', light: { dy: -10, r: 64, col: [0.35, 1, 0.82], i: 1.2, flicker: 0.25 } });
def('chest', 16, 13, (p) => { p.rect(1, 4, 14, 9, P.tarwood); p.rect(1, 1, 14, 4, P.oldwood); p.hline(1, 14, 4, P.deadwood); for (const x of [3, 12]) p.vline(x, 1, 12, P.gold); p.rect(7, 5, 2, 3, P.gold); }, { ax: 8, ay: 12, shadow: [7, 2], r: 0.35 });
def('chestopen', 16, 15, (p) => { p.rect(1, 6, 14, 9, P.tarwood); p.rect(1, 0, 14, 4, P.oldwood); p.rect(2, 4, 12, 3, P.abyss); for (let x = 3; x < 13; x++) p.lit(x, 6, P.lantern, 0.6); for (const x of [3, 12]) p.vline(x, 6, 14, P.gold); }, { ax: 8, ay: 14, shadow: [7, 2], r: 0.35 });
def('torch', 8, 22, (p) => { p.vline(4, 6, 21, P.tarwood); p.rect(2, 4, 5, 3, '#3A3E44'); }, { ax: 4, ay: 21, shadow: [2, 1], r: 0.12, anim: 'fire', light: { dy: -20, r: 44, col: [1, 0.55, 0.22], i: 1, flicker: 0.3 } });
def('skullpile', 18, 12, (p) => { for (const [x, y] of [[4, 7], [9, 8], [14, 7], [7, 3], [11, 3]]) { p.blob(x, y, 3, 2.5, P.bone, { dither: false }); p.px(x - 1, y, P.abyss); p.px(x + 1, y, P.abyss); } }, { ax: 9, ay: 11, shadow: [8, 2] });
def('grave', 12, 18, (p) => { p.rect(2, 2, 8, 14, P.basalt); p.ellipse(6, 3, 4, 2, P.basalt); p.hline(4, 7, 6, P.basaltDark); p.vline(6, 4, 9, P.basaltDark); p.hline(1, 10, 16, P.mossDark); }, { ax: 6, ay: 16, shadow: [5, 2], r: 0.25 });
def('waterfall', 48, 64, (p) => { for (let y = 0; y < 64; y++) for (let x = 0; x < 48; x++) p.px(x, y, (x + y * 0.3) % 6 < 2 ? P.foam : (x % 5 === 0 ? P.glint : P.water), 230); }, { ax: 24, ay: 64, anim: 'waterfall' });
def('portal', 40, 24, (p) => { p.ellipse(20, 12, 19, 11, P.abyss); }, { ax: 20, ay: 12, anim: 'portal', light: { r: 70, col: [0.4, 1, 0.85], i: 1.2, pulse: 2 } });
def('foggate', 40, 44, (p) => { for (let y = 0; y < 44; y++) for (let x = 0; x < 40; x++) { const dx = (x - 20) / 20, top = 44 - Math.sqrt(Math.max(0, 1 - dx * dx)) * 44; if (y < top) continue; const n = Math.sin(x * 0.5 + y * 0.15) * 0.5 + Math.sin(x * 0.17 - y * 0.3) * 0.5; if (BAYER[(y & 3) * 4 + (x & 3)] > 0.35 + n * 0.25) continue; p.px(x, y, n > 0.2 ? '#8FB8B4' : '#4A6A70'); if (n > 0.55) p.glow(x, y, P.soul, 0.15); } }, { ax: 20, ay: 43, anim: 'fog', light: { dy: -20, r: 30, col: [0.4, 0.9, 0.8], i: 0.25, pulse: 3 } });

// ---------------------------------------------------------------- animated pieces painted per frame
function flameFrame(p, f, w, h, cols) {
    for (let y = 0; y < h; y++) for (let x = 0; x < w; x++) {
        const t = y / h, cx = w / 2 + Math.sin(f * 1.7 + y * 0.6) * (1 - t) * 1.6;
        const r = (w / 2) * Math.pow(t, 0.7) * (0.8 + Math.sin(f * 2.3 + y * 0.9) * 0.15);
        const d = Math.abs(x + 0.5 - cx);
        if (d > r) continue;
        const k = d / Math.max(r, 0.01) * 0.6 + (1 - t) * 0.6;
        const c = k < 0.45 ? cols[0] : k < 0.75 ? cols[1] : cols[2];
        if (k > 0.85 && BAYER[(y & 3) * 4 + (x & 3)] < 0.5) continue;
        p.lit(x, y, c, k < 0.45 ? 1.6 : 1.1);
    }
}
export function buildProps(atlas) {
    const out = {};
    for (const [name, d] of Object.entries(DEF)) {
        const p = new Pix(d.w, d.h);
        d.paint(p);
        p.outline({ skipBottom: true });
        const rect = atlas.add('p:' + name, p);
        const meta = { ...d, frames: [rect], paint: undefined };
        // split tall vegetation/houses into trunk (entity) + overhead crown
        if (d.overheadFrom) {
            const top = new Pix(d.w, d.overheadFrom); const bot = new Pix(d.w, d.h);
            for (let y = 0; y < d.h; y++) for (let x = 0; x < d.w; x++) {
                const i = (y * d.w + x) * 4; if (!p.col[i + 3]) continue;
                const c = [p.col[i], p.col[i + 1], p.col[i + 2]];
                if (y < d.overheadFrom) { top.px(x, y, c); if (p.glw[i + 3]) top.glow(x, y, [p.glw[i], p.glw[i + 1], p.glw[i + 2]]); } else { bot.px(x, y, c); if (p.glw[i + 3]) bot.glow(x, y, [p.glw[i], p.glw[i + 1], p.glw[i + 2]]); }
            }
            meta.frames = [atlas.add('pb:' + name, bot)];
            meta.overheadRect = atlas.add('po:' + name, top);
        }
        out[name] = meta;
    }
    // flames (soul teal and fire orange), 6 frames
    const flames = (name, w, h, cols) => {
        const frames = [];
        for (let f = 0; f < 6; f++) { const p = new Pix(w, h); flameFrame(p, f, w, h, cols); frames.push(atlas.add(`${name}:${f}`, p)); }
        out[name] = { frames, w, h, ax: w / 2, ay: h - 1 };
    };
    flames('soulflame', 12, 18, [P.soulLight, P.soul, '#2E8A7A']);
    flames('fireflame', 8, 12, ['#FFE0A0', P.lantern, P.ember]);
    flames('bigflame', 16, 24, ['#FFE0A0', P.lantern, P.ember]);
    flames('forgeflame', 10, 10, ['#FFE0A0', P.lava, P.ember]);
    // flag: blood red with skull, 4 frames of wind
    {
        const frames = [];
        for (let f = 0; f < 4; f++) {
            const p = new Pix(16, 12);
            for (let y = 0; y < 10; y++) for (let x = 0; x < 13; x++) { const dy = Math.round(Math.sin(x * 0.6 + f * 1.5) * 1.2); p.px(x + 1, y + dy + 1, (x + f) % 4 === 0 ? '#5A1218' : P.blood); }
            p.px(6, 5, P.bone); p.px(7, 5, P.bone); p.px(6, 6, P.abyss); p.px(8, 6, P.bone);
            frames.push(atlas.add(`flagcloth:${f}`, p));
        }
        out.flagcloth = { frames, w: 16, h: 12, ax: 0, ay: 0 };
    }
    // waterfall strip: 4 frames scrolling
    {
        const frames = [];
        for (let f = 0; f < 4; f++) { const p = new Pix(48, 64); for (let y = 0; y < 64; y++) for (let x = 0; x < 48; x++) { const s = (y + f * 4 + Math.floor(h01(x, 0) * 8)) % 9; p.px(x, y, s < 2 ? P.foam : s < 4 ? P.glint : P.water, 235); if (s < 1) p.glow(x, y, P.soul, 0.25); } frames.push(atlas.add(`waterfall:${f}`, p)); }
        out.waterfall.frames = frames;
    }
    // portal whirl (end of the world), 6 frames
    {
        const frames = [];
        for (let f = 0; f < 6; f++) {
            const p = new Pix(40, 24);
            for (let y = 0; y < 24; y++) for (let x = 0; x < 40; x++) {
                const dx = (x - 20) / 19, dy = (y - 12) / 11, d = Math.hypot(dx, dy);
                if (d > 1) continue;
                const a = Math.atan2(dy, dx) + d * 5 - f * 0.5;
                const band = Math.sin(a * 3) * 0.5 + 0.5;
                const c = d < 0.25 ? P.abyss : band > 0.6 ? P.soul : band > 0.3 ? '#1E6A60' : P.waterDeep;
                p.px(x, y, c); if (band > 0.6 && d > 0.25) p.glow(x, y, P.soul, 0.8 * (1 - d));
            }
            frames.push(atlas.add(`portal:${f}`, p));
        }
        out.portal.frames = frames;
    }
    return out;
}
export { shade, rgb };
