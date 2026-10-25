// Creature sprites: crabs, shell crawlers, sharks, the four phase-2 monsters of the kings
// (Chimera, Scylla, Harpy Aello, Talos of coral stone), the burning skeleton megalodon and ships.
// Beasts are drawn in a side view; the game flips them when they face left.
import { Pix, BAYER } from './pix.js';
import { P, ramp, shade } from './pal.js';

const h01 = (x, y, s = 0) => { const v = Math.sin(x * 127.1 + y * 311.7 + s * 74.7) * 43758.5453; return v - Math.floor(v); };
export const BEAST_FRAMES = { idle: 2, walk: 4, atk: 2, cast: 1, hurt: 1, dead: 1 };

function legs(p, xs, y0, len, f, col, phase = 0) {
    xs.forEach((x, i) => {
        const s = Math.round(Math.sin((f + i + phase) * Math.PI / 2) * 1.5);
        p.line(x, y0, x + s - 1, y0 + len, col); p.px(x + s - 2, y0 + len, col);
    });
}
// ---------------------------------------------------------------- small creatures
function crab(p, anim, f, big) {
    const W = p.w, cx = W / 2, by = p.h - (big ? 9 : 6);
    const shell = big ? '#7A2A22' : '#8A3A28';
    const R = ramp(shell);
    const step = anim === 'walk' ? f : anim === 'idle' ? f * 2 : 0;
    legs(p, [cx - 8, cx - 5, cx - 2].map(x => x - (big ? 3 : 0)), by, big ? 6 : 4, step, R[0]);
    legs(p, [cx + 2, cx + 5, cx + 8].map(x => x + (big ? 3 : 0)), by, big ? 6 : 4, step, R[0], 1);
    if (anim === 'dead') { p.blob(cx, by + 3, big ? 11 : 8, big ? 4 : 3, shade(R[2], 0.8).map(v => v) && shell); for (let x = cx - 8; x < cx + 8; x += 3) p.px(x, by + 6, P.bone); return; }
    p.blob(cx, by - (big ? 4 : 2), big ? 11 : 8, big ? 7 : 5, shell);
    for (let x = cx - 5; x <= cx + 5; x += 2) p.px(x, by - (big ? 9 : 5), R[4]);
    // eyes on stalks
    for (const ex of [cx - 3, cx + 3]) { p.vline(ex, by - (big ? 13 : 9), by - (big ? 10 : 7), R[1]); p.lit(ex, by - (big ? 14 : 10), P.soul, 1.1); }
    // claws: raised when attacking
    const up = anim === 'atk' ? (f ? -2 : -7) : anim === 'cast' ? -8 : 0;
    for (const s of [-1, 1]) {
        const x = cx + s * (big ? 14 : 10), y = by - (big ? 6 : 3) + up;
        p.blob(x, y, big ? 5 : 3.5, big ? 4 : 3, shell);
        p.px(x + s * 2, y - 2, R[4]); p.px(x + s * 3, y - 1, P.abyss);
        if (big) { p.line(cx + s * 9, by - 4, x - s * 2, y + 2, R[1]); }
    }
    if (big) { // barnacle armour + a rusty helmet plate
        for (let i = 0; i < 6; i++) p.px(cx - 8 + h01(i, 1) * 16, by - 8 + h01(i, 2) * 6, P.bone);
        p.hline(cx - 6, cx + 6, by - 11, '#5A6068'); p.hline(cx - 5, cx + 5, by - 12, '#8A9098');
    }
}
function shellCrawler(p, anim, f) {
    const cx = p.w / 2, by = p.h - 4;
    const R = ramp(P.coral);
    const st = anim === 'walk' ? f : 0;
    if (anim === 'dead') { p.blob(cx, by - 2, 9, 4, P.coral); return; }
    for (let i = 0; i < 4; i++) p.px(cx - 8 + i * 5 + (st + i) % 2, by + 1, R[0]);
    // spiral shell
    p.blob(cx - 1, by - 6 - (anim === 'atk' && f ? 2 : 0), 9, 7, P.coral);
    for (let a = 0; a < 14; a++) { const r = 6 - a * 0.4, ang = a * 0.8; p.px(cx - 1 + Math.cos(ang) * r, by - 6 + Math.sin(ang) * r * 0.8, R[0]); }
    // soft body with luminous spots
    const hx = anim === 'atk' && f ? cx + 11 : cx + 8;
    p.blob(hx, by - 2, 4, 3, '#8A7A9A');
    p.lit(hx + 2, by - 3, P.soul, 1.2);
    for (let i = 0; i < 4; i++) p.lit(cx - 6 + i * 4, by - 10 + (i % 2) * 3, '#C88AD0', 0.6);
}
function shark(p, anim, f) {
    const W = p.w, cy = p.h / 2 + 2;
    const R = ramp('#3A4A58');
    const sw = anim === 'atk' ? 0 : Math.sin(f * Math.PI / 2) * 2;
    for (let x = 4; x < W - 4; x++) {
        const t = (x - 4) / (W - 8);
        const hh = Math.sin(Math.min(1, t * 1.15) * Math.PI) * 6 + 1;
        const yy = cy + Math.sin(t * 3 + f) * (1 - t) * sw * 0.4;
        for (let y = -hh; y <= hh * 0.7; y++) p.px(x, yy + y, y > hh * 0.2 ? P.bone : R[y < -hh * 0.5 ? 3 : 2]);
    }
    p.line(W * 0.45, cy - 6, W * 0.55, cy - 13, R[2]); p.line(W * 0.55, cy - 13, W * 0.62, cy - 6, R[1]); // dorsal fin
    p.line(6, cy - 1, 1, cy - 7, R[2]); p.line(6, cy + 1, 1, cy + 5, R[1]); // tail
    p.lit(W - 12, cy - 2, P.soul, 1.2);
    if (anim === 'atk') for (let x = W - 10; x < W - 4; x += 2) { p.px(x, cy + 2, P.bone); p.px(x + 1, cy + 3, P.abyss); }
}

// ---------------------------------------------------------------- phase-2 monsters
// Chimera (fire): lion body with an ember mane, a goat head rising from the back, a serpent tail
function chimera(p, anim, f) {
    const W = p.w, H = p.h, gy = H - 4;
    const fur = ramp('#5A3A2A'), mane = P.ember;
    const lunge = anim === 'atk' ? (f ? 8 : -4) : 0, breath = anim === 'idle' ? f : 0;
    const bx = W / 2 + lunge;
    if (anim === 'dead') { p.blob(bx, gy - 8, 30, 9, '#5A3A2A'); for (let i = 0; i < 12; i++) p.lit(bx - 25 + i * 4, gy - 14 + (i % 3), P.ember, 0.4); return; }
    // legs
    for (const [lx, ph] of [[bx - 20, 0], [bx - 12, 1], [bx + 12, 1], [bx + 20, 0]]) {
        const s = anim === 'walk' ? Math.round(Math.sin((f + ph * 2) * Math.PI / 2) * 3) : 0;
        p.rect(lx + s, gy - 16, 5, 14, fur[1]); p.rect(lx + s - 1, gy - 3, 7, 3, fur[0]);
        for (let k = 0; k < 3; k++) p.px(lx + s + k * 2, gy, P.bone);
    }
    // body
    p.blob(bx, gy - 24 - breath, 28, 13, '#5A3A2A');
    for (let x = bx - 22; x < bx + 18; x += 3) p.px(x, gy - 13 - breath, fur[0]);
    // serpent tail
    for (let t = 0; t < 30; t++) { const x = bx - 28 - t * 0.9, y = gy - 26 - Math.sin(t * 0.25 + f) * 8 - t * 0.6; p.ellipse(x, y, 2.6 - t * 0.04, 2.6 - t * 0.04, t % 4 < 2 ? '#2E4A36' : '#3E5A3A'); }
    const tx = bx - 56, ty = gy - 48 - Math.sin(7.5 + f) * 8;
    p.blob(tx, ty, 5, 4, '#3E5A3A'); p.lit(tx - 3, ty - 1, P.lava, 1.3); p.px(tx - 6, ty + 1, P.blood);
    // goat head on the back
    const gx = bx - 6, gyy = gy - 46 - breath;
    p.vline(gx, gyy + 4, gy - 34, fur[2]); p.vline(gx + 1, gyy + 4, gy - 34, fur[1]);
    p.blob(gx, gyy, 6, 5, '#6A5A4A');
    for (let k = 0; k < 7; k++) { p.px(gx - 4 - k, gyy - 4 - Math.round(Math.sin(k * 0.5) * 3), P.bone); p.px(gx + 4 + k, gyy - 4 - Math.round(Math.sin(k * 0.5) * 3), P.bone); }
    p.lit(gx - 2, gyy, P.lava, 1.2); p.lit(gx + 2, gyy, P.lava, 1.2);
    // lion head with a burning mane
    const hx = bx + 30, hy = gy - 30 - breath + (anim === 'atk' && f ? 4 : 0);
    for (let i = 0; i < 70; i++) {
        const a = h01(i, f, 1) * Math.PI * 2, r = 10 + h01(i, f, 2) * 7;
        const x = hx - 3 + Math.cos(a) * r, y = hy + Math.sin(a) * r * 0.9;
        if (BAYER[(Math.floor(y) & 3) * 4 + (Math.floor(x) & 3)] > 0.7) continue;
        p.lit(x, y, r > 14 ? P.lantern : r > 12 ? mane : P.blood, r > 14 ? 1.5 : 1);
    }
    p.blob(hx, hy, 10, 9, '#6A4A32');
    p.rect(hx + 4, hy + 2, 8, 5, '#5A3A2A');
    p.lit(hx + 2, hy - 3, P.lantern, 1.6); p.lit(hx + 5, hy - 3, P.lantern, 1.6);
    if (anim === 'atk' || anim === 'cast') { p.rect(hx + 6, hy + 4, 7, 4, P.abyss); for (let x = hx + 6; x < hx + 13; x += 2) { p.px(x, hy + 4, P.bone); p.px(x, hy + 7, P.bone); } if (anim === 'cast') for (let k = 0; k < 8; k++) p.lit(hx + 13 + k, hy + 5 + (k % 3) - 1, k < 4 ? P.lantern : P.ember, 1.6); }
}
// Scylla (ice): a pale woman rising out of a ring of dog heads and ice tentacles
function scylla(p, anim, f) {
    const W = p.w, H = p.h, gy = H - 6, cx = W / 2;
    const ice = ramp(P.ice), skin = ramp('#B8CCD4');
    if (anim === 'dead') { p.blob(cx, gy - 6, 30, 8, '#4F7A8C'); for (let i = 0; i < 10; i++) p.lit(cx - 25 + i * 5, gy - 10, P.soulLight, 0.5); return; }
    const reach = anim === 'atk' ? (f ? 14 : -4) : 0;
    // tentacles
    for (let i = 0; i < 6; i++) {
        const base = cx - 22 + i * 9;
        for (let t = 0; t < 22; t++) {
            const s = i % 2 ? 1 : -1;
            const x = base + s * Math.sin(t * 0.25 + f * 1.3 + i) * 4 + (i >= 3 ? reach * t / 22 : 0);
            const y = gy - t * 0.9;
            p.ellipse(x, y, 2.6 - t * 0.08, 2, t % 5 === 0 ? P.iceDark : ice[2 - (t > 15 ? 1 : 0)]);
        }
    }
    // hound heads around the waist
    for (const [dx, dy, s] of [[-20, -26, -1], [20, -26, 1], [-11, -20, -1], [11, -20, 1]]) {
        const hx = cx + dx + (s > 0 ? reach * 0.5 : 0), hy = gy + dy;
        p.blob(hx, hy, 6, 4.5, '#5A6A74');
        p.rect(hx + s * 4 - 2, hy, 5, 3, '#4A5864');
        p.lit(hx + s * 2, hy - 2, P.soulLight, 1.2);
        if (anim === 'atk') { p.px(hx + s * 6, hy + 2, P.bone); p.px(hx + s * 5, hy + 3, P.bone); }
        p.px(hx - s * 3, hy - 4, '#3A4650'); p.px(hx - s * 4, hy - 5, '#3A4650');
    }
    // torso + head
    const by = gy - 28 - (anim === 'idle' ? f : 0);
    for (let y = by - 22; y < by; y++) { const w = 5 + (y - by + 22) * 0.18; for (let x = -w; x <= w; x++) p.px(cx + x, y, skin[x < -w / 2 ? 3 : x > w / 2 ? 1 : 2]); }
    p.blob(cx, by - 27, 6, 6.5, '#B8CCD4');
    p.lit(cx - 2, by - 27, P.soul, 1.3); p.lit(cx + 2, by - 27, P.soul, 1.3);
    for (let y = by - 34; y < by - 8; y++) { p.px(cx - 7 - Math.round(Math.sin(y * 0.3)), y, '#E2ECEE'); p.px(cx + 7 + Math.round(Math.sin(y * 0.3)), y, '#C9D6D8'); }
    // ice crown
    for (let k = -5; k <= 5; k += 2) for (let y = 0; y < 4 + (k === -1 || k === 1 ? 3 : 0); y++) p.lit(cx + k, by - 34 - y, '#DFF7FF', 0.9);
    // arms: raised for spells
    const ay = anim === 'cast' ? by - 34 : by - 18;
    p.line(cx - 6, by - 20, cx - 14, ay, skin[2]); p.line(cx + 6, by - 20, cx + 14, ay, skin[1]);
    if (anim === 'cast') { for (const s of [-1, 1]) for (let k = 0; k < 5; k++) p.lit(cx + s * 14 + h01(k, s) * 4 - 2, ay - 2 - k, '#DFF7FF', 1.6); }
}
// Harpy queen Aello (wind): huge storm wings, talons, a woman's face with a crown of feathers
function harpy(p, anim, f) {
    const W = p.w, H = p.h, cx = W / 2;
    const flap = anim === 'atk' ? (f ? 1 : -1) : Math.sin(f * Math.PI) * (anim === 'walk' ? 1 : 0.5);
    const fl = ramp('#4A6170'), skin = ramp('#9AA8AE');
    const by = H / 2 + 8 - (anim === 'idle' ? f * 2 : 0);
    if (anim === 'dead') { const gy = H - 6; p.blob(cx, gy - 5, 34, 6, '#4A6170'); for (let i = 0; i < 16; i++) p.px(cx - 30 + i * 4, gy - 9 + (i % 2) * 2, P.mist); return; }
    // wings: feathers fanned out
    for (const s of [-1, 1]) {
        for (let i = 0; i < 9; i++) {
            const a = (-0.25 - i * 0.13 - flap * 0.35) * Math.PI;
            const len = 34 - Math.abs(i - 3) * 2;
            for (let t = 0; t < len; t++) {
                const x = cx + s * (6 + Math.cos(a) * t * -1 * -1) * 1 + s * Math.cos(a + Math.PI) * 0;
                const xx = cx + s * (6 + t * Math.cos(Math.PI * 0.08 * i) * 0.9);
                const yy = by - 12 + Math.sin(a) * t * 0.6 + t * 0.15 * i * 0.3;
                void x;
                p.px(xx, yy, fl[t > len - 4 ? 0 : i % 2 ? 1 : 2]); p.px(xx, yy + 1, fl[1]);
                if (t > len - 6 && i % 3 === 0) p.glow(xx, yy, '#B8D8F0', 0.4);
            }
        }
    }
    // body
    p.blob(cx, by, 7, 12, '#5A6A74');
    for (let y = by - 4; y < by + 10; y += 3) p.hline(cx - 4, cx + 4, y, fl[1]);
    // talons
    const tY = by + 12 + (anim === 'atk' && f ? 6 : 0);
    for (const s of [-1, 1]) { p.vline(cx + s * 3, by + 8, tY, '#3A3A3E'); for (let k = -1; k <= 1; k++) { p.px(cx + s * 3 + k * 2, tY + 1, '#2A2A2E'); p.px(cx + s * 3 + k * 2 + s, tY + 2, P.bone); } }
    // head + feather crown
    p.blob(cx, by - 17, 5, 6, '#9AA8AE');
    p.lit(cx - 2, by - 18, '#D8E4F0', 1.3); p.lit(cx + 2, by - 18, '#D8E4F0', 1.3);
    p.px(cx, by - 15, skin[0]);
    for (let k = -4; k <= 4; k++) for (let y = 0; y < 5 - Math.abs(k) * 0.6; y++) p.px(cx + k, by - 23 - y, k % 2 ? '#24394A' : '#8FA3A8');
    for (let y = by - 17; y < by - 4; y++) { p.px(cx - 6, y, '#24394A'); p.px(cx + 6, y, '#16232E'); }
    if (anim === 'cast') for (let i = 0; i < 20; i++) { const a = i / 20 * Math.PI * 2; p.lit(cx + Math.cos(a) * 18, by + Math.sin(a) * 10, '#D8E4F0', 0.8); }
}
// Talos of coral stone (earth): a bronze-and-coral colossus, glowing veins, fists like anchors
function talos(p, anim, f) {
    const W = p.w, H = p.h, gy = H - 3, cx = W / 2;
    const st = ramp('#5A6A4A'), br = ramp('#7A5A3A');
    if (anim === 'dead') { for (let i = 0; i < 9; i++) p.blob(cx - 30 + i * 8, gy - 4 - h01(i, 2) * 4, 5, 4, i % 2 ? '#5A6A4A' : '#7A5A3A'); return; }
    const step = anim === 'walk' ? Math.round(Math.sin(f * Math.PI / 2) * 3) : 0;
    const slam = anim === 'atk' ? (f ? 1 : -1) : 0;
    // legs
    for (const s of [-1, 1]) { const lx = cx + s * 10 + (s > 0 ? step : -step); p.rect(lx - 5, gy - 26, 10, 26, st[s > 0 ? 1 : 2]); p.rect(lx - 6, gy - 5, 12, 5, br[1]); p.hline(lx - 5, lx + 4, gy - 26, st[3]); }
    // torso
    const ty = gy - 26 - (anim === 'idle' ? f : 0);
    for (let y = ty - 34; y < ty; y++) { const w = 20 - Math.max(0, y - ty + 12) * 0.5; for (let x = -w; x <= w; x++) p.px(cx + x, y, (x + y) % 9 === 0 ? br[2] : st[x < -w * 0.5 ? 3 : x > w * 0.5 ? 1 : 2]); }
    // coral growth and glowing veins
    for (let i = 0; i < 8; i++) { const x = cx - 16 + h01(i, 3) * 32, y = ty - 30 + h01(i, 4) * 26; for (let k = 0; k < 5; k++) p.px(x + Math.round(Math.sin(k + i)), y - k, k % 2 ? P.coral : P.violet); }
    for (let y = ty - 30; y < ty - 2; y++) { p.lit(cx + Math.round(Math.sin(y * 0.3) * 3), y, '#9ACF6A', y % 3 ? 0.7 : 1.2); }
    // head
    p.blob(cx, ty - 40, 8, 7, '#7A5A3A');
    p.hline(cx - 5, cx + 5, ty - 41, P.abyss); p.lit(cx - 3, ty - 41, '#9ACF6A', 1.6); p.lit(cx + 3, ty - 41, '#9ACF6A', 1.6);
    // arms + fists
    for (const s of [-1, 1]) {
        const ax = cx + s * 22, ay0 = ty - 30, ay1 = ty - 6 + (slam > 0 ? 18 : slam < 0 ? -26 : 0);
        p.line(ax, ay0, ax + s * 4, ay1, st[1]); p.line(ax + s, ay0, ax + s * 5, ay1, st[2]); p.line(ax - s, ay0, ax + s * 3, ay1, st[3]);
        p.blob(ax + s * 4, ay1 + 3, 6, 5, '#7A5A3A');
    }
}
// burning skeleton megalodon: a gigantic jaw of bone with an element flame over the spine
function megalodon(p, anim, f, flame = [P.lantern, P.ember, P.blood]) {
    const W = p.w, H = p.h, cy = H / 2 + 10;
    const bone = ramp(P.bone);
    if (anim === 'dead') { for (let i = 0; i < 18; i++) p.blob(10 + i * 9, H - 8 - h01(i, 1) * 4, 4, 3, P.bone); return; }
    const open = anim === 'atk' ? (f ? 14 : 4) : anim === 'cast' ? 12 : 2 + f;
    const sw = anim === 'idle' || anim === 'walk' ? Math.sin(f * Math.PI) * 3 : 0;
    // spine + ribs
    for (let x = 16; x < W - 50; x += 2) {
        const t = (x - 16) / (W - 66);
        const yy = cy + Math.sin(t * 4 + f) * sw * (1 - t);
        p.px(x, yy - 8, bone[3]); p.px(x, yy - 7, bone[2]);
        if (x % 6 === 0) { const rh = Math.sin(Math.min(1, t * 1.2) * Math.PI) * 14 + 2; for (let y = 0; y < rh; y++) p.px(x + Math.round(y * 0.15), yy - 7 + y, bone[y > rh * 0.7 ? 1 : 2]); }
        // flames along the back
        const fh = 4 + Math.abs(Math.sin(x * 0.3 + f * 2)) * 10 * Math.sin(Math.min(1, t * 1.3) * Math.PI);
        for (let y = 0; y < fh; y++) { if (BAYER[((yy | 0) - 9 - y & 3) * 4 + (x & 3)] > 1 - y / fh * 0.3 && y > fh * 0.5) continue; p.lit(x, yy - 9 - y, y < fh * 0.35 ? flame[0] : y < fh * 0.7 ? flame[1] : flame[2], y < fh * 0.35 ? 1.6 : 1); }
    }
    // tail fin bones
    p.line(16, cy - 8, 2, cy - 26, bone[2]); p.line(16, cy - 8, 4, cy + 8, bone[1]); p.line(10, cy - 16, 3, cy - 2, bone[1]);
    // pectoral fins
    p.line(W * 0.55, cy + 4, W * 0.48, cy + 18, bone[1]); p.line(W * 0.58, cy + 4, W * 0.5, cy + 19, bone[2]);
    // skull with jaws
    const sx = W - 52, top = cy - 18;
    for (let y = 0; y < 16; y++) for (let x = 0; x < 46 - y * 0.6; x++) p.px(sx + x, top + y, bone[y < 3 ? 4 : x > 38 - y ? 1 : 2]);
    for (let y = 0; y < 8; y++) for (let x = 0; x < 40 - y; x++) p.px(sx + 2 + x, top + 16 + open + y, bone[y > 5 ? 1 : 2]);
    p.rect(sx + 4, top + 16, 38, open, P.abyss);
    for (let x = sx + 6; x < sx + 40; x += 3) { p.px(x, top + 16, P.bone); p.px(x, top + 17, bone[1]); p.px(x + 1, top + 15 + open, P.bone); }
    for (let x = sx + 6; x < sx + 40 && open > 6; x += 4) p.lit(x + 1, top + 18 + Math.floor(open / 2), flame[0], 0.8);
    // eye socket
    p.ellipse(sx + 34, top + 6, 3, 3, P.abyss); p.lit(sx + 34, top + 6, flame[0], 1.8); p.lit(sx + 35, top + 6, flame[1], 1.4);
}
// ---------------------------------------------------------------- ships (16 headings)
// heading h: 0 = north (screen up), clockwise. Top-down 3/4 view: hull as a projected outline.
function shipFrame(p, h, ghost = false, mast = true) {
    const W = p.w, H = p.h, cx = W / 2, cy = H / 2 + 6;
    const fx = Math.sin(h), fy = -Math.cos(h) * 0.75; // forward, squashed in y (3/4 view)
    const rx = Math.cos(h), ry = Math.sin(h) * 0.75;   // right
    const hull = ramp(ghost ? '#2E4A4C' : P.tarwood), deck = ramp(ghost ? '#3E5A5A' : P.oldwood);
    const len = 20, beam = 7;
    // hull: sample a lens shape in ship space
    for (let a = -len; a <= len; a += 0.5) for (let b = -beam; b <= beam; b += 0.5) {
        const t = a / len, w = beam * Math.sqrt(Math.max(0, 1 - Math.pow(Math.max(0, t), 2) * 1)) * (t < -0.85 ? 0.8 : 1);
        if (Math.abs(b) > w * (t > 0.6 ? (1 - (t - 0.6) * 2.2) : 1)) continue;
        const x = cx + fx * a + rx * b, y = cy + fy * a + ry * b;
        const edge = Math.abs(b) > w * 0.8 || Math.abs(t) > 0.92;
        p.px(x, y, edge ? hull[(ry * b) > 0 ? 1 : 3] : deck[(Math.round(a) % 4 === 0) ? 1 : 2]);
        if (edge) p.px(x, y + 3, hull[0]); // side planks visible below the deck line
        if (edge && Math.abs(b) > w * 0.8) { p.px(x, y + 1, hull[1]); p.px(x, y + 2, hull[0]); }
    }
    if (ghost) for (let i = 0; i < 20; i++) { const a = h01(i, 2) * 2 * len - len, b = (h01(i, 3) - 0.5) * beam * 2; p.glow(cx + fx * a + rx * b, cy + fy * a + ry * b, P.soul, 0.6); }
    if (!mast) return;
    // masts + sails (sails face the wind: billow towards the bow)
    for (const m of [-7, 6]) {
        const mx = cx + fx * m, my = cy + fy * m;
        p.vline(mx, my - 22, my, ghost ? '#4A6A6A' : P.deadwood);
        const sw = 9, sh = 13;
        for (let y = 0; y < sh; y++) {
            const bulge = Math.sin(y / sh * Math.PI) * 2;
            for (let s = -sw; s <= sw; s++) {
                const x = mx + rx * s + fx * bulge, yy = my - 20 + y + ry * s * 0.3;
                const c = ghost ? (h01(s, y, m) > 0.8 ? null : (y % 4 === 0 ? '#4A7A74' : '#6A9A90')) : (y % 4 === 0 ? '#8F8672' : P.bone);
                if (c) p.px(x, yy, c, ghost ? 170 : 255);
            }
        }
        if (ghost) p.lit(mx, my - 23, P.soul, 1.3); else p.lit(mx, my - 23, P.blood, 0.3);
    }
    p.lit(cx - fx * 18, cy - fy * 18 - 3, ghost ? P.soul : P.lantern, 1.3); // stern lantern
}

const BEASTS = {
    crab: { w: 28, h: 22, fn: (p, a, f) => crab(p, a, f, false), shadow: [9, 3] },
    crabWarrior: { w: 40, h: 32, fn: (p, a, f) => crab(p, a, f, true), shadow: [13, 4] },
    shell: { w: 30, h: 22, fn: shellCrawler, shadow: [10, 3] },
    shark: { w: 48, h: 28, fn: shark, shadow: [0, 0] },
    chimera: { w: 120, h: 88, fn: chimera, shadow: [34, 7] },
    scylla: { w: 96, h: 96, fn: scylla, shadow: [32, 7] },
    harpy: { w: 112, h: 80, fn: harpy, shadow: [20, 5] },
    talos: { w: 88, h: 100, fn: talos, shadow: [28, 7] },
    megalodon: { w: 176, h: 72, fn: megalodon, shadow: [60, 8] }
};
const MEG_FLAMES = { fire: [P.lantern, P.ember, P.blood], ice: ['#DFF7FF', '#9FC3D0', '#4F7A8C'], wind: ['#FFFFFF', '#B8D8F0', '#4A6170'], earth: ['#C8F08A', '#9ACF6A', '#3E5A3A'] };

export function buildBeasts(atlas) {
    const out = {};
    const build = (name, def, fn) => {
        const sheet = { w: def.w, h: def.h, ax: Math.floor(def.w / 2), ay: def.h - 1, shadow: def.shadow, beast: true, anims: {} };
        for (const [anim, n] of Object.entries(BEAST_FRAMES)) {
            sheet.anims[anim] = [];
            for (let f = 0; f < n; f++) { const p = new Pix(def.w, def.h); fn(p, anim, f); p.outline({ skipBottom: true }); sheet.anims[anim].push(atlas.add(`b:${name}:${anim}:${f}`, p)); }
        }
        out[name] = sheet;
    };
    for (const [name, def] of Object.entries(BEASTS)) if (name !== 'megalodon') build(name, def, def.fn);
    for (const [el, fl] of Object.entries(MEG_FLAMES)) build('megalodon_' + el, BEASTS.megalodon, (p, a, f) => megalodon(p, a, f, fl));
    // ships: 16 headings
    for (const [name, ghost] of [['ship', false], ['ghostship', true]]) {
        const frames = [];
        for (let i = 0; i < 16; i++) { const p = new Pix(64, 64); shipFrame(p, i / 16 * Math.PI * 2, ghost); p.outline(); frames.push(atlas.add(`s:${name}:${i}`, p)); }
        out[name] = { w: 64, h: 64, ax: 32, ay: 44, frames };
    }
    return out;
}
