// Main-menu panorama (480×270 view, layers 512 px wide for the parallax drift):
// clouds, basalt mountains with the sea horizon, the soul-tree island, the plain with fortress
// Schwarzwasser and its lighthouse, a fog band and foreground silhouettes.
import { Pix, BAYER } from './pix.js';
import { P, ramp } from './pal.js';
import { Noise } from '../gfx/noise.js';

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
