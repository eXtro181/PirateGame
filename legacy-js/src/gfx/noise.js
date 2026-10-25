// Gradient (Perlin) noise with optional periodic tiling, fBm, ridged noise and Worley cells.
// Used for procedural textures, terrain height and smooth light flicker.

function makePerm(seed) {
    const p = new Uint8Array(512);
    const a = new Uint8Array(256).map((_, i) => i);
    let s = seed >>> 0 || 1;
    for (let i = 255; i > 0; i--) {
        s = (s * 1664525 + 1013904223) >>> 0;
        const j = s % (i + 1);
        [a[i], a[j]] = [a[j], a[i]];
    }
    for (let i = 0; i < 512; i++) p[i] = a[i & 255];
    return p;
}
const fade = t => t * t * t * (t * (t * 6 - 15) + 10);
const lerp = (a, b, t) => a + (b - a) * t;
function grad2(h, x, y) {
    switch (h & 7) {
        case 0: return x + y; case 1: return -x + y; case 2: return x - y; case 3: return -x - y;
        case 4: return x; case 5: return -x; case 6: return y; default: return -y;
    }
}

export class Noise {
    constructor(seed = 1) { this.p = makePerm(seed); }
    // 2D Perlin in [-1, 1]; period > 0 makes it tile every `period` units
    n2(x, y, period = 0, periodY = period) {
        const p = this.p;
        let xi = Math.floor(x), yi = Math.floor(y);
        const xf = x - xi, yf = y - yi;
        let xi1 = xi + 1, yi1 = yi + 1;
        if (period) { xi = ((xi % period) + period) % period; xi1 = (xi + 1) % period; }
        if (periodY) { yi = ((yi % periodY) + periodY) % periodY; yi1 = (yi + 1) % periodY; }
        xi &= 255; yi &= 255; xi1 &= 255; yi1 &= 255;
        const u = fade(xf), v = fade(yf);
        const aa = p[p[xi] + yi], ab = p[p[xi] + yi1], ba = p[p[xi1] + yi], bb = p[p[xi1] + yi1];
        return lerp(lerp(grad2(aa, xf, yf), grad2(ba, xf - 1, yf), u), lerp(grad2(ab, xf, yf - 1), grad2(bb, xf - 1, yf - 1), u), v) * 0.9;
    }
    // fractal Brownian motion in about [-1, 1]
    fbm(x, y, oct = 4, period = 0, periodY = period, gain = 0.5) {
        let a = 1, f = 1, s = 0, n = 0;
        for (let i = 0; i < oct; i++) {
            s += this.n2(x * f, y * f, period * f, periodY * f) * a;
            n += a; a *= gain; f *= 2;
        }
        return s / n;
    }
    ridged(x, y, oct = 4, period = 0, periodY = period) {
        let a = 1, f = 1, s = 0, n = 0;
        for (let i = 0; i < oct; i++) {
            s += (1 - Math.abs(this.n2(x * f, y * f, period * f, periodY * f))) * a;
            n += a; a *= 0.5; f *= 2;
        }
        return s / n;
    }
    // 1D smooth noise for flicker, in [-1, 1]
    n1(t) { return this.n2(t, 0.37); }
}

// Worley / cell noise on a tiling grid of `cells` × `cells`: returns [d1, d2, cellId]
export function worley(x, y, cells, seed = 7) {
    const xi = Math.floor(x), yi = Math.floor(y);
    let d1 = 9, d2 = 9, id = 0;
    for (let j = -1; j <= 1; j++) for (let i = -1; i <= 1; i++) {
        const cx = ((xi + i) % cells + cells) % cells, cy = ((yi + j) % cells + cells) % cells;
        let h = (cx * 374761393 + cy * 668265263 + seed * 2246822519) >>> 0;
        h = Math.imul(h ^ (h >>> 13), 1274126177) >>> 0;
        const px = xi + i + (h & 1023) / 1023 * 0.8 + 0.1, py = yi + j + ((h >>> 10) & 1023) / 1023 * 0.8 + 0.1;
        const d = Math.hypot(px - x, py - y);
        if (d < d1) { d2 = d1; d1 = d; id = h; } else if (d < d2) d2 = d;
    }
    return [d1, d2, id];
}

export const NOISE = new Noise(1337);
