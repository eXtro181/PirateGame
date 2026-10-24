// Pixel painter: draws pixel art into RGBA buffers and derives normal + emission maps.
// Every sprite in the game is painted here at runtime (placeholder for Aseprite exports).

export class Rng {
    constructor(seed = 1) { this.s = seed >>> 0 || 1; }
    next() { // mulberry32
        let t = (this.s += 0x6D2B79F5);
        t = Math.imul(t ^ (t >>> 15), t | 1);
        t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
        return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
    }
    range(a, b) { return a + (b - a) * this.next(); }
    int(a, b) { return Math.floor(this.range(a, b + 1)); }
    pick(arr) { return arr[Math.floor(this.next() * arr.length)]; }
    chance(p) { return this.next() < p; }
}

export function hex(h, a = 255) {
    const v = parseInt(h.replace('#', ''), 16);
    return [(v >> 16) & 255, (v >> 8) & 255, v & 255, a];
}

export function shade(c, f) { // f < 1 darker, > 1 lighter
    return [Math.min(255, c[0] * f) | 0, Math.min(255, c[1] * f) | 0, Math.min(255, c[2] * f) | 0, c[3]];
}

export function mix(a, b, t) {
    return [a[0] + (b[0] - a[0]) * t | 0, a[1] + (b[1] - a[1]) * t | 0, a[2] + (b[2] - a[2]) * t | 0, a[3] + (b[3] - a[3]) * t | 0];
}

// A layered pixel image: color, height (for normals) and emission.
export class Pix {
    constructor(w, h) {
        this.w = w; this.h = h;
        this.col = new Uint8ClampedArray(w * h * 4);
        this.hgt = new Float32Array(w * h);
        this.emi = new Uint8ClampedArray(w * h * 4);
        this.flat = new Uint8Array(w * h); // 1 = normal points straight up (no dome)
    }
    inb(x, y) { return x >= 0 && y >= 0 && x < this.w && y < this.h; }
    px(x, y, c, hgt = null) {
        x |= 0; y |= 0;
        if (!this.inb(x, y) || !c) return;
        const i = (y * this.w + x) * 4;
        if (c[3] < 255 && this.col[i + 3] > 0) { // alpha blend over existing
            const a = c[3] / 255;
            this.col[i] = this.col[i] * (1 - a) + c[0] * a;
            this.col[i + 1] = this.col[i + 1] * (1 - a) + c[1] * a;
            this.col[i + 2] = this.col[i + 2] * (1 - a) + c[2] * a;
        } else {
            this.col[i] = c[0]; this.col[i + 1] = c[1]; this.col[i + 2] = c[2]; this.col[i + 3] = c[3];
        }
        if (hgt !== null) this.hgt[y * this.w + x] = hgt;
    }
    get(x, y) {
        if (!this.inb(x, y)) return null;
        const i = (y * this.w + x) * 4;
        return [this.col[i], this.col[i + 1], this.col[i + 2], this.col[i + 3]];
    }
    alpha(x, y) { return this.inb(x, y) ? this.col[(y * this.w + x) * 4 + 3] : 0; }
    glow(x, y, c) {
        x |= 0; y |= 0;
        if (!this.inb(x, y)) return;
        const i = (y * this.w + x) * 4;
        this.emi[i] = c[0]; this.emi[i + 1] = c[1]; this.emi[i + 2] = c[2]; this.emi[i + 3] = 255;
    }
    rect(x, y, w, h, c, hgt = null) { for (let j = 0; j < h; j++) for (let i = 0; i < w; i++) this.px(x + i, y + j, c, hgt); }
    hline(x0, x1, y, c, hgt = null) { for (let x = x0; x <= x1; x++) this.px(x, y, c, hgt); }
    vline(x, y0, y1, c, hgt = null) { for (let y = y0; y <= y1; y++) this.px(x, y, c, hgt); }
    line(x0, y0, x1, y1, c, hgt = null) {
        x0 |= 0; y0 |= 0; x1 |= 0; y1 |= 0;
        const dx = Math.abs(x1 - x0), sx = x0 < x1 ? 1 : -1, dy = -Math.abs(y1 - y0), sy = y0 < y1 ? 1 : -1;
        let err = dx + dy;
        for (;;) {
            this.px(x0, y0, c, hgt);
            if (x0 === x1 && y0 === y1) break;
            const e2 = 2 * err;
            if (e2 >= dy) { err += dy; x0 += sx; }
            if (e2 <= dx) { err += dx; y0 += sy; }
        }
    }
    disc(cx, cy, r, c, hgt = null) {
        for (let y = Math.floor(cy - r); y <= Math.ceil(cy + r); y++) {
            for (let x = Math.floor(cx - r); x <= Math.ceil(cx + r); x++) {
                const d = Math.hypot(x + 0.5 - cx, y + 0.5 - cy);
                if (d <= r) this.px(x, y, c, hgt);
            }
        }
    }
    ellipse(cx, cy, rx, ry, c, hgt = null) {
        for (let y = Math.floor(cy - ry); y <= Math.ceil(cy + ry); y++) {
            for (let x = Math.floor(cx - rx); x <= Math.ceil(cx + rx); x++) {
                const dx = (x + 0.5 - cx) / rx, dy = (y + 0.5 - cy) / ry;
                if (dx * dx + dy * dy <= 1) this.px(x, y, c, hgt);
            }
        }
    }
    // dark outline around opaque pixels
    outline(c) {
        const add = [];
        for (let y = 0; y < this.h; y++) {
            for (let x = 0; x < this.w; x++) {
                if (this.alpha(x, y)) continue;
                if (this.alpha(x - 1, y) || this.alpha(x + 1, y) || this.alpha(x, y - 1) || this.alpha(x, y + 1)) add.push(x, y);
            }
        }
        for (let i = 0; i < add.length; i += 2) this.px(add[i], add[i + 1], c, 0.05);
    }
    // copy another Pix into this one at offset (optionally mirrored)
    blit(src, ox, oy, flipX = false) {
        for (let y = 0; y < src.h; y++) {
            for (let x = 0; x < src.w; x++) {
                const sx = flipX ? src.w - 1 - x : x;
                const i = (y * src.w + sx) * 4;
                if (src.col[i + 3] === 0) continue;
                const dx = ox + x, dy = oy + y;
                if (!this.inb(dx, dy)) continue;
                const j = (dy * this.w + dx) * 4;
                for (let k = 0; k < 4; k++) { this.col[j + k] = src.col[i + k]; this.emi[j + k] = src.emi[i + k]; }
                this.hgt[dy * this.w + dx] = src.hgt[y * src.w + sx];
                this.flat[dy * this.w + dx] = src.flat[y * src.w + sx];
            }
        }
    }
    // Auto-height: a soft dome from the distance to the transparent edge, plus luminance detail.
    autoHeight(dome = 1, detail = 0.35) {
        const { w, h } = this;
        const dist = new Float32Array(w * h);
        const R = 6;
        for (let y = 0; y < h; y++) {
            for (let x = 0; x < w; x++) {
                if (!this.alpha(x, y)) { dist[y * w + x] = 0; continue; }
                let best = R;
                for (let j = -R; j <= R; j++) {
                    for (let i = -R; i <= R; i++) {
                        if (!this.alpha(x + i, y + j)) {
                            const d = Math.hypot(i, j);
                            if (d < best) best = d;
                        }
                    }
                }
                dist[y * w + x] = best;
            }
        }
        for (let k = 0; k < w * h; k++) {
            if (!this.col[k * 4 + 3]) continue;
            const lum = (this.col[k * 4] * 0.3 + this.col[k * 4 + 1] * 0.55 + this.col[k * 4 + 2] * 0.15) / 255;
            const d = Math.min(dist[k], R) / R;
            this.hgt[k] = Math.max(this.hgt[k], dome * Math.sqrt(d) + detail * lum);
        }
    }
    // Normal map in tangent space: R = +u (right), G = +v (image down), B = up.
    normalData(strength = 2.2, cell = 0) {
        const { w, h } = this;
        const out = new Uint8ClampedArray(w * h * 4);
        let bx = 0, by = 0, ex = w - 1, ey = h - 1;
        const H = (x, y) => {
            x = Math.max(bx, Math.min(ex, x)); y = Math.max(by, Math.min(ey, y));
            return this.alpha(x, y) ? this.hgt[y * w + x] : 0;
        };
        for (let y = 0; y < h; y++) {
            for (let x = 0; x < w; x++) {
                if (cell) { bx = x - x % cell; by = y - y % cell; ex = bx + cell - 1; ey = by + cell - 1; }
                const k = y * w + x;
                let nx = 0, ny = 0, nz = 1;
                if (!this.flat[k]) {
                    nx = -(H(x + 1, y) - H(x - 1, y)) * strength;
                    ny = -(H(x, y + 1) - H(x, y - 1)) * strength;
                    const l = Math.hypot(nx, ny, nz);
                    nx /= l; ny /= l; nz /= l;
                }
                out[k * 4] = (nx * 0.5 + 0.5) * 255;
                out[k * 4 + 1] = (ny * 0.5 + 0.5) * 255;
                out[k * 4 + 2] = (nz * 0.5 + 0.5) * 255;
                out[k * 4 + 3] = 255;
            }
        }
        return out;
    }
    toCanvas(data = this.col) {
        const c = document.createElement('canvas');
        c.width = this.w; c.height = this.h;
        const ctx = c.getContext('2d');
        const img = ctx.createImageData(this.w, this.h);
        img.data.set(data);
        ctx.putImageData(img, 0, 0);
        return c;
    }
}

// Simple shelf packer to build atlases from many Pix images.
export class Atlas {
    constructor(w, h) {
        this.img = new Pix(w, h);
        this.x = 1; this.y = 1; this.rowH = 0;
        this.rects = {};
    }
    add(name, pix) {
        if (this.x + pix.w + 1 > this.img.w) { this.x = 1; this.y += this.rowH + 2; this.rowH = 0; }
        if (this.y + pix.h > this.img.h) throw new Error('atlas full: ' + name);
        this.img.blit(pix, this.x, this.y);
        this.rects[name] = { x: this.x, y: this.y, w: pix.w, h: pix.h };
        this.x += pix.w + 2;
        this.rowH = Math.max(this.rowH, pix.h);
        return this.rects[name];
    }
}
