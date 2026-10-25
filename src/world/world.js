// The world: one tile map (16 px tiles) with seven islands — the start coast, the hub Schiffshain,
// the four king regions and the End of the World (reachable only through the whirlpool).
// Height levels: raised plateaus are walkable; their bottom three rows are basalt cliff faces,
// stairs lead up and down. Generation is deterministic. Entity positions are world pixels.
import { G, TS, WALKABLE } from '../art/tiles.js';
import { Noise } from '../gfx/noise.js';

export const MW = 320, MH = 240;
const N = new Noise(4242);
const FH = 3; // cliff face height in tiles

export class Rng {
    constructor(seed = 1) { this.s = seed >>> 0 || 1; }
    next() { let t = (this.s += 0x6D2B79F5); t = Math.imul(t ^ (t >>> 15), t | 1); t ^= t + Math.imul(t ^ (t >>> 7), t | 61); return ((t ^ (t >>> 14)) >>> 0) / 4294967296; }
    range(a, b) { return a + (b - a) * this.next(); }
    int(a, b) { return Math.floor(this.range(a, b + 1)); }
    pick(arr) { return arr[Math.floor(this.next() * arr.length)]; }
    chance(p) { return this.next() < p; }
}

export const ZONES = {
    kueste: { name: 'Strand der Ertränkten', biome: 'coast', grade: 'coast', music: 'salz', cx: 160, cy: 200, rx: 46, ry: 26, seed: 1 },
    hain: { name: 'Schiffshain', biome: 'hub', grade: 'hub', music: 'hain', cx: 160, cy: 112, rx: 48, ry: 34, seed: 2 },
    asche: { name: 'Aschebucht', biome: 'fire', grade: 'fire', music: 'glut', cx: 268, cy: 100, rx: 42, ry: 30, seed: 3, king: 'fire' },
    frost: { name: 'Frostgrotte', biome: 'ice', grade: 'ice', music: 'frost', cx: 56, cy: 58, rx: 44, ry: 32, seed: 4, king: 'ice' },
    raben: { name: 'Hafen Rabenwacht', biome: 'wind', grade: 'wind', music: 'sturm', cx: 56, cy: 176, rx: 44, ry: 32, seed: 5, king: 'wind' },
    mangroven: { name: 'Fluchmangroven', biome: 'earth', grade: 'earth', music: 'tiefe', cx: 268, cy: 196, rx: 42, ry: 30, seed: 6, king: 'earth' },
    ende: { name: 'Ende der Welt', biome: 'end', grade: 'end', music: 'ende', cx: 272, cy: 28, rx: 22, ry: 15, seed: 7, sealed: true }
};
export const BIOMES = ['coast', 'hub', 'fire', 'ice', 'wind', 'earth', 'end'];
export const FIRE_NAMES = {
    kueste: 'Strand der Ertränkten', hain: 'Schiffshain · Seelenbaum', asche: 'Aschebucht · Landung', asche2: 'Vor dem Glutthron',
    frost: 'Frostgrotte · Eingang', frost2: 'Am gefrorenen Fall', raben: 'Rabenwacht · Mole', raben2: 'Rabenwacht · Markt',
    mangroven: 'Fluchmangroven · Steg', mangroven2: 'Wurzelkathedrale', ende: 'Ende der Welt'
};
// vegetation and deco tables per biome: [prop, weight]
const FLORA = {
    coast: { grove: [['palm', 3], ['tealPalm', 2], ['blossomTree0', 2], ['blueTree0', 1], ['greenTree0', 2], ['tealTree0', 1]], bush: ['bushGreen', 'bushTeal', 'bushBlue', 'bushPink', 'fern', 'banana'], deco: ['flowersW', 'flowersY', 'tuft', 'tuft', 'shell', 'pebbles', 'tallgrass', 'mushroomR'] },
    hub: { grove: [['blossomTree0', 2], ['blossomTree1', 1], ['pinkTree0', 2], ['violetTree0', 2], ['violetTree1', 1], ['blueTree0', 2], ['blueTree1', 1], ['tealTree0', 2], ['redTree0', 1]], bush: ['bushBlue', 'bushTeal', 'bushPink', 'bushViolet', 'bushGreen', 'fern', 'bushRed'], deco: ['flowersW', 'flowersW', 'flowersY', 'flowersP', 'flowersB', 'tuft', 'tuft', 'tallgrass', 'mushroomR'] },
    fire: { grove: [['berryTree0', 3], ['berryTree1', 1], ['emberTree0', 1], ['deadTree', 2]], bush: ['bushViolet', 'bushRed', 'lanternPole', 'lanternPole'], deco: ['lanternStalk', 'lanternStalk', 'ember', 'crack', 'skull', 'pebbles', 'flowersR'] },
    ice: { grove: [['snowTree0', 3], ['snowTree1', 1], ['tealTree0', 1], ['deadTree', 1]], bush: ['bushSnow', 'bushTeal', 'crystalBlue', 'crystalBlue'], deco: ['snowtuft', 'snowtuft', 'pebbles', 'flowersB', 'bones'] },
    wind: { grove: [['greenTree0', 2], ['tealTree0', 2], ['blueTree0', 1], ['violetTree0', 1]], bush: ['bushGreen', 'bushTeal', 'bushBlue', 'fern'], deco: ['flowersW', 'tuft', 'tuftDark', 'pebbles', 'puddle', 'tallgrass'] },
    earth: { grove: [['swampTree', 4], ['greenTree0', 1], ['tealTree1', 1]], bush: ['bigLeaf', 'banana', 'bushSwamp', 'fern', 'reedClump', 'cattail'], deco: ['tuftDark', 'tuftDark', 'reeds', 'lily', 'mushroomR', 'coralbud', 'flowersR'] },
    end: { grove: [['deadTree', 2], ['emberTree0', 1]], bush: ['lanternPole', 'bushRed'], deco: ['ember', 'crack', 'skull', 'coralbud'] }
};

export class World {
    constructor(props) {
        this.PROPS = props;
        const n = MW * MH;
        this.ground = new Uint8Array(n); this.biome = new Uint8Array(n); this.zone = new Array(n).fill(null);
        this.level = new Uint8Array(n);      // height level 0..2
        this.cliff = new Uint8Array(n);      // 1 = cliff face tile (blocks unless stairs)
        this.face = new Int8Array(n).fill(-1); // face row 0..2 (0 = top, with the grass lip)
        this.stairs = new Uint8Array(n);     // 1 = stairs through the face
        this.path = new Uint8Array(n);       // reserved walkways (no props)
        this.shoreDist = new Float32Array(n);
        this.props = []; this.deco = []; this.fires = []; this.spawns = []; this.npcs = []; this.items = []; this.chests = [];
        this.docks = []; this.gates = []; this.arenas = {}; this.slots = []; this.lightsStatic = []; this.ghostRoutes = []; this.walkers = []; this.waterfalls = [];
        this.grid = new Map();
        this.generate();
    }
    idx(tx, ty) { return ty * MW + tx; }
    inb(tx, ty) { return tx >= 0 && ty >= 0 && tx < MW && ty < MH; }
    g(tx, ty) { return this.inb(tx, ty) ? this.ground[ty * MW + tx] : G.WATER; }
    set(tx, ty, g) { if (this.inb(tx, ty)) this.ground[ty * MW + tx] = g; }
    lv(tx, ty) { return this.inb(tx, ty) ? this.level[ty * MW + tx] : 0; }
    zoneAt(x, y) {
        const tx = x / TS, ty = y / TS;
        let best = null, bd = 1e9;
        for (const [k, z] of Object.entries(ZONES)) { const d = Math.hypot((tx - z.cx) / z.rx, (ty - z.cy) / z.ry); if (d < bd) { bd = d; best = k; } }
        return best;
    }
    isLand(tx, ty) { return this.g(tx, ty) !== G.WATER; }
    T(tx, ty) { return { x: tx * TS + 8, y: ty * TS + 12 }; }

    // ------------------------------------------------------------------ generation
    generate() {
        for (const [key, z] of Object.entries(ZONES)) this.island(key, z);
        const L = { kueste: this.layoutCoast, hain: this.layoutHub, asche: this.layoutFire, frost: this.layoutIce, raben: this.layoutWind, mangroven: this.layoutEarth, ende: this.layoutEnd };
        for (const [key, z] of Object.entries(ZONES)) { this.cur = key; const rng = new Rng(z.seed * 977); L[key].call(this, z, rng, key); }
        this.computeFaces();
        this.computeShore();
        for (const [key, z] of Object.entries(ZONES)) this.scatter(key, z, new Rng(z.seed * 131));
        this.whirlpool = { x: 160 * TS, y: 64 * TS };
        this.ghostRoutes = [[[214, 150], [226, 160], [206, 168], [196, 150]], [[104, 96], [112, 80], [118, 140]], [[204, 46], [226, 54], [214, 70]], [[108, 210], [124, 226], [96, 224]], [[100, 120], [92, 140], [112, 152]]];
    }
    island(key, z) {
        const bi = BIOMES.indexOf(z.biome);
        const box = [Math.floor(z.cx - z.rx - 8), Math.floor(z.cy - z.ry - 8), Math.ceil(z.cx + z.rx + 8), Math.ceil(z.cy + z.ry + 8)];
        for (let ty = box[1]; ty <= box[3]; ty++) for (let tx = box[0]; tx <= box[2]; tx++) {
            if (!this.inb(tx, ty)) continue;
            const dx = (tx - z.cx) / z.rx, dy = (ty - z.cy) / z.ry;
            const f = 1 - (dx * dx + dy * dy) + N.fbm(tx * 0.06 + z.seed * 10, ty * 0.06, 3, 0) * 0.42;
            const i = this.idx(tx, ty);
            if (f > 0) { this.ground[i] = z.biome === 'ice' ? G.SNOW : z.biome === 'fire' ? G.HEX : z.biome === 'end' ? G.HEX : G.GRASS; this.biome[i] = bi; this.zone[i] = key; }
            else if (!this.zone[i]) { this.biome[i] = bi; this.zone[i] = key; }
        }
        // beach rim + meadow patches
        for (let ty = box[1]; ty <= box[3]; ty++) for (let tx = box[0]; tx <= box[2]; tx++) {
            if (!this.inb(tx, ty) || this.g(tx, ty) === G.WATER || this.zone[this.idx(tx, ty)] !== key) continue;
            let near = false;
            for (let k = -2; k <= 2 && !near; k++) for (let j = -2; j <= 2; j++) if (this.g(tx + j, ty + k) === G.WATER && Math.abs(j) + Math.abs(k) <= 3) { near = true; break; }
            if (near) this.set(tx, ty, z.biome === 'ice' ? G.ICE : z.biome === 'fire' ? G.ASH : z.biome === 'earth' ? G.MUD : z.biome === 'end' ? G.ASH : G.SAND);
            else if ((z.biome === 'coast' || z.biome === 'hub' || z.biome === 'wind') && N.fbm(tx * 0.12 + 40, ty * 0.12, 2, 0) > 0.28) this.set(tx, ty, G.MEADOW);
            else if (z.biome === 'fire' && N.fbm(tx * 0.1 + 7, ty * 0.1, 2, 0) > 0.15) this.set(tx, ty, G.ASH);
            else if (z.biome === 'earth' && N.fbm(tx * 0.1 + 9, ty * 0.1, 2, 0) > 0.2) this.set(tx, ty, G.MUD);
        }
    }
    computeShore() {
        const n = MW * MH, q = new Int32Array(n); let h = 0, t = 0;
        const d = this.shoreDist; d.fill(99);
        for (let i = 0; i < n; i++) if (this.ground[i] !== G.WATER) { d[i] = 0; q[t++] = i; }
        while (h < t) {
            const i = q[h++], x = i % MW, y = (i / MW) | 0;
            for (const [dx, dy] of [[1, 0], [-1, 0], [0, 1], [0, -1]]) {
                const nx = x + dx, ny = y + dy; if (!this.inb(nx, ny)) continue;
                const j = ny * MW + nx; if (d[j] <= d[i] + 1) continue;
                d[j] = d[i] + 1; if (d[j] < 14) q[t++] = j;
            }
        }
    }
    // ------------------------------------------------------------------ painting helpers
    paint(tx, ty, g, r = 0, mark = true) {
        for (let y = -r; y <= r; y++) for (let x = -r; x <= r; x++) {
            if (x * x + y * y > r * r + r) continue;
            const X = Math.round(tx + x), Y = Math.round(ty + y);
            if (!this.inb(X, Y)) continue;
            if (g === G.BRIDGE || g === G.WATER || this.g(X, Y) !== G.WATER) { this.set(X, Y, g); if (mark) this.path[this.idx(X, Y)] = 1; }
        }
    }
    road(pts, g, r = 1) {
        for (let i = 0; i < pts.length - 1; i++) {
            const [x0, y0] = pts[i], [x1, y1] = pts[i + 1];
            const steps = Math.ceil(Math.hypot(x1 - x0, y1 - y0) * 2);
            for (let s = 0; s <= steps; s++) { const t = s / steps; this.paint(x0 + (x1 - x0) * t + N.n2(s * 0.15, i + 3) * 0.9, y0 + (y1 - y0) * t + N.n2(i + 5, s * 0.15) * 0.6, g, r); }
        }
    }
    // ellipse of ground; also flattens the level
    plaza(cx, cy, rx, ry, g, lvl = null) {
        for (let y = -ry; y <= ry; y++) for (let x = -rx; x <= rx; x++) {
            if ((x / rx) ** 2 + (y / ry) ** 2 > 1.05) continue;
            const X = Math.round(cx + x), Y = Math.round(cy + y);
            if (!this.inb(X, Y)) continue;
            this.paint(X, Y, g, 0, true);
            if (lvl !== null) this.level[this.idx(X, Y)] = lvl;
        }
    }
    pond(cx, cy, rx, ry, rim = null, noise = 0.25) {
        for (let y = -ry - 2; y <= ry + 2; y++) for (let x = -rx - 2; x <= rx + 2; x++) {
            const X = Math.round(cx + x), Y = Math.round(cy + y); if (!this.inb(X, Y)) continue;
            const d = (x / rx) ** 2 + (y / ry) ** 2 + N.n2(X * 0.3, Y * 0.3) * noise;
            const i = this.idx(X, Y);
            if (d <= 1) { this.ground[i] = G.WATER; this.path[i] = 1; }
            else if (rim !== null && d <= 1.45 && this.ground[i] !== G.WATER) { this.ground[i] = rim; this.path[i] = 1; }
        }
    }
    // raised plateau of `lvl` (irregular outline); faces are computed later from the bottom rows
    plateau(x0, y0, w, h, lvl = 1, { rough = 0.8, keepPaths = true } = {}) {
        for (let y = y0; y < y0 + h; y++) for (let x = x0; x < x0 + w; x++) {
            if (!this.inb(x, y)) continue;
            const i = this.idx(x, y);
            if (this.ground[i] === G.WATER) continue;
            const ex = Math.min(x - x0, x0 + w - 1 - x), ey = Math.min(y - y0, y0 + h - 1 - y);
            if (Math.min(ex, ey) < 2 && N.n2(x * 0.45 + lvl, y * 0.45) * rough > Math.min(ex, ey) * 0.45 - 0.1) continue;
            if (keepPaths && this.path[i] && this.level[i] < lvl - 1) continue;
            this.level[i] = Math.max(this.level[i], lvl);
        }
    }
    computeFaces() {
        for (let x = 0; x < MW; x++) for (let y = MH - 1; y >= 0; y--) {
            const i = this.idx(x, y), L = this.level[i];
            if (!L) continue;
            // how many tiles of the same (or higher) level lie directly below?
            let k = 0; while (k < FH && this.inb(x, y + k + 1) && this.level[this.idx(x, y + k + 1)] >= L) k++;
            if (k < FH && (!this.inb(x, y + k + 1) || this.level[this.idx(x, y + k + 1)] < L)) {
                // inside the bottom FH rows of this column: a face of the step L-1 → L
                const lowBelow = this.inb(x, y + k + 1) ? this.level[this.idx(x, y + k + 1)] : 0;
                if (lowBelow < L) { this.cliff[i] = 1; this.face[i] = FH - 1 - k; }
            }
        }
        // roads that run across a face become stairs
        for (let i = 0; i < MW * MH; i++) if (this.cliff[i] && this.path[i] && this.ground[i] !== G.WATER) this.stairs[i] = 1;
        // stair columns: make sure a whole column of the face is stairs
        for (let x = 0; x < MW; x++) for (let y = 0; y < MH; y++) {
            const i = this.idx(x, y);
            if (!this.stairs[i]) continue;
            for (let k = -FH; k <= FH; k++) { const j = this.idx(x, Math.max(0, Math.min(MH - 1, y + k))); if (this.cliff[j] && this.level[j] === this.level[i]) this.stairs[j] = 1; }
        }
    }
    stairsAt(x, y) { // force a stair column at tile x below the plateau edge near y
        for (let k = 0; k < 8; k++) { const i = this.idx(x, y + k); if (this.cliff[i]) this.stairs[i] = 1; }
    }
    prop(kind, x, y, extra = {}) {
        const def = this.PROPS[kind]; if (!def) { console.warn('prop?', kind); return null; }
        const p = { kind, def, x, y, flip: !!extra.flip, ...extra };
        this.props.push(p);
        if ((def.r || def.box) && !extra.noCollide) this.addCollider(p);
        if (def.box && def.h > 40) { // keep big buildings free of trees and bushes (roof included)
            const x0 = Math.floor((x - def.w / 2) / TS), x1 = Math.floor((x + def.w / 2) / TS), y0 = Math.floor((y - def.h) / TS), y1 = Math.floor(y / TS);
            for (let ty = y0; ty <= y1; ty++) for (let tx = x0; tx <= x1; tx++) if (this.inb(tx, ty)) this.path[this.idx(tx, ty)] = 1;
        }
        if (def.light) this.lightsStatic.push({ x: x + (def.light.dx || 0) * (p.flip ? -1 : 1), y: y + (def.light.dy || 0), r: def.light.r, col: def.light.col, i: def.light.i, flicker: def.light.flicker, pulse: def.light.pulse, prop: p });
        return p;
    }
    propT(kind, tx, ty, extra) { const t = this.T(tx, ty); return this.prop(kind, t.x, t.y, extra); }
    addCollider(p) {
        const d = p.def;
        const c = d.box ? { kind: 'box', x0: p.x - d.box[0] * TS, x1: p.x + d.box[0] * TS, y0: p.y - d.box[1] * TS, y1: p.y + 3, p } : { kind: 'circle', x: p.x, y: p.y - 2, r: d.r * TS, p };
        p.collider = c;
        const bx0 = Math.floor((c.kind === 'box' ? c.x0 : c.x - c.r) / 64), bx1 = Math.floor((c.kind === 'box' ? c.x1 : c.x + c.r) / 64);
        const by0 = Math.floor((c.kind === 'box' ? c.y0 : c.y - c.r) / 64), by1 = Math.floor((c.kind === 'box' ? c.y1 : c.y + c.r) / 64);
        for (let by = by0; by <= by1; by++) for (let bx = bx0; bx <= bx1; bx++) { const k = by * 1000 + bx; if (!this.grid.has(k)) this.grid.set(k, []); this.grid.get(k).push(c); }
    }
    freeT(tx, ty, r = 1) {
        const L = this.lv(tx, ty);
        for (let y = -r; y <= r; y++) for (let x = -r; x <= r; x++) {
            const X = tx + x, Y = ty + y; if (!this.inb(X, Y)) return false;
            const i = this.idx(X, Y);
            if (this.ground[i] === G.WATER || this.cliff[i] || this.path[i] || this.ground[i] === G.BRIDGE || this.ground[i] === G.PLANK || this.level[i] !== L) return false;
        }
        // the tiles below must not be a face of a higher plateau (props would stand in front of it)
        if (this.cliff[this.idx(tx, Math.min(MH - 1, ty + 1))]) return false;
        return !this.blocked(tx * TS + 8, ty * TS + 10, r * 10);
    }
    fire(id, tx, ty) { const p = this.propT('saberfire', tx, ty, { fireId: id, interact: 'fire' }); this.paint(tx, ty + 1, G.FLOOR, 1); this.fires.push({ id, x: p.x, y: p.y, prop: p, zone: this.zoneAt(p.x, p.y), name: FIRE_NAMES[id] }); }
    spawn(type, tx, ty, extra = {}) { const t = this.T(tx, ty); this.spawns.push({ id: `${type}@${tx},${ty}`, type, x: t.x, y: t.y, ...extra }); }
    npc(id, tx, ty, extra = {}) { const t = this.T(tx, ty); this.npcs.push({ id, x: t.x, y: t.y, ...extra }); }
    walker(preset, pts) { this.walkers.push({ preset, pts: pts.map(([x, y]) => this.T(x, y)) }); }
    item(id, tx, ty) { const t = this.T(tx, ty); this.items.push({ id: `${id}@${tx},${ty}`, item: id, x: t.x, y: t.y }); }
    chest(id, tx, ty, loot, mimic = null) { const p = this.propT('chest', tx, ty, { chestId: id, loot, interact: 'chest', mimic }); this.chests.push(p); }
    dock(zone, tx, ty, dx, dy) {
        let x = tx, y = ty, n = 0;
        while (this.g(x, y) !== G.WATER && n < 40) { x += dx; y += dy; n++; }
        for (let k = -3; k < 6; k++) { const X = x + dx * k, Y = y + dy * k; this.paint(X, Y, G.BRIDGE, 0); if (dx) this.paint(X, Y + 1, G.BRIDGE, 0); else this.paint(X + 1, Y, G.BRIDGE, 0); this.level[this.idx(X, Y)] = 0; }
        for (let k = 0; k < 6; k += 2) { const X = x + dx * k, Y = y + dy * k; this.propT('bridgePost', X + (dx ? 0 : -1), Y + (dx ? -1 : 0)); this.propT('bridgePost', X + (dx ? 0 : 2), Y + (dx ? 2 : 0)); }
        const ex = x + dx * 6, ey = y + dy * 6;
        this.propT('lamppost', x + dx * 5 + (dx ? 0 : -1), y + dy * 5 + (dx ? -1 : 0));
        this.propT('rowboat', x + dx * 2 + (dx ? 0 : 3), y + dy * 2 + (dx ? 3 : 0), { noCollide: true });
        this.docks.push({ zone, x: (ex + dx * 2) * TS + 8, y: (ey + dy * 2) * TS + 8, px: (x + dx * 5) * TS + 8, py: (y + dy * 5) * TS + 8, heading: Math.atan2(dx, -dy) });
    }
    arena(key, cx, cy, r, g, gateDir = [0, 1]) {
        this.plaza(cx, cy, r, Math.round(r * 0.75), g, 0);
        const ring = new Set();
        for (let a = 0; a < Math.PI * 2; a += 0.04) {
            const x = Math.round(cx + Math.cos(a) * (r + 1)), y = Math.round(cy + Math.sin(a) * (r * 0.75 + 1));
            if (Math.cos(a) * gateDir[0] + Math.sin(a) * gateDir[1] > 0.985) continue;
            const k = y * MW + x; if (ring.has(k)) continue; ring.add(k);
            if (!this.inb(x, y)) continue;
            this.path[this.idx(x, y)] = 1;
            if (this.g(x, y) === G.WATER) continue;
            this.propT(['stoneColumn', 'rock1', 'rock2'][ring.size % 3], x, y, { flip: ring.size % 2 === 0 });
        }
        const gx = Math.round(cx + gateDir[0] * (r + 1.5)), gy = Math.round(cy + gateDir[1] * (r * 0.75 + 1.5));
        for (let k = -1; k <= 3; k++) this.paint(gx + gateDir[0] * k, gy + gateDir[1] * k, g, 1);
        const t = this.T(gx, gy);
        const gate = { key, x: t.x, y: t.y, dir: gateDir, open: true, prop: this.prop('foggate', t.x, t.y + 4, { noCollide: true, gateKey: key, interact: 'gate' }) };
        this.gates.push(gate);
        this.arenas[key] = { x: cx * TS + 8, y: cy * TS + 8, r: r * TS, ry: r * 0.75 * TS, gate };
        return gate;
    }
    fenceRow(x0, x1, y) { for (let x = x0; x <= x1; x += 2) this.propT('fenceH', x, y); }
    fenceCol(x, y0, y1) { for (let y = y0; y <= y1; y++) this.propT('fencePost', x, y); }
    grove(cx, cy, r, rng, n, table) {
        const tbl = table || FLORA[ZONES[this.cur].biome].grove;
        const sum = tbl.reduce((s, t) => s + t[1], 0);
        for (let i = 0; i < n * 4 && n > 0; i++) {
            const a = rng.next() * Math.PI * 2, d = Math.sqrt(rng.next()) * r;
            const tx = Math.round(cx + Math.cos(a) * d), ty = Math.round(cy + Math.sin(a) * d * 0.8);
            if (!this.freeT(tx, ty, 1)) continue;
            let k = rng.next() * sum, kind = tbl[0][0];
            for (const [p, w] of tbl) { k -= w; if (k <= 0) { kind = p; break; } }
            const t = this.T(tx, ty);
            this.prop(kind, t.x + rng.int(-4, 4), t.y, { flip: rng.chance(0.5) });
            n--;
        }
    }
    waterfall(tx, ty, w = 3) { // a fall down the face below plateau row ty, with a pool
        for (let x = tx; x < tx + w; x++) for (let y = ty - 2; y < ty + 4; y++) { const i = this.idx(x, y); this.ground[i] = G.WATER; this.path[i] = 1; }
        this.waterfalls.push({ tx, ty, w });
    }

    // ------------------------------------------------------------------ layouts
    layoutCoast(z, rng) {
        const cx = z.cx, cy = z.cy;
        // ridges and terraces
        this.plateau(cx - 40, cy - 22, 16, 12, 1); this.plateau(cx - 36, cy - 22, 8, 6, 2);
        this.plateau(cx + 18, cy - 24, 22, 12, 1); this.plateau(cx + 26, cy - 24, 10, 6, 2);
        this.plateau(cx - 6, cy + 8, 12, 7, 1);
        this.plaza(cx - 10, cy + 4, 5, 3, G.SAND, 0);
        this.fire('kueste', cx - 10, cy + 3);
        this.road([[cx - 10, cy + 4], [cx - 2, cy], [cx + 12, cy - 4], [cx + 26, cy - 2], [cx + 34, cy + 4]], G.DIRT, 1);
        this.road([[cx - 2, cy], [cx, cy - 12], [cx, cy - 22]], G.DIRT, 1);
        this.road([[cx + 26, cy - 2], [cx + 28, cy - 14], [cx + 30, cy - 18]], G.DIRT, 1); // up the terrace
        this.road([[cx - 10, cy + 4], [cx - 26, cy + 2], [cx - 32, cy - 6], [cx - 32, cy - 14]], G.DIRT, 1);
        this.dock('kueste', cx, cy - 22, 0, -1);
        this.waterfall(cx - 34, cy - 12, 3); this.pond(cx - 32, cy - 7, 4, 2, G.SAND);
        this.propT('wreck', cx - 26, cy + 14, { flip: true });
        for (let i = 0; i < 6; i++) this.propT('grave', cx + 8 + i * 2, cy + 8 + (i % 2));
        this.propT('skullpile', cx + 18, cy + 5); this.propT('torch', cx - 7, cy + 2); this.propT('torch', cx - 13, cy + 2);
        this.propT('anchorDeco', cx - 18, cy + 9); this.propT('nets', cx - 6, cy + 14); this.propT('barrels', cx + 2, cy + 12); this.propT('crates', cx - 22, cy + 6);
        this.propT('tent', cx - 18, cy - 2); this.propT('logPile', cx - 14, cy - 3);
        this.propT('signpost', cx - 4, cy + 2);
        for (const [x, y] of [[-30, 6], [24, 8], [38, -6], [-38, -2], [10, -14], [-14, -14]]) this.grove(cx + x, cy + y, 5, rng, 5);
        this.spawn('skeleton', cx + 8, cy - 2); this.spawn('crab', cx - 18, cy + 10); this.spawn('crab', cx + 22, cy + 8);
        this.spawn('skeleton', cx + 18, cy - 4); this.spawn('skelGunner', cx + 24, cy - 8); this.spawn('crab', cx - 2, cy - 9);
        this.spawn('skelBrute', cx + 32, cy + 2); this.spawn('livingChest', cx - 36, cy - 18); this.spawn('skelkraken', cx + 30, cy - 18, { elite: true });
        this.chest('k1', cx + 34, cy - 20, { item: 'rum' }); this.chest('k2', cx - 36, cy - 16, { item: 'bombs' });
        this.item('pearls', cx - 22, cy + 6); this.item('bullets', cx + 12, cy + 4); this.item('pearls', cx + 28, cy - 20);
        this.npc('herald', cx - 6, cy + 6, { preset: 'carver', name: 'Alter Galionsschnitzer', dialog: 'herald', sit: true });
    }
    layoutHub(z, rng) {
        const cx = z.cx, cy = z.cy;
        // clover basin: the soul tree stands in water, a marble rim, a platform with the bonfire at the front
        const bx = cx, by = cy - 6;
        for (const [ox, oy, rx, ry] of [[0, -3, 7, 5], [-6, 1, 5, 4], [6, 1, 5, 4], [0, 4, 4, 3]]) this.pond(bx + ox, by + oy, rx, ry, null, 0);
        for (let y = by - 12; y <= by + 10; y++) for (let x = bx - 15; x <= bx + 15; x++) {
            const i = this.idx(x, y); if (this.ground[i] === G.WATER) continue;
            let near = false; for (let k = -1; k <= 1 && !near; k++) for (let j = -1; j <= 1; j++) if (this.g(x + j, y + k) === G.WATER) { near = true; break; }
            if (near) { this.ground[i] = G.FLOOR; this.path[i] = 1; }
        }
        this.plaza(bx, by + 10, 4, 2, G.FLOOR, 0); this.plaza(bx, by + 13, 2, 2, G.FLOOR, 0);
        this.prop('hubTree', bx * TS + 8, (by - 1) * TS + 8);
        this.prop('hubCrown', bx * TS + 8, (by - 3) * TS, { noCollide: true });
        this.fire('hain', bx, by + 10);
        this.propT('marbleArch', bx, by + 6, { noCollide: true });
        for (const [x, y] of [[-9, -1], [9, -1], [-6, 6], [6, 6], [-12, 3], [12, 3]]) this.propT('marblePillar', bx + x, by + y);
        for (const [x, y] of [[-13, 7], [13, 7], [-15, -4], [15, -4]]) this.propT('marbleStone', bx + x, by + y);
        for (const [x, y] of [[-4, 12], [4, 12]]) this.propT('rimPost', bx + x, by + y);
        // guardian statues with lanterns on the far side
        this.propT('guardian', bx - 10, by - 9); this.propT('guardian', bx + 10, by - 9);
        // bamboo fences behind the basin
        for (let x = bx - 14; x <= bx + 14; x += 2) if (Math.abs(x - bx) > 8) this.propT('fencePost', x, by - 12 + Math.round(Math.abs(x - bx) / 6));
        // roads: platform south to the dock, west to the forge, east to the tavern
        this.road([[bx, by + 14], [bx, cy + 22], [bx, cy + 30]], G.DIRT, 1);
        this.road([[bx - 4, by + 11], [bx - 18, by + 10], [cx - 30, cy - 12], [cx - 34, cy - 18]], G.DIRT, 1);
        this.road([[bx + 4, by + 11], [bx + 18, by + 10], [cx + 30, cy - 12], [cx + 34, cy - 18]], G.DIRT, 1);
        this.road([[bx - 18, by + 10], [cx - 26, cy + 14], [cx - 34, cy + 22]], G.DIRT, 1);
        this.road([[bx + 18, by + 10], [cx + 26, cy + 14], [cx + 34, cy + 22]], G.DIRT, 1);
        this.road([[cx - 30, cy - 12], [cx - 40, cy - 2]], G.DIRT, 1);
        this.road([[cx + 30, cy - 12], [cx + 40, cy - 2]], G.DIRT, 1);
        // forge top-left
        this.plaza(cx - 32, cy - 20, 7, 4, G.STONE, 0);
        this.propT('forge', cx - 33, cy - 22); this.propT('crates', cx - 26, cy - 22); this.propT('barrels', cx - 38, cy - 18); this.propT('logPile', cx - 27, cy - 17); this.propT('cannon', cx - 38, cy - 23); this.propT('anchorDeco', cx - 29, cy - 25);
        this.fenceRow(cx - 40, cx - 26, cy - 15);
        // tavern top-right with tables, benches, a stall and laundry
        this.plaza(cx + 32, cy - 20, 9, 5, G.DIRT, 0);
        this.propT('tavern', cx + 34, cy - 24);
        this.propT('flag', cx + 42, cy - 24, { flag: true });
        for (const [x, y] of [[26, -18], [36, -16]]) { this.propT('table', cx + x, cy + y); this.propT('bench', cx + x, cy + y + 2); }
        this.propT('stall', cx + 24, cy - 24); this.propT('barrels', cx + 42, cy - 19); this.propT('laundry', cx + 30, cy - 12); this.propT('well', cx + 44, cy - 14);
        // the ship graveyard: four wrecks without figureheads on the beaches
        const wrecks = [['fire', cx + 30, cy + 24, false], ['ice', cx - 30, cy + 24, true], ['wind', cx - 42, cy + 2, true], ['earth', cx + 42, cy + 2, false]];
        for (const [el, x, y, flip] of wrecks) {
            this.propT('wreck', x, y, { flip });
            const sx = flip ? x + 6 : x - 6;
            const p = this.propT('figureheadSlot', sx, y + 1, { slot: el, interact: 'slot' });
            this.slots.push({ el, prop: p, x: p.x, y: p.y });
        }
        // terraces in the back with a waterfall
        this.plateau(cx - 24, cy - 34, 16, 9, 1); this.plateau(cx + 10, cy - 34, 16, 9, 1);
        this.waterfall(cx - 17, cy - 26, 3); this.pond(cx - 16, cy - 21, 4, 2, G.FLOOR, 0.1);
        this.road([[cx + 18, cy - 20], [cx + 18, cy - 28]], G.DIRT, 1);
        this.chest('h1', cx + 20, cy - 31, { item: 'shard' });
        this.dock('hain', bx, cy + 26, 0, 1);
        // trees and bushes all around
        for (const [x, y, r, n] of [[-38, -6, 6, 7], [38, -6, 6, 7], [-24, 18, 6, 6], [24, 18, 6, 6], [-16, -24, 5, 4], [16, -24, 5, 4], [-44, 14, 4, 4], [44, 14, 4, 4], [0, 28, 6, 3], [-6, -30, 6, 3]]) this.grove(cx + x, cy + y, r, rng, n);
        this.npc('maid', bx - 3, by + 11, { preset: 'lanternMaid', name: 'Die Laternenmagd', dialog: 'maid' });
        this.npc('smith', cx - 30, cy - 19, { preset: 'smith', name: 'Schmied Harrow', dialog: 'smith' });
        this.npc('innkeep', cx + 30, cy - 20, { preset: 'innkeep', name: 'Wirt Bilge', dialog: 'innkeep' });
        this.crewSpots = [[cx + 25, cy - 16], [cx + 28, cy - 16], [cx + 35, cy - 14], [cx + 38, cy - 14]].map(([x, y]) => this.T(x, y));
        this.walker('villagerA', [[bx - 18, by + 10], [cx - 30, cy - 12], [cx - 26, cy + 14]]);
        this.walker('villagerB', [[bx + 18, by + 10], [cx + 30, cy - 12], [cx + 26, cy + 14]]);
        this.walker('villagerC', [[bx, cy + 22], [bx + 18, by + 10], [bx - 18, by + 10]]);
        this.walker('villagerD', [[cx + 26, cy - 18], [cx + 38, cy - 17], [cx + 32, cy - 12]]);
    }
    layoutFire(z, rng) {
        const cx = z.cx, cy = z.cy;
        // blood tides: hex basalt mesas, a gorge with a broken bridge, lantern poles
        this.plateau(cx - 34, cy - 26, 20, 12, 1); this.plateau(cx - 30, cy - 26, 10, 6, 2);
        this.plateau(cx + 4, cy + 10, 20, 12, 1); this.plateau(cx - 10, cy - 8, 12, 9, 1);
        this.dock('asche', cx - 24, cy + 8, -1, 0);
        this.fire('asche', cx - 28, cy + 6);
        this.road([[cx - 28, cy + 7], [cx - 14, cy + 4], [cx - 4, cy - 2], [cx + 6, cy - 4], [cx + 12, cy - 8]], G.ASH, 1);
        this.road([[cx - 14, cy + 4], [cx - 24, cy - 6], [cx - 24, cy - 18]], G.ASH, 1);
        // gorge with a broken plank bridge
        for (let y = cy - 30; y < cy + 30; y++) for (let k = -1; k <= 1; k++) { const x = cx - 18 + Math.round(Math.sin(y * 0.2) * 2) + k; if (this.zone[this.idx(x, y)] === 'asche' && this.g(x, y) !== G.WATER && !(y >= cy + 3 && y <= cy + 5)) { this.set(x, y, G.WATER); } }
        for (let x = cx - 21; x <= cx - 15; x++) for (let y = cy + 3; y <= cy + 5; y++) if (this.g(x, y) === G.WATER || Math.abs(x - (cx - 18)) <= 2) this.paint(x, y, G.BRIDGE, 0);
        this.arena('asche', cx + 22, cy - 8, 10, G.HEX, [-1, 0]);
        this.fire('asche2', cx + 6, cy - 6);
        for (const [x, y] of [[-8, 12], [-2, 6], [8, 2], [14, 8], [-20, -10], [0, -16], [-28, 14]]) this.propT('lanternPole', cx + x, cy + y);
        for (const [x, y, r, n] of [[-30, -10, 6, 6], [12, 18, 7, 6], [-6, 20, 5, 4], [-34, 16, 5, 4], [2, -24, 6, 5]]) this.grove(cx + x, cy + y, r, rng, n);
        this.propT('glowFlower', cx - 6, cy - 2); this.propT('stoneColumn', cx - 2, cy + 2); this.propT('stoneColumn', cx + 2, cy + 3);
        for (const [t, x, y] of [['ashSkeleton', -10, 2], ['ashSkeleton', -4, -4], ['crabWarrior', 2, 6], ['cannonAngler', 6, -10], ['ashSkeleton', -8, 8], ['skelBrute', 8, 4], ['shell', -14, -6], ['cannonAngler', -26, -14], ['ashSkeleton', 14, 14], ['mimicChest', -30, -22]]) this.spawn(t, cx + x, cy + y);
        this.spawn('turtleWorm', cx + 14, cy + 16, { elite: true });
        this.chest('a1', cx - 6, cy + 14, { item: 'phial' }); this.chest('a2', cx - 28, cy - 22, { item: 'shard' });
        this.item('soulflower', cx + 8, cy + 8); this.item('pearls', cx - 12, cy - 4); this.item('fireball', cx - 4, cy - 12);
        this.npc('crewFire', cx - 24, cy + 2, { preset: 'crewFire', name: 'Kanonierin Asra', dialog: 'crewFire', sit: true, crew: 'fire' });
    }
    layoutIce(z, rng) {
        const cx = z.cx, cy = z.cy;
        // tidal grotto: basalt columns 3–4 tiles high, a waterfall into stepped pools
        this.plateau(cx - 30, cy - 28, 24, 14, 1); this.plateau(cx - 26, cy - 28, 12, 8, 2);
        this.plateau(cx - 4, cy - 30, 18, 10, 1); this.plateau(cx + 14, cy + 4, 14, 10, 1); this.plateau(cx - 38, cy + 2, 12, 10, 1);
        this.dock('frost', cx, cy + 22, 0, 1);
        this.fire('frost', cx - 2, cy + 18);
        this.road([[cx - 2, cy + 19], [cx - 10, cy + 8], [cx - 6, cy - 2], [cx + 8, cy - 6]], G.ICE, 1);
        this.road([[cx - 10, cy + 8], [cx - 18, cy - 8], [cx - 18, cy - 18]], G.ICE, 1);
        this.waterfall(cx - 14, cy - 15, 3);
        this.pond(cx - 13, cy - 9, 5, 3, G.ICE, 0.2); this.pond(cx - 11, cy - 3, 3, 2, G.ICE, 0.2);
        this.propT('sunkenfigurehead', cx - 15, cy - 9, { noCollide: true });
        this.arena('frost', cx + 22, cy - 10, 10, G.ICE, [-1, 0]);
        this.fire('frost2', cx + 6, cy - 9);
        for (const [x, y, r, n] of [[-30, 10, 6, 6], [10, 14, 6, 5], [-24, -20, 6, 5], [24, 16, 5, 4], [-36, -4, 5, 4]]) this.grove(cx + x, cy + y, r, rng, n);
        for (const [x, y] of [[-6, 6], [4, 2], [-20, 2], [12, -2], [-26, -10]]) this.propT('crystalBlue', cx + x, cy + y);
        for (const [t, x, y] of [['drownedIce', -8, 6], ['drownedIce', -14, 2], ['skeleton', 2, -2], ['drownedIce', 8, 2], ['skelGunner', -24, -4], ['cannonEater', -12, -6], ['crab', 0, 10], ['stormHarpy', -22, -20]]) this.spawn(t, cx + x, cy + y);
        this.spawn('wyvern', cx - 20, cy - 22, { elite: true });
        this.chest('f1', cx - 26, cy - 24, { item: 'ring' });
        this.item('pearls', cx + 4, cy + 6); this.item('bolt', cx - 20, cy + 10);
        this.npc('crewIce', cx + 6, cy + 16, { preset: 'crewIce', name: 'Navigatorin Lene', dialog: 'crewIce', sit: true, crew: 'ice' });
    }
    layoutWind(z, rng) {
        const cx = z.cx, cy = z.cy;
        // harbour town: cobble streets, half-timbered houses with scale roofs, a market with a well
        this.dock('raben', cx + 24, cy, 1, 0);
        this.road([[cx + 24, cy], [cx - 26, cy]], G.STONE, 2);
        this.road([[cx - 2, cy + 18], [cx - 2, cy - 22]], G.STONE, 2);
        this.road([[cx - 22, cy + 12], [cx + 18, cy + 12]], G.STONE, 1);
        this.road([[cx - 22, cy - 10], [cx + 18, cy - 10]], G.STONE, 1);
        this.plaza(cx + 10, cy + 3, 5, 3, G.STONE, 0);
        this.fire('raben', cx + 18, cy + 3);
        // rows of town houses facing the streets (front = south side)
        const rows = [[-15, [-24, -18, -11, 6, 12, 18]], [-3, [-24, -17, -10, 6, 13, 19]], [9, [-24, -17, -10, 6, 12, 18]], [21, [-24, -17, -10, 6, 12]]];
        let hi = 0;
        for (const [y, xs] of rows) for (const x of xs) { this.propT(`townhouse${hi % 3}`, cx + x, cy + y, { flip: hi % 2 === 0 }); hi++; }
        for (const x of [-24, -16, -8, 4, 10, 16]) { this.propT('lamppost', cx + x, cy - 2); this.propT('lamppost', cx + x + 2, cy + 3); }
        this.propT('well', cx + 10, cy + 2); this.propT('stall', cx + 4, cy + 2); this.propT('stall', cx + 14, cy - 1);
        this.propT('barrels', cx + 20, cy - 3); this.propT('crates', cx + 20, cy + 5); this.propT('nets', cx + 22, cy + 8); this.propT('cannon', cx + 22, cy - 6);
        this.propT('laundry', cx - 16, cy + 13); this.propT('laundry', cx + 10, cy - 9);
        for (const [x, y] of [[-24, 4], [0, 14], [-8, -7], [12, 14]]) this.propT('flowerPot', cx + x, cy + y);
        this.arena('raben', cx - 2, cy - 28, 9, G.STONE, [0, 1]);
        this.fire('raben2', cx + 2, cy - 19);
        this.plateau(cx + 20, cy - 26, 14, 10, 1); this.plateau(cx - 38, cy + 14, 12, 10, 1);
        for (const [x, y, r, n] of [[-32, -14, 6, 6], [26, 18, 6, 5], [-34, 4, 5, 4], [24, -14, 5, 4]]) this.grove(cx + x, cy + y, r, rng, n);
        for (const [t, x, y] of [['stormSailor', -6, 0], ['stormSailor', 8, 0], ['skelGunner', -18, 1], ['skeleton', -2, 10], ['stormSailor', -2, -12], ['skelBrute', -24, -1], ['stormHarpy', 12, -12], ['stormHarpy', -14, 12]]) this.spawn(t, cx + x, cy + y);
        this.spawn('orcCaptain', cx - 14, cy - 10, { elite: true }); this.spawn('bellConstruct', cx + 26, cy - 20, { elite: true });
        this.chest('r1', cx - 30, cy, { item: 'coat' }); this.chest('r2', cx + 28, cy - 22, { item: 'shard' });
        this.item('pearls', cx + 8, cy - 2); this.item('bombs', cx - 10, cy + 3);
        this.npc('crewWind', cx + 16, cy + 6, { preset: 'crewWind', name: 'Bootsmann Kalo', dialog: 'crewWind', sit: true, crew: 'wind' });
        this.walker('villagerA', [[cx - 20, cy], [cx + 16, cy], [cx + 16, cy + 12]]);
        this.walker('villagerB', [[cx - 2, cy + 16], [cx - 2, cy - 8], [cx + 14, cy - 10]]);
        this.walker('villagerC', [[cx + 4, cy + 4], [cx + 14, cy + 4], [cx + 10, cy - 1]]);
        this.walker('villagerD', [[cx - 20, cy + 12], [cx + 14, cy + 12]]);
    }
    layoutEarth(z, rng) {
        const cx = z.cx, cy = z.cy;
        this.dock('mangroven', cx - 24, cy - 4, -1, 0);
        this.fire('mangroven', cx - 28, cy - 2);
        // channels of black water crossed by plank bridges
        for (const [x0, y0, x1, y1] of [[cx - 8, cy - 28, cx - 6, cy + 28], [cx + 8, cy - 28, cx + 10, cy + 28]]) for (let t = 0; t <= 1; t += 0.01) { const X = Math.round(x0 + (x1 - x0) * t + Math.sin(t * 9) * 3), Y = Math.round(y0 + (y1 - y0) * t); for (let k = -2; k <= 1; k++) if (this.zone[this.idx(X + k, Y)] === 'mangroven') this.set(X + k, Y, G.WATER); }
        this.road([[cx - 28, cy - 2], [cx - 14, cy - 1], [cx - 2, cy - 1], [cx + 6, cy - 2], [cx + 16, cy - 3]], G.MUD, 1);
        for (let x = cx - 14; x <= cx + 16; x++) for (const y of [cy - 2, cy - 1, cy]) if (this.g(x, y) === G.WATER) this.paint(x, y, G.BRIDGE, 0);
        this.arena('mangroven', cx + 24, cy + 6, 9, G.MUD, [-1, 0]);
        this.fire('mangroven2', cx + 2, cy + 10);
        this.road([[cx + 6, cy - 2], [cx + 2, cy + 10]], G.MUD, 1);
        this.plateau(cx - 32, cy + 6, 16, 12, 1); this.plateau(cx - 4, cy - 26, 12, 10, 1); this.plateau(cx + 18, cy - 26, 16, 12, 1);
        for (const [x, y, r, n] of [[-20, 12, 7, 8], [-20, -14, 7, 8], [0, 18, 6, 6], [16, -14, 7, 7], [-2, -16, 5, 4], [30, 18, 5, 4]]) this.grove(cx + x, cy + y, r, rng, n);
        for (const [t, x, y] of [['abyssPirate', -16, 2], ['crabWarrior', -18, -10], ['shell', 0, 6], ['abyssPirate', 2, -10], ['shell', 16, -12], ['crabWarrior', 16, 4], ['abyssPirate', -2, 16], ['voodoo', 14, 12], ['cannonEater', -12, 14]]) this.spawn(t, cx + x, cy + y);
        this.spawn('trueMimic', cx + 24, cy - 20, { elite: true });
        this.chest('m1', cx - 2, cy - 22, { item: 'phial' }); this.chest('m2', cx - 28, cy + 12, { item: 'shard' });
        this.item('pearls', cx + 14, cy + 14); this.item('soulflower', cx - 16, cy + 10);
        this.npc('crewEarth', cx - 20, cy + 4, { preset: 'crewEarth', name: 'Zimmermann Ode', dialog: 'crewEarth', sit: true, crew: 'earth' });
    }
    layoutEnd(z, rng) {
        const cx = z.cx, cy = z.cy;
        this.plaza(cx, cy, 17, 11, G.HEX, 0);
        this.fire('ende', cx - 15, cy + 8);
        this.arenas.ende = { x: cx * TS + 8, y: cy * TS + 8, r: 15 * TS, ry: 10 * TS, gate: null };
        this.endArrive = this.T(cx - 13, cy + 8);
        for (let a = 0; a < Math.PI * 2; a += Math.PI / 8) this.propT(a % 2 < 1 ? 'lanternPole' : 'stoneColumn', Math.round(cx + Math.cos(a) * 18), Math.round(cy + Math.sin(a) * 12));
        void rng;
    }

    // ------------------------------------------------------------------ vegetation + deco
    scatter(key, z, rng) {
        const F = FLORA[z.biome];
        for (let ty = z.cy - z.ry - 8; ty <= z.cy + z.ry + 8; ty++) for (let tx = z.cx - z.rx - 8; tx <= z.cx + z.rx + 8; tx++) {
            if (!this.inb(tx, ty) || this.zone[this.idx(tx, ty)] !== key) continue;
            const g = this.g(tx, ty), i = this.idx(tx, ty);
            if (g === G.WATER) {
                if (this.shoreDist[i] <= 1 && rng.chance(z.biome === 'earth' ? 0.2 : 0.05)) this.deco.push({ k: rng.pick(['lily', 'reeds']), x: tx * TS + rng.int(0, 10), y: ty * TS + rng.int(2, 12) });
                continue;
            }
            if (this.cliff[i]) continue;
            // deco: dense, clustered by noise
            const dens = 0.18 + Math.max(0, N.fbm(tx * 0.15 + z.seed, ty * 0.15, 2, 0)) * 0.5 + (g === G.MEADOW ? 0.25 : 0);
            const nd = rng.next() < dens ? (rng.next() < 0.3 ? 2 : 1) : 0;
            for (let k = 0; k < nd; k++) {
                let dk = rng.pick(F.deco);
                if (this.path[i] && g !== G.MEADOW) { if (rng.next() < 0.75) continue; dk = rng.pick(['pebbles', 'tuft']); }
                if (g === G.MEADOW && rng.chance(0.5)) dk = rng.pick(F.deco.filter(d => d.startsWith('flowers')).concat(['flowersW']));
                this.deco.push({ k: dk, x: tx * TS + rng.int(-2, 10), y: ty * TS + rng.int(0, 12) });
            }
            if (this.path[i] || g === G.BRIDGE || g === G.PLANK) continue;
            const t = this.T(tx, ty);
            if (this.fires.some(f => Math.hypot(f.x - t.x, f.y - t.y) < 48) || this.npcs.some(n => Math.hypot(n.x - t.x, n.y - t.y) < 40) || this.spawns.some(s => Math.hypot(s.x - t.x, s.y - t.y) < 20)) continue;
            // trees in clusters (noise), bushes in between
            const forest = N.fbm(tx * 0.08 + z.seed * 3, ty * 0.08, 3, 0);
            const dense = z.biome === 'earth' ? 2 : 1;
            if (forest > 0.12 && rng.chance(0.1 * dense) && this.freeT(tx, ty, 1)) { this.grove(tx, ty, 0.1, rng, 1, F.grove); continue; }
            if (rng.chance((0.04 + Math.max(0, forest) * 0.08) * dense) && this.freeT(tx, ty, 0)) this.prop(rng.pick(F.bush) + (rng.chance(0.4) && !['fern', 'banana', 'bigLeaf', 'reedClump', 'cattail', 'lanternPole', 'crystalBlue'].includes('x') ? '' : ''), t.x + rng.int(-4, 4), t.y, { flip: rng.chance(0.5) });
            else if (rng.chance(0.012) && this.freeT(tx, ty, 0)) this.prop(rng.pick(['rock0', 'rock1']), t.x, t.y, { flip: rng.chance(0.5) });
        }
    }

    // ------------------------------------------------------------------ collision with height levels
    // the levels a mover standing at (x,y) may walk on (bitmask)
    levelMask(x, y) {
        const tx = Math.floor(x / TS), ty = Math.floor(y / TS);
        if (!this.inb(tx, ty)) return 1;
        const i = this.idx(tx, ty), L = this.level[i];
        if (this.stairs[i]) return (1 << L) | (1 << (L - 1));
        return 1 << L;
    }
    tileBlocks(tx, ty, sailing, mask = 1) {
        if (!this.inb(tx, ty)) return true;
        const i = this.idx(tx, ty), g = this.ground[i];
        if (sailing) return g !== G.WATER || this.shoreDist[i] < 2;
        if (g === G.WATER) return true;
        const L = this.level[i];
        if (this.stairs[i]) return !(mask & ((1 << L) | (1 << (L - 1))));
        if (this.cliff[i]) return true;
        return !(mask & (1 << L));
    }
    blocked(x, y, r, sailing = false, mask = null) {
        if (mask === null) mask = sailing ? 1 : this.levelMask(x, y);
        const x0 = Math.floor((x - r) / TS), x1 = Math.floor((x + r) / TS), y0 = Math.floor((y - r) / TS), y1 = Math.floor((y + r) / TS);
        for (let ty = y0; ty <= y1; ty++) for (let tx = x0; tx <= x1; tx++) {
            if (!this.tileBlocks(tx, ty, sailing, mask)) continue;
            const nx = Math.max(tx * TS, Math.min(x, tx * TS + TS)), ny = Math.max(ty * TS, Math.min(y, ty * TS + TS));
            if ((nx - x) ** 2 + (ny - y) ** 2 < r * r) return true;
        }
        if (sailing) return this.zoneAt(x, y) === 'ende';
        const bx0 = Math.floor((x - r) / 64), bx1 = Math.floor((x + r) / 64), by0 = Math.floor((y - r) / 64), by1 = Math.floor((y + r) / 64);
        for (let by = by0; by <= by1; by++) for (let bx = bx0; bx <= bx1; bx++) {
            const arr = this.grid.get(by * 1000 + bx); if (!arr) continue;
            for (const c of arr) {
                if (c.p.open || c.p.hidden) continue;
                if (c.kind === 'circle') { if ((c.x - x) ** 2 + (c.y - y) ** 2 < (c.r + r) ** 2) return true; }
                else { const nx = Math.max(c.x0, Math.min(x, c.x1)), ny = Math.max(c.y0, Math.min(y, c.y1)); if ((nx - x) ** 2 + (ny - y) ** 2 < r * r) return true; }
            }
        }
        for (const e of this.dynamicBlockers || []) if (e.active && x > e.x0 && x < e.x1 && y > e.y0 && y < e.y1) return true;
        return false;
    }
    move(x, y, dx, dy, r, sailing = false) {
        const mask = sailing ? 1 : this.levelMask(x, y);
        const B = (X, Y) => this.blocked(X, Y, r, sailing, mask);
        if (!B(x + dx, y + dy)) return [x + dx, y + dy];
        let nx = x, ny = y;
        if (dx && !B(x + dx, y)) nx = x + dx;
        if (dy && !B(nx, y + dy)) ny = y + dy;
        if (nx === x && ny === y) for (const s of [0.7, -0.7]) { const ax = dx * Math.cos(s) - dy * Math.sin(s), ay = dx * Math.sin(s) + dy * Math.cos(s); if (!B(x + ax * 0.7, y + ay * 0.7)) return [x + ax * 0.7, y + ay * 0.7]; }
        return [nx, ny];
    }
    sameLevel(a, b) { return (this.levelMask(a.x, a.y) & this.levelMask(b.x, b.y)) !== 0; }
    lineClear(x0, y0, x1, y1) {
        if (!(this.levelMask(x0, y0) & this.levelMask(x1, y1))) return false;
        const n = Math.ceil(Math.hypot(x1 - x0, y1 - y0) / 8);
        for (let i = 1; i < n; i++) { const t = i / n; const tx = Math.floor((x0 + (x1 - x0) * t) / TS), ty = Math.floor((y0 + (y1 - y0) * t) / TS); if (this.inb(tx, ty) && this.cliff[this.idx(tx, ty)] && !this.stairs[this.idx(tx, ty)]) return false; }
        return true;
    }
    walkable(x, y) { const tx = Math.floor(x / TS), ty = Math.floor(y / TS); return WALKABLE.has(this.g(tx, ty)) && !this.cliff[this.idx(tx, ty)]; }
}
