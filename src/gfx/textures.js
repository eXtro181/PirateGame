// Procedural PBR texture sets, generated once at start-up.
// Every set has an albedo (sRGB, dark and desaturated on purpose), a tangent-space normal map
// and a "surface" map: R = glossiness, G = height (used for height-based terrain blending).
// Also: alpha cards (palm fronds, grass), decals (blood, puddles, burn, moss) and particle sprites.
import { Noise, worley } from './noise.js';

const N = new Noise(4242);
const N2 = new Noise(777);
const clamp = (v, a = 0, b = 1) => Math.max(a, Math.min(b, v));
const mix = (a, b, t) => a + (b - a) * t;
const smooth = (a, b, x) => { const t = clamp((x - a) / (b - a)); return t * t * (3 - 2 * t); };
const hex = h => [parseInt(h.slice(1, 3), 16), parseInt(h.slice(3, 5), 16), parseInt(h.slice(5, 7), 16)];
const mixc = (a, b, t) => [mix(a[0], b[0], t), mix(a[1], b[1], t), mix(a[2], b[2], t)];

function canvas(w, h = w) { const c = document.createElement('canvas'); c.width = w; c.height = h; return c; }

// Generic generator. fn(u, v) -> [r, g, b, height(0..1), gloss(0..1)]
function generate(S, fn, normalStrength = 3) {
    const alb = canvas(S), srf = canvas(S), nrm = canvas(S);
    const ad = alb.getContext('2d').createImageData(S, S), sd = srf.getContext('2d').createImageData(S, S), nd = nrm.getContext('2d').createImageData(S, S);
    const H = new Float32Array(S * S);
    for (let y = 0; y < S; y++) for (let x = 0; x < S; x++) {
        const o = fn(x / S, y / S);
        const i = y * S + x, k = i * 4;
        ad.data[k] = clamp(o[0], 0, 255); ad.data[k + 1] = clamp(o[1], 0, 255); ad.data[k + 2] = clamp(o[2], 0, 255); ad.data[k + 3] = 255;
        H[i] = o[3];
        sd.data[k] = clamp(o[4]) * 255; sd.data[k + 1] = clamp(o[3]) * 255; sd.data[k + 2] = 0; sd.data[k + 3] = 255;
    }
    const at = (x, y) => H[((y + S) % S) * S + ((x + S) % S)];
    for (let y = 0; y < S; y++) for (let x = 0; x < S; x++) {
        const dx = (at(x + 1, y) - at(x - 1, y)) * normalStrength * S / 256;
        const dy = (at(x, y + 1) - at(x, y - 1)) * normalStrength * S / 256;
        let nx = -dx, ny = dy, nz = 1; // +y (green) up in tangent space
        const l = Math.hypot(nx, ny, nz); nx /= l; ny /= l; nz /= l;
        const k = (y * S + x) * 4;
        nd.data[k] = (nx * 0.5 + 0.5) * 255; nd.data[k + 1] = (ny * 0.5 + 0.5) * 255; nd.data[k + 2] = (nz * 0.5 + 0.5) * 255; nd.data[k + 3] = 255;
    }
    alb.getContext('2d').putImageData(ad, 0, 0); srf.getContext('2d').putImageData(sd, 0, 0); nrm.getContext('2d').putImageData(nd, 0, 0);
    return { albedo: alb, normal: nrm, surface: srf };
}

// ------------------------------------------------------------------ surfaces
const SETS = {
    sand(u, v) {
        const P = 4;
        const n = N.fbm(u * P * 2, v * P * 2, 4, P * 2);
        const fine = N2.n2(u * 64, v * 64, 64);
        const rip = Math.sin((v * 10 + N.fbm(u * 3, v * 3, 2, 3) * 0.9 + u * 2) * Math.PI * 2) * 0.5 + 0.5;
        const [d1, , id] = worley(u * 14, v * 14, 14, 3);
        const peb = (id & 7) === 0 ? smooth(0.22, 0.12, d1) : 0;
        let c = mixc(hex('#4f4838'), hex('#625946'), n * 0.5 + 0.5);
        c = mixc(c, hex('#3a352b'), (1 - rip) * 0.25);
        c = mixc(c, hex('#5e5b55'), peb);
        const k = 1 + fine * 0.08;
        return [c[0] * k, c[1] * k, c[2] * k, 0.35 + rip * 0.12 + n * 0.1 + peb * 0.3 + fine * 0.03, 0.18 + peb * 0.2];
    },
    grass(u, v) {
        const P = 4;
        const n = N.fbm(u * P, v * P, 4, P);
        const patch = smooth(0.25, 0.45, N2.fbm(u * 3, v * 3, 3, 3));
        const blade = N.n2(u * 96, v * 24, 96, 24) * 0.5 + N.n2(u * 144, v * 40, 144, 40) * 0.5;
        const b = blade * 0.5 + 0.5;
        let c = mixc(hex('#1f2a1a'), hex('#33402a'), b);
        c = mixc(c, hex('#3d4a2c'), smooth(0.3, 0.8, n) * 0.5);
        c = mixc(c, hex('#3a3024'), patch * 0.55);
        return [c[0], c[1], c[2], 0.4 + b * 0.25 + n * 0.1 - patch * 0.1, 0.12 + b * 0.08];
    },
    dirt(u, v) {
        const P = 4;
        const n = N.fbm(u * P, v * P, 5, P);
        const wet = smooth(-0.05, -0.3, N2.fbm(u * 2, v * 2, 3, 2));
        const [d1, , id] = worley(u * 10, v * 10, 10, 9);
        const stone = (id & 3) === 0 ? smooth(0.3, 0.15, d1) : 0;
        let c = mixc(hex('#2e2219'), hex('#45342a'), n * 0.5 + 0.5);
        c = mixc(c, hex('#1f1712'), wet * 0.6);
        c = mixc(c, hex('#4c4842'), stone);
        return [c[0], c[1], c[2], 0.4 + n * 0.2 - wet * 0.25 + stone * 0.35, 0.2 + wet * 0.65 + stone * 0.1];
    },
    stone(u, v) { // irregular flagstones with mossy, wet joints
        const [d1, d2, id] = worley(u * 5, v * 5, 5, 21);
        const edge = d2 - d1;
        const gap = smooth(0.08, 0.02, edge);
        const n = N.fbm(u * 16, v * 16, 4, 16);
        const tint = ((id >>> 3) & 255) / 255;
        let c = mixc(hex('#3c3a38'), hex('#55514b'), tint * 0.7 + n * 0.2);
        c = mixc(c, hex('#1c2418'), gap * 0.85);
        const moss = gap * smooth(0, 0.4, N2.fbm(u * 6, v * 6, 3, 6) + 0.2);
        c = mixc(c, hex('#2f4223'), moss * 0.8);
        const dome = smooth(0, 0.18, edge);
        return [c[0], c[1], c[2], dome * 0.6 + n * 0.12, 0.35 + gap * 0.4 + n * 0.08];
    },
    rock(u, v) {
        const r = N.ridged(u * 4, v * 6, 5, 4, 6);
        const strata = Math.sin((v * 9 + N.fbm(u * 2, v * 2, 3, 2) * 1.6) * Math.PI * 2) * 0.5 + 0.5;
        const n = N2.fbm(u * 12, v * 12, 4, 12);
        const crack = smooth(0.94, 0.99, N.ridged(u * 8, v * 8 + 3, 3, 8));
        let c = mixc(hex('#34312d'), hex('#544e46'), r * 0.6 + n * 0.25 + strata * 0.15);
        c = mixc(c, hex('#1a1816'), crack * 0.7);
        const moss = smooth(0.15, 0.45, N2.fbm(u * 5, v * 5 + 7, 3, 5)) * 0.5;
        c = mixc(c, hex('#2c3a22'), moss);
        return [c[0], c[1], c[2], r * 0.6 + strata * 0.15 - crack * 0.3 + n * 0.1, 0.3 + crack * 0.3 + n * 0.1];
    },
    wood(u, v) { // weathered planks running along u
        const planks = 5;
        const pv = v * planks, pi = Math.floor(pv), pf = pv - pi;
        const seam = smooth(0.06, 0.0, pf) + smooth(0.94, 1.0, pf);
        const off = (pi * 0.37) % 1;
        const grain = Math.sin(((u + off) * 30 + N.fbm((u + off) * 4, pv * 2, 3, 4, 10) * 4) * Math.PI * 2) * 0.5 + 0.5;
        const n = N2.fbm(u * 8, v * 8, 3, 8);
        const tint = ((pi * 2654435761) >>> 0) % 100 / 100;
        let c = mixc(hex('#2c2018'), hex('#4a3626'), tint * 0.6 + grain * 0.25 + n * 0.15);
        c = mixc(c, hex('#120d0a'), seam * 0.9);
        const nail = ((u * 4 + off) % 1 < 0.025 && Math.abs(pf - 0.5) < 0.07) ? 1 : 0;
        c = mixc(c, hex('#5a5550'), nail);
        return [c[0], c[1], c[2], 0.5 - seam * 0.5 + grain * 0.12 + n * 0.08, 0.22 + grain * 0.08 + nail * 0.4];
    },
    bark(u, v) {
        const ring = Math.sin(v * 22 * Math.PI * 2) * 0.5 + 0.5;
        const fib = N.n2(u * 40, v * 6, 40, 6) * 0.5 + 0.5;
        const n = N2.fbm(u * 6, v * 6, 3, 6);
        const c = mixc(hex('#2a2119'), hex('#4a3b2c'), ring * 0.5 + fib * 0.3 + n * 0.2);
        return [c[0], c[1], c[2], ring * 0.55 + fib * 0.2 + n * 0.1, 0.15];
    },
    brick(u, v) { // ruin masonry trim: rows of large stone blocks
        const rows = 4, rv = v * rows, ri = Math.floor(rv), rf = rv - ri;
        const cols = 2, cu = u * cols + (ri % 2) * 0.5, ci = Math.floor(cu), cf = cu - ci;
        const mort = Math.max(smooth(0.06, 0.0, rf), smooth(0.94, 1, rf), smooth(0.035, 0, cf), smooth(0.965, 1, cf));
        const n = N.fbm(u * 10, v * 10, 4, 10);
        const chip = smooth(0.55, 0.75, N2.fbm(u * 7, v * 7, 3, 7));
        const tint = (((ri * 7 + ci) * 2654435761) >>> 0) % 100 / 100;
        let c = mixc(hex('#3a3835'), hex('#57524b'), tint * 0.6 + n * 0.3);
        c = mixc(c, hex('#1a1a18'), mort * 0.85);
        c = mixc(c, hex('#2a2826'), chip * 0.5);
        const moss = smooth(0.55, 1, v) * smooth(0.0, 0.5, N2.fbm(u * 5, v * 3, 3, 5, 3) + 0.3);
        c = mixc(c, hex('#28361f'), moss * 0.8);
        return [c[0], c[1], c[2], 0.6 - mort * 0.6 + n * 0.12 - chip * 0.2, 0.28 + mort * 0.2 + moss * 0.15];
    },
    cloth(u, v) {
        const w = (Math.sin(u * 160 * Math.PI) * Math.sin(v * 160 * Math.PI)) * 0.5 + 0.5;
        const n = N.fbm(u * 6, v * 6, 4, 6);
        const stain = smooth(0.2, 0.5, N2.fbm(u * 3, v * 3, 3, 3));
        const k = 205 + w * 25 + n * 30 - stain * 60;
        return [k, k, k, w * 0.4 + n * 0.2, 0.08];
    },
    leather(u, v) {
        const n = N.fbm(u * 10, v * 10, 5, 10);
        const wr = N.ridged(u * 6, v * 6, 3, 6);
        const k = 190 + n * 40 - wr * 30;
        return [k, k * 0.96, k * 0.9, n * 0.3 + wr * 0.2, 0.3 + n * 0.1];
    },
    skin(u, v) {
        const n = N.fbm(u * 12, v * 12, 4, 12);
        const k = 235 + n * 18;
        return [k, k, k, 0.5 + n * 0.1, 0.32];
    },
    metal(u, v) {
        const n = N.fbm(u * 8, v * 8, 4, 8);
        const rust = smooth(0.15, 0.45, N2.fbm(u * 5, v * 5, 4, 5));
        const scr = smooth(0.96, 1, N.ridged(u * 20, v * 3, 2, 20, 3));
        let c = mixc(hex('#3c3d40'), hex('#55565a'), n * 0.5 + 0.5);
        c = mixc(c, hex('#4a2c1c'), rust * 0.75);
        c = mixc(c, hex('#8a8a88'), scr * 0.6);
        return [c[0], c[1], c[2], 0.5 + n * 0.1 - rust * 0.1, 0.62 - rust * 0.45 + scr * 0.2];
    },
    bone(u, v) {
        const n = N.fbm(u * 10, v * 10, 4, 10);
        const crack = smooth(0.95, 0.99, N.ridged(u * 6, v * 6, 3, 6));
        let c = mixc(hex('#8f8672'), hex('#b3aa92'), n * 0.5 + 0.5);
        c = mixc(c, hex('#3a3428'), crack * 0.8);
        return [c[0], c[1], c[2], 0.5 + n * 0.2 - crack * 0.3, 0.3];
    }
};

export function makeSurface(kind, S = 512) {
    const strength = { sand: 2.2, grass: 2.2, dirt: 3, stone: 5, rock: 5, wood: 3.5, bark: 4, brick: 5, cloth: 1.5, leather: 2.5, skin: 0.8, metal: 1.8, bone: 2.5 }[kind] || 3;
    return generate(S, SETS[kind], strength);
}

// water: only a normal map (two of them scroll against each other)
export function waterNormal(S = 256) {
    return generate(S, (u, v) => {
        const h = N.fbm(u * 4, v * 4, 5, 4) * 0.6 + N2.fbm(u * 9, v * 9, 3, 9) * 0.25;
        return [0, 0, 0, h * 0.5 + 0.5, 1];
    }, 6).normal;
}

// ------------------------------------------------------------------ alpha cards
export function frondCard() { // palm frond: rib plus drooping leaflets, painted with Canvas2D
    const W = 128, H = 512, c = canvas(W, H), g = c.getContext('2d');
    g.lineCap = 'round';
    for (let i = 0; i < 46; i++) {
        const y = 20 + i * 10.4, len = 52 * Math.sin(Math.PI * (0.15 + 0.85 * i / 46));
        for (const s of [-1, 1]) {
            const shade = 40 + Math.random() * 30;
            g.strokeStyle = `rgb(${shade * 0.55},${shade + 20},${shade * 0.45})`;
            g.lineWidth = 5;
            g.beginPath(); g.moveTo(W / 2, y); g.quadraticCurveTo(W / 2 + s * len * 0.6, y + 6, W / 2 + s * len, y + 24 + Math.random() * 6); g.stroke();
        }
    }
    g.strokeStyle = '#5a5236'; g.lineWidth = 6; g.beginPath(); g.moveTo(W / 2, 0); g.lineTo(W / 2, H); g.stroke();
    return c;
}
export function grassCard() {
    const W = 256, H = 256, c = canvas(W, H), g = c.getContext('2d');
    for (let i = 0; i < 70; i++) {
        const x = 10 + Math.random() * (W - 20), h = 90 + Math.random() * 150, bend = (Math.random() - 0.5) * 50;
        const k = 30 + Math.random() * 40;
        g.fillStyle = `rgb(${k * 0.7},${k + 15},${k * 0.45})`;
        g.beginPath(); g.moveTo(x - 4, H); g.quadraticCurveTo(x + bend * 0.3, H - h * 0.5, x + bend, H - h); g.quadraticCurveTo(x + bend * 0.3 + 2, H - h * 0.5, x + 4, H); g.fill();
    }
    return c;
}
export function leafCard() { // bush leaves
    const S = 256, c = canvas(S), g = c.getContext('2d');
    for (let i = 0; i < 90; i++) {
        const x = 30 + Math.random() * (S - 60), y = 30 + Math.random() * (S - 60), a = Math.random() * Math.PI * 2, r = 12 + Math.random() * 14;
        const k = 28 + Math.random() * 40;
        g.fillStyle = `rgb(${k * 0.65},${k + 12},${k * 0.5})`;
        g.save(); g.translate(x, y); g.rotate(a); g.beginPath(); g.ellipse(0, 0, r, r * 0.42, 0, 0, Math.PI * 2); g.fill(); g.restore();
    }
    return c;
}

// ------------------------------------------------------------------ decals (RGBA, straight alpha)
export function decalAtlas() { // 2×2: blood, puddle, burn, moss
    const S = 256, c = canvas(S * 2), g = c.getContext('2d');
    const blot = (ox, oy, col, rough, alphaMul) => {
        const id = g.createImageData(S, S);
        for (let y = 0; y < S; y++) for (let x = 0; x < S; x++) {
            const u = x / S - 0.5, v = y / S - 0.5;
            const d = Math.hypot(u, v) * 2 + N.fbm(x / S * 4 + ox, y / S * 4 + oy, 4) * rough;
            const a = smooth(0.95, 0.55, d) * alphaMul;
            const k = (y * S + x) * 4;
            const n = N2.n2(x / 9, y / 9) * 0.15;
            id.data[k] = col[0] * (1 + n); id.data[k + 1] = col[1] * (1 + n); id.data[k + 2] = col[2] * (1 + n); id.data[k + 3] = a * 255;
        }
        g.putImageData(id, ox, oy);
    };
    blot(0, 0, [70, 6, 6], 0.7, 0.92);
    blot(S, 0, [14, 16, 18], 0.4, 0.75);
    blot(0, S, [10, 8, 6], 0.9, 0.85);
    blot(S, S, [34, 52, 26], 1.0, 0.8);
    return c;
}

// ------------------------------------------------------------------ particle sprites
export function softDot(S = 64) {
    const c = canvas(S), g = c.getContext('2d');
    const gr = g.createRadialGradient(S / 2, S / 2, 0, S / 2, S / 2, S / 2);
    gr.addColorStop(0, 'rgba(255,255,255,1)'); gr.addColorStop(0.35, 'rgba(255,255,255,0.55)'); gr.addColorStop(1, 'rgba(255,255,255,0)');
    g.fillStyle = gr; g.fillRect(0, 0, S, S);
    return c;
}
export function smokePuff(S = 128) {
    const c = canvas(S), g = c.getContext('2d');
    const id = g.createImageData(S, S);
    for (let y = 0; y < S; y++) for (let x = 0; x < S; x++) {
        const u = x / S - 0.5, v = y / S - 0.5;
        const d = Math.hypot(u, v) * 2;
        const n = N.fbm(x / S * 5, y / S * 5, 4) * 0.5 + 0.5;
        const a = smooth(1, 0.2, d) * (0.4 + n * 0.6);
        const k = (y * S + x) * 4;
        id.data[k] = id.data[k + 1] = id.data[k + 2] = 255; id.data[k + 3] = a * 255;
    }
    g.putImageData(id, 0, 0);
    return c;
}
export function flameSprite(S = 128) { // tear-drop flame for particles
    const c = canvas(S), g = c.getContext('2d');
    const id = g.createImageData(S, S);
    for (let y = 0; y < S; y++) for (let x = 0; x < S; x++) {
        const u = (x / S - 0.5) * 2, v = y / S;
        const w = Math.sin(Math.PI * Math.pow(v, 0.7)) * 0.75;
        const d = Math.abs(u) / Math.max(0.01, w);
        const n = N.fbm(x / S * 4, y / S * 6, 3) * 0.3;
        const a = smooth(1, 0.4, d + n) * smooth(0, 0.15, v);
        const k = (y * S + x) * 4;
        id.data[k] = id.data[k + 1] = id.data[k + 2] = 255; id.data[k + 3] = a * 255;
    }
    g.putImageData(id, 0, 0);
    return c;
}
// tiling scrolling noise for fog layers and the whirlpool
export function noiseTex(S = 256, scale = 4) {
    const c = canvas(S), g = c.getContext('2d');
    const id = g.createImageData(S, S);
    for (let y = 0; y < S; y++) for (let x = 0; x < S; x++) {
        const n = N.fbm(x / S * scale, y / S * scale, 5, scale) * 0.5 + 0.5;
        const k = (y * S + x) * 4;
        id.data[k] = id.data[k + 1] = id.data[k + 2] = 255; id.data[k + 3] = smooth(0.35, 0.85, n) * 255;
    }
    g.putImageData(id, 0, 0);
    return c;
}
