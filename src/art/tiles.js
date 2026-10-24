// Terrain tile atlas (16x16 tiles, 16 per row) with coast masks.
import { Pix, Rng, hex, shade, mix } from './painter.js';

export const T = { WATER: 0, SAND: 1, GRASS: 2, DARK: 3, DIRT: 4, STONE: 5, ROCK: 6, DOCK: 7 };
export const SOLID_WALK = new Set([T.WATER, T.ROCK]);
export const VARIANTS = 4;

const TS = 16;

function base(rng, c, h, speck = [], speckP = 0.15, hj = 0.1) {
    const p = new Pix(TS, TS);
    for (let y = 0; y < TS; y++) {
        for (let x = 0; x < TS; x++) {
            let col = shade(c, rng.range(0.94, 1.06));
            let hh = h + rng.range(-hj, hj);
            if (speck.length && rng.chance(speckP)) { col = rng.pick(speck); hh += 0.15; }
            p.px(x, y, col, hh);
        }
    }
    return p;
}

function sand(rng) {
    const p = base(rng, hex('#8f7a52'), 0.3, [hex('#a08a5e'), hex('#7d6946'), hex('#ab9568')], 0.12, 0.04);
    for (let i = 0; i < 3; i++) { // ripples
        const y = rng.int(1, 14), x = rng.int(0, 10);
        for (let k = 0; k < 5; k++) p.px(x + k, y + (k > 2 ? 1 : 0), hex('#9c865b'), 0.42);
    }
    if (rng.chance(0.25)) { const x = rng.int(3, 12), y = rng.int(3, 12); p.px(x, y, hex('#e8dcc8'), 0.6); p.px(x + 1, y, hex('#cfc2ad'), 0.55); } // shell
    return p;
}

function grass(rng, dark) {
    const c = dark ? hex('#2b4630') : hex('#3c6438');
    const p = base(rng, c, 0.3, [shade(c, 0.8)], 0.2, 0.05);
    const blade = dark ? hex('#3b5c3c') : hex('#58844a');
    const tip = dark ? hex('#4d7048') : hex('#7aa35c');
    for (let i = 0; i < 10; i++) {
        const x = rng.int(0, 15), y = rng.int(1, 15);
        p.px(x, y, blade, 0.6); p.px(x, y - 1, tip, 0.75);
    }
    if (!dark && rng.chance(0.2)) { const x = rng.int(2, 13), y = rng.int(2, 13); p.px(x, y, hex('#e8d26a'), 0.7); }
    return p;
}

function dirt(rng) {
    const p = base(rng, hex('#6a4c34'), 0.25, [hex('#7d5d40'), hex('#563c28')], 0.2, 0.05);
    for (let i = 0; i < 3; i++) { const x = rng.int(1, 14), y = rng.int(1, 14); p.px(x, y, hex('#8b8178'), 0.6); p.px(x + 1, y, hex('#6e665e'), 0.5); }
    return p;
}

function stone(rng) {
    const p = new Pix(TS, TS);
    const mortar = hex('#2c2a2e');
    p.rect(0, 0, TS, TS, mortar, 0.0);
    const rows = [[0, 7], [8, 15]];
    rows.forEach(([y0, y1], ri) => {
        let x = ri ? -4 : 0;
        while (x < TS) {
            const w = rng.int(6, 9);
            const c = shade(hex('#6f6a72'), rng.range(0.85, 1.1));
            for (let y = y0; y < y1; y++) {
                for (let xx = Math.max(0, x); xx < Math.min(TS, x + w - 1); xx++) {
                    const edge = (y === y0 || xx === x || y === y1 - 1 || xx === x + w - 2);
                    p.px(xx, y, edge ? shade(c, 0.85) : shade(c, rng.range(0.95, 1.05)), edge ? 0.55 : 0.8 + rng.range(-0.05, 0.05));
                }
            }
            if (rng.chance(0.4)) { const mx = rng.int(Math.max(0, x + 1), Math.min(14, x + w - 3)); p.px(mx, y0 + 2, hex('#4f6b3e'), 0.7); p.px(mx + 1, y0 + 2, hex('#3f5a32'), 0.7); } // moss
            x += w;
        }
    });
    return p;
}

function rock(rng) {
    const p = base(rng, hex('#34313a'), 0.2, [], 0, 0.02);
    for (let i = 0; i < 4; i++) {
        const cx = rng.range(3, 13), cy = rng.range(3, 13), r = rng.range(3.5, 6);
        const c = shade(hex('#6b6670'), rng.range(0.8, 1.05));
        for (let y = 0; y < TS; y++) {
            for (let x = 0; x < TS; x++) {
                const d = Math.hypot(x + 0.5 - cx, y + 0.5 - cy) / r;
                if (d < 1) {
                    const h = 0.5 + Math.sqrt(1 - d * d) * 1.4;
                    const lit = d < 0.5 && y < cy ? 1.12 : 1;
                    if (h > p.hgt[y * TS + x]) p.px(x, y, shade(c, lit * rng.range(0.95, 1.05)), h);
                }
            }
        }
    }
    return p;
}

function dock(rng) {
    const p = new Pix(TS, TS);
    for (let b = 0; b < 4; b++) {
        const y0 = b * 4;
        const c = shade(hex('#7a5536'), rng.range(0.85, 1.1));
        for (let y = y0; y < y0 + 3; y++) {
            for (let x = 0; x < TS; x++) p.px(x, y, y === y0 ? shade(c, 1.15) : shade(c, rng.range(0.92, 1.02)), y === y0 + 2 ? 0.5 : 0.7);
        }
        p.px(rng.int(1, 3), y0 + 1, hex('#a8a39a'), 0.9); p.px(rng.int(12, 14), y0 + 1, hex('#a8a39a'), 0.9);
        // row y0+3 stays transparent: water shows through the gaps
    }
    return p;
}

// mask a land tile toward water neighbours (bit 1=N, 2=E, 4=S, 8=W); returns new Pix with foam rim
function coastMask(src, mask, rng) {
    const p = new Pix(TS, TS);
    p.blit(src, 0, 0);
    const foam = hex('#b7c6c8');
    const wet = hex('#5e5640');
    for (let y = 0; y < TS; y++) {
        for (let x = 0; x < TS; x++) {
            // distance into the tile from each water side
            let d = 99;
            if (mask & 1) d = Math.min(d, y);
            if (mask & 2) d = Math.min(d, TS - 1 - x);
            if (mask & 4) d = Math.min(d, TS - 1 - y);
            if (mask & 8) d = Math.min(d, x);
            // round convex corners
            const cn = (mask & 1) && (mask & 8) ? Math.hypot(x - 4.5, y - 4.5) - 4.5 : -99;
            const ce = (mask & 1) && (mask & 2) ? Math.hypot(x - 10.5, y - 4.5) - 4.5 : -99;
            const cs = (mask & 4) && (mask & 2) ? Math.hypot(x - 10.5, y - 10.5) - 4.5 : -99;
            const cw = (mask & 4) && (mask & 8) ? Math.hypot(x - 4.5, y - 10.5) - 4.5 : -99;
            const inCornerZone = (cx, cy) => Math.abs(x - cx) >= 0 && ((cx < 8 ? x < cx : x > cx) && (cy < 8 ? y < cy : y > cy));
            let cut = false;
            if (cn > 0 && inCornerZone(4.5, 4.5)) cut = true;
            if (ce > 0 && inCornerZone(10.5, 4.5)) cut = true;
            if (cs > 0 && inCornerZone(10.5, 10.5)) cut = true;
            if (cw > 0 && inCornerZone(4.5, 10.5)) cut = true;
            const jag = (x * 7 + y * 13) % 3 === 0 ? 1 : 0;
            const i = (y * TS + x) * 4;
            if (cut || d < 1 + jag) { p.col[i + 3] = 0; continue; }
            if (d < 2 + jag || (cut === false && (cn > -1.2 && inCornerZone(4.5, 4.5) || ce > -1.2 && inCornerZone(10.5, 4.5) || cs > -1.2 && inCornerZone(10.5, 10.5) || cw > -1.2 && inCornerZone(4.5, 10.5)))) {
                p.px(x, y, rng.chance(0.7) ? foam : shade(foam, 0.85), 0.45);
                p.glow(x, y, [24, 34, 38, 255]); // faint glow so the surf reads at night
            } else if (d < 4 + jag) {
                const c = p.get(x, y);
                p.px(x, y, mix(c, wet, 0.5), p.hgt[y * TS + x]);
            }
        }
    }
    return p;
}

// Build the atlas. Layout: row per tile type (VARIANTS columns), then coast rows.
export function buildTileAtlas() {
    const rng = new Rng(4711);
    const cols = 16;
    const painters = {
        [T.SAND]: () => sand(rng), [T.GRASS]: () => grass(rng, false), [T.DARK]: () => grass(rng, true),
        [T.DIRT]: () => dirt(rng), [T.STONE]: () => stone(rng), [T.ROCK]: () => rock(rng), [T.DOCK]: () => dock(rng)
    };
    const rowsNeeded = 8 + 4; // types + coast (16 masks x 4 variants = 64 tiles = 4 rows)
    const atlas = new Pix(cols * TS, rowsNeeded * TS);
    const uv = {}; // key -> {x,y,w,h}
    const sandVariants = [];
    for (const [t, fn] of Object.entries(painters)) {
        for (let v = 0; v < VARIANTS; v++) {
            const p = fn();
            if (+t === T.SAND) sandVariants.push(p);
            const x = v * TS, y = (+t) * TS;
            atlas.blit(p, x, y);
            uv[`${t}:${v}`] = { x, y, w: TS, h: TS };
        }
    }
    for (let m = 1; m < 16; m++) {
        for (let v = 0; v < VARIANTS; v++) {
            const p = coastMask(sandVariants[v], m, rng);
            const idx = m * VARIANTS + v;
            const x = (idx % cols) * TS, y = (8 + Math.floor(idx / cols)) * TS;
            atlas.blit(p, x, y);
            uv[`coast:${m}:${v}`] = { x, y, w: TS, h: TS };
        }
    }
    return { pix: atlas, uv };
}
