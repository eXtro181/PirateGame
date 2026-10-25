// Main-menu panorama (480×270 view, layers 512 px wide for the parallax drift):
// clouds, basalt mountains with the sea horizon, the soul-tree island, the plain with fortress
// Schwarzwasser and its lighthouse, a fog band and foreground silhouettes.
import { Pix, BAYER } from './pix.js';
import { P, ramp } from './pal.js';
import { Noise } from '../gfx/noise.js';
import { clusterCrown, FOL, wisteria } from './flora.js';

const N = new Noise(7);
const h01 = (x, y, s = 0) => { const v = Math.sin(x * 127.1 + y * 311.7 + s * 74.7) * 43758.5453; return v - Math.floor(v); };
export const MENU_W = 480, MENU_H = 270, LAYER_W = 512, HORIZON = Math.round(270 * 0.62);

function clouds(w, h, seed, fill, edge, glint, density) {
    const p = new Pix(w, h), e = new Pix(w, h);
    for (let y = 0; y < h; y++) for (let x = 0; x < w; x++) {
        // tileable in x: blend two samples
        const t = x / w;
        const n = N.fbm(x * 0.012 + seed, y * 0.035, 4, 0) * (1 - t) + N.fbm((x - w) * 0.012 + seed, y * 0.035, 4, 0) * t;
        const shape = n + 0.15 - Math.abs(y / h - 0.45) * 1.2 + density;
        if (shape < 0) continue;
        p.px(x, y, fill);
    }
    // edges: top-left lit rim (glint), bottom rim (edge)
    for (let y = 0; y < h; y++) for (let x = 0; x < w; x++) {
        if (!p.a(x, y)) continue;
        const topOpen = !p.a(x, y - 1) || !p.a(x, y - 2);
        const botOpen = !p.a(x, y + 1);
        if (topOpen) { p.px(x, y, !p.a(x, y - 1) ? glint : edge); e.px(x, y, [255, 255, 255]); }
        else if (botOpen) p.px(x, y, edge);
        else if (!p.a(x, y - 3) && BAYER[(y & 3) * 4 + (x & 3)] < 0.5) p.px(x, y, edge);
    }
    return { p, e };
}

export function buildMenuArt(atlas) {
    const A = {};
    // 1: moon + halo rings
    { const p = new Pix(30, 30); for (let y = 0; y < 30; y++) for (let x = 0; x < 30; x++) { const d = Math.hypot(x - 14.5, y - 14.5); if (d > 13) continue; const l = -(x - 14.5) - (y - 14.5); p.px(x, y, d > 11.5 ? '#9FB0B4' : l > 6 ? '#E2E8E4' : l > -4 ? '#C9D2D0' : '#A8B4B6'); if (N.n2(x * 0.3, y * 0.3) > 0.35 && d < 11) p.px(x, y, '#8FA0A4'); p.glow(x, y, '#9FB0B4', 0.25); } A.moon = atlas.add('m:moon', p); }
    for (const [k, r] of [['halo1', 22], ['halo2', 32], ['halo3', 46]]) { const p = new Pix(r * 2, r * 2); for (let y = 0; y < r * 2; y++) for (let x = 0; x < r * 2; x++) { const d = Math.hypot(x + 0.5 - r, y + 0.5 - r); if (d < r) p.px(x, y, [200, 220, 220]); } A[k] = atlas.add('m:' + k, p); }
    // 2 + 3: clouds
    { const c = clouds(960, 60, 3, P.storm, P.mistBlue, P.mist, -0.12); A.cloudsFar = atlas.add('m:cf', c.p); A.cloudsFarEdge = atlas.add('m:cfe', c.e); }
    { const c = clouds(960, 50, 11, '#1C2D3A', P.storm, P.mistBlue, -0.08); A.cloudsNear = atlas.add('m:cn', c.p); A.cloudsNearEdge = atlas.add('m:cne', c.e); }
    // 4: basalt mountains + sea to the horizon
    {
        const H = 120, p = new Pix(LAYER_W, H);
        const base = H - 30; // horizon line inside the layer
        for (const [amp, off, col, seed] of [[70, 10, '#24394A', 1], [46, 0, '#1A2A38', 5], [26, -4, '#121D27', 9]]) {
            const R = ramp(col);
            for (let x = 0; x < LAYER_W; x++) {
                const m = N.ridged(x * 0.006 + seed, seed, 4);
                const top = base - off - Math.pow(m, 2.2) * amp;
                for (let y = Math.max(0, Math.floor(top)); y < base; y++) {
                    const k = y - top < 1 ? 3 : (N.n2(x * 0.15, y * 0.05 + seed) > 0.35 && y - top > 6) ? 1 : 2; // ridge light + basalt columns
                    p.px(x, y, R[k]);
                }
            }
        }
        for (let y = base; y < H; y++) for (let x = 0; x < LAYER_W; x++) {
            const t = (y - base) / (H - base);
            p.px(x, y, t < 0.15 ? '#173763' : t < 0.5 ? P.waterMid : P.waterDeep);
            if (h01(x, y, 2) > 0.985 - t * 0.02) p.px(x, y, P.glint);
        }
        A.mountains = atlas.add('m:mtn', p); A.mountainsH = H; A.mountainsBase = base;
    }
    // ghost ship silhouette
    {
        const p = new Pix(40, 30);
        for (let x = 4; x < 36; x++) { const d = Math.round(Math.sin((x - 4) / 32 * Math.PI) * 3); for (let y = 22; y < 25 + d; y++) p.px(x, y, '#2E4A4C'); }
        for (const mx of [14, 26]) { p.vline(mx, 4, 22, '#3E5A5A'); for (let y = 6; y < 18; y++) for (let s = -6; s <= 5; s++) if (h01(s, y, mx) > 0.25) p.px(mx + s + Math.round(Math.sin(y * 0.3)), y, '#5A8A84'); }
        p.lit(35, 21, P.soul, 1.2); p.lit(14, 3, P.soul, 1); p.lit(26, 3, P.soul, 1);
        A.ghostShip = atlas.add('m:gship', p);
    }
    // 5: soul-tree island
    {
        const W = 170, H = 150, p = new Pix(W, H);
        const cx = W / 2, sea = H - 20;
        for (let y = sea - 26; y < sea + 4; y++) for (let x = 0; x < W; x++) { const w = 70 - (sea - y) * 1.4 + N.fbm(x * 0.1, y * 0.1, 2, 0) * 10; if (Math.abs(x - cx) < w) p.px(x, y, y > sea ? '#16232E' : (y - (sea - 26) < 2 ? '#34473A' : ramp(P.basalt)[x < cx ? 1 : 0])); }
        for (let y = sea; y < H; y++) for (let x = 20; x < W - 20; x++) if (h01(x, y) > 0.6) p.px(x, y, '#2E4A6A'); // reflection
        // trunk + branches
        for (let y = 40; y < sea - 22; y++) { const w = 3 + Math.max(0, y - (sea - 40)) * 0.35; for (let k = -w; k <= w; k++) p.px(cx + k + Math.sin(y * 0.1) * 2, y, k < -w / 3 ? '#2A2A30' : '#141418'); }
        for (const [a, l] of [[-2.5, 30], [-0.6, 32], [-1.6, 26], [-2.9, 20], [-0.2, 20]]) p.line(cx, 58, cx + Math.cos(a) * l, 58 + Math.sin(a) * l * 0.8, '#141418');
        // mist crown
        for (let y = 0; y < 70; y++) for (let x = 0; x < W; x++) {
            const dx = (x - cx) / 62, dy = (y - 36) / 32, d = dx * dx + dy * dy + N.fbm(x * 0.08, y * 0.08, 3, 0) * 0.5;
            if (d > 1) continue;
            if (BAYER[(y & 3) * 4 + (x & 3)] > 1.15 - d) continue;
            p.px(x, y, d < 0.25 ? '#6FB8A6' : d < 0.6 ? '#3E7470' : P.mistBlue);
            p.glow(x, y, P.soul, Math.max(0, 0.6 - d) * 0.22);
        }
        for (let i = 0; i < 8; i++) { const x = 30 + i * 15, y0 = 56 + (i % 3) * 4; for (let y = y0; y < y0 + 10 + (i % 2) * 8; y++) p.px(x, y, P.kelp); p.lit(x, y0 + 12 + (i % 2) * 8, P.soul, 1.4); }
        A.island = atlas.add('m:island', p); A.islandW = W; A.islandH = H; A.islandSea = sea;
    }
    // 6: the plain with fortress Schwarzwasser, rivers and the lighthouse
    {
        const W = LAYER_W, H = 150, p = new Pix(W, H);
        const bands = ['#1E2E36', '#1A2A2C', '#172420', '#131E1A', '#0F1814'];
        for (let y = 0; y < H; y++) for (let x = 0; x < W; x++) {
            const top = 48 + N.fbm(x * 0.012, 3, 3, 0) * 6;
            if (y < top) continue;
            const t = (y - top) / (H - top) * (bands.length - 1) + (BAYER[(y & 3) * 4 + (x & 3)] - 0.5) * 0.9;
            p.px(x, y, bands[Math.max(0, Math.min(bands.length - 1, Math.round(t)))]);
            if (y - top < 1) p.px(x, y, '#2E4048');
            else if (h01(x, y, 7) > 0.985) p.px(x, y, '#2A3A30');
        }
        // rivers: two winding glints
        for (const [x0, ph] of [[110, 0], [350, 2]]) for (let y = 54; y < H; y++) { const x = x0 + Math.sin(y * 0.07 + ph) * 18 + (y - 54) * 0.6; for (let k = -(y - 54) / 30; k <= (y - 54) / 30; k++) p.px(x + k, y, y % 3 ? '#173763' : '#22508A'); if (y % 5 === 0) p.px(x, y, P.glint); }
        // fortress: walls, towers, keep
        const fx = 196, fy = 22;
        const S = ramp('#141C24');
        p.rect(fx - 30, fy + 8, 120, 26, S[1]);
        for (let x = fx - 30; x < fx + 90; x += 4) p.rect(x, fy + 5, 2, 3, S[1]);
        for (const [tx, th] of [[fx - 34, 30], [fx + 4, 44], [fx + 30, 56], [fx + 88, 34]]) {
            p.rect(tx, fy + 34 - th, 10, th, S[2]); p.vline(tx, fy + 34 - th, fy + 34, S[3]);
            for (let k = 0; k < 6; k++) p.px(tx - 1 + k * 2, fy + 32 - th - (k % 2), S[2]);
            p.line(tx - 1, fy + 33 - th, tx + 5, fy + 25 - th, S[1]); p.line(tx + 5, fy + 25 - th, tx + 11, fy + 33 - th, S[0]);
        }
        A.windows = [];
        for (let i = 0; i < 14; i++) { const x = fx - 28 + Math.floor(h01(i, 1) * 116), y = fy + 12 + Math.floor(h01(i, 2) * 18); p.px(x, y, '#2A1A10'); A.windows.push([x, y]); }
        // lighthouse on the right cliff
        const lx = 440, ly = 8;
        for (let y = ly + 10; y < ly + 46; y++) { const w = 3 + (y - ly) * 0.06; for (let k = -w; k <= w; k++) p.px(lx + k, y, (Math.floor((y - ly) / 6) % 2) ? '#5A5450' : '#2A2626'); }
        p.rect(lx - 4, ly + 4, 9, 6, '#2A2626'); p.lit(lx, ly + 7, '#FFE0A0', 2); p.lit(lx - 1, ly + 7, P.lantern, 1.5); p.lit(lx + 1, ly + 7, P.lantern, 1.5);
        A.plain = atlas.add('m:plain', p); A.plainH = H; A.lighthouse = [lx, ly + 7];
    }
    // 7: fog band
    {
        const W = LAYER_W, H = 46, p = new Pix(W, H);
        for (let y = 0; y < H; y++) for (let x = 0; x < W; x++) {
            const n = N.fbm(x * 0.02, y * 0.08, 3, 0) + 0.25 - Math.abs(y / H - 0.5) * 1.4;
            const q = Math.floor(Math.max(0, n) * 4) / 4; // alpha quantized to 4 steps via dither density
            if (q <= 0 || BAYER[(y & 3) * 4 + (x & 3)] > q * 1.6) continue;
            p.px(x, y, '#8FA3A8');
        }
        A.fog = atlas.add('m:fog', p); A.fogH = H;
    }
    // 8: foreground silhouettes (dead tree, grass, rocks, broken mast with a rag)
    {
        const W = LAYER_W, H = 130, p = new Pix(W, H), C = '#070A0E', C2 = '#0E141A';
        for (let x = 0; x < W; x++) { const top = H - 18 - Math.abs(N.fbm(x * 0.02, 9, 3, 0)) * 26 - (x < 120 ? 14 : 0) - (x > 420 ? 10 : 0); for (let y = Math.floor(top); y < H; y++) p.px(x, y, y - top < 1 ? C2 : C); }
        for (let i = 0; i < 90; i++) { const x = Math.floor(h01(i, 4) * W), hgt = 4 + Math.floor(h01(i, 5) * 10); let top = 0; for (let y = 0; y < H; y++) if (p.a(x, y)) { top = y; break; } for (let y = top - hgt; y < top; y++) p.px(x + Math.round((top - y) * (h01(i, 6) - 0.5) * 0.4), y, C2); }
        // dead tree
        const tx = 54; for (let y = 20; y < 112; y++) { const w = 3 + Math.max(0, y - 90) * 0.3; for (let k = -w; k <= w; k++) p.px(tx + k + Math.sin(y * 0.08) * 2, y, C); }
        for (const [a, l, y0] of [[-2.4, 40, 40], [-0.7, 36, 34], [-2.0, 26, 60], [-0.4, 30, 70], [-1.5, 24, 24]]) { p.line(tx, y0, tx + Math.cos(a) * l, y0 + Math.sin(a) * l * 0.7, C); p.line(tx + 1, y0, tx + 1 + Math.cos(a) * l, y0 + Math.sin(a) * l * 0.7, C); }
        for (let i = 0; i < 5; i++) { const x = tx - 30 + i * 13, y0 = 36 + (i % 2) * 8; for (let y = y0; y < y0 + 12 + i * 2; y++) p.px(x + Math.round(Math.sin(y * 0.5)), y, C2); }
        // broken mast + rag
        const mx = 452; p.line(mx, 120, mx + 8, 30, C); p.line(mx + 1, 120, mx + 9, 30, C); p.line(mx - 12, 44, mx + 26, 40, C);
        for (let y = 44; y < 66; y++) for (let x = mx + 10; x < mx + 22 - (y - 44) * 0.3; x++) if (h01(x, y) > 0.2) p.px(x, y, '#1A0A0E');
        A.fore = atlas.add('m:fore', p); A.foreH = H;
    }
    // rune ring dot + lighthouse beam texel use the white texel; rain drop
    { const p = new Pix(3, 5); p.px(2, 0, [255, 255, 255]); p.px(1, 2, [255, 255, 255]); p.px(1, 3, [255, 255, 255]); p.px(0, 4, [255, 255, 255]); A.rain = atlas.add('m:rain', p); }
    return A;
}

// ------------------------------------------------------------------ bright panorama (the look of the reference)
export function buildMenuArtBright(atlas) {
    const A = {};
    { const c = clouds(960, 80, 5, '#E4EEF8', '#A8C0DC', '#FFFFFF', 0.05); A.cloudsFar = atlas.add('mb:cf', c.p); A.cloudsFarEdge = atlas.add('mb:cfe', c.e); }
    { const c = clouds(960, 64, 13, '#F4F8FC', '#B8CCE4', '#FFFFFF', 0.02); A.cloudsNear = atlas.add('mb:cn', c.p); A.cloudsNearEdge = atlas.add('mb:cne', c.e); }
    // blue mountains on the left, a forest band and the sea on the right
    {
        const H = 120, p = new Pix(LAYER_W, H), base = H - 30;
        for (const [amp, off, col, seed, from, to] of [[80, 6, '#7FA4CE', 2, 0, 220], [56, 0, '#5A86B8', 6, 0, 170]]) {
            const R = ramp(col, 0.8);
            for (let x = from; x < to; x++) {
                const fall = Math.max(0, (x - (to - 90)) / 90);
                const top = base - off - Math.pow(N.ridged(x * 0.008 + seed, seed, 4), 1.6) * amp * (1 - fall);
                for (let y = Math.max(0, Math.floor(top)); y < base; y++) p.px(x, y, R[y - top < 2 ? 4 : (N.n2(x * 0.08, y * 0.08) > 0.2 ? 3 : 2)]);
            }
        }
        for (let x = 0; x < LAYER_W; x++) { const top = base - 6 - Math.abs(N.fbm(x * 0.06, 2, 2, 0)) * 8; for (let y = Math.floor(top); y < base; y++) p.px(x, y, (x * 7 + y * 3) % 11 === 0 ? '#3E7A5A' : '#4E8A6A'); }
        for (let y = base; y < H; y++) for (let x = 0; x < LAYER_W; x++) { const t = (y - base) / (H - base); p.px(x, y, x < 300 ? '#5AA048' : t < 0.2 ? '#6EB8E0' : '#3E8EC8'); if (x >= 300 && h01(x, y, 2) > 0.97) p.px(x, y, '#C8EEF8'); }
        A.mountains = atlas.add('mb:mtn', p); A.mountainsH = H; A.mountainsBase = base;
    }
    { const p = new Pix(40, 30); for (let x = 4; x < 36; x++) { const d = Math.round(Math.sin((x - 4) / 32 * Math.PI) * 3); for (let y = 22; y < 25 + d; y++) p.px(x, y, '#7A5A3E'); } for (const mx of [14, 26]) { p.vline(mx, 4, 22, '#4A3326'); for (let y = 6; y < 18; y++) for (let s = -6; s <= 5; s++) p.px(mx + s + Math.round(Math.sin(y * 0.3)), y, y % 4 ? '#F4F0E4' : '#D8D0BC'); } A.ghostShip = atlas.add('mb:gship', p); }
    // golden hill with a white terraced temple and the great tree
    {
        const W = 200, H = 176, p = new Pix(W, H), cx = W / 2, sea = H - 18;
        const GOLD = ramp('#C8A84A');
        // rounded golden hill made of foliage clusters
        clusterCrown(p, cx, sea - 30, 96, 36, [GOLD[0], GOLD[1], GOLD[2], GOLD[3], GOLD[4]], 21, 520, 4);
        for (let y = sea - 20; y < sea; y++) for (let x = 0; x < W; x++) { const w = 96 - (sea - y) * 0.6; if (Math.abs(x - cx) < w && !p.a(x, y)) p.px(x, y, GOLD[1]); }
        // white terraced temple
        for (const [w, y] of [[16, 86], [24, 102], [32, 118]]) { p.rect(cx - w, y, w * 2, 12, '#E8F0F4'); p.hline(cx - w, cx + w - 1, y, '#FFFFFF'); for (let x = cx - w + 3; x < cx + w - 2; x += 5) p.rect(x, y + 3, 2, 8, '#7AB4D8'); p.hline(cx - w, cx + w - 1, y + 11, '#9AB4C4'); p.vline(cx - w, y, y + 11, '#FFFFFF'); }
        p.rect(cx - 5, 124, 10, 6, '#5A86B8');
        // the great tree with a white and pink crown
        for (let y = 34; y < 88; y++) { const w = 4 + Math.max(0, y - 72) * 0.5; for (let k = -w; k <= w; k++) p.px(cx + k + Math.sin(y * 0.1) * 2, y, k < -w / 3 ? '#4A5878' : '#2E3A58'); }
        for (const [a, l] of [[-2.5, 30], [-0.6, 32], [-1.6, 24]]) p.line(cx, 50, cx + Math.cos(a) * l, 50 + Math.sin(a) * l * 0.8, '#2E3A58');
        clusterCrown(p, cx, 30, 46, 28, FOL.blossom, 33, 150, 6);
        wisteria(p, cx - 40, cx + 40, 44, FOL.pink, 3, 16, 16);
        A.island = atlas.add('mb:island', p); A.islandW = W; A.islandH = H; A.islandSea = sea;
    }
    // green plain with meadows, rivers and a walled town with teal roofs
    {
        const W = LAYER_W, H = 150, p = new Pix(W, H);
        const greens = ['#9AD86A', '#7CC85A', '#62B04A', '#4E9A44', '#3E8040'];
        for (let y = 0; y < H; y++) for (let x = 0; x < W; x++) {
            const top = 46 + N.fbm(x * 0.012, 3, 3, 0) * 6;
            if (y < top) continue;
            const t = (y - top) / (H - top) * 3 + N.fbm(x * 0.03, y * 0.06, 3, 0) * 1.6 + (BAYER[(y & 3) * 4 + (x & 3)] - 0.5) * 0.7;
            p.px(x, y, greens[Math.max(0, Math.min(4, Math.round(t)))]);
            if (N.fbm(x * 0.02 + 9, y * 0.05, 2, 0) > 0.3 && y > top + 10) p.px(x, y, (x + y) % 3 ? '#E8D86A' : '#D4C454');
        }
        for (const [x0, ph] of [[150, 0], [330, 2]]) for (let y = 52; y < H; y++) { const x = x0 + Math.sin(y * 0.06 + ph) * 22 + (y - 52) * 0.5; const w = 1 + (y - 52) / 22; for (let k = -w; k <= w; k++) p.px(x + k, y, Math.abs(k) > w - 1 ? '#6EB8E0' : '#3E8EC8'); }
        const fx = 236, fy = 70;
        p.rect(fx - 40, fy + 6, 110, 22, '#8C9098'); p.hline(fx - 40, fx + 69, fy + 6, '#B8BCC2');
        for (let x = fx - 40; x < fx + 70; x += 4) p.rect(x, fy + 3, 2, 3, '#8C9098');
        for (let i = 0; i < 10; i++) { const hx = fx - 34 + i * 10, hy = fy - 2 - (i % 3) * 3; p.rect(hx, hy + 4, 8, 6, '#E8E0CC'); for (let k = 0; k < 5; k++) p.hline(hx - 1 + k, hx + 8 - k, hy + 4 - k, k % 2 ? '#2A8A90' : '#46AEAE'); }
        p.rect(fx + 10, fy + 16, 8, 12, '#3A3E46');
        A.plain = atlas.add('mb:plain', p); A.plainH = H; A.lighthouse = [460, 20]; A.windows = [];
    }
    { const W = LAYER_W, H = 46, p = new Pix(W, H); for (let y = 0; y < H; y++) for (let x = 0; x < W; x++) { const n = N.fbm(x * 0.02, y * 0.08, 3, 0) + 0.2 - Math.abs(y / H - 0.5) * 1.4; const q = Math.floor(Math.max(0, n) * 4) / 4; if (q <= 0 || BAYER[(y & 3) * 4 + (x & 3)] > q * 1.6) continue; p.px(x, y, '#F4F8FC'); } A.fog = atlas.add('mb:fog', p); A.fogH = H; }
    // foreground: lush bushes and a blossom tree edge
    {
        const W = LAYER_W, H = 130, p = new Pix(W, H);
        const G2 = ramp('#3E8040');
        for (let x = 0; x < W; x++) { const top = H - 14 - Math.abs(N.fbm(x * 0.03, 9, 3, 0)) * 22 - (x < 90 ? 18 : 0) - (x > 440 ? 14 : 0); for (let y = Math.floor(top); y < H; y++) p.px(x, y, G2[y - top < 2 ? 3 : y - top < 6 ? 2 : 1]); }
        for (let i = 0; i < 22; i++) { const x = h01(i, 3) * W, y = H - 16 - h01(i, 4) * 16; clusterCrown(p, x, y, 12 + h01(i, 5) * 8, 8 + h01(i, 6) * 4, [FOL.green, FOL.teal, FOL.blue][i % 3], i, 14, 5); }
        clusterCrown(p, 40, 36, 40, 30, FOL.pink, 8, 60, 6); wisteria(p, 6, 76, 50, FOL.violet, 2, 10, 18);
        A.fore = atlas.add('mb:fore', p); A.foreH = H;
    }
    A.moon = null; A.rain = null; A.bright = true;
    return A;
}
