// World generation for the first playable region: the Emerald Coast at night.
import { Rng } from '../art/painter.js';
import { T, VARIANTS } from '../art/tiles.js';
import { PROP_INFO } from '../art/props.js';

export const MAP_W = 128, MAP_H = 96;

function vnoise(seed) { // value noise
    const rng = new Rng(seed);
    const g = new Float32Array(64 * 64).map(() => rng.next());
    const at = (x, y) => g[((y & 63) * 64) + (x & 63)];
    return (x, y) => {
        const xi = Math.floor(x), yi = Math.floor(y), xf = x - xi, yf = y - yi;
        const s = t => t * t * (3 - 2 * t);
        const a = at(xi, yi), b = at(xi + 1, yi), c = at(xi, yi + 1), d = at(xi + 1, yi + 1);
        return a + (b - a) * s(xf) + (c - a) * s(yf) + (a - b - c + d) * s(xf) * s(yf);
    };
}

export class WorldMap {
    constructor() {
        this.w = MAP_W; this.h = MAP_H;
        this.tiles = new Uint8Array(this.w * this.h);
        this.props = [];
        this.colliders = [];
        this.fires = [];
        this.spawns = [];
        this.lights = [];
        this.chests = [];
        this.generate();
    }
    idx(x, y) { return y * this.w + x; }
    get(x, y) { return (x < 0 || y < 0 || x >= this.w || y >= this.h) ? T.WATER : this.tiles[this.idx(x, y)]; }
    set(x, y, t) { if (x >= 0 && y >= 0 && x < this.w && y < this.h) this.tiles[this.idx(x, y)] = t; }
    variant(x, y) { return ((x * 73856093) ^ (y * 19349663)) >>> 0 & (VARIANTS - 1); }
    isLand(x, y) { const t = this.get(x, y); return t !== T.WATER && t !== T.DOCK; }
    waterMask(x, y) {
        const w = (a, b) => this.get(a, b) === T.WATER || this.get(a, b) === T.DOCK;
        return (w(x, y - 1) ? 1 : 0) | (w(x + 1, y) ? 2 : 0) | (w(x, y + 1) ? 4 : 0) | (w(x - 1, y) ? 8 : 0);
    }

    island(cx, cy, r, seed, rough = 0.35) {
        const n = vnoise(seed);
        for (let y = Math.floor(cy - r * 1.6); y <= cy + r * 1.6; y++) {
            for (let x = Math.floor(cx - r * 1.6); x <= cx + r * 1.6; x++) {
                const d = Math.hypot((x - cx) / 1.15, y - cy) / r;
                const k = d - (n(x * 0.12, y * 0.12) - 0.5) * rough * 2 - (n(x * 0.35 + 9, y * 0.35) - 0.5) * 0.15;
                if (k < 1) this.set(x, y, T.GRASS);
            }
        }
    }

    generate() {
        const rng = new Rng(20261024);
        // islands
        this.island(42, 50, 23, 11);
        this.island(98, 22, 7.5, 5, 0.25);
        this.island(112, 46, 2.6, 8, 0.2);
        this.island(70, 82, 2.2, 9, 0.2);
        this.island(16, 18, 3, 12, 0.3);
        // remove single-tile water holes and lone land tiles
        for (let pass = 0; pass < 2; pass++) {
            for (let y = 1; y < this.h - 1; y++) for (let x = 1; x < this.w - 1; x++) {
                const land = [this.get(x - 1, y), this.get(x + 1, y), this.get(x, y - 1), this.get(x, y + 1)].filter(t => t !== T.WATER).length;
                if (this.get(x, y) === T.WATER && land >= 3) this.set(x, y, T.GRASS);
                if (this.get(x, y) !== T.WATER && land <= 1) this.set(x, y, T.WATER);
            }
        }
        // distance to water -> beaches
        const dist = new Float32Array(this.w * this.h).fill(99);
        for (let y = 0; y < this.h; y++) for (let x = 0; x < this.w; x++) {
            if (this.get(x, y) === T.WATER) { dist[this.idx(x, y)] = 0; continue; }
            let best = 99;
            for (let j = -4; j <= 4; j++) for (let i = -4; i <= 4; i++) if (this.get(x + i, y + j) === T.WATER) best = Math.min(best, Math.hypot(i, j));
            dist[this.idx(x, y)] = best;
        }
        this.dist = dist;
        const n = vnoise(77);
        for (let y = 0; y < this.h; y++) for (let x = 0; x < this.w; x++) {
            if (this.get(x, y) === T.WATER) continue;
            const d = dist[this.idx(x, y)];
            if (d <= 2.3 + n(x * 0.3, y * 0.3) * 1.5) this.set(x, y, T.SAND);
            else if (n(x * 0.09 + 30, y * 0.09) > 0.62) this.set(x, y, T.DARK);
        }
        // northern cliff ridge on the main island
        for (let x = 30; x <= 54; x++) {
            const yy = 33 + Math.round(Math.sin(x * 0.35) * 1.5 + (x - 42) * (x - 42) * 0.012);
            for (let k = 0; k < 2 + (x % 3 === 0 ? 1 : 0); k++) if (this.get(x, yy - k) !== T.WATER && dist[this.idx(x, yy - k)] > 2) this.set(x, yy - k, T.ROCK);
        }
        // a gap in the ridge (pass to the north beach)
        for (let k = -4; k <= 0; k++) for (let x = 40; x <= 41; x++) if (this.get(x, 33 + k) === T.ROCK) this.set(x, 33 + k, T.DIRT);
        for (let y = 38; y <= 46; y++) for (let x = 26; x <= 30; x++) if (n(x * 0.7, y * 0.7) > 0.55 && dist[this.idx(x, y)] > 3) this.set(x, y, T.ROCK);

        // ruins (stone floor) in the island centre
        const rx = 44, ry = 40;
        for (let y = ry; y < ry + 8; y++) for (let x = rx; x < rx + 11; x++) if (rng.next() > 0.12) this.set(x, y, T.STONE);

        // fires
        const fire1 = this.findLand(24, 60, 3);
        this.fires.push({ id: 'kueste', name: 'Strand der Ertränkten', kind: 'land', x: fire1.x + 0.5, z: fire1.y + 0.5 });
        const fire2 = this.findLand(98, 24, 3);
        this.fires.push({ id: 'insel', name: 'Knocheninsel', kind: 'land', x: fire2.x + 0.5, z: fire2.y + 0.5 });
        this.fires.push({ id: 'wrack', name: 'Brennendes Wrack', kind: 'sea', x: 92, z: 72 });
        this.start = { x: fire1.x - 3.5, z: fire1.y + 3.5 };
        if (!this.walkableTile(Math.floor(this.start.x), Math.floor(this.start.z))) this.start = { x: fire1.x + 0.5, z: fire1.y + 2.5 };

        // dock on the east coast
        let dx = 55, dy = 52;
        while (this.get(dx + 1, dy) !== T.WATER && dx < 100) dx++;
        for (let k = 1; k <= 6; k++) { this.set(dx + k, dy, T.DOCK); this.set(dx + k, dy + 1, T.DOCK); }
        this.dock = { x: dx + 6.5, z: dy + 1, shipX: dx + 10.2, shipZ: dy + 1, heading: Math.PI / 2 };
        this.props.push({ type: 'post', v: 0, x: dx + 6.6, z: dy + 0.4 }, { type: 'post', v: 0, x: dx + 6.6, z: dy + 2.3 });

        // dirt paths: fire1 -> ruins -> dock, ruins -> ridge pass
        this.path(fire1.x, fire1.y - 1, rx + 2, ry + 8);
        this.path(rx + 10, ry + 6, dx - 1, dy);
        this.path(41, ry - 1, 41, 30);

        // ruin dressing
        const pillars = [[rx + 1, ry + 1], [rx + 5, ry], [rx + 9, ry + 1], [rx + 1, ry + 6], [rx + 9, ry + 6]];
        pillars.forEach(([x, y], i) => this.addProp('pillar', i % 2, x + 0.5, y + 0.9));
        this.addProp('wall', 0, rx + 3.5, ry + 7.8); this.addProp('wall', 1, rx + 8, ry - 0.2);
        this.addProp('torch', 0, rx + 3.5, ry + 3.6); this.addProp('torch', 0, rx + 7.5, ry + 3.6);
        this.lights.push({ kind: 'torch', x: rx + 3.5, z: ry + 3.2 }, { kind: 'torch', x: rx + 7.5, z: ry + 3.2 });
        this.addProp('skulls', 0, rx + 6, ry + 5); this.addProp('skulls', 0, rx + 2.4, ry + 4.2);
        this.chests.push({ id: 'ruine', x: rx + 5.5, z: ry + 2.8, pearls: 300, item: 'flask' });
        this.colliders.push({ x: rx + 5.5, z: ry + 2.55, r: 0.45 });
        // graves on the islet + camp
        this.addProp('grave', 0, fire2.x - 2, fire2.y - 2); this.addProp('grave', 0, fire2.x + 3, fire2.y - 1.5); this.addProp('grave', 0, fire2.x - 3, fire2.y + 1);
        this.addProp('crate', 0, fire2.x + 3, fire2.y + 2); this.addProp('barrel', 0, fire2.x + 4, fire2.y + 2.6);
        this.chests.push({ id: 'insel', x: fire2.x + 1.5, z: fire2.y - 3.5, pearls: 600, item: 'shard' });
        this.colliders.push({ x: fire2.x + 1.5, z: fire2.y - 3.75, r: 0.45 });
        // crates by the dock
        this.addProp('crate', 0, dx - 0.5, dy - 1.2); this.addProp('barrel', 0, dx - 1.4, dy - 0.6); this.addProp('barrel', 0, dx + 0.3, dy + 3.2);
        this.addProp('torch', 0, dx + 0.5, dy - 0.6); this.lights.push({ kind: 'torch', x: dx + 0.5, z: dy - 1.0 });
        // the saber fires and wreck as props
        this.addProp('saberFire', 0, this.fires[0].x, this.fires[0].z + 0.3);
        this.addProp('saberFire', 0, this.fires[1].x, this.fires[1].z + 0.3);
        this.colliders.push({ x: 92, z: 72, r: 2.4, seaOnly: true });

        // vegetation
        const veg = vnoise(99);
        for (let y = 1; y < this.h - 1; y++) for (let x = 1; x < this.w - 1; x++) {
            const t = this.get(x, y);
            if (t === T.WATER || t === T.DOCK || t === T.STONE || t === T.ROCK || t === T.DIRT) continue;
            if (this.nearKey(x + 0.5, y + 0.5) || (x >= rx - 1 && x <= rx + 11 && y >= ry - 1 && y <= ry + 8)) continue;
            const v = veg(x * 0.2, y * 0.2);
            const d = dist[this.idx(x, y)];
            const r = rng.next();
            const jx = x + rng.range(0.25, 0.75), jy = y + rng.range(0.4, 0.9);
            if (t === T.SAND && d > 1.5 && r < 0.07) this.addProp('palm', rng.int(0, 2), jx, jy);
            else if (t !== T.SAND && r < 0.05 + v * 0.06 && d > 3) this.addProp('palm', rng.int(0, 2), jx, jy);
            else if (t !== T.SAND && r < 0.12 + v * 0.1) this.addProp('bush', rng.int(0, 2), jx, jy);
            else if (r < 0.16) this.addProp(rng.chance(0.75) ? 'boulder' : 'boulderBig', rng.int(0, 1), jx, jy);
            else if (t !== T.SAND && r < 0.38) this.addProp('tuft', rng.int(0, 2), jx, jy);
            else if (t !== T.SAND && r < 0.43) this.addProp('flowers', rng.int(0, 2), jx, jy);
        }

        // enemies
        const beach = [];
        for (let y = 0; y < this.h; y++) for (let x = 0; x < 75; x++) if (this.get(x, y) === T.SAND && !this.nearKey(x, y, 6) && this.walkableTile(x, y)) beach.push([x, y]);
        for (let i = 0; i < 9; i++) { const [x, y] = rng.pick(beach); this.spawns.push({ type: 'crab', x: x + 0.5, z: y + 0.5 }); }
        [[rx + 2, ry + 2], [rx + 8, ry + 3], [rx + 5, ry + 6], [rx + 3, ry - 3], [36, 46]].forEach(([x, y]) => this.spawns.push({ type: 'skeleton', x: x + 0.5, z: y + 0.5 }));
        [[fire2.x - 4, fire2.y - 3], [fire2.x + 4, fire2.y - 4], [fire2.x, fire2.y + 4]].forEach(([x, y]) => this.spawns.push({ type: 'skeleton', x: x + 0.5, z: y + 0.5 }));
        this.spawns.push({ type: 'shark', x: 104, z: 60 });
        this.whirlpools = [{ x: 78, z: 30, r: 9 }];
        // fireflies over dark grass
        for (let i = 0; i < 26; i++) {
            const x = rng.range(22, 62), y = rng.range(30, 70);
            if (this.isLand(Math.floor(x), Math.floor(y))) this.lights.push({ kind: 'firefly', x, z: y });
        }
    }
    nearKey(x, y, r = 2.5) {
        for (const f of this.fires) if (Math.hypot(f.x - x, f.z - y) < r) return true;
        if (this.dock && Math.abs(y - this.dock.z) < 3 && x > this.dock.x - 9) return true;
        if (Math.hypot(x - (this.start?.x ?? -99), y - (this.start?.z ?? -99)) < r) return true;
        return false;
    }
    findLand(x, y, minDist) {
        for (let r = 0; r < 30; r++) for (let j = -r; j <= r; j++) for (let i = -r; i <= r; i++) {
            const xx = x + i, yy = y + j;
            if (this.isLand(xx, yy) && this.dist[this.idx(xx, yy)] >= minDist) return { x: xx, y: yy };
        }
        return { x, y };
    }
    path(x0, y0, x1, y1) {
        let x = x0, y = y0;
        const rng = new Rng(x0 * 31 + y1);
        let guard = 0;
        while ((x !== x1 || y !== y1) && guard++ < 400) {
            if (this.get(x, y) !== T.WATER && this.get(x, y) !== T.DOCK && this.get(x, y) !== T.STONE) this.set(x, y, T.DIRT);
            if (rng.chance(0.5) && x !== x1) x += Math.sign(x1 - x); else if (y !== y1) y += Math.sign(y1 - y); else x += Math.sign(x1 - x);
            if (this.get(x, y) === T.ROCK) this.set(x, y, T.DIRT);
        }
    }
    addProp(type, v, x, z) {
        this.props.push({ type, v, x, z });
        const info = PROP_INFO[type];
        if (info.r > 0) this.colliders.push({ x, z: z - 0.25, r: info.r });
        if (info.box) this.colliders.push({ x, z: z - 0.4, hw: info.box[0] / 2 + 0.1, hd: info.box[1] / 2 + 0.1, box: true });
    }
    walkableTile(x, y) { const t = this.get(x, y); return t !== T.WATER && t !== T.ROCK; }
    // circle vs world (walker)
    blockedWalk(x, z, r = 0.3) {
        const d = r * 0.72; // sample a circle, not a square: corners no longer snag
        for (const [ox, oz] of [[-d, -d], [d, -d], [-d, d], [d, d], [r, 0], [-r, 0], [0, r], [0, -r], [0, 0]]) {
            if (!this.walkableTile(Math.floor(x + ox), Math.floor(z + oz))) return true;
        }
        for (const c of this.colliders) {
            if (c.seaOnly) continue;
            if (c.box) { if (Math.abs(x - c.x) < c.hw + r && Math.abs(z - c.z) < c.hd + r) return true; continue; }
            if ((x - c.x) ** 2 + (z - c.z) ** 2 < (c.r + r) ** 2) return true;
        }
        return false;
    }
    // move a walking circle by (dx, dz) and slide along whatever it touches.
    // Returns the new position and whether the full move was blocked.
    moveCircle(x, z, dx, dz, r) {
        if (!dx && !dz) return { x, z, hit: false };
        if (!this.blockedWalk(x + dx, z + dz, r)) return { x: x + dx, z: z + dz, hit: false };
        // 1. slide along circle colliders: drop the part of the motion pointing into them
        for (const c of this.colliders) {
            if (c.seaOnly || c.box) continue;
            const ox = x + dx - c.x, oz = z + dz - c.z, l = Math.hypot(ox, oz), min = c.r + r;
            if (l >= min || l < 1e-5) continue;
            const nx = ox / l, nz = oz / l, into = dx * nx + dz * nz;
            if (into < 0) {
                const sx = dx - nx * into, sz = dz - nz * into;
                // keep the speed and push gently out of the overlap
                const k = Math.hypot(dx, dz) / (Math.hypot(sx, sz) || 1);
                const tx = x + sx * Math.min(k, 1.4) + nx * (min - l) * 0.5, tz = z + sz * Math.min(k, 1.4) + nz * (min - l) * 0.5;
                if (!this.blockedWalk(tx, tz, r)) return { x: tx, z: tz, hit: true };
            }
        }
        // 2. axis slides (walls, tile edges)
        if (Math.abs(dx) > 1e-6 && !this.blockedWalk(x + dx, z, r)) return { x: x + dx, z, hit: true };
        if (Math.abs(dz) > 1e-6 && !this.blockedWalk(x, z + dz, r)) return { x, z: z + dz, hit: true };
        // 3. try deflected directions around corners
        const sp = Math.hypot(dx, dz), a = Math.atan2(dz, dx);
        for (const off of [0.6, -0.6, 1.1, -1.1]) {
            const tx = x + Math.cos(a + off) * sp * 0.8, tz = z + Math.sin(a + off) * sp * 0.8;
            if (!this.blockedWalk(tx, tz, r)) return { x: tx, z: tz, hit: true };
        }
        // 4. stuck inside something (e.g. after a knockback): small nudge out
        if (this.blockedWalk(x, z, r)) {
            for (let k = 0; k < 8; k++) {
                const ang = k * Math.PI / 4, tx = x + Math.cos(ang) * 0.06, tz = z + Math.sin(ang) * 0.06;
                if (!this.blockedWalk(tx, tz, r * 0.8)) return { x: tx, z: tz, hit: true };
            }
        }
        return { x, z, hit: true };
    }

    // heading given: test the long ship hull (bow, stern, both sides) instead of a square
    blockedSail(x, z, r = 0.8, heading = null) {
        let pts;
        if (heading !== null) {
            const fx = Math.sin(heading), fz = -Math.cos(heading), sx = -fz, sz = fx, half = r * 2.05, side = r * 0.78;
            pts = [[0, 0], [fx * half, fz * half], [-fx * half, -fz * half], [sx * side, sz * side], [-sx * side, -sz * side],
                [fx * half * 0.8 + sx * side * 0.6, fz * half * 0.8 + sz * side * 0.6], [fx * half * 0.8 - sx * side * 0.6, fz * half * 0.8 - sz * side * 0.6], [-fx * half * 0.85 + sx * side, -fz * half * 0.85 + sz * side], [-fx * half * 0.85 - sx * side, -fz * half * 0.85 - sz * side],
                [fx * half * 0.5 + sx * side * 0.9, fz * half * 0.5 + sz * side * 0.9], [fx * half * 0.5 - sx * side * 0.9, fz * half * 0.5 - sz * side * 0.9],
                [-fx * half * 0.5 + sx * side * 0.9, -fz * half * 0.5 + sz * side * 0.9], [-fx * half * 0.5 - sx * side * 0.9, -fz * half * 0.5 - sz * side * 0.9]];
        } else pts = [[-r, -r], [r, -r], [-r, r], [r, r], [0, 0], [0, -r * 1.4], [0, r * 1.4]];
        for (const [ox, oz] of pts) {
            const t = this.get(Math.floor(x + ox), Math.floor(z + oz));
            if (t !== T.WATER) return true;
        }
        for (const c of this.colliders) if (c.seaOnly && (x - c.x) ** 2 + (z - c.z) ** 2 < (c.r + r) ** 2) return true;
        return x < 1 || z < 1 || x > this.w - 1 || z > this.h - 1;
    }
}
