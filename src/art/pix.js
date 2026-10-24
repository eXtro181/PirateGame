// Pixel painting toolkit: an RGBA image with a parallel emission (glow) layer, primitives,
// ramp shading lit from the top-left, dithering, selective outlines, and an atlas packer.
import { rgb, ramp, outlineOf } from './pal.js';

export const C = (h) => (Array.isArray(h) ? h : rgb(h));

export class Pix {
    constructor(w, h) {
        this.w = w; this.h = h;
        this.col = new Uint8ClampedArray(w * h * 4);
        this.glw = new Uint8ClampedArray(w * h * 4);
    }
    inb(x, y) { return x >= 0 && y >= 0 && x < this.w && y < this.h; }
    px(x, y, c, a = 255) {
        x = Math.floor(x); y = Math.floor(y);
        if (!this.inb(x, y) || !c) return;
        c = C(c);
        const i = (y * this.w + x) * 4;
        if (a < 255 && this.col[i + 3]) {
            const t = a / 255;
            this.col[i] += (c[0] - this.col[i]) * t; this.col[i + 1] += (c[1] - this.col[i + 1]) * t; this.col[i + 2] += (c[2] - this.col[i + 2]) * t;
        } else { this.col[i] = c[0]; this.col[i + 1] = c[1]; this.col[i + 2] = c[2]; this.col[i + 3] = a; }
    }
    glow(x, y, c, k = 1) {
        x = Math.floor(x); y = Math.floor(y);
        if (!this.inb(x, y)) return;
        c = C(c);
        const i = (y * this.w + x) * 4;
        this.glw[i] = c[0] * k; this.glw[i + 1] = c[1] * k; this.glw[i + 2] = c[2] * k; this.glw[i + 3] = 255;
    }
    // pixel + glow in one call (for lanterns, soul stones, eyes)
    lit(x, y, c, k = 1) { this.px(x, y, c); this.glow(x, y, c, k); }
    get(x, y) { if (!this.inb(x, y)) return null; const i = (y * this.w + x) * 4; return [this.col[i], this.col[i + 1], this.col[i + 2], this.col[i + 3]]; }
    a(x, y) { return this.inb(x, y) ? this.col[(y * this.w + x) * 4 + 3] : 0; }
    clear(x, y) { if (!this.inb(x, y)) return; const i = (y * this.w + x) * 4; this.col[i + 3] = 0; this.glw[i + 3] = 0; }
    rect(x, y, w, h, c) { for (let j = 0; j < h; j++) for (let i = 0; i < w; i++) this.px(x + i, y + j, c); }
    hline(x0, x1, y, c) { for (let x = Math.min(x0, x1); x <= Math.max(x0, x1); x++) this.px(x, y, c); }
    vline(x, y0, y1, c) { for (let y = Math.min(y0, y1); y <= Math.max(y0, y1); y++) this.px(x, y, c); }
    line(x0, y0, x1, y1, c) {
        x0 = Math.round(x0); y0 = Math.round(y0); x1 = Math.round(x1); y1 = Math.round(y1);
        const dx = Math.abs(x1 - x0), sx = x0 < x1 ? 1 : -1, dy = -Math.abs(y1 - y0), sy = y0 < y1 ? 1 : -1;
        let err = dx + dy;
        for (;;) {
            this.px(x0, y0, c);
            if (x0 === x1 && y0 === y1) break;
            const e2 = 2 * err;
            if (e2 >= dy) { err += dy; x0 += sx; }
            if (e2 <= dx) { err += dx; y0 += sy; }
        }
    }
    ellipse(cx, cy, rx, ry, c) {
        for (let y = Math.floor(cy - ry); y <= Math.ceil(cy + ry); y++) for (let x = Math.floor(cx - rx); x <= Math.ceil(cx + rx); x++) {
            const dx = (x + 0.5 - cx) / rx, dy = (y + 0.5 - cy) / ry;
            if (dx * dx + dy * dy <= 1) this.px(x, y, c);
        }
    }
    // shaded ellipse: 5-step ramp, light from the top-left, ordered dither between steps
    blob(cx, cy, rx, ry, hex, { spread = 1, dither = true, rim = true } = {}) {
        const R = ramp(hex, spread);
        for (let y = Math.floor(cy - ry); y <= Math.ceil(cy + ry); y++) for (let x = Math.floor(cx - rx); x <= Math.ceil(cx + rx); x++) {
            const dx = (x + 0.5 - cx) / rx, dy = (y + 0.5 - cy) / ry, d2 = dx * dx + dy * dy;
            if (d2 > 1) continue;
            // light vector from the top-left
            const nz = Math.sqrt(Math.max(0, 1 - d2));
            let l = (-dx * 0.55 - dy * 0.65 + nz * 0.55) * 0.9 + 0.1;
            if (rim && d2 > 0.82 && dx > 0.2 && dy > 0.1) l -= 0.25;
            let v = (l + 0.35) * 2.6; // 0..~4
            if (dither) v += BAYER[(y & 3) * 4 + (x & 3)] * 0.9 - 0.45;
            this.px(x, y, R[Math.max(0, Math.min(4, Math.floor(v)))]);
        }
    }
    // vertical gradient band in a material ramp (top light → bottom dark), dithered
    vramp(x, y, w, h, hex, { from = 3, to = 0, dither = true } = {}) {
        const R = ramp(hex);
        for (let j = 0; j < h; j++) {
            const t = h > 1 ? j / (h - 1) : 0;
            for (let i = 0; i < w; i++) {
                let v = from + (to - from) * t;
                if (dither) v += BAYER[((y + j) & 3) * 4 + ((x + i) & 3)] * 0.9 - 0.45;
                this.px(x + i, y + j, R[Math.max(0, Math.min(4, Math.round(v)))]);
            }
        }
    }
    // selective outline: transparent pixels next to opaque ones get the neighbour's material outline
    outline({ inner = false, skipBottom = false } = {}) {
        const add = [];
        for (let y = 0; y < this.h; y++) for (let x = 0; x < this.w; x++) {
            if (this.a(x, y)) continue;
            for (const [dx, dy] of [[0, 1], [0, -1], [1, 0], [-1, 0]]) {
                if (skipBottom && dy === -1) continue;
                const n = this.get(x + dx, y + dy);
                if (n && n[3]) { add.push(x, y, outlineOf(n)); break; }
            }
        }
        for (let i = 0; i < add.length; i += 3) this.px(add[i], add[i + 1], add[i + 2]);
        if (inner) { // darken the lower-right inner edge a touch
            for (let y = 0; y < this.h; y++) for (let x = 0; x < this.w; x++) {
                if (!this.a(x, y)) continue;
                if (!this.a(x + 1, y) || !this.a(x, y + 1)) { const c = this.get(x, y); this.px(x, y, [c[0] * 0.85, c[1] * 0.85, c[2] * 0.9]); }
            }
        }
    }
    blit(src, ox, oy, flip = false) {
        for (let y = 0; y < src.h; y++) for (let x = 0; x < src.w; x++) {
            const sx = flip ? src.w - 1 - x : x, i = (y * src.w + sx) * 4;
            if (!src.col[i + 3]) continue;
            const dx = ox + x, dy = oy + y;
            if (!this.inb(dx, dy)) continue;
            const j = (dy * this.w + dx) * 4;
            for (let k = 0; k < 4; k++) { this.col[j + k] = src.col[i + k]; this.glw[j + k] = src.glw[i + k]; }
        }
    }
    canvas(which = 'col') {
        const c = document.createElement('canvas'); c.width = this.w; c.height = this.h;
        const g = c.getContext('2d'); const img = g.createImageData(this.w, this.h);
        img.data.set(which === 'col' ? this.col : this.glw);
        g.putImageData(img, 0, 0);
        return c;
    }
}

export const BAYER = [0, 8, 2, 10, 12, 4, 14, 6, 3, 11, 1, 9, 15, 7, 13, 5].map(v => v / 16);

// shelf packer → two textures (colour + glow) sharing the same rects
export class Atlas {
    constructor(w = 2048, h = 2048) {
        this.img = new Pix(w, h);
        this.img.px(0, 0, [255, 255, 255]); // white texel for solid rects
        this.x = 4; this.y = 4; this.rowH = 0;
        this.rects = {};
    }
    add(name, pix) {
        if (this.x + pix.w + 2 > this.img.w) { this.x = 4; this.y += this.rowH + 3; this.rowH = 0; }
        if (this.y + pix.h + 2 > this.img.h) throw new Error('Atlas voll: ' + name);
        this.img.blit(pix, this.x, this.y);
        let g = false;
        for (let i = 3; i < pix.glw.length && !g; i += 4) if (pix.glw[i]) g = true;
        const r = { x: this.x, y: this.y, w: pix.w, h: pix.h, glow: g };
        this.rects[name] = r;
        this.x += pix.w + 3;
        this.rowH = Math.max(this.rowH, pix.h);
        return r;
    }
    // upload to the GPU: every rect gets .tex (colour) and .gtex (glow)
    upload(renderer) {
        this.tex = renderer.texture(this.img.canvas('col'));
        this.gtex = renderer.texture(this.img.canvas('glw'));
        for (const r of Object.values(this.rects)) { r.tex = this.tex; r.gtex = this.gtex; }
        this.white = { x: 0, y: 0, w: 1, h: 1, tex: this.tex };
        return this;
    }
}
