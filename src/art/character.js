// Paper-doll character painter. Used for the player (from the character creator) and skeletons.
import { Pix, hex, shade } from './painter.js';

export const FW = 20, FH = 26; // frame size; feet at y = 24
export const SKIN = ['#f0c8a2', '#dfab82', '#c78c63', '#a6704b', '#875336', '#673e27', '#4c2c1b', '#c9c2b4'];
export const HAIR_COLORS = ['#1c1714', '#3b2617', '#6a4126', '#9a6a3a', '#c9a050', '#d9d0c0', '#7e7e86', '#7a2a1e', '#2a3a5a', '#4a1f3a'];
export const CLOTH = ['#7a2a2a', '#2a3f6a', '#3e5a32', '#6a4a2a', '#4a2a5a', '#2e2e34', '#8a7a5a', '#5a1e3a', '#1e4a4a', '#a0522d', '#3a3a6a', '#bcae8c'];
export const SCARF = ['#a8322e', '#2e5aa8', '#d8b040', '#2e8a5a', '#7a3aa0', '#e0e0d0', '#202024', '#d06a2a'];
export const HAIR_STYLES = ['Kurz', 'Lang', 'Zopf', 'Locken', 'Glatze', 'Dreadlocks'];
export const BEARDS = ['Keiner', 'Stoppeln', 'Vollbart', 'Kinnbart'];
export const HATS = ['Keine', 'Kopftuch', 'Dreispitz', 'Kapitänsmütze'];
export const BODIES = ['Schmal', 'Breit'];

export const DEFAULT_LOOK = { body: 0, skin: 1, hair: 0, hairColor: 1, beard: 1, hat: 1, coat: 0, accent: 5, scarf: 0, patch: false, scar: false };

const OUT = hex('#140e16');
const BOOT = hex('#2e2018');
const STEEL = hex('#c3ccd2'), STEEL_D = hex('#7d868d');
const GOLD = hex('#c9a24a');

// pose: { leg: -1|0|1, arm: -1|0|1, atk: 0|1|2, roll: 0|1|2, drink, bob }
function drawDown(p, ox, oy, a, pose, up) {
    const skin = hex(a.skinHex), hair = hex(a.hairHex), coat = hex(a.coatHex), acc = hex(a.accHex), scarf = hex(a.scarfHex);
    const S = a.skeleton;
    const bone = hex('#dcd4c0');
    const wide = a.body === 1 ? 1 : 0;
    const by = oy + (pose.bob || 0);
    const P = (x, y, c, h = 0.8) => p.px(ox + x, y, c, h);
    // legs
    const legL = pose.leg === 1 ? -1 : 0, legR = pose.leg === -1 ? -1 : 0;
    const legC = S ? bone : shade(acc, 0.8);
    for (let y = 19; y <= 23 + legL; y++) { P(7, by + y, legC); if (!S) P(8, by + y, legC); }
    for (let y = 19; y <= 23 + legR; y++) { P(12, by + y, legC); if (!S) P(11, by + y, legC); }
    P(7, by + 23 + legL, BOOT); P(8, by + 23 + legL, BOOT); P(11, by + 23 + legR, BOOT); P(12, by + 23 + legR, BOOT);
    // torso
    for (let y = 12; y <= 19; y++) {
        for (let x = 6 - wide; x <= 13 + wide; x++) {
            let c = coat;
            if (S) c = (y % 2 === 0 && x > 6 && x < 13) ? bone : shade(coat, 0.6);
            if (!S && !up && (x === 9 || x === 10) && y > 13) c = shade(coat, 0.8); // coat opening
            P(x, by + y, c, 0.9 - Math.abs(x - 9.5) * 0.05);
        }
    }
    if (!S) { for (let x = 6 - wide; x <= 13 + wide; x++) P(x, by + 18, hex('#3a2418'), 0.95); P(9, by + 18, GOLD, 1); P(10, by + 18, GOLD, 1); }
    P(6 - wide, by + 20, coat); P(13 + wide, by + 20, coat); // coat tails
    // arms
    const aL = pose.arm === 1 ? -1 : (pose.arm === -1 ? 1 : 0);
    const aR = -aL;
    for (let y = 13; y <= 17; y++) { P(5 - wide, by + y + aL, S ? bone : shade(coat, 0.85)); P(14 + wide, by + y + aR, S ? bone : shade(coat, 0.85)); }
    P(5 - wide, by + 18 + aL, S ? bone : skin, 1); P(14 + wide, by + 18 + aR, S ? bone : skin, 1);
    // sword in right hand
    if (!up) {
        if (pose.atk === 1) { // wind-up: blade raised
            for (let k = 0; k < 6; k++) P(15 + wide, by + 17 - k, k ? STEEL : GOLD, 1.3);
        } else if (pose.atk === 2) { // strike: blade down
            for (let k = 0; k < 7; k++) P(14 + wide - (k > 3 ? 1 : 0), by + 19 + k, k ? STEEL : GOLD, 1.3);
        } else {
            P(15 + wide, by + 19 + aR, GOLD, 1.2); P(15 + wide, by + 20 + aR, STEEL, 1.2); P(16 + wide, by + 21 + aR, STEEL, 1.2); P(16 + wide, by + 22 + aR, STEEL_D, 1.2);
        }
    }
    // scarf
    if (!S) {
        for (let x = 7; x <= 12; x++) { P(x, by + 12, scarf, 1); P(x, by + 13, shade(scarf, 0.85), 1); }
        if (!up) { P(12, by + 14, scarf, 1); P(13, by + 15, shade(scarf, 0.8), 1); }
    }
    // head
    for (let y = 4; y <= 11; y++) for (let x = 6; x <= 13; x++) {
        if ((y === 4 || y === 11) && (x === 6 || x === 13)) continue;
        P(x, by + y, S ? bone : skin, 1.2 - Math.abs(x - 9.5) * 0.06);
    }
    if (!up) {
        if (S) { // skull face
            P(8, by + 8, OUT, 0.9); P(11, by + 8, OUT, 0.9); P(7, by + 7, OUT, 0.9); P(12, by + 7, OUT, 0.9);
            p.glow(ox + 8, by + 8, [255, 40, 30, 255]); p.glow(ox + 11, by + 8, [255, 40, 30, 255]);
            P(9, by + 10, OUT, 0.9); P(10, by + 10, OUT, 0.9); P(8, by + 11, bone); P(11, by + 11, bone);
        } else {
            P(8, by + 8, hex('#1a1a26'), 1); P(11, by + 8, hex('#1a1a26'), 1);
            P(9, by + 10, shade(skin, 0.78), 1.1); P(10, by + 10, shade(skin, 0.78), 1.1);
            P(5, by + 8, shade(skin, 0.9), 1); P(14, by + 8, shade(skin, 0.9), 1);
            if (a.beard === 1) for (let x = 7; x <= 12; x++) if ((x + 1) % 2) P(x, by + 10 + (x % 2), shade(skin, 0.7), 1.1);
            if (a.beard === 2) { for (let y = 9; y <= 12; y++) for (let x = 6; x <= 13; x++) if (!(y === 10 && (x === 9 || x === 10))) P(x, by + y, y === 12 && (x < 8 || x > 11) ? null : hair, 1.25); }
            if (a.beard === 3) { P(9, by + 11, hair, 1.2); P(10, by + 11, hair, 1.2); P(9, by + 12, hair, 1.2); P(10, by + 12, hair, 1.2); }
            if (a.patch) { P(8, by + 8, OUT, 1.3); P(7, by + 8, OUT, 1.3); P(8, by + 7, OUT, 1.3); for (let x = 6; x <= 13; x++) P(x, by + 6, hex('#26201e'), 1.3); }
            if (a.scar) { P(11, by + 7, hex('#b5524a'), 1.2); P(12, by + 9, hex('#b5524a'), 1.2); P(11, by + 9, hex('#b5524a'), 1.2); }
        }
    }
    // hair
    if (!S) {
        const h = a.hair;
        if (h !== 4) {
            for (let x = 6; x <= 13; x++) { P(x, by + 3, hair, 1.4); P(x, by + 4, hair, 1.4); }
            for (let x = 7; x <= 12; x++) P(x, by + 2, hair, 1.45);
            P(6, by + 5, hair, 1.35); P(13, by + 5, hair, 1.35);
            if (up) for (let y = 5; y <= 10; y++) for (let x = 6; x <= 13; x++) P(x, by + y, shade(hair, 0.95 + ((x + y) % 3) * 0.04), 1.3);
            if (h === 1 || h === 5) for (let y = 6; y <= 14; y++) { P(5, by + y, hair, 1.2); P(14, by + y, hair, 1.2); if (up) for (let x = 6; x <= 13; x++) if (h === 1 || x % 2) P(x, by + y + 1, shade(hair, 0.9), 1.2); }
            if (h === 2 && up) for (let y = 10; y <= 16; y++) { P(9, by + y, hair, 1.3); P(10, by + y, shade(hair, 0.85), 1.3); }
            if (h === 3) { for (let x = 5; x <= 14; x += 2) { P(x, by + 2, hair, 1.5); P(x, by + 5, hair, 1.4); } P(5, by + 4, hair, 1.4); P(14, by + 4, hair, 1.4); }
            if (h === 5) for (let x = 6; x <= 13; x += 2) P(x, by + 6, shade(hair, 0.8), 1.3);
        } else if (!up) { P(8, by + 5, shade(skin, 1.15), 1.3); }
    }
    // hats
    const hat = S ? 2 : a.hat;
    if (hat === 1) { for (let x = 6; x <= 13; x++) { P(x, by + 3, acc, 1.5); P(x, by + 4, shade(acc, 0.85), 1.5); } for (let x = 7; x <= 12; x++) P(x, by + 2, acc, 1.5); P(up ? 9 : 14, by + 5, acc, 1.4); P(up ? 10 : 15, by + 6, shade(acc, 0.8), 1.3); }
    if (hat === 2) {
        const tc = S ? hex('#2a2024') : hex('#24201f');
        for (let x = 3; x <= 16; x++) { P(x, by + 4, tc, 1.6); }
        for (let x = 5; x <= 14; x++) { P(x, by + 3, tc, 1.7); P(x, by + 2, tc, 1.7); }
        for (let x = 7; x <= 12; x++) P(x, by + 1, tc, 1.75);
        P(3, by + 3, tc, 1.6); P(16, by + 3, tc, 1.6);
        if (!S) for (let x = 5; x <= 14; x++) P(x, by + 4, GOLD, 1.65);
    }
    if (hat === 3) { for (let x = 5; x <= 14; x++) { P(x, by + 2, hex('#1e2440'), 1.6); P(x, by + 3, hex('#1e2440'), 1.6); } for (let x = 4; x <= 15; x++) P(x, by + 4, hex('#15182a'), 1.6); P(9, by + 2, GOLD, 1.7); P(10, by + 2, GOLD, 1.7); }
}

function drawSide(p, ox, oy, a, pose) {
    const skin = hex(a.skinHex), hair = hex(a.hairHex), coat = hex(a.coatHex), acc = hex(a.accHex), scarf = hex(a.scarfHex);
    const S = a.skeleton, bone = hex('#dcd4c0');
    const by = oy + (pose.bob || 0);
    const P = (x, y, c, h = 0.8) => p.px(ox + x, y, c, h);
    // legs (stride)
    const s = pose.leg || 0;
    const legC = S ? bone : shade(acc, 0.8);
    for (let y = 19; y <= 23; y++) { P(9 - s * (y - 18) / 5 * 2 | 0, by + y, legC); P(11 + s * (y - 18) / 5 * 2 | 0, by + y, shade(legC, 0.85)); }
    P(9 - s * 2, by + 23, BOOT); P(10 - s * 2, by + 23, BOOT); P(11 + s * 2, by + 23, BOOT); P(12 + s * 2, by + 23, BOOT);
    // torso
    for (let y = 12; y <= 19; y++) for (let x = 7; x <= 12; x++) P(x, by + y, S ? ((y % 2 === 0 && x > 7) ? bone : shade(coat, 0.6)) : coat, 0.9);
    if (!S) { for (let x = 7; x <= 12; x++) P(x, by + 18, hex('#3a2418'), 0.95); P(11, by + 18, GOLD, 1); }
    P(6, by + 19, coat); P(6, by + 20, coat); // tail behind
    // head
    for (let y = 4; y <= 11; y++) for (let x = 7; x <= 12; x++) P(x, by + y, S ? bone : skin, 1.2);
    P(13, by + 8, S ? bone : skin, 1.2); // nose
    if (S) { P(11, by + 7, OUT, 1); p.glow(ox + 11, by + 7, [255, 40, 30, 255]); P(12, by + 10, OUT, 1); }
    else {
        P(11, by + 7, hex('#1a1a26'), 1.1); P(12, by + 10, shade(skin, 0.78), 1.1);
        if (a.beard === 2) for (let y = 9; y <= 12; y++) for (let x = 9; x <= 12; x++) P(x, by + y, hair, 1.25);
        if (a.beard === 3) { P(12, by + 11, hair, 1.2); P(12, by + 12, hair, 1.2); }
        if (a.patch) { P(11, by + 7, OUT, 1.3); for (let x = 7; x <= 12; x++) P(x, by + 6, hex('#26201e'), 1.3); }
        if (a.scar) { P(10, by + 8, hex('#b5524a'), 1.2); P(10, by + 9, hex('#b5524a'), 1.2); }
    }
    if (!S && a.hair !== 4) {
        for (let x = 7; x <= 12; x++) { P(x, by + 3, hair, 1.4); P(x, by + 4, hair, 1.4); }
        for (let y = 5; y <= 9; y++) { P(7, by + y, hair, 1.3); P(8, by + y, hair, 1.3); }
        if (a.hair === 1 || a.hair === 5) for (let y = 9; y <= 14; y++) P(7, by + y, hair, 1.2);
        if (a.hair === 2) for (let y = 7; y <= 13; y++) P(6, by + y, hair, 1.25);
        if (a.hair === 3) { P(6, by + 3, hair, 1.4); P(6, by + 5, hair, 1.4); P(9, by + 2, hair, 1.5); P(11, by + 2, hair, 1.5); }
    }
    const hat = S ? 2 : a.hat;
    if (hat === 1) { for (let x = 7; x <= 12; x++) { P(x, by + 3, acc, 1.5); P(x, by + 4, shade(acc, 0.85), 1.5); } P(6, by + 5, acc, 1.4); P(5, by + 6, acc, 1.3); }
    if (hat === 2) { const tc = hex('#24201f'); for (let x = 4; x <= 15; x++) P(x, by + 4, tc, 1.6); for (let x = 6; x <= 13; x++) { P(x, by + 3, tc, 1.7); P(x, by + 2, tc, 1.7); } P(4, by + 3, tc, 1.6); P(15, by + 3, tc, 1.6); }
    if (hat === 3) { for (let x = 7; x <= 12; x++) { P(x, by + 2, hex('#1e2440'), 1.6); P(x, by + 3, hex('#1e2440'), 1.6); } for (let x = 7; x <= 15; x++) P(x, by + 4, hex('#15182a'), 1.6); }
    if (!S) { P(8, by + 12, scarf, 1); P(9, by + 12, scarf, 1); P(10, by + 12, scarf, 1); P(11, by + 12, scarf, 1); P(7, by + 13, scarf, 1); P(6, by + 14, shade(scarf, 0.8), 1); }
    // arm + sword
    const sw = pose.arm || 0;
    if (pose.atk === 1) { // raised behind
        for (let y = 12; y <= 15; y++) P(8, by + y, S ? bone : shade(coat, 0.85));
        for (let k = 0; k < 7; k++) P(7 - (k > 3 ? 1 : 0), by + 11 - k, k ? STEEL : GOLD, 1.3);
    } else if (pose.atk === 2) { // thrust forward
        for (let x = 10; x <= 13; x++) P(x, by + 15, S ? bone : shade(coat, 0.85));
        P(14, by + 15, S ? bone : skin, 1); P(15, by + 15, GOLD, 1.2);
        for (let x = 16; x <= 19; x++) P(x, by + 15, STEEL, 1.3);
    } else {
        for (let y = 13; y <= 17; y++) P(10 + (y > 15 ? sw : 0), by + y, S ? bone : shade(coat, 0.85));
        P(10 + sw, by + 18, S ? bone : skin, 1);
        P(11 + sw, by + 18, GOLD, 1.2); P(12 + sw, by + 19, STEEL, 1.2); P(13 + sw, by + 20, STEEL, 1.2); P(14 + sw, by + 21, STEEL_D, 1.2);
    }
}

function drawRoll(p, ox, oy, a, k) {
    const coat = hex(a.coatHex), hair = hex(a.hairHex), scarf = hex(a.scarfHex);
    const cx = ox + 10, cy = oy + 18;
    p.disc(cx, cy, 5.5, coat, 1.2);
    const ang = k * Math.PI / 2;
    p.disc(cx + Math.cos(ang) * 3, cy + Math.sin(ang) * 3, 2.5, a.hair === 4 ? hex(a.skinHex) : hair, 1.5);
    p.px(cx - Math.cos(ang) * 3, cy - Math.sin(ang) * 3, hex(a.skinHex), 1.3);
    p.px(cx + Math.sin(ang) * 4, cy - Math.cos(ang) * 4, scarf, 1.3);
    p.px(cx - Math.sin(ang) * 4, cy + Math.cos(ang) * 4, BOOT, 1.2);
}

function drawDead(p, ox, oy, a) {
    // lying on the side: draw side frame rotated 90° by hand-copy
    const tmp = new Pix(FW, FH);
    drawSide(tmp, 0, 0, a, { leg: 0 });
    for (let y = 0; y < FH; y++) for (let x = 0; x < FW; x++) {
        const c = tmp.get(x, y);
        if (!c || !c[3]) continue;
        const nx = ox + Math.round(y * 0.75) - 1, ny = oy + FH - 3 - Math.round(x * 0.5);
        p.px(nx, ny, c, tmp.hgt[y * FW + x]);
    }
}

function drawDrink(p, ox, oy, a) {
    drawDown(p, ox, oy, a, { leg: 0, arm: 0 }, false);
    const skin = hex(a.skinHex);
    for (let y = 11; y <= 15; y++) p.px(ox + 13, oy + y, shade(hex(a.coatHex), 0.85), 0.9);
    p.px(ox + 12, oy + 10, skin, 1.3);
    p.px(ox + 11, oy + 9, hex('#3d6b3a'), 1.5); p.px(ox + 11, oy + 10, hex('#3d6b3a'), 1.5); p.px(ox + 11, oy + 8, hex('#c9a050'), 1.5); // bottle
    p.glow(ox + 11, oy + 9, [80, 120, 40, 255]);
}

// Animations: each entry is a list of [dir, pose] frames.
const ANIMS = {
    stand: [{ leg: 0, arm: 0 }],
    walk: [{ leg: 1, arm: 1, bob: -1 }, { leg: 0, arm: 0 }, { leg: -1, arm: -1, bob: -1 }, { leg: 0, arm: 0 }],
    windup: [{ leg: 0, arm: 0, atk: 1 }],
    strike: [{ leg: 1, arm: 0, atk: 2 }],
    hurt: [{ leg: 0, arm: 0, bob: 1 }]
};

export function resolveLook(look) {
    return {
        ...look,
        skinHex: SKIN[look.skin] || SKIN[1],
        hairHex: HAIR_COLORS[look.hairColor] || HAIR_COLORS[0],
        coatHex: CLOTH[look.coat] || CLOTH[0],
        accHex: CLOTH[look.accent] || CLOTH[5],
        scarfHex: SCARF[look.scarf] || SCARF[0]
    };
}

export const SKELETON_LOOK = { skeleton: true, body: 0, skinHex: '#dcd4c0', hairHex: '#000000', coatHex: '#3a3440', accHex: '#2a2a30', scarfHex: '#000000', hair: 4, hat: 2, beard: 0 };

// Paint a full sprite sheet. Returns { pix, frames: { 'walk-down': [rects...] , ... } }
export function paintCharacterSheet(look) {
    const a = look.skeleton ? look : resolveLook(look);
    const dirs = ['down', 'up', 'side'];
    const cols = 4 + 1 + 1 + 1 + 2 + 2; // stand, walk4, windup, strike, hurt, roll2/ pile, dead, drink...
    const pix = new Pix(FW * 12, FH * 4);
    const frames = {};
    const add = (name, rect) => { (frames[name] = frames[name] || []).push(rect); };
    dirs.forEach((dir, row) => {
        let col = 0;
        for (const [name, poses] of Object.entries(ANIMS)) {
            for (const pose of poses) {
                const ox = col * FW, oy = row * FH;
                const one = new Pix(FW, FH);
                if (dir === 'side') drawSide(one, 0, 0, a, pose); else drawDown(one, 0, 0, a, pose, dir === 'up');
                one.outline(OUT);
                pix.blit(one, ox, oy);
                add(`${name}-${dir}`, { x: ox, y: oy, w: FW, h: FH });
                col++;
            }
        }
    });
    // extra row: roll (4), dead, drink, pile
    let col = 0;
    const extra = (name, fn) => {
        const one = new Pix(FW, FH); fn(one); one.outline(OUT);
        pix.blit(one, col * FW, 3 * FH);
        add(name, { x: col * FW, y: 3 * FH, w: FW, h: FH });
        col++;
    };
    for (let k = 0; k < 4; k++) extra('roll', one => drawRoll(one, 0, 0, a, k));
    extra('dead', one => drawDead(one, 0, 0, a));
    if (!a.skeleton) extra('drink', one => drawDrink(one, 0, 0, a));
    if (a.skeleton) {
        extra('pile', (one) => {
            const bone = hex('#dcd4c0');
            for (let k = 0; k < 7; k++) one.line(4 + k * 1.6, 22 - (k % 3), 7 + k * 1.4, 20 + (k % 2) * 2, bone, 0.8);
            one.disc(10, 19, 2.6, bone, 1.1); one.px(9, 19, OUT, 0.9); one.px(11, 19, OUT, 0.9);
            one.glow(9, 19, [120, 20, 15, 255]); one.glow(11, 19, [120, 20, 15, 255]);
        });
    }
    pix.autoHeight(0.9, 0.25);
    return { pix, frames };
}
