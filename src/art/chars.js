// Character sprites: 32×32 frames (body ~16×30, head ≈ ⅓ of the height), three directions
// (down, up, side → left is the side frame flipped), procedural poses for every animation.
// Humanoids: player (all creator options), NPCs, skeletons, the drowned, ghost kings (scale 1.5).
import { Pix } from './pix.js';
import { P, ramp, rgb, shade } from './pal.js';

export const FRAMES = { idle: 2, walk: 6, atk: 4, roll: 4, hurt: 1, dead: 1, cast: 2, drink: 2, sit: 1, shoot: 1 };
export const DIRS3 = ['down', 'up', 'side'];

// ---------------------------------------------------------------- creator options
export const SKIN = ['#C9A07E', '#B08A6E', '#8A6248', '#5E4030'];
export const HAIR = ['#2A1E1A', '#5A3A22', '#8A6A3A', '#9A9A92', '#7A2A1E'];
export const COAT = [
    { name: 'Weinrot', c: '#5A2A30' }, { name: 'Marine', c: '#2E3E52' }, { name: 'Moos', c: '#3E4A34' },
    { name: 'Leder', c: '#5A4230' }, { name: 'Teer', c: '#2A2A30' }, { name: 'Verblichen', c: '#5A4A6A' }
];
export const LOOK_OPTS = [
    ['Statur', 'body', ['Schlank', 'Kräftig']],
    ['Haut', 'skin', ['Hell', 'Sonnengebräunt', 'Bronze', 'Dunkel']],
    ['Haar', 'hair', ['Glatze', 'Kurz', 'Lang', 'Zopf', 'Wild']],
    ['Haarfarbe', 'hairCol', ['Schwarz', 'Braun', 'Strohblond', 'Grau', 'Rost']],
    ['Bart', 'beard', ['Keiner', 'Stoppeln', 'Vollbart', 'Zöpfe']],
    ['Kopf', 'hat', ['Nichts', 'Dreispitz', 'Kopftuch', 'Kapuze', 'Kapitänshut']],
    ['Mantel', 'coat', COAT.map(c => c.name)],
    ['Augenklappe', 'patch', ['Nein', 'Ja']]
];
export const DEFAULT_LOOK = { body: 0, skin: 1, hair: 1, hairCol: 1, beard: 1, hat: 1, coat: 0, patch: 0 };

const h01 = (x, y, s = 0) => { const v = Math.sin(x * 127.1 + y * 311.7 + s * 74.7) * 43758.5453; return v - Math.floor(v); };

// ---------------------------------------------------------------- weapons
const STEEL = '#B8C0C8', STEEL_D = '#6A7280', GOLD = P.gold;
function drawWeapon(p, hx, hy, ang, kind, S = 1) {
    const ca = Math.cos(ang), sa = Math.sin(ang);
    const along = (t, c, side = 0) => p.px(Math.round(hx + ca * t - sa * side), Math.round(hy + sa * t + ca * side), c);
    const L = { cutlass: 10, rapier: 13, dagger: 6, hammer: 12, club: 10, fetish: 10, trident: 15, anchor: 14, greatsword: 16 }[kind] * S;
    if (kind === 'hammer' || kind === 'anchor') {
        for (let t = -2; t < L; t++) along(t, t < 0 ? P.tarwood : '#5A4A3A');
        for (let t = L - 3 * S; t <= L; t++) for (let s = -3 * S; s <= 3 * S; s++) along(t, s, Math.abs(s) > 2 * S ? STEEL_D : (t === L ? STEEL : '#8A9098'));
        return;
    }
    if (kind === 'club') { for (let t = -1; t < L; t++) { along(t, t > L * 0.6 ? P.oldwood : P.tarwood); if (t > L * 0.5) along(t, P.deadwood, 1); } return; }
    if (kind === 'fetish') { for (let t = -1; t < L; t++) along(t, P.deadwood); for (let s = -2; s <= 2; s++) along(L, P.bone, s); along(L + 1, P.bone); const x = Math.round(hx + ca * (L + 1)), y = Math.round(hy + sa * (L + 1)); p.lit(x, y, P.soul, 1.3); return; }
    if (kind === 'trident') { for (let t = -2; t < L; t++) along(t, '#4A5258'); for (const s of [-2, 0, 2]) for (let t = L - 3; t <= L + (s ? 0 : 1); t++) along(t, STEEL, s); along(L - 3, STEEL, -1); along(L - 3, STEEL, 1); return; }
    // blades: hilt + guard + blade
    along(-1, P.tarwood); along(0, GOLD);
    for (let s = -2; s <= 2; s++) along(1, GOLD, s);
    const w = kind === 'greatsword' ? 1 : 0;
    for (let t = 2; t < L; t++) {
        const curve = kind === 'cutlass' ? Math.round((t / L) * (t / L) * 2) : 0;
        along(t, STEEL, -curve); if (kind !== 'rapier') along(t, STEEL_D, -curve + 1);
        if (w) along(t, STEEL_D, -curve - 1);
    }
}
function drawPistol(p, x, y, dirx) { for (let k = 0; k < 5; k++) p.px(x + k * dirx, y, k < 2 ? P.tarwood : STEEL_D); p.px(x, y + 1, P.tarwood); p.px(x + 4 * dirx, y - 1, STEEL); }

// ---------------------------------------------------------------- humanoid painter
// pose: { bob, legA, legB (−2..2), armA, armB, wAng, wShow, cast, drink, sit, shoot, lean }
function paintHuman(p, dir, pose, L, S = 1) {
    const q = v => Math.round(v * S);
    const W = p.w, cx = Math.floor(W / 2), foot = p.h - 1;
    const sk = L.skeleton, dr = L.drowned;
    const skin = ramp(sk ? P.bone : L.skinHex), coat = ramp(L.coatHex), pants = ramp(L.pantsHex || '#3A3430');
    const hair = ramp(L.hairHex || HAIR[0]), shirt = ramp(L.shirtHex || '#8F8672');
    const boot = '#22181A';
    const by = foot - q(31) + (pose.bob || 0) + (pose.sit ? q(6) : 0); // frame top for the body
    const Y = v => by + q(v);
    const X = v => cx + q(v - 16);
    const side = dir === 'side', up = dir === 'up';
    const wide = L.body ? 1 : 0;
    const weaponFirst = up || (side && pose.wBehind);
    const weapon = () => {
        if (!pose.wShow || !L.weapon) return;
        const hand = side ? [X(18 + (pose.armA || 0)), Y(23)] : up ? [X(9 - wide), Y(22)] : [X(22 + wide), Y(23)];
        drawWeapon(p, hand[0], hand[1], pose.wAng, L.weapon, S);
    };
    if (weaponFirst) weapon();
    // ---- legs
    const legs = () => {
        if (pose.sit) { // folded legs in front
            const ly = Y(26);
            if (side) { p.rect(X(13), ly, q(9), q(3), pants[2]); p.rect(X(21), ly, q(2), q(4), boot); }
            else { p.rect(X(10), ly, q(5), q(3), pants[2]); p.rect(X(17), ly, q(5), q(3), pants[2]); p.rect(X(9), ly + q(2), q(2), q(2), boot); p.rect(X(21), ly + q(2), q(2), q(2), boot); }
            return;
        }
        const leg = (lx, off, near) => {
            const lift = off > 0 ? 1 : 0;
            const x = side ? X(lx + off) : X(lx), top = Y(24), bot = foot - lift;
            const R = sk ? skin : pants;
            const lw = sk ? 1 : q(3);
            for (let y = top; y <= bot; y++) for (let k = 0; k < lw; k++) {
                const isBoot = !sk && y > bot - q(3);
                p.px(x + k, y, isBoot ? (k === 0 ? '#3A2A26' : boot) : R[k === 0 ? 3 : near ? 2 : 1]);
            }
            if (sk) { p.px(x - 1, bot, skin[1]); p.px(x + 1, bot, skin[2]); p.px(x, Y(27), skin[4]); }
            if (!side && !sk && off !== 0) p.px(x + 1, bot + 1, boot);
        };
        if (side) { leg(15, -(pose.legA || 0), false); leg(16, pose.legA || 0, true); }
        else { leg(12 - wide, pose.legA || 0, true); leg(17 + wide, pose.legB || 0, false); }
    };
    // ---- torso + coat
    const torso = () => {
        const t0 = Y(15), t1 = Y(24);
        const x0 = side ? X(13) : X(11 - wide), x1 = side ? X(19) : X(20 + wide);
        if (sk) { // ribcage
            for (let y = t0; y <= t1; y++) for (let x = x0 + 1; x < x1; x++) p.px(x, y, '#141418');
            for (let y = t0 + 1; y < t1 - 2; y += 2) p.hline(x0 + 1, x1 - 1, y, skin[y % 4 ? 2 : 3]);
            p.vline(Math.floor((x0 + x1) / 2), t0, t1, skin[3]);
            p.hline(x0 + 2, x1 - 2, t1 - 1, skin[1]);
            if (L.rags) for (let y = t0; y <= t1 + q(3); y++) { if (h01(y, 3) > 0.35) p.px(x0, y, coat[1]); if (h01(y, 4) > 0.35) p.px(x1, y, coat[1]); if (y > t1 && h01(y, 5) > 0.5) p.px(x0 + 1, y, coat[0]); }
            return;
        }
        for (let y = t0; y <= t1; y++) for (let x = x0; x <= x1; x++) {
            const rel = (x - x0) / Math.max(1, x1 - x0);
            const open = !up && !side && rel > 0.36 && rel < 0.64 && !L.robe;
            const R = open ? shirt : coat;
            p.px(x, y, R[rel < 0.2 ? 3 : rel > 0.8 ? 1 : 2]);
        }
        // coat tails
        if (!pose.sit) for (let y = t1 + 1; y <= Y(27 + (L.robe ? 3 : 0)); y++) {
            if (side) { p.px(x0, y, coat[1]); p.px(x0 + 1, y, coat[2]); if (L.robe) for (let x = x0 + 2; x <= x1; x++) p.px(x, y, coat[2]); }
            else if (L.robe) for (let x = x0; x <= x1; x++) p.px(x, y, coat[x === x0 ? 3 : x === x1 ? 1 : 2]);
            else { p.px(x0, y, coat[3]); p.px(x0 + 1, y, coat[2]); p.px(x1, y, coat[1]); p.px(x1 - 1, y, coat[2]); if (up) for (let x = x0 + 2; x < x1 - 1; x++) p.px(x, y, coat[1]); }
        }
        // belt + buckle, shoulder line
        p.hline(x0, x1, Y(22), '#22181A');
        if (!up) p.px(side ? x1 - 1 : Math.floor((x0 + x1) / 2), Y(22), GOLD);
        p.hline(x0, x1, t0, coat[4]);
        if (L.apron) for (let y = Y(18); y <= Y(27); y++) for (let x = x0 + 2; x <= x1 - 2; x++) p.px(x, y, ramp('#5A4A3A')[x === x0 + 2 ? 3 : 2]);
        if (dr) for (let i = 0; i < 6; i++) { const x = x0 + Math.floor(h01(i, 1) * (x1 - x0)); for (let y = t0 + Math.floor(h01(i, 2) * 4); y < t1 + 3; y++) p.px(x, y, i % 2 ? P.kelp : '#2E5A3A'); }
        if (L.sash) p.hline(x0, x1, Y(21), L.sash);
    };
    // ---- arms
    const arms = () => {
        const R = sk ? skin : coat;
        const aw = sk ? 1 : q(2);
        const arm = (ax, swing, near, hold) => {
            let top = Y(16), bot = Y(22) + swing;
            if (pose.cast) { top = Y(8); bot = Y(16); }
            for (let y = top; y <= bot; y++) for (let k = 0; k < aw; k++) p.px(ax + k, y, R[near ? (k ? 2 : 3) : 1]);
            const hy = pose.cast ? top - 1 : bot + 1;
            for (let k = 0; k < aw; k++) { p.px(ax + k, hy, skin[2]); p.px(ax + k, hy + (pose.cast ? -1 : 1), skin[1]); }
            if (pose.cast) { p.lit(ax, hy - 2, L.castCol || P.soul, 1.4); p.lit(ax + 1, hy - 2, L.castCol || P.soul, 1.4); if (pose.cast > 1) { p.lit(ax, hy - 3, P.soulLight, 1.6); p.lit(ax - 1, hy - 2, P.soul, 1); p.lit(ax + 2, hy - 2, P.soul, 1); } }
            if (hold === 'lantern') { p.rect(ax - 1, hy + 1, 3, 4, P.deadwood); for (let y = hy + 2; y < hy + 4; y++) p.lit(ax, y, L.lanternCol || P.soul, 1.5); }
        };
        if (pose.drink) { // right hand at the mouth with a bottle
            const hx = side ? X(19) : X(18), hy = Y(12);
            for (let y = hy; y <= Y(20); y++) p.px(hx + (side ? 0 : 1), y, R[2]);
            p.rect(hx - 1, hy - (pose.drink > 1 ? 4 : 3), 2, 4, '#3A5A3A'); p.px(hx - 1, hy - (pose.drink > 1 ? 4 : 3), P.oldwood);
            if (!side) arm(X(9 - wide), 0, true);
            return;
        }
        if (pose.shoot) {
            if (side) { for (let x = X(17); x <= X(23); x++) p.px(x, Y(18), R[2]); drawPistol(p, X(24), Y(18), 1); }
            else if (up) { arm(X(9 - wide), 0, false); for (let y = Y(10); y <= Y(17); y++) p.px(X(21 + wide), y, R[2]); }
            else { arm(X(9 - wide), 0, true); for (let y = Y(17); y <= Y(25); y++) p.px(X(21 + wide), y, R[2]); for (let k = 0; k < 4; k++) p.px(X(21 + wide), Y(26) + k, k < 1 ? P.tarwood : STEEL_D); }
            return;
        }
        if (side) arm(X(16 + (pose.armA || 0)), 0, true, L.hold);
        else { arm(X(9 - wide), pose.armA || 0, !up, up ? null : L.hold); arm(X(21 + wide), pose.armB || 0, up, up ? L.hold : null); }
    };
    // ---- head
    const head = () => {
        const lean = pose.lean || 0;
        const hx0 = (side ? X(12) : X(11)) + lean, hx1 = (side ? X(20) : X(20)) + lean, hy0 = Y(5), hy1 = Y(14);
        for (let y = hy0; y <= hy1; y++) for (let x = hx0; x <= hx1; x++) {
            const corner = (y === hy0 || y === hy1) && (x === hx0 || x === hx1);
            if (corner) continue;
            const rel = (x - hx0) / (hx1 - hx0), relY = (y - hy0) / (hy1 - hy0);
            p.px(x, y, skin[relY > 0.8 ? 1 : rel < 0.3 ? 3 : rel > 0.8 ? 1 : 2]);
        }
        if (sk) {
            const ey = Y(9);
            if (!up) {
                const eyes = side ? [hx1 - 3] : [hx0 + 2, hx1 - 3];
                for (const e of eyes) { p.px(e, ey, '#0B0B0E'); p.px(e + 1, ey, '#0B0B0E'); p.px(e, ey + 1, '#0B0B0E'); p.lit(e + (side ? 1 : 0), ey, L.eyeCol || P.soul, 1.3); }
                p.px(side ? hx1 - 1 : Math.floor((hx0 + hx1) / 2), ey + 2, '#1A1A1E');
                for (let x = hx0 + 2; x < hx1 - 1; x += 2) p.px(x, hy1 - 1, '#1A1A1E');
            }
        } else if (!up) {
            const ey = Y(10) + (L.hat === 3 ? 0 : 0);
            const eyeC = dr ? null : '#14101A';
            if (side) { p.px(hx1 - 2, ey, eyeC || '#14101A'); if (dr) p.lit(hx1 - 2, ey, L.eyeCol || P.soul, 1.2); p.px(hx1 + 1, ey + 1, skin[2]); if (L.patch) { p.px(hx1 - 2, ey, '#0B0B0E'); p.hline(hx0 + 2, hx1, ey - 2, '#0B0B0E'); } }
            else {
                const e1 = hx0 + 2, e2 = hx1 - 3;
                p.px(e1, ey, '#14101A'); p.px(e2, ey, '#14101A'); p.px(e1, ey - 1, skin[0]); p.px(e2, ey - 1, skin[0]);
                if (dr) { p.lit(e1, ey, L.eyeCol || P.soul, 1.2); p.lit(e2, ey, L.eyeCol || P.soul, 1.2); }
                if (L.patch) { p.rect(e2 - 1, ey - 1, 3, 2, '#0B0B0E'); p.line(hx0, ey - 3, e2, ey - 1, '#0B0B0E'); }
                p.px(Math.floor((hx0 + hx1) / 2), ey + 2, skin[1]);
            }
        }
        // hair
        const hc = hair;
        if (!sk && L.hair) {
            for (let x = hx0; x <= hx1; x++) { p.px(x, hy0, hc[2]); p.px(x, hy0 + 1, hc[x % 3 ? 2 : 1]); if (up || L.hair >= 2) p.px(x, hy0 + 2, hc[1]); }
            if (up) for (let y = hy0; y <= hy1 - (L.hair >= 2 ? 0 : 2); y++) for (let x = hx0; x <= hx1; x++) p.px(x, y, hc[(x + y) % 4 === 0 ? 1 : 2]);
            if (side) for (let y = hy0; y <= hy0 + 5 + (L.hair >= 2 ? 4 : 0); y++) for (let x = hx0; x <= hx0 + 3; x++) p.px(x, y, hc[x === hx0 ? 1 : 2]);
            else if (!up) for (let y = hy0; y <= hy0 + (L.hair >= 2 ? 9 : 4); y++) { p.px(hx0, y, hc[1]); p.px(hx1, y, hc[1]); }
            if (L.hair === 2 && !side) for (let y = hy1 - 2; y <= Y(18); y++) { p.px(hx0 - 1, y, hc[1]); p.px(hx1 + 1, y, hc[1]); }
            if (L.hair === 3) { const tx = side ? hx0 - 1 : up ? Math.floor((hx0 + hx1) / 2) : hx1 + 1; for (let y = hy0 + 4; y <= Y(19); y++) p.px(tx, y, hc[y % 2 ? 1 : 2]); }
            if (L.hair === 4) for (let x = hx0 - 1; x <= hx1 + 1; x += 2) p.px(x, hy0 - 1, hc[2]);
        }
        // beard
        if (!sk && !up && L.beard) {
            const bx0 = side ? hx1 - 4 : hx0 + 1, bx1 = hx1 - (side ? 0 : 1);
            if (L.beard === 1) for (let x = bx0; x <= bx1; x++) { if ((x + hy1) % 2) p.px(x, hy1 - 1, hc[1]); p.px(x, hy1, hc[1]); }
            else for (let y = hy1 - 2; y <= hy1 + (L.beard === 3 ? 4 : 2); y++) for (let x = bx0; x <= bx1; x++) { if (L.beard === 3 && y > hy1 + 1 && (x - bx0) % 3 !== 1) continue; p.px(x, y, hc[(x + y) % 3 ? 2 : 1]); }
            if (L.beard >= 2) p.px(side ? hx1 - 2 : Math.floor((hx0 + hx1) / 2), hy1 - 2, '#2A1414');
        }
        // headgear
        const H = L.hat;
        if (H === 1 || H === 4 || L.crown) { // tricorn / captain hat
            const hat = ramp(L.hatHex || '#22202A');
            const brim0 = hx0 - 2, brim1 = hx1 + 2, top = hy0 - (H === 4 ? 6 : 4);
            for (let y = top; y <= hy0 + 1; y++) for (let x = brim0; x <= brim1; x++) {
                const rim = y >= hy0;
                const inCrown = x > hx0 && x < hx1;
                if (!rim && !inCrown) continue;
                if (rim && (x === brim0 || x === brim1) && y === hy0 + 1) continue;
                p.px(x, y, hat[rim ? (y === hy0 ? 3 : 1) : 2]);
            }
            p.hline(brim0 + 1, brim1 - 1, hy0, GOLD);
            if (H === 4) { for (let k = 0; k < 6; k++) p.px(hx1 - k, top - 1 - Math.floor(k / 2), k < 3 ? P.blood : '#A83030'); }
            if (L.crown) for (let x = hx0; x <= hx1; x += 2) { p.lit(x, top - 1, L.crownCol || GOLD, 0.9); p.lit(x, top - 2, L.crownCol || GOLD, 1.1); }
        } else if (H === 2) { // bandana
            const b = ramp(L.bandHex || '#7A1A22');
            for (let y = hy0 - 1; y <= hy0 + 2; y++) for (let x = hx0; x <= hx1; x++) p.px(x, y, b[y === hy0 - 1 ? 3 : 2]);
            const kx = side ? hx0 - 1 : up ? Math.floor((hx0 + hx1) / 2) : hx1 + 1;
            if (side || up) { p.px(kx, hy0 + 2, b[1]); p.px(kx - (side ? 1 : 0), hy0 + 3, b[1]); p.px(kx - (side ? 1 : 0), hy0 + 4, b[0]); }
        } else if (H === 3) { // hood: covers hair, shadows the face
            for (let y = hy0 - 2; y <= hy1 + 1; y++) for (let x = hx0 - 1; x <= hx1 + 1; x++) {
                const inner = !up && x > hx0 + (side ? 2 : 0) && x < hx1 - (side ? -1 : 0) && y > hy0 + 1;
                if (inner) { const c = p.get(x, y); if (c && c[3]) p.px(x, y, shade(c, 0.6)); continue; }
                if ((y === hy0 - 2) && (x === hx0 - 1 || x === hx1 + 1)) continue;
                p.px(x, y, coat[x === hx0 - 1 ? 3 : x === hx1 + 1 ? 1 : 2]);
            }
            if (!up && !L.skeleton) { const ey = Y(10); if (!side) { p.px(hx0 + 2, ey, '#0B0B0E'); p.px(hx1 - 3, ey, '#0B0B0E'); } }
            if (L.hoodGlow && !up) { const ey = Y(10); if (side) p.lit(hx1 - 2, ey, L.eyeCol || P.soul, 1.3); else { p.lit(hx0 + 2, ey, L.eyeCol || P.soul, 1.3); p.lit(hx1 - 3, ey, L.eyeCol || P.soul, 1.3); } }
        }
        if (L.horns) { p.px(hx0 - 1, hy0 - 1, P.bone); p.px(hx0 - 2, hy0 - 2, P.bone); p.px(hx1 + 1, hy0 - 1, P.bone); p.px(hx1 + 2, hy0 - 2, P.bone); }
    };
    legs(); torso();
    if (side) { head(); arms(); } else if (up) { arms(); head(); } else { arms(); head(); }
    if (!weaponFirst) weapon();
    if (L.aura) for (let i = 0; i < 18; i++) { const x = Math.floor(h01(i, pose.seed || 0, 3) * W), y = Math.floor(h01(i, pose.seed || 0, 4) * (p.h - 4)); if (!p.a(x, y)) continue; p.glow(x, y, L.aura, 0.55); }
}

// poses per animation frame
function poseFor(anim, f, dir) {
    const side = dir === 'side', up = dir === 'up';
    const base = { wShow: true, wAng: side ? 1.9 : up ? 1.9 : 1.25 };
    switch (anim) {
        case 'idle': return { ...base, bob: f, seed: f };
        case 'walk': {
            const s = [0, 1, 2, 0, -1, -2][f], b = [0, -1, 0, 0, -1, 0][f];
            return { ...base, bob: b, legA: side ? s : (s > 0 ? 1 : 0), legB: side ? 0 : (s < 0 ? 1 : 0), armA: side ? -s : (s < 0 ? 1 : 0), armB: s > 0 ? 1 : 0, seed: f };
        }
        case 'atk': {
            const ang = side ? [-2.0, -0.1, 0.7, 1.4][f] : up ? [Math.PI + 1.2, Math.PI + 0.2, Math.PI - 0.8, 2.0][f] : [-1.1, 0.4, 2.1, 1.6][f];
            return { ...base, wAng: ang, bob: f === 1 ? 1 : 0, armA: side ? [-2, 3, 4, 2][f] : 0, lean: f === 0 ? -1 : f === 1 ? 1 : 0, wBehind: side && f === 0, seed: f };
        }
        case 'hurt': return { ...base, lean: -2, bob: 1, wAng: base.wAng + 0.6 };
        case 'cast': return { wShow: false, cast: f + 1, bob: -f, seed: f };
        case 'drink': return { wShow: false, drink: f + 1, lean: -f };
        case 'sit': return { wShow: false, sit: true };
        case 'shoot': return { wShow: false, shoot: true };
        default: return base;
    }
}

function rollFrame(p, dir, f, L) {
    const coat = ramp(L.skeleton ? P.bone : L.coatHex), skin = ramp(L.skeleton ? P.bone : L.skinHex), hair = ramp(L.hairHex || HAIR[0]);
    const cx = p.w / 2, cy = p.h - 9;
    p.blob(cx, cy, 7, 7, L.skeleton ? '#8F8672' : L.coatHex);
    const a = f / 4 * Math.PI * 2 + (dir === 'up' ? Math.PI : 0) - Math.PI / 2;
    const hx = cx + Math.cos(a) * 4, hy = cy + Math.sin(a) * 4;
    p.ellipse(hx, hy, 3.4, 3.4, skin[2]);
    if (!L.skeleton && L.hair) p.ellipse(hx - Math.cos(a), hy - Math.sin(a), 3, 3, hair[2]);
    // tucked boots
    const ba = a + Math.PI * 0.9;
    p.rect(Math.round(cx + Math.cos(ba) * 5) - 1, Math.round(cy + Math.sin(ba) * 5) - 1, 3, 3, '#22181A');
    for (let k = 0; k < 7; k++) p.px(cx - 6 + k * 2, cy + 6, coat[1]);
}

// rotate a standing frame onto the ground (dead); skeletons collapse into a bone pile
function deadFrame(p, L, S) {
    if (L.skeleton) {
        const cx = p.w / 2, cy = p.h - 4;
        for (let i = 0; i < 9; i++) { const x = cx - 9 + h01(i, 1) * 18, y = cy - 2 + h01(i, 2) * 4; p.hline(x, x + 3 + Math.floor(h01(i, 3) * 3), y, P.bone); p.px(x, y - 1, P.bone); }
        p.blob(cx + 3, cy - 3, 3.5 * S, 3 * S, P.bone, { dither: false }); p.px(cx + 2, cy - 3, P.abyss); p.px(cx + 4, cy - 3, P.abyss);
        if (L.rags) p.hline(cx - 6, cx + 1, cy + 1, ramp(L.coatHex)[1]);
        return;
    }
    const tmp = new Pix(p.w, p.h);
    paintHuman(tmp, 'down', { wShow: false }, L, S);
    for (let y = 0; y < tmp.h; y++) for (let x = 0; x < tmp.w; x++) {
        const c = tmp.get(x, y); if (!c[3]) continue;
        // rotate 90° clockwise around the frame, then drop to the floor
        const nx = tmp.h - 1 - y, ny = x;
        const fy = ny - Math.floor(tmp.w / 2) + p.h - Math.round(8 * S);
        p.px(nx, fy, shade(c, 0.8));
    }
    for (let k = 0; k < 6; k++) p.px(p.w / 2 - 6 + k * 3, p.h - 2, P.blood);
}

// build one character sheet into the atlas
export function buildHumanSheet(atlas, key, look, { S = 1, anims = FRAMES } = {}) {
    const L = resolveLook(look);
    const fw = Math.round(32 * S), fh = Math.round(32 * S) + (L.crown || L.hat === 4 ? Math.round(4 * S) : 0);
    const sheet = { w: fw, h: fh, ax: Math.floor(fw / 2), ay: fh - 1, dirs: {} };
    for (const dir of DIRS3) {
        sheet.dirs[dir] = {};
        for (const [anim, n] of Object.entries(anims)) {
            sheet.dirs[dir][anim] = [];
            for (let f = 0; f < n; f++) {
                const p = new Pix(fw, fh);
                if (anim === 'roll') rollFrame(p, dir, f, L);
                else if (anim === 'dead') deadFrame(p, L, S);
                else paintHuman(p, dir, poseFor(anim, f, dir), L, S);
                p.outline({ skipBottom: true });
                sheet.dirs[dir][anim].push(atlas.add(`${key}:${dir}:${anim}:${f}`, p));
            }
        }
    }
    return sheet;
}

// look (creator values or a preset) → painter parameters
export function resolveLook(look) {
    const L = { ...look };
    if (L.skinHex === undefined) L.skinHex = SKIN[look.skin ?? 1];
    if (L.hairHex === undefined) L.hairHex = HAIR[look.hairCol ?? 0];
    if (L.coatHex === undefined) L.coatHex = COAT[look.coat ?? 0].c;
    return L;
}

// presets for enemies, NPCs and kings
export const PRESETS = {
    skeleton: { skeleton: true, rags: true, coatHex: '#3A2E2A', hat: 2, bandHex: '#4A1A1E', weapon: 'cutlass' },
    skelBrute: { skeleton: true, rags: true, coatHex: '#2A2A30', hat: 1, hatHex: '#1A1A20', weapon: 'hammer', body: 1 },
    skelGunner: { skeleton: true, rags: true, coatHex: '#2E3E52', hat: 0, weapon: 'dagger' },
    drowned: { drowned: true, skinHex: '#5C7454', hairHex: '#1F4A4C', hair: 2, coatHex: '#2E3A36', pantsHex: '#24302C', weapon: 'trident' },
    drownedIce: { drowned: true, skinHex: '#8EA6B0', hairHex: '#C9D6D8', hair: 2, coatHex: '#3E4E5A', pantsHex: '#2E3A44', weapon: 'trident', eyeCol: '#B8FFE8' },
    ashSkeleton: { skeleton: true, rags: true, coatHex: '#4A2A22', hat: 2, bandHex: '#8A3A1E', weapon: 'cutlass', eyeCol: P.ember, aura: P.ember },
    stormSailor: { skinHex: '#8FA3A8', hairHex: '#24394A', hair: 4, coatHex: '#24394A', hat: 3, hoodGlow: true, eyeCol: '#B8FFE8', weapon: 'rapier', drowned: true },
    // NPCs
    lanternMaid: { skinHex: '#C9A07E', hairHex: '#9A9A92', hair: 2, coatHex: '#4A6170', hat: 3, robe: true, hold: 'lantern', lanternCol: P.soul, weapon: null },
    smith: { skinHex: '#8A6248', hairHex: '#2A1E1A', hair: 0, beard: 2, body: 1, coatHex: '#3A3430', apron: true, weapon: 'hammer' },
    innkeep: { skinHex: '#C9A07E', hairHex: '#7A2A1E', hair: 1, beard: 3, body: 1, coatHex: '#5A4230', hat: 2, bandHex: '#2E3E52', weapon: null },
    carver: { skinHex: '#B08A6E', hairHex: '#9A9A92', hair: 3, beard: 2, coatHex: '#3E4A34', hat: 0, weapon: null, patch: 1 },
    crewFire: { skinHex: '#8A6248', hairHex: '#2A1E1A', hair: 3, coatHex: '#5A2A30', hat: 2, bandHex: '#C2502E', weapon: 'cutlass' },
    crewIce: { skinHex: '#C9A07E', hairHex: '#C9D6D8', hair: 2, coatHex: '#2E3E52', hat: 0, weapon: 'rapier' },
    crewWind: { skinHex: '#5E4030', hairHex: '#2A1E1A', hair: 4, beard: 1, coatHex: '#4A6170', hat: 1, weapon: 'dagger' },
    crewEarth: { skinHex: '#B08A6E', hairHex: '#5A3A22', hair: 1, beard: 2, body: 1, coatHex: '#3E4A34', hat: 2, bandHex: '#34473A', weapon: 'club' },
    villagerA: { skinHex: '#C9A07E', hairHex: '#7A2A1E', hair: 3, coatHex: '#4A6A8A', robe: true, hat: 2, bandHex: '#E8DCC0', weapon: null },
    villagerB: { skinHex: '#B08A6E', hairHex: '#9A9A92', hair: 1, beard: 2, coatHex: '#6A5A3A', hat: 0, weapon: null },
    villagerC: { skinHex: '#8A6248', hairHex: '#2A1E1A', hair: 2, coatHex: '#8A3A4A', robe: true, hat: 0, weapon: null },
    villagerD: { skinHex: '#C9A07E', hairHex: '#5A3A22', hair: 4, beard: 1, coatHex: '#3E6A5A', hat: 1, hatHex: '#4A3A2A', weapon: null },
    // ghost kings (scale 1.5)
    kingFire: { skinHex: '#4A2A28', hairHex: '#C2502E', hair: 4, beard: 3, coatHex: '#5A1A1E', hat: 4, hatHex: '#2A1A1A', crown: true, crownCol: P.lava, body: 1, weapon: 'greatsword', drowned: true, eyeCol: P.lava, aura: P.ember, castCol: P.lava },
    kingIce: { skinHex: '#9FC3D0', hairHex: '#E2ECEE', hair: 2, coatHex: '#3E5A6A', robe: true, crown: true, crownCol: '#B8FFE8', weapon: 'rapier', drowned: true, eyeCol: '#B8FFE8', aura: '#9FC3D0', castCol: '#B8FFE8' },
    kingWind: { skinHex: '#6A7A80', hairHex: '#24394A', hair: 4, beard: 2, coatHex: '#24394A', hat: 1, hatHex: '#16232E', crown: true, crownCol: '#D8E4F0', weapon: 'cutlass', drowned: true, eyeCol: '#D8E4F0', aura: '#8FA3A8', castCol: '#D8E4F0' },
    kingEarth: { skinHex: '#5C7454', hairHex: '#1F4A4C', hair: 2, beard: 3, body: 1, coatHex: '#34473A', robe: true, crown: true, crownCol: '#9ACF6A', weapon: 'anchor', drowned: true, eyeCol: '#9ACF6A', aura: '#5C7454', castCol: '#9ACF6A', horns: true }
};
export { rgb };
