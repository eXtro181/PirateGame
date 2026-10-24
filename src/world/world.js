// The world: one tile map (16 px tiles) with seven islands — the start coast, the hub
// Schiffshain, the four king regions and the End of the World (reachable only through the
// whirlpool). Generation is deterministic. Units: entity positions are world pixels.
import { G, TS, WALKABLE } from '../art/tiles.js';
import { Noise } from '../gfx/noise.js';

export const MW = 240, MH = 180;
const N = new Noise(4242);

export class Rng {
    constructor(seed = 1) { this.s = seed >>> 0 || 1; }
    next() { let t = (this.s += 0x6D2B79F5); t = Math.imul(t ^ (t >>> 15), t | 1); t ^= t + Math.imul(t ^ (t >>> 7), t | 61); return ((t ^ (t >>> 14)) >>> 0) / 4294967296; }
    range(a, b) { return a + (b - a) * this.next(); }
    int(a, b) { return Math.floor(this.range(a, b + 1)); }
    pick(arr) { return arr[Math.floor(this.next() * arr.length)]; }
    chance(p) { return this.next() < p; }
}

// zones: name, biome, grade, music, island centre and radii (tiles)
export const ZONES = {
    kueste: { name: 'Strand der Ertränkten', biome: 'coast', grade: 'coast', music: 'salz', cx: 120, cy: 152, rx: 27, ry: 15, seed: 1 },
    hain: { name: 'Schiffshain', biome: 'hub', grade: 'hub', music: 'hain', cx: 120, cy: 92, rx: 25, ry: 17, seed: 2 },
    asche: { name: 'Aschebucht', biome: 'fire', grade: 'fire', music: 'glut', cx: 196, cy: 74, rx: 27, ry: 19, seed: 3, king: 'fire' },
    frost: { name: 'Frostgrotte', biome: 'ice', grade: 'ice', music: 'frost', cx: 44, cy: 46, rx: 28, ry: 19, seed: 4, king: 'ice' },
    raben: { name: 'Hafen Rabenwacht', biome: 'wind', grade: 'wind', music: 'sturm', cx: 42, cy: 128, rx: 28, ry: 19, seed: 5, king: 'wind' },
    mangroven: { name: 'Fluchmangroven', biome: 'earth', grade: 'earth', music: 'tiefe', cx: 196, cy: 146, rx: 27, ry: 19, seed: 6, king: 'earth' },
    ende: { name: 'Ende der Welt', biome: 'end', grade: 'end', music: 'ende', cx: 212, cy: 20, rx: 17, ry: 12, seed: 7, sealed: true }
};
export const BIOMES = ['coast', 'hub', 'fire', 'ice', 'wind', 'earth', 'end'];

export const FIRE_NAMES = {
    kueste: 'Strand der Ertränkten', hain: 'Schiffshain · Seelenbaum', asche: 'Aschebucht · Landung', asche2: 'Vor dem Glutthron',
    frost: 'Frostgrotte · Eingang', frost2: 'Am gefrorenen Fall', raben: 'Rabenwacht · Mole', raben2: 'Rabenwacht · Markt',
    mangroven: 'Fluchmangroven · Steg', mangroven2: 'Wurzelkathedrale', ende: 'Ende der Welt'
};

export class World {
    constructor(props) {
        this.PROPS = props; // prop sprite metadata (art/props.js)
        const n = MW * MH;
        this.ground = new Uint8Array(n);      // G.*
        this.biome = new Uint8Array(n);       // index into BIOMES
        this.zone = new Array(n).fill(null);
        this.cliff = new Uint8Array(n);       // 1 = raised basalt (blocks)
        this.face = new Int8Array(n).fill(-1); // cliff face row 0..2 (drawn below the top)
        this.path = new Uint8Array(n);        // reserved walkways (no props)
        this.shoreDist = new Float32Array(n);
        this.props = []; this.deco = []; this.fires = []; this.spawns = []; this.npcs = []; this.items = []; this.chests = [];
        this.docks = []; this.gates = []; this.arenas = {}; this.slots = []; this.lightsStatic = []; this.crew = []; this.ghostRoutes = [];
        this.grid = new Map(); // collider buckets (4×4 tiles)
        this.generate();
    }
    idx(tx, ty) { return ty * MW + tx; }
    inb(tx, ty) { return tx >= 0 && ty >= 0 && tx < MW && ty < MH; }
    g(tx, ty) { return this.inb(tx, ty) ? this.ground[ty * MW + tx] : G.WATER; }
    set(tx, ty, g) { if (this.inb(tx, ty)) this.ground[ty * MW + tx] = g; }
    zoneAt(x, y) { // world px → zone key (nearest island)
        const tx = x / TS, ty = y / TS;
        let best = null, bd = 1e9;
        for (const [k, z] of Object.entries(ZONES)) { const d = Math.hypot((tx - z.cx) / z.rx, (ty - z.cy) / z.ry); if (d < bd) { bd = d; best = k; } }
        return best;
    }
    isLand(tx, ty) { const g = this.g(tx, ty); return g !== G.WATER; }

    // ------------------------------------------------------------------ generation
    generate() {
        for (const [key, z] of Object.entries(ZONES)) this.island(key, z);
        this.computeShore();
        const L = { kueste: this.layoutCoast, hain: this.layoutHub, asche: this.layoutFire, frost: this.layoutIce, raben: this.layoutWind, mangroven: this.layoutEarth, ende: this.layoutEnd };
        for (const [key, z] of Object.entries(ZONES)) { const rng = new Rng(z.seed * 977); L[key].call(this, z, rng, key); }
        this.computeFaces();
        for (const [key, z] of Object.entries(ZONES)) this.scatter(key, z, new Rng(z.seed * 131));
        this.computeShore();
        // whirlpool spot (north of the hub) and ghost ship routes
        this.whirlpool = { x: (120) * TS, y: (66) * TS };
        this.ghostRoutes = [[[160, 112], [170, 120], [150, 128], [140, 116]], [[80, 72], [90, 64], [96, 80]], [[150, 40], [170, 46], [160, 56]], [[80, 160], [96, 168], [70, 168]]];
    }
    island(key, z) {
        const bi = BIOMES.indexOf(z.biome);
        for (let ty = Math.floor(z.cy - z.ry - 6); ty <= z.cy + z.ry + 6; ty++) for (let tx = Math.floor(z.cx - z.rx - 6); tx <= z.cx + z.rx + 6; tx++) {
            if (!this.inb(tx, ty)) continue;
            const dx = (tx - z.cx) / z.rx, dy = (ty - z.cy) / z.ry;
            const f = 1 - (dx * dx + dy * dy) + N.fbm(tx * 0.09 + z.seed * 10, ty * 0.09, 3, 0) * 0.45;
            const i = this.idx(tx, ty);
            if (f > 0) { this.ground[i] = z.biome === 'coast' ? G.GRASS : G.GRASS; this.biome[i] = bi; this.zone[i] = key; }
            else if (!this.zone[i]) { this.biome[i] = bi; this.zone[i] = key; }
        }
        // beach rim
        for (let ty = Math.floor(z.cy - z.ry - 6); ty <= z.cy + z.ry + 6; ty++) for (let tx = Math.floor(z.cx - z.rx - 6); tx <= z.cx + z.rx + 6; tx++) {
            if (!this.inb(tx, ty) || this.g(tx, ty) === G.WATER || this.zone[this.idx(tx, ty)] !== key) continue;
            let near = false;
            for (let k = -2; k <= 2 && !near; k++) for (let j = -2; j <= 2; j++) if (this.g(tx + j, ty + k) === G.WATER && Math.abs(j) + Math.abs(k) <= 3) { near = true; break; }
            if (near) this.set(tx, ty, z.biome === 'ice' ? G.SNOW : z.biome === 'fire' ? G.ASH : z.biome === 'earth' ? G.MUD : z.biome === 'end' ? G.HEX : G.SAND);
        }
    }
    computeShore() { // distance from land for water tiles (BFS, in tiles, capped)
        const n = MW * MH, q = new Int32Array(n); let h = 0, t = 0;
        const d = this.shoreDist; d.fill(99);
        for (let i = 0; i < n; i++) if (this.ground[i] !== G.WATER) { d[i] = 0; q[t++] = i; }
        while (h < t) {
            const i = q[h++], x = i % MW, y = (i / MW) | 0;
            for (const [dx, dy] of [[1, 0], [-1, 0], [0, 1], [0, -1]]) {
                const nx = x + dx, ny = y + dy; if (!this.inb(nx, ny)) continue;
                const j = ny * MW + nx; if (d[j] <= d[i] + 1) continue;
                d[j] = d[i] + 1; if (d[j] < 12) q[t++] = j;
            }
        }
    }
    // ---------------------------------------------------------------- helpers for layouts
    T(tx, ty) { return { x: tx * TS + 8, y: ty * TS + 12 }; }
    paint(tx, ty, g, r = 0, pathMark = true) {
        for (let y = -r; y <= r; y++) for (let x = -r; x <= r; x++) {
            if (x * x + y * y > r * r + r) continue;
            const X = Math.round(tx + x), Y = Math.round(ty + y);
            if (!this.inb(X, Y)) continue;
            if (g === G.BRIDGE || this.g(X, Y) !== G.WATER) { this.set(X, Y, g); if (pathMark) this.path[this.idx(X, Y)] = 1; this.cliff[this.idx(X, Y)] = 0; }
        }
    }
    road(pts, g, r = 1) {
        for (let i = 0; i < pts.length - 1; i++) {
            const [x0, y0] = pts[i], [x1, y1] = pts[i + 1];
            const steps = Math.ceil(Math.hypot(x1 - x0, y1 - y0) * 2);
            for (let s = 0; s <= steps; s++) { const t = s / steps; this.paint(x0 + (x1 - x0) * t + N.n2(s * 0.2, i) * 0.8, y0 + (y1 - y0) * t, g, r); }
        }
    }
    plaza(cx, cy, rx, ry, g) { for (let y = -ry; y <= ry; y++) for (let x = -rx; x <= rx; x++) if ((x / rx) ** 2 + (y / ry) ** 2 <= 1.05) this.paint(cx + x, cy + y, g, 0); }
    // raised basalt block; the bottom `fh` rows become the visible face
    plateau(x0, y0, w, h) {
        for (let y = y0; y < y0 + h; y++) for (let x = x0; x < x0 + w; x++) {
            if (!this.inb(x, y) || this.g(x, y) === G.WATER || this.path[this.idx(x, y)]) continue;
            const edge = (x === x0 || x === x0 + w - 1) && (y === y0) ;
            if (edge && N.n2(x * 0.7, y * 0.7) > 0.2) continue;
            this.cliff[this.idx(x, y)] = 1;
        }
    }
    computeFaces() {
        for (let y = 0; y < MH; y++) for (let x = 0; x < MW; x++) {
            const i = this.idx(x, y);
            if (!this.cliff[i]) continue;
            // rows from the bottom of a cliff column: 0 = bottom-most
            let k = 0; while (k < 3 && this.inb(x, y + k + 1) && this.cliff[this.idx(x, y + k + 1)]) k++;
            const fh = 2;
            if (k < fh) this.face[i] = fh - 1 - k; // 0 top face row, fh-1 bottom
        }
    }
    prop(kind, x, y, extra = {}) { // x,y world px (foot)
        const def = this.PROPS[kind]; if (!def) { console.warn('prop?', kind); return null; }
        const p = { kind, def, x, y, flip: !!extra.flip, ...extra };
        this.props.push(p);
        if ((def.r || def.box) && !extra.noCollide) this.addCollider(p);
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
    removeCollider(p) { if (!p.collider) return; for (const arr of this.grid.values()) { const i = arr.indexOf(p.collider); if (i >= 0) arr.splice(i, 1); } p.collider = null; }
    freeT(tx, ty, r = 1) { // no path, no cliff, land, nothing else close
        for (let y = -r; y <= r; y++) for (let x = -r; x <= r; x++) { const X = tx + x, Y = ty + y; if (!this.inb(X, Y)) return false; const i = this.idx(X, Y); if (this.ground[i] === G.WATER || this.cliff[i] || this.path[i] || this.ground[i] === G.BRIDGE || this.ground[i] === G.PLANK) return false; }
        return !this.blocked(tx * TS + 8, ty * TS + 10, r * 10);
    }
    fire(id, tx, ty) { const p = this.propT('saberfire', tx, ty, { fireId: id, interact: 'fire' }); this.paint(tx, ty + 1, G.FLOOR, 1); this.fires.push({ id, x: p.x, y: p.y, prop: p, zone: this.zoneAt(p.x, p.y), name: FIRE_NAMES[id] }); }
    spawn(type, tx, ty, extra = {}) { const t = this.T(tx, ty); this.spawns.push({ id: `${type}@${tx},${ty}`, type, x: t.x, y: t.y, ...extra }); }
    npc(id, tx, ty, extra = {}) { const t = this.T(tx, ty); this.npcs.push({ id, x: t.x, y: t.y, ...extra }); }
    item(id, tx, ty) { const t = this.T(tx, ty); this.items.push({ id: `${id}@${tx},${ty}`, item: id, x: t.x, y: t.y }); }
    chest(id, tx, ty, loot) { const p = this.propT('chest', tx, ty, { chestId: id, loot, interact: 'chest' }); this.chests.push(p); }
    // pier from a shore point out into the water in direction (dx,dy); ship rises at the end
    dock(zone, tx, ty, dx, dy) {
        let x = tx, y = ty, n = 0;
        while (this.g(x, y) !== G.WATER && n < 30) { x += dx; y += dy; n++; }
        for (let k = -2; k < 6; k++) { const X = x + dx * k, Y = y + dy * k; this.paint(X, Y, G.BRIDGE, 0); if (dx) { this.paint(X, Y + 1, G.BRIDGE, 0); } else this.paint(X + 1, Y, G.BRIDGE, 0); }
        const ex = x + dx * 6, ey = y + dy * 6;
        this.propT('lamppost', x + dx * 5 + (dx ? 0 : -1), y + dy * 5 + (dx ? -1 : 0));
        this.docks.push({ zone, x: (ex + dx * 2) * TS + 8, y: (ey + dy * 2) * TS + 8, px: (x + dx * 5) * TS + 8, py: (y + dy * 5) * TS + 8, heading: Math.atan2(dx, -dy) });
    }
    arena(key, cx, cy, r, g, gateDir = [0, 1]) {
        this.plaza(cx, cy, r, Math.round(r * 0.75), g);
        // ring of basalt boulders with a gap at the gate
        const ring = [];
        for (let a = 0; a < Math.PI * 2; a += 0.05) {
            const x = Math.round(cx + Math.cos(a) * (r + 1)), y = Math.round(cy + Math.sin(a) * (r * 0.75 + 1));
            if (Math.cos(a) * gateDir[0] + Math.sin(a) * gateDir[1] > 0.985) continue;
            const k = y * MW + x;
            if (ring.includes(k)) continue;
            ring.push(k);
            if (this.inb(x, y)) { this.path[this.idx(x, y)] = 1; if (this.g(x, y) === G.WATER) continue; }
            this.propT(['rock1', 'rock2', 'rock0'][ring.length % 3], x, y, { flip: ring.length % 2 === 0, arenaRock: true });
        }
        const gx = Math.round(cx + gateDir[0] * (r + 1.5)), gy = Math.round(cy + gateDir[1] * (r * 0.75 + 1.5));
        for (let k = -1; k <= 3; k++) this.paint(gx + gateDir[0] * k, gy + gateDir[1] * k, g, 1);
                const t = this.T(gx, gy);
        const gate = { key, x: t.x, y: t.y, dir: gateDir, open: true, prop: this.prop('foggate', t.x, t.y + 4, { noCollide: true, gateKey: key, interact: 'gate' }) };
        this.gates.push(gate);
        this.arenas[key] = { x: cx * TS + 8, y: cy * TS + 8, r: r * TS, ry: r * 0.75 * TS, gate };
        return gate;
    }

    // ---------------------------------------------------------------- layouts
    layoutCoast(z, rng) {
        const cx = z.cx, cy = z.cy;
        this.plaza(cx - 6, cy + 4, 4, 3, G.SAND);
        this.fire('kueste', cx - 6, cy + 3);
        this.road([[cx - 6, cy + 4], [cx + 2, cy], [cx + 12, cy - 4], [cx + 18, cy - 2]], G.DIRT, 1);
        this.road([[cx + 2, cy], [cx, cy - 12]], G.DIRT, 1);
        this.dock('kueste', cx, cy - 12, 0, -1);
        // the wreck you washed up from and the beached graves
        this.propT('wreck', cx - 18, cy + 9, { flip: true });
        for (let i = 0; i < 5; i++) this.propT('grave', cx + 6 + i * 2, cy + 7 + (i % 2));
        this.propT('skullpile', cx + 14, cy + 3); this.propT('torch', cx - 3, cy + 2); this.propT('torch', cx - 9, cy + 2);
        this.plateau(cx + 8, cy - 11, 8, 4); this.plateau(cx - 20, cy - 6, 7, 4);
        this.spawn('skeleton', cx + 6, cy - 2); this.spawn('crab', cx - 14, cy + 6); this.spawn('crab', cx + 18, cy + 5);
        this.spawn('skeleton', cx + 15, cy - 3); this.spawn('skelGunner', cx + 20, cy - 6); this.spawn('crab', cx - 2, cy - 7);
        this.spawn('skelBrute', cx + 21, cy + 1);
        this.chest('k1', cx + 22, cy - 3, { item: 'rum' });
        this.item('pearls', cx - 18, cy + 4); this.item('bullets', cx + 10, cy + 3);
        this.npc('herald', cx - 3, cy + 6, { preset: 'carver', name: 'Alter Galionsschnitzer', dialog: 'herald', sit: true });
    }
    layoutHub(z, rng) {
        const cx = z.cx, cy = z.cy;
        // clover basin: soul tree on top, flagstone platform at the bottom, wet paths in between
        this.plaza(cx, cy - 5, 5, 3, G.FLOOR);
        this.plaza(cx, cy + 9, 6, 2, G.FLOOR);
        this.plaza(cx, cy + 2, 3, 2, G.FLOOR);
        this.road([[cx, cy + 8], [cx, cy - 2]], G.DIRT, 1);
        this.road([[cx - 9, cy + 2], [cx + 9, cy + 2]], G.DIRT, 1);
        this.road([[cx, cy + 10], [cx, cy + 17]], G.DIRT, 1);
        this.propT('soultree', cx, cy - 5);
        this.soulCrown = this.T(cx, cy - 12);
        this.fire('hain', cx, cy + 2);
        // basin pools around the tree
        for (const [x, y] of [[cx - 6, cy - 4], [cx + 6, cy - 4]]) this.propT('basin', x, y);
        // basalt arches with soul stones, guardians with lanterns, oar fence
        this.propT('arch', cx, cy + 12);
        this.propT('arch', cx - 14, cy + 2); this.propT('arch', cx + 14, cy + 2);
        this.propT('guardian', cx - 4, cy - 10); this.propT('guardian', cx + 4, cy - 10); this.propT('guardian', cx, cy - 13);
        for (let i = -3; i <= 3; i++) if (Math.abs(i) > 0) this.propT('oarfence', cx + i * 2 + (i > 0 ? 2 : -2), cy + 11);
        // forge top-left, tavern top-right
        this.plaza(cx - 15, cy - 8, 5, 3, G.STONE); this.propT('forge', cx - 15, cy - 9);
        this.plaza(cx + 15, cy - 8, 6, 3, G.STONE); this.propT('tavern', cx + 16, cy - 10);
        this.propT('flag', cx + 22, cy - 10, { flag: true });
        this.propT('barreltable', cx + 12, cy - 6); this.propT('barrel', cx + 20, cy - 7); this.propT('maptable', cx + 9, cy - 7);
        this.propT('cannon', cx - 19, cy - 6); this.propT('crate', cx - 11, cy - 7);
        this.propT('gallows', cx + 19, cy + 7);
        // the ship graveyard: four wrecks without figureheads along the beaches
        const wrecks = [['fire', cx + 17, cy + 13, false], ['ice', cx - 17, cy + 13, true], ['wind', cx - 21, cy - 2, true], ['earth', cx + 21, cy - 1, false]];
        for (const [el, x, y, flip] of wrecks) {
            this.propT('wreck', x, y, { flip });
            const sx = flip ? x + 6 : x - 6;
            const p = this.propT('figureheadSlot', sx, y + 1, { slot: el, interact: 'slot' });
            this.slots.push({ el, prop: p, x: p.x, y: p.y });
            this.road([[cx, cy + 2], [sx, y + 2]], G.DIRT, 0);
        }
        this.dock('hain', cx, cy + 14, 0, 1);
        this.npc('maid', cx - 3, cy + 3, { preset: 'lanternMaid', name: 'Die Laternenmagd', dialog: 'maid' });
        this.npc('smith', cx - 13, cy - 6, { preset: 'smith', name: 'Schmied Harrow', dialog: 'smith' });
        this.npc('innkeep', cx + 14, cy - 6, { preset: 'innkeep', name: 'Wirt Bilge', dialog: 'innkeep' });
        this.crewSpots = [[cx + 11, cy - 4], [cx + 13, cy - 3], [cx + 17, cy - 4], [cx + 19, cy - 3]].map(([x, y]) => this.T(x, y));
    }
    layoutFire(z, rng) {
        const cx = z.cx, cy = z.cy;
        this.plaza(cx, cy, 14, 9, G.HEX);
        this.dock('asche', cx - 14, cy + 6, -1, 0);
        this.fire('asche', cx - 16, cy + 4);
        this.road([[cx - 16, cy + 5], [cx - 6, cy + 2], [cx + 2, cy - 2], [cx + 8, cy - 6]], G.ASH, 1);
        this.plateau(cx - 10, cy - 12, 8, 5); this.plateau(cx + 4, cy + 8, 9, 4);
        this.arena('asche', cx + 14, cy - 6, 9, G.HEX, [-1, 0]);
        this.fire('asche2', cx + 1, cy - 5);
        for (const [t, x, y] of [['ashSkeleton', -8, 1], ['ashSkeleton', -3, -2], ['crabWarrior', 0, 4], ['skelGunner', 4, -8], ['ashSkeleton', -6, 6], ['skelBrute', 6, 2], ['shell', -12, -4]]) this.spawn(t, cx + x, cy + y);
        this.chest('a1', cx - 4, cy + 9, { item: 'phial' });
        this.item('soulflower', cx + 6, cy + 6); this.item('pearls', cx - 10, cy - 4); this.item('fireball', cx - 2, cy - 9);
        this.npc('crewFire', cx - 12, cy - 2, { preset: 'crewFire', name: 'Kanonierin Asra', dialog: 'crewFire', sit: true, crew: 'fire' });
    }
    layoutIce(z, rng) {
        const cx = z.cx, cy = z.cy;
        this.dock('frost', cx, cy + 12, 0, 1);
        this.fire('frost', cx - 2, cy + 10);
        this.road([[cx - 2, cy + 11], [cx - 8, cy + 3], [cx - 4, cy - 4], [cx + 6, cy - 6]], G.ICE, 1);
        // grotto: basalt columns with a waterfall into stepped pools
        this.plateau(cx - 14, cy - 14, 10, 6); this.plateau(cx - 2, cy - 18, 12, 5); this.plateau(cx + 10, cy + 2, 8, 5); this.plateau(cx - 22, cy + 2, 6, 5);
        for (const [x, y, r] of [[cx - 9, cy - 6, 2], [cx - 12, cy - 4, 1]]) for (let j = -r; j <= r; j++) for (let i = -r - 1; i <= r + 1; i++) { const X = x + i, Y = y + j; this.set(X, Y, G.WATER); this.cliff[this.idx(X, Y)] = 0; this.path[this.idx(X, Y)] = 1; }
        this.propT('waterfall', cx - 9, cy - 8, { noCollide: true });
        this.propT('sunkenfigurehead', cx - 12, cy - 4);
        this.arena('frost', cx + 14, cy - 8, 9, G.ICE, [-1, 0]);
        this.fire('frost2', cx + 1, cy - 9);
        for (const [t, x, y] of [['drownedIce', -6, 4], ['drownedIce', -10, 0], ['skeleton', 2, -2], ['drownedIce', 6, 0], ['skelGunner', -16, -2], ['crab', 0, 6]]) this.spawn(t, cx + x, cy + y);
        this.chest('f1', cx - 18, cy - 4, { item: 'ring' });
        this.item('pearls', cx + 2, cy + 4); this.item('bolt', cx - 14, cy + 6);
        this.npc('crewIce', cx + 4, cy + 8, { preset: 'crewIce', name: 'Navigatorin Lene', dialog: 'crewIce', sit: true, crew: 'ice' });
    }
    layoutWind(z, rng) {
        const cx = z.cx, cy = z.cy;
        this.dock('raben', cx + 14, cy, 1, 0);
        this.road([[cx + 14, cy], [cx - 14, cy]], G.STONE, 1);
        this.road([[cx - 2, cy + 10], [cx - 2, cy - 12]], G.STONE, 1);
        this.fire('raben', cx + 10, cy + 2);
        // half-timbered houses with slate scale roofs along the streets
        for (const [x, y] of [[-12, -4], [-6, -4], [4, -4], [10, -4], [-12, 7], [-6, 7], [4, 7], [10, 7]]) this.propT(rng.chance(0.5) ? 'house0' : 'house1', cx + x, cy + y, { flip: rng.chance(0.5) });
        for (const x of [-15, -9, -3, 3, 9]) { this.propT('lamppost', cx + x, cy - 2); this.propT('lamppost', cx + x + 2, cy + 2); }
        this.arena('raben', cx - 6, cy - 16, 8, G.STONE, [0, 1]);
        this.fire('raben2', cx + 2, cy - 9);
        this.propT('barrel', cx + 6, cy + 2); this.propT('crate', cx + 7, cy + 3); this.propT('crate', cx - 10, cy + 2); this.propT('cannon', cx + 12, cy - 3);
        for (const [t, x, y] of [['stormSailor', -4, 0], ['stormSailor', 6, 0], ['skelGunner', -12, 1], ['skeleton', -2, 6], ['stormSailor', -2, -8], ['skelBrute', -14, -1]]) this.spawn(t, cx + x, cy + y);
        this.chest('r1', cx - 18, cy, { item: 'coat' });
        this.item('pearls', cx + 6, cy - 2); this.item('bombs', cx - 8, cy + 3);
        this.npc('crewWind', cx + 8, cy + 3, { preset: 'crewWind', name: 'Bootsmann Kalo', dialog: 'crewWind', sit: true, crew: 'wind' });
    }
    layoutEarth(z, rng) {
        const cx = z.cx, cy = z.cy;
        this.dock('mangroven', cx - 14, cy - 4, -1, 0);
        this.fire('mangroven', cx - 16, cy - 2);
        // channels of black water crossed by plank bridges
        for (const [x0, y0, x1, y1] of [[cx - 6, cy - 16, cx - 4, cy + 16], [cx + 6, cy - 16, cx + 8, cy + 16]]) for (let t = 0; t <= 1; t += 0.02) { const X = Math.round(x0 + (x1 - x0) * t + Math.sin(t * 9) * 2), Y = Math.round(y0 + (y1 - y0) * t); for (let k = -1; k <= 1; k++) { this.set(X + k, Y, G.WATER); this.cliff[this.idx(X + k, Y)] = 0; } }
        this.road([[cx - 16, cy - 2], [cx - 8, cy - 1], [cx - 2, cy - 1], [cx + 4, cy - 2], [cx + 10, cy - 3]], G.MUD, 1);
        for (const x of [cx - 7, cx - 6, cx - 5, cx - 4, cx - 3, cx + 5, cx + 6, cx + 7, cx + 8, cx + 9]) for (const y of [cy - 2, cy - 1]) { if (this.g(x, y) === G.WATER) this.paint(x, y, G.BRIDGE, 0); }
        this.arena('mangroven', cx + 16, cy + 4, 8, G.MUD, [-1, 0]);
        this.fire('mangroven2', cx + 2, cy + 6);
        this.road([[cx + 4, cy - 2], [cx + 2, cy + 6]], G.MUD, 1);
        for (const [t, x, y] of [['drowned', -10, 2], ['crabWarrior', -12, -6], ['shell', 0, 4], ['drowned', 2, -6], ['shell', 12, -8], ['crabWarrior', 12, 2], ['drowned', -1, 9]]) this.spawn(t, cx + x, cy + y);
        this.chest('m1', cx - 1, cy - 10, { item: 'phial' });
        this.item('pearls', cx + 11, cy + 8); this.item('soulflower', cx - 11, cy + 6);
        this.npc('crewEarth', cx - 12, cy + 4, { preset: 'crewEarth', name: 'Zimmermann Ode', dialog: 'crewEarth', sit: true, crew: 'earth' });
    }
    layoutEnd(z, rng) {
        const cx = z.cx, cy = z.cy;
        this.plaza(cx, cy, 15, 10, G.HEX);
        this.fire('ende', cx - 13, cy + 7);
        this.arenas.ende = { x: cx * TS + 8, y: cy * TS + 8, r: 13 * TS, ry: 9 * TS, gate: null };
        this.endArrive = this.T(cx - 11, cy + 7);
        for (let a = 0; a < Math.PI * 2; a += Math.PI / 7) this.propT(a % 2 < 1 ? 'lanternshroom' : 'coral', Math.round(cx + Math.cos(a) * 15), Math.round(cy + Math.sin(a) * 10.5));
    }

    // ---------------------------------------------------------------- vegetation + deco
    scatter(key, z, rng) {
        const B = z.biome;
        const table = {
            coast: [['kelppalm', 0.012], ['bush0', 0.02], ['bush1', 0.012], ['rock0', 0.01], ['rock1', 0.006], ['deadtree', 0.004], ['willow', 0.004]],
            hub: [['willow', 0.01], ['bush2', 0.012], ['rock0', 0.006], ['kelppalm', 0.006]],
            fire: [['emberpalm', 0.016], ['lanternshroom', 0.02], ['rock1', 0.01], ['rock2', 0.004], ['deadtree', 0.006]],
            ice: [['icecrystal', 0.02], ['rock0', 0.01], ['rock2', 0.006], ['deadtree', 0.008]],
            wind: [['bush1', 0.012], ['rock0', 0.008], ['deadtree', 0.008], ['willow', 0.004]],
            earth: [['mangrove', 0.03], ['bush2', 0.02], ['coral', 0.008], ['willow', 0.006]],
            end: [['coral', 0.01], ['lanternshroom', 0.01]]
        }[B];
        const deco = {
            coast: ['shell', 'pebbles', 'bones', 'boneshroom', 'plankbits', 'rope'], hub: ['boneshroom', 'boneshroom2', 'puddle', 'soulspark', 'pebbles', 'skull'],
            fire: ['ember', 'crack', 'bones', 'skull', 'pebbles'], ice: ['snowtuft', 'pebbles', 'bones', 'snowtuft'], wind: ['puddle', 'pebbles', 'rope', 'plankbits', 'puddle'],
            earth: ['lily', 'reeds', 'puddle', 'coralbud', 'boneshroom'], end: ['ember', 'crack', 'skull', 'coralbud']
        }[B];
        for (let ty = z.cy - z.ry - 6; ty <= z.cy + z.ry + 6; ty++) for (let tx = z.cx - z.rx - 6; tx <= z.cx + z.rx + 6; tx++) {
            if (!this.inb(tx, ty) || this.zone[this.idx(tx, ty)] !== key) continue;
            const g = this.g(tx, ty), i = this.idx(tx, ty);
            if (g === G.WATER) { if (B === 'earth' && this.shoreDist[i] <= 2 && rng.chance(0.08)) this.deco.push({ k: rng.pick(['lily', 'reeds']), x: tx * TS + rng.int(0, 10), y: ty * TS + rng.int(2, 12) }); continue; }
            if (this.cliff[i]) {
                if (this.face[i] < 0 && rng.chance(0.06)) this.deco.push({ k: rng.pick(deco), x: tx * TS + rng.int(0, 10), y: ty * TS + rng.int(0, 10), onCliff: true });
                continue;
            }
            if (rng.chance(0.07)) this.deco.push({ k: rng.pick(deco), x: tx * TS + rng.int(0, 10), y: ty * TS + rng.int(0, 12) });
            if (this.path[i] || g === G.BRIDGE || g === G.PLANK) continue;
            // keep a clear radius around fires, docks, NPCs and spawns
            const t = this.T(tx, ty);
            if (this.fires.some(f => Math.hypot(f.x - t.x, f.y - t.y) < 48) || this.npcs.some(n => Math.hypot(n.x - t.x, n.y - t.y) < 40)) continue;
            for (const [kind, p] of table) if (rng.chance(p) && this.freeT(tx, ty, 1)) { this.prop(kind, t.x + rng.int(-3, 3), t.y, { flip: rng.chance(0.5) }); break; }
        }
    }

    // ---------------------------------------------------------------- collision
    tileBlocks(tx, ty, sailing) {
        if (!this.inb(tx, ty)) return true;
        const i = this.idx(tx, ty), g = this.ground[i];
        if (sailing) return g !== G.WATER || this.shoreDist[i] < 2;
        return g === G.WATER || !!this.cliff[i];
    }
    blocked(x, y, r, sailing = false) {
        const x0 = Math.floor((x - r) / TS), x1 = Math.floor((x + r) / TS), y0 = Math.floor((y - r) / TS), y1 = Math.floor((y + r) / TS);
        for (let ty = y0; ty <= y1; ty++) for (let tx = x0; tx <= x1; tx++) {
            if (!this.tileBlocks(tx, ty, sailing)) continue;
            const nx = Math.max(tx * TS, Math.min(x, tx * TS + TS)), ny = Math.max(ty * TS, Math.min(y, ty * TS + TS));
            if ((nx - x) ** 2 + (ny - y) ** 2 < r * r) return true;
        }
        if (sailing) {
            if (this.zoneAt(x, y) === 'ende') return true; // the edge of the world cannot be sailed
            return false;
        }
        const bx0 = Math.floor((x - r) / 64), bx1 = Math.floor((x + r) / 64), by0 = Math.floor((y - r) / 64), by1 = Math.floor((y + r) / 64);
        for (let by = by0; by <= by1; by++) for (let bx = bx0; bx <= bx1; bx++) {
            const arr = this.grid.get(by * 1000 + bx); if (!arr) continue;
            for (const c of arr) {
                if (c.p.open) continue;
                if (c.kind === 'circle') { if ((c.x - x) ** 2 + (c.y - y) ** 2 < (c.r + r) ** 2) return true; }
                else { const nx = Math.max(c.x0, Math.min(x, c.x1)), ny = Math.max(c.y0, Math.min(y, c.y1)); if ((nx - x) ** 2 + (ny - y) ** 2 < r * r) return true; }
            }
        }
        for (const extra of this.dynamicBlockers || []) { if (extra.active && x > extra.x0 && x < extra.x1 && y > extra.y0 && y < extra.y1) return true; }
        return false;
    }
    // move with sliding; returns the new position
    move(x, y, dx, dy, r, sailing = false) {
        if (!this.blocked(x + dx, y + dy, r, sailing)) return [x + dx, y + dy];
        let nx = x, ny = y;
        if (dx && !this.blocked(x + dx, y, r, sailing)) nx = x + dx;
        if (dy && !this.blocked(nx, y + dy, r, sailing)) ny = y + dy;
        if (nx === x && ny === y) { // nudge around corners
            for (const s of [0.7, -0.7]) { const ax = dx * Math.cos(s) - dy * Math.sin(s), ay = dx * Math.sin(s) + dy * Math.cos(s); if (!this.blocked(x + ax * 0.7, y + ay * 0.7, r, sailing)) return [x + ax * 0.7, y + ay * 0.7]; }
        }
        return [nx, ny];
    }
    lineClear(x0, y0, x1, y1) { const n = Math.ceil(Math.hypot(x1 - x0, y1 - y0) / 8); for (let i = 1; i < n; i++) { const t = i / n; const tx = Math.floor((x0 + (x1 - x0) * t) / TS), ty = Math.floor((y0 + (y1 - y0) * t) / TS); if (this.inb(tx, ty) && this.cliff[this.idx(tx, ty)]) return false; } return true; }
    walkable(x, y) { const tx = Math.floor(x / TS), ty = Math.floor(y / TS); return WALKABLE.has(this.g(tx, ty)) && !this.cliff[this.idx(tx, ty)]; }
}
