// HUD and menu sprites: brass health frame with an anchor hook, portholes, item icons,
// compass arrow, saber-and-anchor emblem, bevelled logo lettering, soft shadows, light blobs.
import { Pix, BAYER } from './pix.js';
import { P, ramp } from './pal.js';

const BR = ramp(P.lantern), BRD = ramp(P.oldwood);
function brassRect(p, x, y, w, h) {
    for (let i = 0; i < w; i++) { p.px(x + i, y, BR[3]); p.px(x + i, y + h - 1, BRD[1]); }
    for (let j = 0; j < h; j++) { p.px(x, y + j, BR[2]); p.px(x + w - 1, y + j, BRD[1]); }
    for (let i = 1; i < w - 1; i++) { p.px(x + i, y + 1, BRD[2]); p.px(x + i, y + h - 2, BRD[0]); }
    for (let j = 1; j < h - 1; j++) { p.px(x + 1, y + j, BRD[2]); p.px(x + w - 2, y + j, BRD[0]); }
}
function anchor(p, x, y, col = BR, s = 1) {
    p.vline(x, y + 2 * s, y + 10 * s, col[2]); p.vline(x + 1, y + 2 * s, y + 10 * s, col[1]);
    p.ellipse(x + 0.5, y + 1 * s, 1.8 * s, 1.8 * s, col[3]); p.px(x, y + s, P.abyss);
    p.hline(x - 3 * s, x + 4 * s, y + 4 * s, col[3]);
    for (let k = 0; k <= 4 * s; k++) { p.px(x - k, y + 10 * s - Math.round(Math.sqrt(k) * 1.4), col[2]); p.px(x + 1 + k, y + 10 * s - Math.round(Math.sqrt(k) * 1.4), col[1]); }
    p.px(x - 4 * s, y + 7 * s, col[4]); p.px(x + 5 * s, y + 7 * s, col[4]);
}
function porthole(p, d) {
    const c = (d - 1) / 2;
    for (let y = 0; y < d; y++) for (let x = 0; x < d; x++) {
        const r = Math.hypot(x - c, y - c);
        if (r > c + 0.5) continue;
        if (r > c - 2.5) { const l = -(x - c) - (y - c); p.px(x, y, l > 2 ? BR[3] : l > -2 ? BR[2] : BRD[1]); if ((Math.round(Math.atan2(y - c, x - c) * 4) % 3 === 0) && r > c - 1.5) p.px(x, y, BRD[0]); }
        else p.px(x, y, r < c - 4 ? '#0E161E' : '#16232E');
    }
}
function icon(name) {
    const p = new Pix(16, 16);
    switch (name) {
        case 'saber': for (let t = 0; t < 11; t++) { const x = 3 + t, y = 13 - t + Math.round((t / 11) * (t / 11) * 2); p.px(x, y, '#C8D0D8'); p.px(x, y + 1, '#6A7280'); } p.line(1, 12, 5, 16, P.gold); p.px(2, 14, P.tarwood); p.px(1, 15, P.tarwood); break;
        case 'pistol': p.rect(3, 6, 10, 3, '#6A7280'); p.hline(3, 12, 6, '#B8C0C8'); p.rect(2, 8, 4, 6, P.tarwood); p.px(2, 13, P.oldwood); p.px(7, 9, '#3A3E44'); p.px(12, 5, '#B8C0C8'); break;
        case 'rum': p.rect(5, 6, 6, 9, '#3A5A3A'); p.rect(6, 3, 4, 3, '#3A5A3A'); p.rect(6, 1, 4, 2, P.oldwood); p.rect(5, 9, 6, 3, P.bone); p.px(7, 10, P.blood); p.vline(6, 7, 13, '#5A8A5A'); break;
        case 'phial': p.rect(6, 1, 4, 3, P.oldwood); p.ellipse(8, 10, 5, 5, '#16232E'); for (let y = 7; y < 15; y++) for (let x = 4; x < 13; x++) if (Math.hypot(x - 8, y - 10) < 4) p.lit(x, y, y < 9 ? P.soulLight : P.soul, 1.1); p.rect(7, 4, 2, 2, '#4A6170'); break;
        case 'bomb': p.blob(8, 10, 5, 5, '#2A2E34'); p.line(10, 5, 12, 2, P.oldwood); p.lit(13, 1, P.lantern, 1.4); p.lit(12, 1, P.ember, 1); break;
        case 'bullets': for (const [x, y] of [[4, 6], [8, 9], [11, 5]]) { p.ellipse(x, y, 2, 2, '#8A9098'); p.px(x - 1, y - 1, '#D8DCE0'); } break;
        case 'fireball': for (let y = 2; y < 15; y++) for (let x = 2; x < 15; x++) { const r = Math.hypot(x - 9, y - 9) + (x < 9 ? (9 - x) * 0.3 : 0); if (r < 5.5) p.lit(x, y, r < 2.5 ? '#FFE0A0' : r < 4 ? P.lantern : P.ember, 1.3); } break;
        case 'bolt': p.line(10, 1, 6, 8, '#DFF7FF'); p.line(6, 8, 10, 8, '#DFF7FF'); p.line(10, 8, 5, 15, '#DFF7FF'); for (let y = 1; y < 16; y++) for (let x = 0; x < 16; x++) if (p.a(x, y)) p.glow(x, y, P.soulLight, 1.3); break;
        case 'hammer': p.line(3, 13, 10, 6, P.tarwood); p.rect(9, 2, 6, 5, '#6A7280'); p.hline(9, 14, 2, '#B8C0C8'); break;
        case 'rapier': p.line(2, 14, 14, 2, '#C8D0D8'); p.ellipse(4, 12, 2, 2, P.gold); break;
        case 'dagger': p.line(4, 12, 11, 5, '#C8D0D8'); p.line(2, 11, 5, 14, P.gold); p.px(3, 13, P.tarwood); break;
        case 'club': p.line(3, 13, 12, 4, P.tarwood); p.line(4, 13, 13, 4, P.oldwood); p.ellipse(12, 4, 2.5, 2.5, P.oldwood); break;
        case 'fetish': p.line(4, 14, 10, 6, P.deadwood); p.ellipse(11, 5, 3.5, 3, P.bone); p.lit(11, 5, P.soul, 1.3); break;
        case 'tide': for (let x = 2; x < 15; x++) { const y = 9 + Math.round(Math.sin(x * 0.8) * 2); p.lit(x, y, P.soul, 0.9); p.px(x, y + 1, '#22508A'); } break;
        case 'pearl': p.ellipse(8, 8, 3.5, 3.5, '#C8D0D8'); p.px(7, 6, '#FFFFFF'); for (let y = 4; y < 13; y++) for (let x = 4; x < 13; x++) if (p.a(x, y)) p.glow(x, y, P.soulLight, 0.5); break;
        case 'figurehead': p.blob(8, 6, 4, 4, P.bone); p.blob(8, 12, 4, 3, P.bone); p.lit(7, 5, P.soul, 1.2); p.lit(9, 5, P.soul, 1.2); break;
        case 'key': p.ellipse(5, 6, 3, 3, P.gold); p.px(5, 6, P.abyss); p.hline(7, 14, 6, P.gold); p.vline(12, 6, 9, P.gold); p.vline(14, 6, 8, P.gold); break;
        case 'shard': p.line(8, 1, 12, 8, P.soulLight); p.line(12, 8, 8, 15, P.soul); p.line(8, 15, 4, 8, P.soul); p.line(4, 8, 8, 1, P.soulLight); for (let y = 0; y < 16; y++) for (let x = 0; x < 16; x++) if (p.a(x, y)) p.glow(x, y, P.soul, 1); break;
        case 'coat': p.rect(4, 3, 8, 11, '#5A2A30'); p.rect(2, 4, 2, 7, '#5A2A30'); p.rect(12, 4, 2, 7, '#5A2A30'); p.vline(8, 3, 13, P.bone); break;
        case 'ring': p.ellipse(8, 9, 4, 4, P.gold); p.ellipse(8, 9, 2, 2, [0, 0, 0]); for (let y = 0; y < 16; y++) for (let x = 0; x < 16; x++) { const c = p.get(x, y); if (c && c[3] && c[0] === 0 && c[1] === 0 && c[2] === 0) p.clear(x, y); } p.lit(8, 4, P.blood, 1.2); break;
        case 'shanty': p.vline(10, 2, 11, P.bone); p.ellipse(8, 12, 2.5, 2, P.bone); p.line(10, 2, 13, 5, P.bone); break;
    }
    p.outline();
    return p;
}

export function buildUI(atlas) {
    const U = {};
    // health bar frame: 140×8 inner, brass frame and an anchor hook on the left
    { const p = new Pix(158, 16); brassRect(p, 12, 2, 144, 12); anchor(p, 5, 2); U.hpFrame = atlas.add('ui:hp', p); }
    { const p = new Pix(108, 10); brassRect(p, 0, 0, 108, 10); U.mpFrame = atlas.add('ui:mp', p); }
    { const p = new Pix(22, 22); porthole(p, 22); U.porthole = atlas.add('ui:port', p); }
    { const p = new Pix(26, 26); porthole(p, 26); U.porthole26 = atlas.add('ui:port26', p); }
    { const p = new Pix(16, 16); porthole(p, 16); U.porthole16 = atlas.add('ui:port16', p); }
    for (const n of ['saber', 'pistol', 'rum', 'phial', 'bomb', 'bullets', 'fireball', 'bolt', 'hammer', 'rapier', 'dagger', 'club', 'fetish', 'tide', 'pearl', 'figurehead', 'key', 'shard', 'coat', 'ring', 'shanty']) U['i_' + n] = atlas.add('ui:i:' + n, icon(n));
    // compass arrow, 4 frames (spinning needle glint)
    U.compass = [];
    for (let f = 0; f < 4; f++) {
        const p = new Pix(9, 9);
        p.line(1, 4, 7, 4, BR[2]); p.line(5, 2, 7, 4, BR[3]); p.line(5, 6, 7, 4, BR[1]);
        p.px(1 + f * 2, 4, '#FFF0C0'); p.glow(1 + f * 2, 4, P.lantern, 0.8);
        U.compass.push(atlas.add('ui:comp:' + f, p));
    }
    // emblem: crossed saber and anchor
    {
        const p = new Pix(34, 34);
        anchor(p, 16, 3, ramp(P.bone), 2.6);
        for (let t = 0; t < 28; t++) { const x = 4 + t, y = 30 - t + Math.round((t / 28) * (t / 28) * 4); p.px(x, y, '#C8D0D8'); p.px(x, y + 1, '#6A7280'); }
        p.line(2, 26, 9, 33, P.gold); p.px(4, 30, P.tarwood); p.px(3, 31, P.tarwood);
        p.lit(16, 5, P.soul, 1);
        p.outline();
        U.emblem = atlas.add('ui:emblem', p);
    }
    // soft ellipse used for shadows (multiplied) and light blobs
    for (const [k, rx, ry] of [['shadowS', 6, 2], ['shadowM', 10, 3], ['shadowL', 18, 5], ['shadowXL', 34, 8], ['shadowXXL', 60, 10]]) {
        const p = new Pix(rx * 2 + 2, ry * 2 + 2);
        for (let y = 0; y < p.h; y++) for (let x = 0; x < p.w; x++) {
            const d = Math.hypot((x + 0.5 - p.w / 2) / rx, (y + 0.5 - p.h / 2) / ry);
            if (d > 1) continue;
            if (d > 0.7 && BAYER[(y & 3) * 4 + (x & 3)] < (d - 0.7) / 0.3) continue; // dithered rim
            p.px(x, y, [255, 255, 255]);
        }
        U[k] = atlas.add('ui:' + k, p);
    }
    return U;
}

// bevelled logo lettering: every font pixel becomes a 2×2 cell lit from the top-left
export function buildLogo(atlas, font, text, key = 'logo') {
    const W = font.width(text) * 2 + 4, H = 20;
    const p = new Pix(W, H);
    const src = atlas.img;
    let cx = 1;
    const LR = ramp('#C9B48A');
    for (const ch of text) {
        const g = font.glyphs[ch] || font.glyphs['?'];
        if (g.rect) for (let y = 0; y < 8; y++) for (let x = 0; x < g.w; x++) {
            const i = ((g.rect.y + y) * src.w + g.rect.x + x) * 4;
            if (!src.col[i + 3]) continue;
            const X = cx + x * 2, Y = 1 + y * 2;
            const t = y / 7;
            const base = t < 0.3 ? 4 : t < 0.6 ? 3 : 2;
            p.px(X, Y, LR[Math.min(4, base)]); p.px(X + 1, Y, LR[base - 1]); p.px(X, Y + 1, LR[base - 1]); p.px(X + 1, Y + 1, LR[Math.max(0, base - 2)]);
        }
        cx += (g.w + 1) * 2;
    }
    p.outline();
    // second, darker outline pass for weight
    const q = new Pix(W + 2, H + 2); q.blit(p, 1, 1); q.outline();
    return atlas.add(key, q);
}
