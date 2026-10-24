// Shared world definitions: tile types, prop collision info and a seeded random generator.
export const T = { WATER: 0, SAND: 1, GRASS: 2, DARK: 3, DIRT: 4, STONE: 5, ROCK: 6, DOCK: 7 };
export const SOLID_WALK = new Set([T.WATER, T.ROCK]);
export const VARIANTS = 4;

// r = walk collider radius, box = [w, d] box collider, variants = model variants
export const PROP_INFO = {
    palm: { r: 0.25, variants: 3 },
    bush: { r: 0.28, variants: 3 },
    boulder: { r: 0.5, variants: 2 },
    boulderBig: { r: 0.8, variants: 2 },
    pillar: { r: 0.34, variants: 2 },
    crate: { r: 0.38, variants: 1 },
    barrel: { r: 0.32, variants: 1 },
    skulls: { r: 0, variants: 1 },
    grave: { r: 0.2, variants: 1 },
    torch: { r: 0.15, variants: 1 },
    wall: { r: 0, variants: 2, box: [2.1, 0.5] },
    wallV: { r: 0, variants: 2, box: [0.5, 2.1] },
    palisade: { r: 0, variants: 1, box: [2.2, 0.35] },
    tuft: { r: 0, variants: 3 },
    flowers: { r: 0, variants: 3 },
    post: { r: 0.2, variants: 1 },
    chest: { r: 0.38, variants: 1 },
    saberFire: { r: 0.45, variants: 1 }
};

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
