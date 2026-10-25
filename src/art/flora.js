// Props in the style of the reference shots: blossom trees built from small flower clusters with
// hanging wisteria, round bushes, ferns, banana leaves, bamboo fences, marble arches and pillars
// with blue gems, guardian stones, benches, tables, market stalls, lantern poles, boats, wells.
import { BAYER } from './pix.js';
import { rgb, shade } from './pal.js';

const h01 = (x, y, s = 0) => { const v = Math.sin(x * 127.1 + y * 311.7 + s * 74.7) * 43758.5453; return v - Math.floor(v); };
const R = (...h) => h.map(rgb);
const P_GOLD = '#C9A04A';

// palettes (dark → light)
export const FOL = {
    pink: R('#7A3A5A', '#B0587E', '#E08AAE', '#F4B8CE', '#FCE4EC'),
    blossom: R('#8A5A7A', '#C08AA8', '#E8B8CE', '#F8DCE6', '#FFF6F8'),
    violet: R('#3E2A5E', '#5E3E8A', '#8A5EB8', '#B08AD8', '#D4B8F0'),
    blue: R('#1E2E6A', '#2E4A9A', '#4A6EC8', '#6E98E0', '#A8C8F4'),
    teal: R('#0E3A44', '#1A5A64', '#2A8088', '#46A8A8', '#7ACCC4'),
    green: R('#123A1E', '#1E5A2A', '#2E7E36', '#4AA444', '#7CC85A'),
    leaf: R('#163A22', '#22562E', '#34783C', '#4E9A48', '#78BC5E'),
    red: R('#5A1418', '#8E2424', '#C83A2E', '#EC6A3E', '#F8A060'),
    berry: R('#2A1A40', '#44285E', '#64408A', '#8A64B2', '#B496D8'),
    snowy: R('#6A8090', '#90A8B4', '#B8CCD4', '#DCE8EC', '#FFFFFF'),
    ember: R('#4A1410', '#7E2418', '#B83E20', '#E8743A', '#FFC070'),
    swamp: R('#0E2618', '#183A22', '#26522E', '#3A6E3A', '#58904A'),
    marble: R('#7E7666', '#A89E8A', '#D0C6AE', '#E8E0CC', '#FBF6EA'),
    bark: R('#140E10', '#24181A', '#382628', '#4E3A36', '#6A5246'),
    wood: R('#3A2618', '#5A3C24', '#7A5434', '#98704A', '#B48C62'),
    bamboo: R('#4A3A1E', '#7A6230', '#A88A44', '#C8AA5E', '#E4CC84'),
    stone: R('#3A3E46', '#5A6068', '#7E848A', '#A2A6A8', '#C8CAC6')
};

// a crown made of many small flower clusters, lit from the top-left
export function clusterCrown(p, cx, cy, rx, ry, pal, seed, n = 60, cr = 4) {
    const pts = [];
    for (let i = 0; i < n; i++) {
        const a = h01(i, seed, 1) * Math.PI * 2, d = Math.sqrt(h01(i, seed, 2));
        pts.push([cx + Math.cos(a) * rx * d, cy + Math.sin(a) * ry * d, cr * (0.75 + h01(i, seed, 3) * 0.55)]);
    }
    pts.sort((a, b) => a[1] - b[1]);
    for (const [x0, y0, r] of pts) {
        const lx = (x0 - cx) / rx, ly = (y0 - cy) / ry;
        const base = 2.3 - lx * 0.6 - ly * 0.9;
        // dark under-rim first so every cluster reads as its own puff
        for (let y = Math.floor(y0 - r); y <= y0 + r + 1; y++) for (let x = Math.floor(x0 - r); x <= x0 + r + 1; x++) {
            const dx = (x + 0.5 - x0 - 0.6) / r, dy = (y + 0.5 - y0 - 0.9) / r;
            if (dx * dx + dy * dy <= 1) p.px(x, y, pal[Math.max(0, Math.min(4, Math.round(base - 1.6)))]);
        }
        for (let y = Math.floor(y0 - r); y <= y0 + r; y++) for (let x = Math.floor(x0 - r); x <= x0 + r; x++) {
            const dx = (x + 0.5 - x0) / r, dy = (y + 0.5 - y0) / r, d2 = dx * dx + dy * dy;
            if (d2 > 1) continue;
            if (d2 > 0.72 && h01(x, y, seed) > 0.55) continue; // petal edge
            let v = base + (-dx * 0.7 - dy * 0.9) * 1.1;
            v += (BAYER[(y & 3) * 4 + (x & 3)] - 0.5) * 0.5;
            p.px(x, y, pal[Math.max(0, Math.min(4, Math.round(v)))]);
        }
        // little highlight petals
        p.px(x0 - r * 0.4, y0 - r * 0.5, pal[4]); if (r > 4) p.px(x0 - r * 0.4 + 1, y0 - r * 0.5, pal[Math.min(4, Math.round(base + 1))]);
    }
}
// hanging wisteria strands from the lower crown
export function wisteria(p, x0, x1, y0, pal, seed, n = 12, maxLen = 26) {
    for (let i = 0; i < n; i++) {
        const x = Math.round(x0 + h01(i, seed, 5) * (x1 - x0)), len = 8 + Math.floor(h01(i, seed, 6) * maxLen);
        const sy = y0 + Math.floor(h01(i, seed, 7) * 6);
        for (let k = 0; k < len; k += 2) {
            const t = k / len, xx = x + Math.round(Math.sin(k * 0.3 + i) * 0.8);
            const c = pal[Math.max(0, Math.min(4, Math.round(3.4 - t * 2.4 + (k % 4 ? 0 : 0.6))))];
            p.px(xx, sy + k, c); p.px(xx + 1, sy + k, shade(c, 0.85)); p.px(xx, sy + k + 1, shade(c, 0.9));
            if (t < 0.6 && k % 4 === 0) p.px(xx - 1, sy + k + 1, c);
        }
    }
}
function trunk(p, x0, y0, x1, y1, w, pal) {
    const steps = Math.max(Math.abs(x1 - x0), Math.abs(y1 - y0)) | 0;
    for (let i = 0; i <= steps; i++) {
        const t = i / Math.max(1, steps), x = x0 + (x1 - x0) * t, y = y0 + (y1 - y0) * t, ww = w * (1 - t * 0.4);
        for (let k = -ww / 2; k <= ww / 2; k++) p.px(x + k, y, pal[k < -ww / 4 ? 3 : k > ww / 4 ? 0 : (i % 5 === 0 ? 1 : 2)]);
    }
}
// twisted roots spreading on the ground
function roots(p, cx, cy, spread, pal, n = 7, seed = 1) {
    for (let i = 0; i < n; i++) {
        const dir = (i / (n - 1) - 0.5) * 2, len = spread * (0.6 + h01(i, seed) * 0.5);
        for (let t = 0; t <= len; t++) {
            const x = cx + dir * t, y = cy - 2 + t * 0.25 + Math.sin(t * 0.4 + i) * 1.2 - (Math.abs(dir) < 0.3 ? -t * 0.3 : 0);
            const w = Math.max(1, 3 - t / len * 3);
            for (let k = 0; k < w; k++) p.px(x, y + k, pal[k === 0 ? 3 : 1]);
        }
    }
}
const blobLeaf = (p, cx, cy, rx, ry, pal) => p.blob(cx, cy, rx, ry, pal === FOL.green ? '#2E7E36' : pal[2]);

export function defineFlora(def) {
    // ---------------------------------------------------------------- trees
    const treeVariants = [['blossomTree', FOL.blossom, FOL.pink], ['pinkTree', FOL.pink, FOL.violet], ['violetTree', FOL.violet, FOL.violet], ['blueTree', FOL.blue, null], ['tealTree', FOL.teal, null], ['redTree', FOL.red, null], ['berryTree', FOL.berry, null], ['snowTree', FOL.snowy, null], ['emberTree', FOL.ember, null], ['greenTree', FOL.green, null]];
    treeVariants.forEach(([name, pal, wis], vi) => {
        for (let s = 0; s < 2; s++) {
            const W = 92 + s * 32, H = 112 + s * 28, cy = 38 + s * 10;
            def(`${name}${s}`, W, H, (p) => {
                const cx = W / 2, foot = H - 2;
                roots(p, cx, foot, 10 + s * 3, FOL.bark, 5, vi + s);
                trunk(p, cx, foot, cx - 2, cy + 10, 9 + s * 2, FOL.bark);
                trunk(p, cx - 1, cy + 24, cx - 18, cy + 6, 4, FOL.bark);
                trunk(p, cx - 1, cy + 20, cx + 17, cy + 4, 4, FOL.bark);
                clusterCrown(p, cx, cy, 40 + s * 14, 30 + s * 10, pal, vi * 7 + s, 80 + s * 40, 6 + s);
                if (wis) wisteria(p, cx - 34 - s * 10, cx + 34 + s * 10, cy + 18, wis, vi + s, 16 + s * 6, 22 + s * 6);
            }, { ax: W / 2, ay: H - 2, shadow: [26 + s * 8, 6], r: 0.5, sway: 1, overheadFrom: cy + 34 + s * 10 });
        }
    });
    // palms (kelp palm teal, coconut green)
    for (const [name, pal] of [['palm', FOL.green], ['tealPalm', FOL.teal]]) def(name, 48, 64, (p) => {
        for (let y = 20; y < 63; y++) { const x = 24 + Math.round(Math.sin(y * 0.09) * 4); for (let k = -1; k <= 1; k++) p.px(x + k, y, FOL.wood[k < 0 ? 3 : k > 0 ? 0 : (y % 4 === 0 ? 1 : 2)]); }
        for (let i = 0; i < 9; i++) {
            const a = -Math.PI + i * Math.PI / 8 + 0.06, len = 17 + h01(i, 4) * 6;
            for (let t = 0; t < len; t++) {
                const x = 24 + Math.cos(a) * t, y = 19 + Math.sin(a) * t * 0.55 + t * t * 0.035;
                p.px(x, y, pal[t < 5 ? 4 : 3]); p.px(x, y + 1, pal[1]);
                if (t % 2 === 0) { p.px(x + Math.sin(a) * 2, y + 2, pal[2]); p.px(x - Math.sin(a) * 2, y + 2, pal[1]); }
            }
        }
        p.blob(24, 21, 3, 2.5, '#5A3A24');
    }, { ax: 24, ay: 63, shadow: [12, 3], r: 0.3, sway: 1.5, overheadFrom: 36 });
    // swamp tree: dark trunk with moss strands, big drooping canopy (canopy at 70 % in the mangroves)
    def('swampTree', 80, 88, (p) => {
        const cx = 40; roots(p, cx, 86, 16, FOL.bark, 7, 9);
        trunk(p, cx, 86, cx + 3, 34, 9, FOL.bark);
        clusterCrown(p, cx, 30, 34, 22, FOL.swamp, 41, 90, 5);
        for (let i = 0; i < 14; i++) { const x = 10 + i * 4 + Math.floor(h01(i, 3) * 3); for (let y = 36; y < 46 + (i % 4) * 6; y++) p.px(x, y, y % 3 ? '#3E6E3A' : '#5A8A4A'); }
    }, { ax: 40, ay: 86, shadow: [24, 6], r: 0.5, sway: 0.6, overheadFrom: 56, overheadAlpha: 0.7 });
    def('deadTree', 40, 52, (p) => { trunk(p, 20, 51, 20, 16, 5, FOL.bark); trunk(p, 20, 26, 7, 12, 2, FOL.bark); trunk(p, 20, 22, 33, 8, 2, FOL.bark); trunk(p, 20, 18, 18, 3, 2, FOL.bark); roots(p, 20, 51, 7, FOL.bark, 4, 3); }, { ax: 20, ay: 50, shadow: [10, 3], r: 0.3 });
    // ---------------------------------------------------------------- bushes, ferns, leaves
    for (const [name, pal] of [['bushBlue', FOL.blue], ['bushTeal', FOL.teal], ['bushGreen', FOL.green], ['bushPink', FOL.pink], ['bushViolet', FOL.violet], ['bushRed', FOL.red], ['bushSwamp', FOL.swamp], ['bushSnow', FOL.snowy]]) {
        def(name, 34, 28, (p) => clusterCrown(p, 17, 15, 14, 10, pal, name.length * 13, 22, 5), { ax: 17, ay: 27, shadow: [13, 3], r: 0.4, sway: 0.5 });
        def(name + 'S', 18, 14, (p) => clusterCrown(p, 9, 8, 7, 5, pal, name.length * 7, 12, 3), { ax: 9, ay: 13, shadow: [7, 2], sway: 0.4 });
    }
    def('fern', 22, 16, (p) => {
        for (let i = 0; i < 7; i++) { const a = -Math.PI + 0.2 + i * 0.45, len = 9 + (i % 2) * 3; for (let t = 0; t < len; t++) { const x = 11 + Math.cos(a) * t, y = 15 + Math.sin(a) * t * 0.9 + t * t * 0.04; p.px(x, y, FOL.leaf[t < 3 ? 1 : t % 2 ? 3 : 2]); if (t % 2) p.px(x + 1, y + 1, FOL.leaf[1]); } }
    }, { ax: 11, ay: 15, shadow: [7, 2], sway: 0.6 });
    def('banana', 36, 34, (p) => {
        for (let i = 0; i < 6; i++) {
            const a = -Math.PI * 0.95 + i * 0.36, len = 14 + (i % 3) * 3;
            for (let t = 0; t < len; t++) { const x = 18 + Math.cos(a) * t, y = 32 + Math.sin(a) * t * 1.2 + t * t * 0.02; const w = Math.sin(t / len * Math.PI) * 3.5; for (let k = -w; k <= w; k++) p.px(x - Math.sin(a) * k, y + Math.cos(a) * k * 0.6, FOL.teal[k < 0 ? 4 : k > w * 0.5 ? 1 : 3]); p.px(x, y, FOL.teal[1]); }
        }
    }, { ax: 18, ay: 32, shadow: [10, 3], r: 0.3, sway: 0.8 });
    def('bigLeaf', 30, 26, (p) => {
        for (let i = 0; i < 4; i++) { const cx = 7 + i * 6, cy = 8 + (i % 2) * 5; p.ellipse(cx, cy, 6, 5, FOL.swamp[2]); p.ellipse(cx - 1, cy - 1, 4, 3, FOL.swamp[3]); p.line(cx, cy, 15, 25, FOL.swamp[1]); p.px(cx + 2, cy + 2, '#C83A5A'); }
    }, { ax: 15, ay: 25, shadow: [10, 3], sway: 0.6 });
    def('reedClump', 18, 20, (p) => { for (let k = 0; k < 9; k++) { const x = 2 + k * 1.7, h = 10 + (k * 7) % 9; for (let j = 0; j < h; j++) p.px(x + Math.round(j * (k - 4) * 0.04), 19 - j, j > h - 3 ? '#C8A858' : j > 4 ? '#6A9A3E' : '#3E6A2E'); } }, { ax: 9, ay: 19, sway: 0.8 });
    def('cattail', 12, 22, (p) => { for (const [x, h] of [[3, 18], [6, 21], [9, 16]]) { p.vline(x, 21 - h, 21, '#4E7A34'); p.rect(x - 1, 21 - h, 2, 5, '#6A3E24'); } }, { ax: 6, ay: 21, sway: 0.7 });
    def('crystalBlue', 18, 22, (p) => { for (const [x, h, w] of [[9, 20, 3], [4, 12, 2], [14, 14, 2]]) for (let y = 0; y < h; y++) { const ww = Math.max(0, Math.round(w * Math.min(1, y / 3))); for (let k = -ww; k <= ww; k++) p.px(x + k, 21 - h + y, k < 0 ? '#E0F6FF' : k > 0 ? '#4A8AB8' : '#9AD0F0'); } for (const [x, y] of [[9, 4], [4, 12], [14, 10]]) p.lit(x, y, '#B8F0FF', 0.7); }, { ax: 9, ay: 21, shadow: [7, 2], r: 0.3, light: { dy: -10, r: 26, col: [0.5, 0.85, 1], i: 0.5 } });
    def('lanternPole', 10, 34, (p) => { p.vline(5, 10, 33, '#3A1E26'); p.vline(4, 14, 33, '#2A141A'); p.ellipse(5, 5, 4, 5, '#C83A3A'); for (let y = 2; y < 9; y++) for (let x = 2; x < 9; x++) if (p.a(x, y)) p.lit(x, y, y < 5 ? '#FFD08A' : '#F05A4A', 1.3); }, { ax: 5, ay: 33, shadow: [3, 1], r: 0.15, light: { dy: -28, r: 40, col: [1, 0.45, 0.3], i: 0.9, flicker: 0.15 } });
    def('glowFlower', 16, 18, (p) => { p.vline(8, 9, 17, '#3E6A3A'); for (let i = 0; i < 5; i++) { const a = i / 5 * Math.PI * 2 - Math.PI / 2; p.ellipse(8 + Math.cos(a) * 4, 7 + Math.sin(a) * 3, 2.5, 2, '#6E98E0'); } p.blob(8, 7, 3, 3, '#F8F4E8', { dither: false }); for (let y = 4; y < 11; y++) for (let x = 5; x < 12; x++) if (p.a(x, y)) p.glow(x, y, '#FFF4D8', 0.5); }, { ax: 8, ay: 17, shadow: [5, 2], light: { dy: -10, r: 30, col: [1, 0.95, 0.8], i: 0.6, pulse: 3 } });
    // ---------------------------------------------------------------- the soul tree of the hub
    def('hubTree', 180, 130, (p) => {
        const cx = 90;
        for (let y = 0; y < 112; y++) { const w = 14 + Math.max(0, y - 70) * 0.9 + Math.sin(y * 0.3) * 2; for (let k = -w; k <= w; k++) { const x = cx + k + Math.sin(y * 0.09) * 4; const tw = Math.sin((k + y * 0.6) * 0.45); p.px(x, y, FOL.bark[k < -w * 0.5 ? 3 : k > w * 0.55 ? 0 : tw > 0.6 ? 4 : tw > 0 ? 2 : 1]); } }
        for (let i = 0; i < 14; i++) {
            const dir = (i / 13 - 0.5) * 2.4, len = 36 + h01(i, 2) * 34;
            for (let t = 0; t < len; t++) { const x = cx + dir * t * 1.15 + Math.sin(t * 0.22 + i) * 3, y = 100 + t * 0.4 + Math.abs(dir) * t * 0.08; const w = Math.max(1, 7 - t / len * 7); for (let k = 0; k < w; k++) p.px(x, y + k, FOL.bark[k === 0 ? 4 : k === 1 ? 3 : k < w - 1 ? 2 : 1]); }
        }
    }, { ax: 90, ay: 112, shadow: [0, 0], box: [3.4, 1.4] });
    def('hubCrown', 300, 190, (p) => {
        clusterCrown(p, 150, 80, 140, 72, FOL.blossom, 77, 900, 7);
        clusterCrown(p, 150, 70, 100, 50, FOL.blossom, 79, 300, 7);
        wisteria(p, 20, 280, 110, FOL.pink, 5, 56, 50);
        wisteria(p, 40, 260, 118, FOL.violet, 9, 24, 34);
        for (let i = 0; i < 12; i++) { const x = 40 + i * 20 + Math.floor(h01(i, 3) * 8), y = 128 + Math.floor(h01(i, 4) * 26); p.vline(x, y - 14, y, '#4A3A36'); p.rect(x - 1, y, 3, 4, '#3A2E2A'); p.lit(x, y + 1, '#B8FFE8', 1.4); p.lit(x, y + 2, '#5FF2C8', 1.3); }
    }, { ax: 150, ay: 190, overhead: true, light: { dy: -60, r: 160, col: [0.9, 0.75, 0.85], i: 0.35 } });
    // ---------------------------------------------------------------- marble pieces (hub)
    def('marbleArch', 56, 50, (p) => {
        const M = 'marble';
        for (const x0 of [3, 43]) { p.vramp(x0, 14, 10, 34, '#D0C6AE', { from: 4, to: 1 }); p.vline(x0, 14, 47, FOL.marble[4]); p.vline(x0 + 9, 14, 47, FOL.marble[0]); p.rect(x0 - 2, 44, 14, 4, FOL.marble[2]); p.hline(x0 - 2, x0 + 11, 44, FOL.marble[4]); }
        for (let x = 3; x < 53; x++) { const y = 14 - Math.round(Math.sin((x - 3) / 49 * Math.PI) * 10); for (let k = 0; k < 7; k++) p.px(x, y + k, FOL.marble[k === 0 ? 4 : k === 6 ? 0 : k < 3 ? 3 : 2]); }
        p.blob(28, 6, 6, 7, '#D0C6AE'); p.ellipse(28, 6, 3, 4, '#2E4A9A');
        for (let y = 3; y < 10; y++) for (let x = 25; x < 32; x++) if (Math.hypot(x - 28, (y - 6) * 0.8) < 3) p.lit(x, y, y < 6 ? '#A8E0FF' : '#4A8AE0', 1.1);
        void M;
    }, { ax: 28, ay: 47, shadow: [26, 3], box: [3.4, 0.5], light: { dy: -40, r: 34, col: [0.45, 0.75, 1], i: 0.6, pulse: 3 } });
    def('marblePillar', 18, 40, (p) => {
        p.vramp(4, 8, 10, 28, '#D0C6AE', { from: 4, to: 1 }); p.vline(4, 8, 35, FOL.marble[4]); p.vline(13, 8, 35, FOL.marble[0]);
        p.rect(2, 34, 14, 5, FOL.marble[2]); p.hline(2, 15, 34, FOL.marble[4]);
        p.blob(9, 6, 6, 5, '#D0C6AE');
        for (let y = 3; y < 9; y++) for (let x = 6; x < 13; x++) if (Math.hypot(x - 9, y - 6) < 2.6) p.lit(x, y, y < 6 ? '#A8E0FF' : '#3A7AE0', 1.2);
    }, { ax: 9, ay: 38, shadow: [8, 2], r: 0.4, light: { dy: -32, r: 26, col: [0.45, 0.75, 1], i: 0.5, pulse: 3 } });
    def('marbleStone', 18, 22, (p) => { p.blob(9, 12, 7, 9, '#D0C6AE'); for (let y = 6; y < 18; y += 4) p.hline(4, 14, y, FOL.marble[1]); p.rect(3, 19, 12, 3, FOL.marble[1]); }, { ax: 9, ay: 21, shadow: [7, 2], r: 0.35 });
    def('rimPost', 12, 20, (p) => { p.vramp(2, 4, 8, 14, '#D0C6AE', { from: 4, to: 1 }); p.blob(6, 4, 4, 3, '#D0C6AE'); p.rect(1, 17, 10, 3, FOL.marble[1]); }, { ax: 6, ay: 19, shadow: [5, 2], r: 0.3 });
    // ---------------------------------------------------------------- village
    def('fenceH', 34, 20, (p) => {
        for (const x of [2, 13, 24]) { for (let y = 2; y < 19; y++) { p.px(x, y, FOL.bamboo[3]); p.px(x + 1, y, FOL.bamboo[2]); p.px(x + 2, y, FOL.bamboo[1]); if (y % 6 === 0) p.hline(x, x + 2, y, FOL.bamboo[0]); } p.hline(x, x + 2, 1, FOL.bamboo[4]); }
        for (let x = 0; x < 34; x++) { p.px(x, 7 + Math.round(Math.sin(x * 0.25)), '#A88A5A'); p.px(x, 12 + Math.round(Math.sin(x * 0.25 + 1)), '#7A6A3A'); }
    }, { ax: 17, ay: 18, shadow: [15, 2], box: [1.05, 0.25] });
    def('fencePost', 8, 22, (p) => { for (let y = 2; y < 21; y++) { p.px(2, y, FOL.bamboo[3]); p.px(3, y, FOL.bamboo[2]); p.px(4, y, FOL.bamboo[1]); if (y % 6 === 0) p.hline(2, 4, y, FOL.bamboo[0]); } p.hline(2, 4, 1, FOL.bamboo[4]); }, { ax: 3, ay: 20, shadow: [3, 1], r: 0.15 });
    def('bench', 30, 16, (p) => { p.rect(1, 4, 28, 4, FOL.wood[3]); p.hline(1, 28, 4, FOL.wood[4]); p.hline(1, 28, 7, FOL.wood[1]); for (const x of [3, 25]) p.rect(x, 8, 2, 7, FOL.wood[1]); }, { ax: 15, ay: 14, shadow: [13, 2], box: [0.9, 0.3] });
    def('table', 34, 22, (p) => { p.rect(1, 3, 32, 10, FOL.wood[3]); for (let x = 1; x < 33; x += 5) p.vline(x, 3, 12, FOL.wood[2]); p.hline(1, 32, 3, FOL.wood[4]); p.hline(1, 32, 12, FOL.wood[1]); for (const x of [3, 29]) p.rect(x, 13, 2, 8, FOL.wood[1]); p.ellipse(10, 6, 3, 2, '#E8DCC0'); p.ellipse(22, 7, 3, 2, '#C8A858'); p.rect(16, 4, 2, 4, '#3A6A3A'); }, { ax: 17, ay: 20, shadow: [15, 3], box: [1.0, 0.5] });
    def('stall', 48, 46, (p) => {
        for (let y = 0; y < 14; y++) for (let x = 0; x < 48; x++) p.px(x, y + Math.floor(x / 6) % 2, ((x / 6) | 0) % 2 ? '#E8DCC0' : '#C83A3A');
        for (const x of [3, 44]) p.rect(x, 12, 2, 32, FOL.wood[1]);
        p.rect(2, 28, 44, 10, FOL.wood[3]); p.hline(2, 45, 28, FOL.wood[4]);
        for (let i = 0; i < 8; i++) p.ellipse(7 + i * 5, 26, 2, 2, ['#E8743A', '#C83A3A', '#8ABE4A', '#F4E07A'][i % 4]);
    }, { ax: 24, ay: 44, shadow: [22, 3], box: [1.4, 0.4] });
    def('crates', 30, 28, (p) => {
        const crate = (x, y, s) => { p.rect(x, y, s, s, FOL.wood[2]); p.rect(x, y, s, 2, FOL.wood[4]); p.line(x + 1, y + 3, x + s - 2, y + s - 2, FOL.wood[1]); p.rect(x, y, 1, s, FOL.wood[3]); p.rect(x + s - 1, y, 1, s, FOL.wood[0]); };
        crate(1, 13, 14); crate(15, 15, 12); crate(7, 2, 11);
    }, { ax: 15, ay: 27, shadow: [13, 3], r: 0.5 });
    def('barrels', 26, 20, (p) => { for (const [x, y] of [[1, 5], [13, 5], [7, 0]]) { p.rect(x, y + 2, 11, 12, FOL.wood[2]); p.vline(x, y + 2, y + 13, FOL.wood[3]); for (const yy of [y + 4, y + 11]) p.hline(x, x + 10, yy, '#3A3E44'); p.ellipse(x + 5.5, y + 2, 5.5, 2, FOL.wood[3]); } }, { ax: 13, ay: 19, shadow: [11, 2], r: 0.45 });
    def('well', 30, 36, (p) => {
        p.ellipse(15, 28, 13, 6, FOL.stone[2]); p.ellipse(15, 27, 10, 4, '#1E4A6A'); p.hline(8, 22, 26, '#4A8AB8');
        for (const x of [4, 25]) p.rect(x, 6, 2, 22, FOL.wood[1]);
        p.rect(2, 3, 26, 4, '#2E7E86'); p.hline(2, 27, 3, '#46A8A8'); p.vline(15, 7, 20, '#A88A5A'); p.rect(13, 20, 5, 4, FOL.wood[2]);
    }, { ax: 15, ay: 33, shadow: [13, 3], r: 0.7 });
    def('rowboat', 40, 18, (p) => { for (let x = 0; x < 40; x++) { const d = Math.round(Math.sin(x / 39 * Math.PI) * 6); for (let y = 8 - d; y < 9 + d * 0.6; y++) p.px(x, y, y < 9 - d + 2 ? FOL.wood[4] : FOL.wood[2]); } p.hline(8, 32, 9, FOL.wood[1]); p.line(26, 2, 34, 12, FOL.wood[1]); }, { ax: 20, ay: 16, shadow: [0, 0] });
    def('anchorDeco', 22, 26, (p) => { p.vline(11, 3, 22, '#5A6068'); p.vline(12, 3, 22, '#3A3E44'); p.ellipse(11.5, 2, 2.5, 2, '#7E848A'); p.hline(5, 18, 7, '#5A6068'); for (let k = 0; k < 8; k++) { p.px(11 - k, 22 - Math.round(Math.sqrt(k) * 2), '#5A6068'); p.px(12 + k, 22 - Math.round(Math.sqrt(k) * 2), '#3A3E44'); } }, { ax: 11, ay: 24, shadow: [8, 2], r: 0.3 });
    def('nets', 26, 20, (p) => { for (let y = 0; y < 18; y++) for (let x = 0; x < 26; x++) if ((x + y) % 4 === 0 || (x - y + 40) % 4 === 0) if (Math.abs(x - 13) < 13 - y * 0.2) p.px(x, y + 2, '#8A7A5A'); for (const x of [0, 25]) p.vline(x, 0, 19, FOL.wood[1]); }, { ax: 13, ay: 19, shadow: [10, 2], box: [0.8, 0.2] });
    def('signpost', 18, 28, (p) => { p.vline(8, 6, 27, FOL.wood[1]); p.vline(9, 6, 27, FOL.wood[2]); p.rect(1, 4, 16, 6, FOL.wood[3]); p.hline(1, 16, 4, FOL.wood[4]); p.hline(3, 13, 7, FOL.wood[1]); p.px(16, 7, FOL.wood[3]); p.px(17, 7, FOL.wood[3]); }, { ax: 8, ay: 27, shadow: [4, 1], r: 0.15 });
    def('laundry', 48, 30, (p) => { for (const x of [1, 46]) p.rect(x, 4, 2, 25, FOL.wood[1]); for (let x = 2; x < 47; x++) p.px(x, 6 + Math.round(Math.sin((x - 2) / 44 * Math.PI) * 3), '#C8B890'); [[8, '#E8DCC0'], [18, '#6E98E0'], [28, '#C83A3A'], [38, '#E8DCC0']].forEach(([x, c]) => { const y = 7 + Math.round(Math.sin((x - 2) / 44 * Math.PI) * 3); p.rect(x, y, 7, 9, c); p.hline(x, x + 6, y, shade(rgb(c), 1.1)); }); }, { ax: 24, ay: 28, shadow: [20, 2], box: [1.5, 0.2] });
    // town houses after the reference: cream walls, timber frame, arched windows with flower boxes,
    // teal fish-scale roofs (the roof is overhead and fades when you walk behind it)
    for (let v = 0; v < 3; v++) {
        const W = [80, 96, 72][v], H = [92, 104, 88][v], roofH = [46, 52, 44][v];
        def(`townhouse${v}`, W, H, (p) => {
            const wall = R('#8A7A60', '#B8A888', '#DCCEB0', '#EDE2C8', '#F8F0DC'), tim = FOL.wood, roof = v === 2 ? R('#3A2A4A', '#5A3E6E', '#7A5A96', '#9A7AB6', '#BCA0D4') : R('#0E3A44', '#16606A', '#2A8A90', '#46AEAE', '#7ACCC8');
            const top = roofH - 6;
            for (let y = top; y < H; y++) for (let x = 3; x < W - 3; x++) p.px(x, y, wall[x < 6 ? 4 : x > W - 8 ? 1 : 2]);
            for (const x of [3, Math.floor(W / 2) - 1, W - 5]) p.rect(x, top, 3, H - top, tim[1]);
            for (const y of [top, top + 22, H - 3]) p.rect(3, y, W - 6, 3, tim[1]);
            // arched windows with flower boxes
            const wins = v === 1 ? [10, 30, 58, 78] : [10, W - 24];
            for (const wx of wins) {
                const wy = top + 6;
                for (let y = 0; y < 12; y++) for (let x = 0; x < 12; x++) { const arch = y < 4 && Math.hypot(x - 5.5, y - 4) > 6; if (arch) continue; p.px(wx + x, wy + y, (x === 0 || x === 11 || y === 11) ? tim[2] : (y < 6 ? '#3A2A20' : '#2A1E18')); }
                p.vline(wx + 6, wy + 2, wy + 10, tim[2]); p.hline(wx + 1, wx + 10, wy + 6, tim[2]);
                for (let y = wy + 2; y < wy + 11; y++) for (let x = wx + 1; x < wx + 11; x++) if (h01(x, y, v) > 0.8 && y > wy + 4) p.lit(x, y, '#FFD08A', 0.6);
                p.rect(wx - 1, wy + 12, 14, 3, tim[3]); for (let k = 0; k < 6; k++) p.px(wx + k * 2, wy + 11, k % 2 ? '#F4A8C8' : '#F8F4E8');
            }
            // door
            const dx = Math.floor(W / 2) - 7; p.rect(dx, H - 22, 14, 19, tim[1]); for (let y = H - 21; y < H - 3; y++) for (let x = dx + 1; x < dx + 13; x++) { if (y < H - 17 && Math.hypot(x - dx - 6.5, y - (H - 17)) > 6) continue; p.px(x, y, (x - dx) % 4 === 0 ? tim[0] : tim[2]); } p.px(dx + 10, H - 12, P_GOLD);
            for (const bx of [dx - 8, dx + 16]) { clusterCrown(p, bx + 3, H - 6, 5, 3, FOL.green, bx, 6, 3); }
            // fish-scale roof (4×3 scales) with a ridge and eaves
            for (let y = 0; y < roofH; y++) {
                const inset = Math.max(0, 18 - y) * 1.0;
                for (let x = Math.round(inset); x < W - Math.round(inset); x++) {
                    const row = Math.floor(y / 3), sx = (x + (row % 2) * 2) % 4, edge = y % 3 === 2;
                    const shadeK = y > roofH - 4 ? 0 : x < W * 0.25 ? 3 : x > W * 0.75 ? 1 : 2;
                    p.px(x, y, edge ? roof[Math.max(0, shadeK - 1)] : sx === 0 ? roof[Math.max(0, shadeK - 1)] : roof[shadeK + (sx === 1 && shadeK < 4 ? 1 : 0)]);
                }
            }
            p.hline(0, W - 1, roofH - 1, tim[1]); p.hline(0, W - 1, roofH, tim[0]);
            for (let x = 18; x < W - 18; x++) p.px(x, 0, roof[4]);
            p.rect(W - 26, 2, 7, 12, '#7E7666'); p.rect(W - 27, 1, 9, 2, '#A89E8A');
        }, { ax: W / 2, ay: H - 2, shadow: [W * 0.45, 4], box: [W / 32 - 0.3, 1.2], overheadFrom: roofH, light: { dy: -30, r: 40, col: [1, 0.7, 0.4], i: 0.35 } });
    }
    def('flowerPot', 12, 12, (p) => { p.rect(2, 6, 8, 6, '#A85A3A'); p.hline(2, 9, 6, '#C8784A'); clusterCrown(p, 6, 4, 5, 3, FOL.pink, 3, 10, 2); }, { ax: 6, ay: 11, shadow: [4, 1] });
    def('stoneColumn', 20, 34, (p) => { p.vramp(3, 4, 14, 30, '#7E848A', { from: 4, to: 1 }); for (let y = 8; y < 30; y += 7) p.hline(3, 16, y, FOL.stone[1]); p.ellipse(10, 4, 7, 3, FOL.stone[3]); for (let x = 4; x < 16; x += 3) p.px(x, 3, '#4AA444'); }, { ax: 10, ay: 33, shadow: [8, 2], r: 0.45 });
    def('bridgePost', 8, 26, (p) => { p.rect(2, 4, 4, 22, FOL.wood[1]); p.vline(2, 4, 25, FOL.wood[3]); p.rect(1, 2, 6, 3, FOL.wood[2]); }, { ax: 4, ay: 25, shadow: [3, 1], r: 0.15 });
    def('logPile', 30, 16, (p) => { for (const [x, y] of [[2, 8], [12, 8], [22, 8], [7, 2], [17, 2]]) { p.ellipse(x + 3, y + 3, 4, 4, FOL.wood[2]); p.ellipse(x + 3, y + 3, 2, 2, FOL.wood[4]); p.px(x + 3, y + 3, FOL.wood[1]); } }, { ax: 15, ay: 15, shadow: [12, 2], r: 0.5 });
    def('haybale', 22, 16, (p) => { p.blob(11, 9, 10, 6, '#C8A858'); for (let x = 3; x < 20; x += 3) p.vline(x, 5, 13, '#A88A44'); }, { ax: 11, ay: 15, shadow: [9, 2], r: 0.45 });
    def('tent', 50, 38, (p) => { for (let y = 0; y < 30; y++) { const w = y * 0.8 + 2; for (let k = -w; k <= w; k++) p.px(25 + k, y + 4, k < 0 ? '#C8B890' : '#9A8A6A'); } p.rect(21, 22, 8, 12, '#3A2A1E'); p.vline(25, 0, 6, FOL.wood[1]); p.px(25, 0, '#C83A3A'); p.px(26, 0, '#C83A3A'); }, { ax: 25, ay: 34, shadow: [22, 3], box: [1.4, 0.6] });
}
