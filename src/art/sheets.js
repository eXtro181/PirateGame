// Animated sprite sheets: flames, crab, ship, shark, effects, whirlpool, pearls.
import { Pix, Rng, hex, shade, mix } from './painter.js';

const OUT = hex('#120d14');

function strip(fw, fh, n, draw, { outline = true, dome = 0.9 } = {}) {
    const pix = new Pix(fw * n, fh);
    const rects = [];
    for (let i = 0; i < n; i++) {
        const one = new Pix(fw, fh);
        draw(one, i);
        if (outline) one.outline(OUT);
        one.autoHeight(dome, 0.2);
        pix.blit(one, i * fw, 0);
        rects.push({ x: i * fw, y: 0, w: fw, h: fh });
    }
    return { pix, rects };
}

// Flames: teal for saber fires, orange for wreck/torches. Fully emissive.
export function flameSheet(kind = 'teal', fw = 14, fh = 22) {
    const pal = kind === 'teal'
        ? [hex('#0b3d3a'), hex('#178a78'), hex('#3fd8b0'), hex('#a8ffe6')]
        : [hex('#5a1a08'), hex('#d0461a'), hex('#ffa23a'), hex('#fff1b8')];
    const rng = new Rng(kind === 'teal' ? 77 : 78);
    const s = strip(fw, fh, 6, (p, f) => {
        const cx = fw / 2;
        for (let y = 0; y < fh; y++) {
            const t = 1 - y / fh; // 0 at bottom
            const wob = Math.sin(y * 0.7 + f * 1.3) * (1 - t) * 1.6;
            const w = Math.max(0, (Math.sin(t * Math.PI * 0.95) * (fw / 2 - 1)) * (0.65 + 0.35 * t));
            for (let x = 0; x < fw; x++) {
                const d = Math.abs(x + 0.5 - cx - wob * (1 - t)) / Math.max(0.5, w);
                const n = rng.next() * 0.25;
                if (d + n < 1 && t > 0.05 + 0.2 * (1 - Math.sin(f + x))) {
                    const heat = (1 - d) * 0.7 + t * 0.6;
                    const c = heat > 1.05 ? pal[3] : heat > 0.8 ? pal[2] : heat > 0.5 ? pal[1] : pal[0];
                    p.px(x, y, c, 1);
                    p.glow(x, y, c);
                }
            }
        }
        // sparks
        for (let k = 0; k < 2; k++) { const x = rng.int(2, fw - 3), y = rng.int(0, fh / 3); p.px(x, y, pal[2], 1); p.glow(x, y, pal[3]); }
    }, { outline: false, dome: 0.3 });
    return { pix: s.pix, frames: { burn: s.rects } };
}

export function crabSheet() {
    const shell = hex('#b5482f'), shellD = hex('#7d2e1e'), leg = hex('#8e3a26');
    const draw = (p, pose) => {
        const cy = 9;
        // legs
        for (let k = 0; k < 3; k++) {
            const off = pose.walk ? ((k + pose.walk) % 2) : 0;
            p.line(5, cy + k, 1, cy + k + 2 - off, leg, 0.6);
            p.line(14, cy + k, 18, cy + k + 2 - off, leg, 0.6);
        }
        // body
        p.ellipse(9.5, cy, 6, 3.6, shell, 1);
        p.hline(5, 14, cy + 2, shellD, 0.8);
        p.px(7, cy - 3, OUT, 1.3); p.px(12, cy - 3, OUT, 1.3); // eye stalks
        p.px(7, cy - 4, hex('#1a1a1a'), 1.4); p.px(12, cy - 4, hex('#1a1a1a'), 1.4);
        // claws
        const raise = pose.windup ? -4 : (pose.attack ? 2 : 0);
        const open = pose.windup ? 2 : 0;
        for (const side of [-1, 1]) {
            const x = 9.5 + side * 7, y = cy - 3 + raise;
            p.disc(x, y, 2.4, shell, 1.4);
            p.px(x + side * 1, y - 2 - open, shellD, 1.4); p.px(x - side * 1, y - 2, shellD, 1.4);
            p.line(9.5 + side * 4, cy - 1, x, y + 1, leg, 0.9);
        }
        if (pose.windup) { p.glow(7, cy - 4, [255, 60, 40, 255]); p.glow(12, cy - 4, [255, 60, 40, 255]); }
    };
    const poses = { idle: [{}, { walk: 1 }], walk: [{ walk: 0 }, { walk: 1 }], windup: [{ windup: true }], attack: [{ attack: true }], dead: [{ dead: true }] };
    const fw = 20, fh = 16;
    const names = Object.keys(poses);
    const total = names.reduce((s, n) => s + poses[n].length, 0);
    const pix = new Pix(fw * total, fh);
    const frames = {};
    let col = 0;
    for (const n of names) {
        frames[n] = [];
        for (const pose of poses[n]) {
            const one = new Pix(fw, fh);
            draw(one, pose);
            if (pose.dead) { // flip upside down = dead crab
                const tmp = new Pix(fw, fh);
                for (let y = 0; y < fh; y++) for (let x = 0; x < fw; x++) { const c = one.get(x, y); if (c[3]) tmp.px(x, fh - 1 - y, shade(c, 0.75), 0.5); }
                tmp.outline(OUT); tmp.autoHeight(0.6, 0.1);
                pix.blit(tmp, col * fw, 0);
            } else { one.outline(OUT); one.autoHeight(1, 0.2); pix.blit(one, col * fw, 0); }
            frames[n].push({ x: col * fw, y: 0, w: fw, h: fh });
            col++;
        }
    }
    return { pix, frames };
}

// The player's sloop, seen from above, bow pointing to -y (image up). Frames = sail levels.
export function shipSheet() {
    const fw = 28, fh = 48;
    const hull = hex('#6a4528'), deck = hex('#9a7048'), rail = hex('#4a2e1a');
    const sailC = hex('#d8ceb4');
    const s = strip(fw, fh, 3, (p, level) => {
        const cx = fw / 2;
        for (let y = 4; y < 46; y++) {
            const t = (y - 4) / 42;
            const half = t < 0.3 ? Math.sqrt(t / 0.3) * 8.5 : (t > 0.88 ? 8.5 - (t - 0.88) * 30 : 8.5);
            for (let x = Math.round(cx - half); x <= Math.round(cx + half); x++) {
                const edge = Math.abs(x + 0.5 - cx) > half - 1.2;
                const plank = (x - Math.round(cx - half)) % 3 === 0;
                p.px(x, y, edge ? rail : (plank ? shade(deck, 0.88) : deck), edge ? 1.2 : 0.8);
            }
        }
        // stern cabin
        p.rect(cx - 6, 36, 12, 7, shade(hull, 1.1), 1.3); p.hline(cx - 6, cx + 5, 36, rail, 1.4);
        p.px(cx - 3, 39, hex('#ffcf6a'), 1.3); p.px(cx + 2, 39, hex('#ffcf6a'), 1.3);
        p.glow(cx - 3, 39, [255, 190, 80, 255]); p.glow(cx + 2, 39, [255, 190, 80, 255]);
        // cannons
        for (const y of [18, 27]) { p.px(cx - 9, y, hex('#2b2b30'), 1.4); p.px(cx + 8, y, hex('#2b2b30'), 1.4); }
        // mast + yard + sail
        p.disc(cx, 20, 1.5, rail, 2.2);
        const sw = level === 0 ? 3 : level === 1 ? 9 : 13;
        const sd = level === 0 ? 2 : level === 1 ? 4 : 6;
        p.hline(cx - 12, cx + 11, 20, rail, 2.3);
        for (let y = 20 - sd; y <= 20 + sd / 2; y++) {
            for (let x = cx - sw; x <= cx + sw - 1; x++) {
                const belly = 1 - Math.abs((x + 0.5 - cx) / sw);
                if (y < 20 - sd * belly) continue;
                p.px(x, y, shade(sailC, 0.9 + belly * 0.15), 2.5 + belly);
            }
        }
        if (level > 0) { p.hline(cx - 2, cx + 1, 17, hex('#7a2a2a'), 2.6); p.hline(cx - 1, cx, 16, hex('#7a2a2a'), 2.6); } // emblem
        // jib at bow
        p.line(cx, 4, cx, 12, rail, 1.5);
        if (level > 0) for (let y = 6; y < 13; y++) p.hline(cx + 1, cx + 1 + (y - 6) * 0.5, y, sailC, 2);
    });
    return { pix: s.pix, frames: { l0: [s.rects[0]], l1: [s.rects[1]], l2: [s.rects[2]] } };
}

// "Rostkiefer": a shark with a rusted anchor for a jaw. Faces +x (image right).
export function sharkSheet() {
    const fw = 44, fh = 20;
    const body = hex('#4b5a66'), belly = hex('#8a9aa4'), rust = hex('#8a4a22');
    const s = strip(fw, fh, 4, (p, f) => {
        const sw = Math.sin(f / 4 * Math.PI * 2) * 2;
        for (let x = 4; x < 38; x++) {
            const t = (x - 4) / 34;
            const half = Math.sin(Math.min(1, t * 1.15) * Math.PI) * 5 + 0.5;
            const yc = 10 + Math.sin(t * 3 + f) * (1 - t) * sw * 0.5;
            for (let y = Math.round(yc - half); y <= Math.round(yc + half); y++) p.px(x, y, y > yc + half * 0.3 ? belly : body, 1 + (1 - Math.abs(y - yc) / half));
        }
        // tail
        p.line(4, 10, 0, 4 + sw, body, 1.2); p.line(4, 10, 0, 16 + sw, body, 1.2); p.line(3, 10, 1, 6 + sw, body, 1.2);
        // dorsal fin
        p.line(20, 8, 24, 3, shade(body, 0.8), 1.8); p.line(21, 8, 26, 7, shade(body, 0.8), 1.6);
        // anchor jaw
        p.line(36, 12, 42, 12, rust, 1.6); p.line(42, 12, 42, 15, rust, 1.6); p.line(42, 15, 39, 16, rust, 1.6); p.px(40, 9, rust, 1.6);
        p.px(33, 8, hex('#e8e0c0'), 1.6); p.glow(33, 8, [255, 220, 120, 255]); // pale eye
        for (let k = 0; k < 3; k++) p.px(34 + k * 2, 12, hex('#e0dccc'), 1.4); // teeth
    });
    return { pix: s.pix, frames: { swim: s.rects } };
}

export function fxSheet() {
    // a grab-bag of small effects in one strip
    const pix = new Pix(256, 32);
    const frames = {};
    let x = 0;
    const put = (name, w, h, n, draw) => {
        frames[name] = [];
        for (let i = 0; i < n; i++) {
            const one = new Pix(w, h); draw(one, i); one.autoHeight(0.6, 0.1);
            pix.blit(one, x, 0); frames[name].push({ x, y: 0, w, h }); x += w + 1;
        }
    };
    // slash arc (emissive, pale blue steel)
    put('slash', 24, 24, 3, (p, f) => {
        const r0 = 6 + f * 2, r1 = 10 + f * 1.5;
        for (let a = -1.4; a <= 1.4; a += 0.03) {
            for (let r = r0; r <= r1; r++) {
                const x = 12 + Math.cos(a - Math.PI / 2) * r, y = 12 + Math.sin(a - Math.PI / 2) * r * -1;
                const k = 1 - Math.abs(a) / 1.4;
                if (k < 0.15 + f * 0.25) continue;
                const c = mix(hex('#4a7aa8'), hex('#f2fbff'), k);
                p.px(x, 24 - y, c, 1); p.glow(x, 24 - y, shade(c, 0.9 - f * 0.2));
            }
        }
    });
    put('spark', 9, 9, 2, (p, f) => {
        const c = hex('#fff2b0');
        if (f === 0) { p.line(4, 0, 4, 8, c); p.line(0, 4, 8, 4, c); } else { p.line(1, 1, 7, 7, c); p.line(7, 1, 1, 7, c); }
        for (let y = 0; y < 9; y++) for (let xx = 0; xx < 9; xx++) if (p.alpha(xx, y)) p.glow(xx, y, c);
    });
    put('pearl', 8, 8, 2, (p, f) => {
        p.disc(4, 4, f ? 2.4 : 3, hex('#bff6ff'), 1.2); p.px(3, 3, hex('#ffffff'), 1.5);
        for (let y = 0; y < 8; y++) for (let xx = 0; xx < 8; xx++) if (p.alpha(xx, y)) p.glow(xx, y, hex('#7fe9ff'));
    });
    put('ball', 5, 5, 1, (p) => { p.disc(2.5, 2.5, 2, hex('#2a2a30'), 1.3); p.px(1, 1, hex('#8a8a96'), 1.5); });
    put('splash', 14, 14, 3, (p, f) => {
        const r = 2 + f * 2.5;
        for (let a = 0; a < Math.PI * 2; a += 0.2) p.px(7 + Math.cos(a) * r, 7 + Math.sin(a) * r * 0.8, f === 2 ? hex('#a8c8d0') : hex('#e8f6fa'), 1);
        if (f === 0) p.disc(7, 7, 2, hex('#ffffff'), 1.5);
    });
    put('smoke', 16, 16, 3, (p, f) => {
        const r = 2.5 + f * 1.8;
        for (const [ox, oy, k] of [[0, 0, 1], [-2, 1, 0.7], [2, -1, 0.75], [1, 2, 0.6]]) p.disc(8 + ox * (1 + f * 0.4), 8 + oy * (1 + f * 0.4), r * k, hex(f === 0 ? '#9a948a' : f === 1 ? '#6a6670' : '#45434c'), 0.8);
        if (f === 2) for (let y = 0; y < 16; y++) for (let x = 0; x < 16; x++) if ((x + y) % 2) { const i = (y * 16 + x) * 4; p.col[i + 3] = 0; }
        if (f === 0) { p.disc(8, 8, 1.5, hex('#ffd27a'), 1); for (let y = 6; y < 11; y++) for (let x = 6; x < 11; x++) if (p.alpha(x, y)) p.glow(x, y, hex('#ff9a3a')); }
    });
    put('blood', 10, 10, 1, (p) => { p.disc(5, 5, 3, hex('#5a0f12'), 0.3); p.px(8, 3, hex('#5a0f12'), 0.3); p.px(2, 7, hex('#5a0f12'), 0.3); });
    put('firefly', 3, 3, 1, (p) => { p.px(1, 1, hex('#d8ff7a'), 1); p.glow(1, 1, hex('#d8ff7a')); });
    return { pix, frames };
}

// A whirlpool: concentric spiral foam on dark water, rotated by the game.
export function whirlSheet() {
    const size = 112;
    const p = new Pix(size, size);
    const c = size / 2;
    for (let y = 0; y < size; y++) {
        for (let x = 0; x < size; x++) {
            const dx = x + 0.5 - c, dy = y + 0.5 - c;
            const r = Math.hypot(dx, dy) / c;
            if (r > 1) continue;
            const a = Math.atan2(dy, dx);
            const spiral = Math.sin(a * 3 + r * 14);
            const dark = mix(hex('#0d2a36'), hex('#03080c'), 1 - r);
            let col = dark;
            let alpha = Math.min(1, (1 - r) * 3);
            if (spiral > 0.55 && r > 0.12) col = mix(hex('#5f8c98'), hex('#d8eef2'), (spiral - 0.55) * 2.2);
            col = [col[0], col[1], col[2], Math.round(alpha * 255)];
            if (col[3] > 128) {
                p.px(x, y, [col[0], col[1], col[2], 255], (1 - r) * -1 + spiral * 0.1);
                if (spiral > 0.8 && r > 0.2) p.glow(x, y, [30, 60, 70, 255]);
            }
        }
    }
    p.autoHeight(0, 0.2);
    return { pix: p, frames: { spin: [{ x: 0, y: 0, w: size, h: size }] } };
}
