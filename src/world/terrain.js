// Height field and material weights derived from the tile map. The 3D terrain mesh, the foam
// band and everything that stands on the ground (figures, props) use the same height lookup.
import { T } from './defs.js';
import { Noise } from '../gfx/noise.js';

export const RES = 2; // vertices per tile
const NZ = new Noise(31);
const smooth = (a, b, x) => { const t = Math.max(0, Math.min(1, (x - a) / (b - a))); return t * t * (3 - 2 * t); };

export class Terrain {
    constructor(map) {
        this.map = map;
        const W = map.w * RES + 1, H = map.h * RES + 1;
        this.W = W; this.H = H;
        this.h = new Float32Array(W * H);
        this.w = new Float32Array(W * H * 4); // splat weights sand, grass, dirt, stone
        this.coast = new Float32Array(W * H); // 0 = deep sea, 1 = inland; 0.5 is the shoreline
        // per-tile one-hot fields
        const tw = map.w, th = map.h;
        const field = (fn) => { const f = new Float32Array(tw * th); for (let y = 0; y < th; y++) for (let x = 0; x < tw; x++) f[y * tw + x] = fn(map.get(x, y)); return f; };
        const land = field(t => (t !== T.WATER && t !== T.DOCK ? 1 : 0));
        const rock = field(t => (t === T.ROCK ? 1 : 0));
        const sand = field(t => (t === T.SAND || t === T.WATER || t === T.DOCK ? 1 : 0));
        const grass = field(t => (t === T.GRASS ? 1 : t === T.DARK ? 0.6 : t === T.ROCK ? 0.4 : 0));
        const dirt = field(t => (t === T.DIRT ? 1 : t === T.DARK ? 0.4 : t === T.ROCK ? 0.6 : 0));
        const stone = field(t => (t === T.STONE ? 1 : 0));
        // sample a tile field blurred with a small box kernel at a fractional tile position
        const sample = (f, x, z, r) => {
            let s = 0, n = 0;
            for (let j = -r; j <= r; j++) for (let i = -r; i <= r; i++) {
                const tx = Math.floor(x + i * 0.5), tz = Math.floor(z + j * 0.5);
                const v = (tx < 0 || tz < 0 || tx >= tw || tz >= th) ? (f === sand ? 1 : 0) : f[tz * tw + tx];
                s += v; n++;
            }
            return s / n;
        };
        for (let j = 0; j < H; j++) for (let i = 0; i < W; i++) {
            const x = i / RES, z = j / RES;
            const L = sample(land, x - 0.5, z - 0.5, 2);
            const R = sample(rock, x - 0.5, z - 0.5, 1);
            const n = NZ.fbm(x * 0.09, z * 0.09, 4);
            const d = NZ.fbm(x * 0.5 + 9, z * 0.5, 3);
            let y = -1.25 + smooth(0.15, 0.75, L) * 1.25; // sea floor up to the beach
            y += smooth(0.7, 1, L) * (n * 0.35 + 0.12 + d * 0.05);
            y += Math.pow(R, 1.3) * (1.9 + NZ.ridged(x * 0.35, z * 0.35, 3) * 0.9);
            const st = sample(stone, x - 0.5, z - 0.5, 1);
            y = y * (1 - st * 0.6) + st * 0.6 * 0.12; // ruins: level floor
            const k = j * W + i;
            this.h[k] = y;
            this.coast[k] = L;
            const ws = [sample(sand, x - 0.5, z - 0.5, 1), sample(grass, x - 0.5, z - 0.5, 1), sample(dirt, x - 0.5, z - 0.5, 1), st];
            const sum = ws[0] + ws[1] + ws[2] + ws[3] || 1;
            for (let c = 0; c < 4; c++) this.w[k * 4 + c] = ws[c] / sum;
        }
    }
    // bilinear ground height at world (x, z)
    height(x, z) {
        const fx = Math.max(0, Math.min(this.W - 1.001, x * RES)), fz = Math.max(0, Math.min(this.H - 1.001, z * RES));
        const i = Math.floor(fx), j = Math.floor(fz), u = fx - i, v = fz - j;
        const W = this.W, h = this.h;
        const a = h[j * W + i], b = h[j * W + i + 1], c = h[(j + 1) * W + i], d = h[(j + 1) * W + i + 1];
        return a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v;
    }
    // walkers never stand below the waterline (docks, shallow beaches)
    ground(x, z) { return Math.max(this.height(x, z), this.map.get(Math.floor(x), Math.floor(z)) === T.DOCK ? 0.32 : -0.05); }
    normal(i, j) {
        const W = this.W, H = this.H;
        const at = (a, b) => this.h[Math.max(0, Math.min(H - 1, b)) * W + Math.max(0, Math.min(W - 1, a))];
        const dx = (at(i + 1, j) - at(i - 1, j)) * RES / 2, dz = (at(i, j + 1) - at(i, j - 1)) * RES / 2;
        const l = Math.hypot(dx, 1, dz);
        return [-dx / l, 1 / l, -dz / l];
    }
}
