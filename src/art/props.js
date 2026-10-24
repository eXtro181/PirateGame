// Static props: palms, rocks, ruins, wreck, saber fire, torches. Packed into one atlas.
import { Pix, Rng, Atlas, hex, shade } from './painter.js';

const OUT = hex('#120d14');

function palm(rng) {
    const p = new Pix(36, 52);
    const trunk = hex('#7b5a3c');
    // curved trunk from base (18,51) up to crown (~20,16)
    const bend = rng.range(-5, 5);
    for (let i = 0; i <= 34; i++) {
        const t = i / 34;
        const x = 18 + bend * t * t;
        const y = 51 - i;
        const ring = i % 4 === 0;
        p.hline(Math.round(x - 1), Math.round(x + 1), y, ring ? shade(trunk, 0.75) : trunk, 0.9);
        p.px(Math.round(x - 1), y, shade(trunk, 0.8), 0.6);
        p.px(Math.round(x + 1), y, shade(trunk, 1.15), 0.9);
    }
    const cx = 18 + bend, cy = 16;
    const leaf = hex('#3f7a3e'), leafD = hex('#2c5a30'), leafL = hex('#68a352');
    for (let k = 0; k < 7; k++) {
        const ang = (k / 7) * Math.PI * 2 + rng.range(-0.2, 0.2);
        const len = rng.range(13, 17);
        for (let s = 0; s < len; s++) {
            const t = s / len;
            const x = cx + Math.cos(ang) * s;
            const y = cy + Math.sin(ang) * s * 0.75 + t * t * 6; // droop
            const wdt = Math.max(0.5, (1 - t) * 2.6);
            for (let o = -wdt; o <= wdt; o += 0.5) {
                const nx = x - Math.sin(ang) * o, ny = y + Math.cos(ang) * o * 0.75;
                p.px(nx, ny, o < -wdt * 0.4 ? leafD : (o > wdt * 0.4 ? leafL : leaf), 1.2 + (1 - t) * 0.6);
            }
        }
    }
    p.disc(cx - 1, cy + 1, 1.6, hex('#5b4026'), 1.9); p.disc(cx + 2, cy + 2, 1.4, hex('#5b4026'), 1.9); // coconuts
    p.outline(OUT);
    p.autoHeight(0.6, 0.2);
    return p;
}

function bush(rng) {
    const p = new Pix(18, 14);
    const c = hex('#35603a');
    for (let k = 0; k < 6; k++) p.disc(rng.range(5, 13), rng.range(5, 9), rng.range(3, 4.5), shade(c, rng.range(0.85, 1.15)), 0.8);
    for (let k = 0; k < 6; k++) p.px(rng.int(3, 14), rng.int(3, 10), hex('#5f9150'), 1);
    if (rng.chance(0.4)) for (let k = 0; k < 3; k++) { const x = rng.int(4, 13), y = rng.int(4, 9); p.px(x, y, hex('#c2383c'), 1.1); p.glow(x, y, [60, 8, 10, 255]); }
    p.outline(OUT); p.autoHeight(1, 0.2);
    return p;
}

function boulder(rng, w, h) {
    const p = new Pix(w, h);
    const c = hex('#56514b');
    p.ellipse(w / 2, h / 2 + 1, w / 2 - 1, h / 2 - 1.5, c, 0.5);
    for (let k = 0; k < 3; k++) p.ellipse(rng.range(w * 0.3, w * 0.7), rng.range(h * 0.35, h * 0.6), rng.range(2, w / 4), rng.range(2, h / 4), shade(c, rng.range(0.9, 1.15)), 0.8);
    for (let k = 0; k < 4; k++) p.px(rng.int(2, w - 3), rng.int(2, h - 4), hex('#56723f'), 0.9); // moss
    p.outline(OUT); p.autoHeight(1.4, 0.25);
    return p;
}

function pillar(rng) {
    const p = new Pix(14, 30);
    const c = hex('#8a8590');
    p.rect(3, 6, 8, 22, c, 0.8);
    for (let y = 6; y < 28; y++) { p.px(3, y, shade(c, 0.75), 0.6); p.px(10, y, shade(c, 1.15), 0.9); p.px(6, y, shade(c, 0.9), 0.7); }
    p.rect(1, 26, 12, 4, shade(c, 0.85), 0.6); // base
    // broken top
    for (let x = 3; x <= 10; x++) { const top = 6 - ((x * 3) % 5); p.vline(x, top, 6, c, 0.9); }
    p.px(5, 14, hex('#4f6b3e'), 0.9); p.px(5, 15, hex('#4f6b3e'), 0.9); p.px(8, 21, hex('#4f6b3e'), 0.9);
    p.outline(OUT); p.autoHeight(0.8, 0.2);
    return p;
}

function crate(rng) {
    const p = new Pix(14, 14);
    const c = hex('#86603c');
    p.rect(1, 1, 12, 12, c, 0.8);
    p.hline(1, 12, 1, shade(c, 1.2), 0.9); p.vline(1, 1, 12, shade(c, 1.1), 0.8);
    p.line(2, 2, 11, 11, shade(c, 0.7), 1); p.line(11, 2, 2, 11, shade(c, 0.7), 1);
    p.outline(OUT); p.autoHeight(0.5, 0.2);
    return p;
}

function barrel() {
    const p = new Pix(12, 14);
    const c = hex('#7a5233');
    p.ellipse(6, 7, 5, 6, c, 0.9);
    p.hline(2, 9, 4, hex('#4a4a50'), 1); p.hline(2, 9, 10, hex('#4a4a50'), 1);
    p.vline(8, 3, 11, shade(c, 1.2), 1);
    p.outline(OUT); p.autoHeight(1, 0.2);
    return p;
}

function skullPile(rng) {
    const p = new Pix(14, 9);
    for (let k = 0; k < 3; k++) {
        const x = 3 + k * 4, y = 4 + (k % 2);
        p.disc(x, y, 2.2, hex('#d8d0bc'), 0.8);
        p.px(x - 1, y, OUT, 0.5); p.px(x + 1, y, OUT, 0.5);
    }
    p.line(1, 7, 12, 6, hex('#c9c0a8'), 0.6);
    p.outline(OUT); p.autoHeight(0.6, 0.2);
    return p;
}

function grave(rng) {
    const p = new Pix(10, 18);
    const c = hex('#6a503a');
    p.rect(4, 2, 2, 15, c, 0.9); p.rect(1, 5, 8, 2, c, 0.9);
    p.ellipse(5, 16, 4, 1.5, hex('#4a3a2c'), 0.3);
    p.outline(OUT); p.autoHeight(0.5, 0.2);
    return p;
}

function torch() {
    const p = new Pix(8, 22);
    p.rect(3, 6, 2, 15, hex('#5a4030'), 0.8);
    p.rect(2, 4, 4, 3, hex('#3a3a40'), 1);
    p.px(3, 3, hex('#ffcf6a'), 1.2); p.px(4, 3, hex('#ffcf6a'), 1.2);
    p.glow(3, 3, [255, 200, 90, 255]); p.glow(4, 3, [255, 200, 90, 255]);
    p.outline(OUT); p.autoHeight(0.5, 0.2);
    return p;
}

// The land bonfire: a saber rammed into the earth, a skull on the hilt.
export function saberFire() {
    const p = new Pix(18, 30);
    // ash ring + stones
    p.ellipse(9, 26, 8, 3, hex('#2b2526'), 0.2);
    for (let k = 0; k < 8; k++) { const a = k / 8 * Math.PI * 2; p.disc(9 + Math.cos(a) * 7, 26 + Math.sin(a) * 2.5, 1.3, hex('#5d5860'), 0.7); }
    p.ellipse(9, 26, 5, 1.6, hex('#160e10'), 0.1);
    // blade (curved cutlass) from ground (9,26) to guard (9,11)
    for (let y = 12; y <= 26; y++) {
        const bx = 9 + Math.round(Math.sin((y - 12) / 14 * 1.2) * 1.5);
        p.px(bx, y, hex('#b9c2c8'), 1.4); p.px(bx + 1, y, hex('#7e868d'), 1.2);
    }
    // guard
    p.hline(5, 13, 11, hex('#a07b3a'), 1.6); p.px(5, 10, hex('#a07b3a'), 1.6); p.px(13, 12, hex('#a07b3a'), 1.6);
    // grip
    p.vline(9, 6, 10, hex('#3a2418'), 1.6);
    // skull on hilt
    p.disc(9, 4.5, 3.4, hex('#dcd3bd'), 1.8);
    p.rect(7, 7, 5, 2, hex('#cfc5ad'), 1.7);
    p.px(7, 4, OUT, 1.2); p.px(8, 4, OUT, 1.2); p.px(10, 4, OUT, 1.2); p.px(11, 4, OUT, 1.2);
    p.glow(7, 4, [40, 255, 200, 255]); p.glow(11, 4, [40, 255, 200, 255]); // eye sockets glow
    p.px(9, 6, OUT, 1.3); p.px(8, 8, OUT, 1.5); p.px(10, 8, OUT, 1.5);
    p.outline(OUT); p.autoHeight(0.5, 0.2);
    return p;
}

// The sea bonfire: a burning shipwreck sticking out of the water.
export function wreck() {
    const p = new Pix(72, 44);
    const wood = hex('#5c3d27'), dark = hex('#3a271b'), burnt = hex('#231816');
    // hull, tilted
    for (let x = 6; x < 66; x++) {
        const t = (x - 6) / 60;
        const top = 14 + Math.round(Math.sin(t * Math.PI) * -6 + t * 6);
        const bot = 38 - Math.round(Math.sin(t * Math.PI) * 4);
        for (let y = top; y <= bot; y++) {
            const plank = (y - top) % 4 === 0;
            const c = x > 48 ? burnt : (plank ? dark : shade(wood, 0.9 + ((x * 7 + y) % 5) * 0.04));
            p.px(x, y, c, 0.6 + (1 - Math.abs((y - (top + bot) / 2) / ((bot - top) / 2 + 1))) * 0.9);
        }
    }
    // broken ribs on the burnt end
    for (let k = 0; k < 4; k++) { const x = 50 + k * 4; p.line(x, 12 + k, x + 2, 30, burnt, 1.2); }
    // broken mast with torn sail
    p.line(30, 22, 22, 2, hex('#4a3322'), 1.8);
    for (let y = 4; y < 16; y++) for (let x = 24; x < 24 + (16 - y) * 0.8; x++) if ((x + y) % 7) p.px(x, y, hex('#8f8572'), 1.5);
    // embers on deck
    for (let k = 0; k < 14; k++) { const x = 36 + (k * 13) % 26, y = 20 + (k * 7) % 12; p.px(x, y, hex('#ff8a3a'), 1); p.glow(x, y, [255, 110, 30, 255]); }
    p.outline(OUT); p.autoHeight(0.6, 0.2);
    return p;
}

function ruinWall(rng) {
    const p = new Pix(34, 22);
    const c = hex('#77727c');
    for (let row = 0; row < 4; row++) {
        let x = (row % 2) * 3;
        const y = 4 + row * 4 + (row ? 0 : 0);
        const len = 34 - (row === 0 ? rng.int(8, 14) : 0);
        while (x < len) {
            const w = rng.int(5, 8);
            p.rect(x, y, w - 1, 3, shade(c, rng.range(0.85, 1.1)), 0.9);
            p.hline(x, x + w - 2, y, shade(c, 1.2), 1);
            x += w;
        }
    }
    p.px(10, 9, hex('#4f6b3e'), 1); p.px(11, 9, hex('#4f6b3e'), 1); p.px(22, 13, hex('#4f6b3e'), 1);
    p.outline(OUT); p.autoHeight(0.6, 0.2);
    return p;
}

function tuft(rng) {
    const p = new Pix(8, 7);
    for (let k = 0; k < 5; k++) { const x = rng.int(1, 6); p.line(x, 6, x + rng.int(-1, 1), rng.int(1, 3), shade(hex('#5d8a4c'), rng.range(0.8, 1.2)), 0.8); }
    return p;
}

function flowers(rng) {
    const p = new Pix(8, 8);
    const col = rng.pick([hex('#d8c45a'), hex('#c9d2e8'), hex('#c85a7a')]);
    for (let k = 0; k < 4; k++) { const x = rng.int(1, 6), y = rng.int(1, 6); p.px(x, y, col, 0.8); p.px(x, y + 1, hex('#3f6b3a'), 0.5); }
    return p;
}

function chestOpen() {
    const p = new Pix(16, 15);
    p.rect(1, 5, 14, 9, hex('#6b4428'), 0.8);
    p.rect(1, 0, 14, 4, hex('#5a3a22'), 1.3); // lid flipped back
    p.rect(2, 5, 12, 3, hex('#1a1210'), 0.4); // dark inside
    p.hline(1, 14, 7, hex('#c9a24a'), 1.2);
    p.px(5, 6, hex('#bff6ff'), 0.6); p.glow(5, 6, [60, 120, 140, 255]);
    p.outline(OUT); p.autoHeight(0.6, 0.2);
    return p;
}

function dockPost() {
    const p = new Pix(6, 10);
    p.disc(3, 5, 2.5, hex('#5a3f2a'), 1);
    p.disc(3, 4.5, 1.2, hex('#7a5a3e'), 1.2);
    p.outline(OUT); p.autoHeight(0.8, 0.2);
    return p;
}

function chest() {
    const p = new Pix(16, 13);
    p.rect(1, 3, 14, 9, hex('#6b4428'), 0.8);
    p.rect(1, 1, 14, 4, hex('#7d522f'), 1.1);
    p.hline(1, 14, 5, hex('#c9a24a'), 1.2);
    p.rect(7, 4, 2, 3, hex('#e2c260'), 1.3);
    p.glow(7, 5, [120, 90, 20, 255]);
    p.outline(OUT); p.autoHeight(0.6, 0.2);
    return p;
}

// Prop catalogue: collision radius (tiles) and caster height (tiles, 0 = no shadow)
export const PROP_INFO = {
    palm: { r: 0.35, caster: 3.0, variants: 3 },
    bush: { r: 0.45, caster: 0.8, variants: 3 },
    boulder: { r: 0.6, caster: 1.0, variants: 2 },
    boulderBig: { r: 0.9, caster: 1.4, variants: 2 },
    pillar: { r: 0.4, caster: 1.8, variants: 2 },
    crate: { r: 0.45, caster: 0.8, variants: 1 },
    barrel: { r: 0.4, caster: 0.8, variants: 1 },
    skulls: { r: 0, caster: 0, variants: 1 },
    grave: { r: 0.2, caster: 1.0, variants: 1 },
    torch: { r: 0.15, caster: 1.3, variants: 1 },
    wall: { r: 0, caster: 1.4, variants: 2, box: [2.1, 0.5] },
    tuft: { r: 0, caster: 0, variants: 3 },
    flowers: { r: 0, caster: 0, variants: 3 },
    post: { r: 0.2, caster: 0.6, variants: 1 },
    chest: { r: 0.45, caster: 0.7, variants: 1 },
    chestOpen: { r: 0, caster: 0, variants: 1 },
    saberFire: { r: 0.45, caster: 0, variants: 1 },
    wreck: { r: 0, caster: 2.0, variants: 1 }
};

export function buildPropAtlas() {
    const rng = new Rng(1234);
    const atlas = new Atlas(512, 256);
    const make = {
        palm: () => palm(rng), bush: () => bush(rng), boulder: () => boulder(rng, 18, 15), boulderBig: () => boulder(rng, 28, 22),
        pillar: () => pillar(rng), crate: () => crate(rng), barrel: () => barrel(), skulls: () => skullPile(rng), grave: () => grave(rng),
        torch: () => torch(), wall: () => ruinWall(rng), tuft: () => tuft(rng), flowers: () => flowers(rng), post: () => dockPost(),
        chest: () => chest(), chestOpen: () => chestOpen(), saberFire: () => saberFire(), wreck: () => wreck()
    };
    for (const [name, info] of Object.entries(PROP_INFO)) {
        for (let v = 0; v < info.variants; v++) atlas.add(`${name}:${v}`, make[name]());
    }
    return atlas;
}
