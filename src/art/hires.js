// High-resolution art painted natively at 4x pixel density with brush-like vector shapes and
// gradients: the player (every look from the character creator), skeleton pirates, the saber fire
// and the teal flames. Frames keep their logical size (20 × 26 for figures), so gameplay is unchanged.
import { Pix } from './painter.js';
import { resolveLook } from './character.js';

export const NATIVE = 4;
const S = NATIVE;
const FW = 20 * S, FH = 26 * S; // 80 × 104
const OUTLINE = '#120c12';
const BOOT = '#2c1d15';
const BONE = '#ddd3bd';

// ---------------------------------------------------------------------------- colour helpers
function rgb(hex) { const v = parseInt(hex.slice(1), 16); return [(v >> 16) & 255, (v >> 8) & 255, v & 255]; }
function tone(hex, f, a = 1) {
    const [r, g, b] = rgb(hex);
    const m = c => Math.max(0, Math.min(255, Math.round(f >= 1 ? c + (255 - c) * (f - 1) * 0.6 : c * f)));
    return `rgba(${m(r)},${m(g)},${m(b)},${a})`;
}
function lin(ctx, x0, y0, x1, y1, hex, hi = 1.3, lo = 0.62) {
    const g = ctx.createLinearGradient(x0, y0, x1, y1);
    g.addColorStop(0, tone(hex, hi)); g.addColorStop(0.45, tone(hex, 1)); g.addColorStop(1, tone(hex, lo));
    return g;
}
function rad(ctx, x, y, r, hex, hi = 1.35, lo = 0.6) {
    const g = ctx.createRadialGradient(x - r * 0.35, y - r * 0.4, r * 0.1, x, y, r * 1.15);
    g.addColorStop(0, tone(hex, hi)); g.addColorStop(0.5, tone(hex, 1)); g.addColorStop(1, tone(hex, lo));
    return g;
}

// draw a closed path with fill + dark outline
function shape(ctx, path, fill, { line = 2.4, stroke = OUTLINE } = {}) {
    ctx.beginPath(); path(ctx); ctx.closePath();
    if (line) { ctx.lineJoin = 'round'; ctx.lineWidth = line; ctx.strokeStyle = stroke; ctx.stroke(); }
    ctx.fillStyle = fill; ctx.fill();
}
const ell = (x, y, rx, ry, rot = 0) => c => c.ellipse(x, y, rx, ry, rot, 0, Math.PI * 2);
const rrect = (x, y, w, h, r) => c => c.roundRect(x, y, w, h, r);
function poly(...pts) { return c => { c.moveTo(pts[0], pts[1]); for (let i = 2; i < pts.length; i += 2) c.lineTo(pts[i], pts[i + 1]); }; }
function strokeLine(ctx, pts, color, w = 1.5) {
    ctx.beginPath(); ctx.moveTo(pts[0], pts[1]);
    for (let i = 2; i < pts.length; i += 2) ctx.lineTo(pts[i], pts[i + 1]);
    ctx.lineCap = 'round'; ctx.lineWidth = w; ctx.strokeStyle = color; ctx.stroke();
}

// limb as a rotated rounded bar from (x, y) with length and angle (0 = straight down)
function limb(ctx, x, y, len, wid, ang, fill, line = 2.2) {
    ctx.save(); ctx.translate(x, y); ctx.rotate(ang);
    shape(ctx, rrect(-wid / 2, 0, wid, len, wid / 2), fill, { line });
    ctx.restore();
}
function handAt(ctx, x, y, skin, r = 4.6) { shape(ctx, ell(x, y, r, r), rad(ctx, x, y, r, skin), { line: 2 }); }

// cutlass: hilt at (x, y), blade direction angle (0 = down), length
function cutlass(ctx, x, y, ang, len, rusty = false) {
    ctx.save(); ctx.translate(x, y); ctx.rotate(ang);
    const steel = rusty ? '#8a7a68' : '#c8d2d8';
    const g = ctx.createLinearGradient(-4, 0, 5, 0);
    g.addColorStop(0, tone(steel, 1.35)); g.addColorStop(0.5, steel); g.addColorStop(1, tone(steel, 0.55));
    shape(ctx, c => { c.moveTo(-2.5, 4); c.quadraticCurveTo(-4, len * 0.6, 1, len); c.quadraticCurveTo(5, len * 0.55, 3, 4); }, g, { line: 1.8 });
    strokeLine(ctx, [0, 6, -0.5, len * 0.8], tone(steel, 1.5, 0.6), 0.8);
    if (rusty) for (let k = 0; k < 4; k++) { ctx.fillStyle = 'rgba(120,60,30,0.7)'; ctx.fillRect(-2 + k % 2, 10 + k * 6, 2, 2); }
    // guard and grip
    shape(ctx, c => { c.ellipse(0, 3, 6.5, 2.4, 0, 0, Math.PI * 2); }, lin(ctx, -6, 0, 6, 4, '#b8913e'), { line: 1.6 });
    shape(ctx, rrect(-1.8, -7, 3.6, 9, 1.5), lin(ctx, -2, -7, 2, 2, '#3a2418'), { line: 1.4 });
    shape(ctx, ell(0, -8, 2.4, 2.4), rad(ctx, 0, -8, 2.4, '#c9a24a'), { line: 1.2 });
    ctx.restore();
}

// ---------------------------------------------------------------------------- figure
function drawFigure(ctx, emi, dir, pose, a) {
    const SK = a.skeleton;
    const skin = SK ? BONE : a.skinHex;
    const hair = a.hairHex, coat = SK ? '#3a3340' : a.coatHex, acc = SK ? '#2a2830' : a.accHex, scarf = a.scarfHex;
    const wide = a.body === 1 ? 3 : 0;
    const bob = (pose.bob || 0) * S;
    const leg = pose.leg || 0, arm = pose.arm || 0, atk = pose.atk || 0;
    if (dir === 'side') return drawSide(ctx, emi, pose, a, { SK, skin, hair, coat, acc, scarf, bob, leg, arm, atk });
    const up = dir === 'up';
    const pants = SK ? BONE : tone(acc, 0.75);
    // --- legs
    const legY = 74;
    const lLift = leg === 1 ? -5 : 0, rLift = leg === -1 ? -5 : 0;
    if (SK) {
        limb(ctx, 34, legY, 20 + lLift, 4.5, 0.05, BONE); limb(ctx, 46, legY, 20 + rLift, 4.5, -0.05, BONE);
    } else {
        shape(ctx, rrect(29, legY, 10, 19 + lLift, 4), lin(ctx, 29, legY, 39, 96, pants === BONE ? BONE : acc, 1.1, 0.5));
        shape(ctx, rrect(41, legY, 10, 19 + rLift, 4), lin(ctx, 41, legY, 51, 96, acc, 1.1, 0.5));
    }
    // boots
    shape(ctx, rrect(27.5, 87 + lLift, 13, 9.5, 3.5), lin(ctx, 27, 87, 40, 97, BOOT, 1.5, 0.6));
    shape(ctx, rrect(39.5, 87 + rLift, 13, 9.5, 3.5), lin(ctx, 39, 87, 52, 97, BOOT, 1.5, 0.6));
    strokeLine(ctx, [29, 89.5 + lLift, 39, 89.5 + lLift], tone(BOOT, 1.6), 1.2);
    strokeLine(ctx, [41, 89.5 + rLift, 51, 89.5 + rLift], tone(BOOT, 1.6), 1.2);
    // --- coat tails (behind)
    ctx.save(); ctx.translate(0, bob);
    if (SK) {
        shape(ctx, poly(25, 66, 55, 66, 58, 84, 53, 80, 49, 86, 45, 79, 40, 85, 35, 79, 30, 86, 26, 80, 22, 84), lin(ctx, 22, 66, 58, 86, coat, 1.1, 0.5));
    } else {
        shape(ctx, poly(24 - wide, 64, 56 + wide, 64, 60 + wide, 85, 50, 82, 40, 84, 30, 82, 20 - wide, 85), lin(ctx, 20, 64, 60, 85, coat, 1.05, 0.5));
    }
    // --- arms behind torso? (down view: arms at the sides, drawn after torso)
    // --- torso
    const tx0 = 24 - wide, tx1 = 56 + wide;
    shape(ctx, c => { c.moveTo(tx0, 47); c.quadraticCurveTo(40, 43, tx1, 47); c.lineTo(tx1 - 3, 75); c.quadraticCurveTo(40, 78, tx0 + 3, 75); }, lin(ctx, tx0, 44, tx1, 78, coat, 1.25, 0.55));
    if (SK) {
        // ribcage through the torn coat
        ctx.fillStyle = 'rgba(10,8,12,0.85)'; ctx.beginPath(); ctx.ellipse(40, 59, 10, 12, 0, 0, Math.PI * 2); ctx.fill();
        for (let k = 0; k < 4; k++) {
            const y = 51 + k * 5;
            ctx.beginPath(); ctx.moveTo(40, y); ctx.quadraticCurveTo(31, y + 1, 31.5, y + 4); ctx.moveTo(40, y); ctx.quadraticCurveTo(49, y + 1, 48.5, y + 4);
            ctx.lineWidth = 2.4; ctx.strokeStyle = BONE; ctx.lineCap = 'round'; ctx.stroke();
        }
        strokeLine(ctx, [40, 49, 40, 70], BONE, 2.6);
    } else if (!up) {
        // shirt V, lapels, buttons
        shape(ctx, poly(35, 46, 45, 46, 40, 58), lin(ctx, 35, 46, 45, 58, '#e6dcc8', 1.1, 0.75), { line: 1.4 });
        shape(ctx, poly(33, 46, 40, 60, 36, 66, 30, 49), lin(ctx, 30, 46, 40, 66, coat, 1.45, 0.8), { line: 1.4 });
        shape(ctx, poly(47, 46, 40, 60, 44, 66, 50, 49), lin(ctx, 40, 46, 50, 66, coat, 1.2, 0.7), { line: 1.4 });
        for (const y of [62, 67]) { shape(ctx, ell(43.5, y, 1.5, 1.5), rad(ctx, 43.5, y, 1.5, '#d8b252'), { line: 0.8 }); }
    } else {
        strokeLine(ctx, [40, 47, 40, 74], tone(coat, 0.6), 1.4); // back seam
    }
    if (!SK) {
        shape(ctx, rrect(tx0 + 1.5, 69, tx1 - tx0 - 3, 5.5, 1.5), lin(ctx, 0, 69, 0, 75, '#4a2c1a', 1.3, 0.6), { line: 1.4 });
        if (!up) shape(ctx, rrect(37, 68.5, 6, 6.5, 1.2), lin(ctx, 37, 68, 43, 75, '#d4ad4e', 1.4, 0.6), { line: 1.2 });
    }
    // --- arms
    const aL = arm === 1 ? -0.18 : arm === -1 ? 0.18 : 0;
    const sleeve = SK ? BONE : lin(ctx, 0, 48, 0, 72, coat, 1.15, 0.55);
    const raiseR = !up && atk === 1, strikeR = !up && atk === 2, drinkR = pose.drink;
    // left arm
    if (SK) limb(ctx, tx0 - 1, 48, 22, 4.2, 0.12 + aL, BONE);
    else { limb(ctx, tx0 - 1, 48, 22, 9, 0.12 + aL, sleeve); }
    handAt(ctx, tx0 - 3.5 - aL * 20, 72, skin);
    // right arm (sword arm)
    let hx, hy;
    if (raiseR) { limb(ctx, tx1 + 1, 50, 20, SK ? 4.2 : 9, Math.PI - 0.35, SK ? BONE : sleeve); hx = tx1 + 8; hy = 32; }
    else if (strikeR) { limb(ctx, tx1, 49, 24, SK ? 4.2 : 9, 0.35, SK ? BONE : sleeve); hx = tx1 - 7; hy = 72; }
    else if (drinkR) { limb(ctx, tx1, 49, 17, 9, 2.1, sleeve); hx = 47; hy = 41; }
    else { limb(ctx, tx1 + 1, 48, 22, SK ? 4.2 : 9, -0.12 - aL, SK ? BONE : sleeve); hx = tx1 + 3.5 - aL * 20; hy = 72; }
    if (!SK && !drinkR) { // cuffs
        ctx.save(); ctx.fillStyle = tone(acc, 1.2); ctx.globalAlpha = 0.9;
        ctx.fillRect(tx0 - 8 - aL * 20, 65, 9, 3); if (!raiseR && !strikeR) ctx.fillRect(tx1 - 1 - aL * 20, 65, 9, 3); ctx.restore();
    }
    // sword
    if (!up) {
        if (raiseR) cutlass(ctx, hx, hy, Math.PI + 0.25, 28, SK);
        else if (strikeR) cutlass(ctx, hx, hy, 0.1, 30, SK);
        else if (!drinkR) cutlass(ctx, hx, hy, -0.45, 24, SK);
    } else cutlass(ctx, hx, hy, -0.25, 22, SK);
    if (!drinkR) handAt(ctx, hx, hy, skin);
    // --- scarf
    if (!SK) {
        shape(ctx, c => { c.moveTo(28, 43); c.quadraticCurveTo(40, 50, 52, 43); c.lineTo(52, 48); c.quadraticCurveTo(40, 55, 28, 48); }, lin(ctx, 28, 43, 52, 55, scarf, 1.3, 0.6), { line: 1.8 });
        if (!up) {
            shape(ctx, ell(50, 49, 3.6, 3), rad(ctx, 50, 49, 3.6, scarf), { line: 1.4 });
            shape(ctx, poly(49, 51, 53, 51, 55, 62, 51, 60), lin(ctx, 49, 51, 55, 62, scarf, 1.1, 0.55), { line: 1.4 });
        } else {
            shape(ctx, poly(38, 50, 42, 50, 44, 60, 40, 58, 36, 61), lin(ctx, 36, 50, 44, 61, scarf, 1.1, 0.55), { line: 1.4 });
        }
    }
    // --- head
    drawHeadFront(ctx, emi, a, up, bob, { SK, skin, hair });
    ctx.restore();
    if (drinkR) { // bottle at the mouth
        ctx.save(); ctx.translate(46, 37 + bob); ctx.rotate(-0.9);
        const g = ctx.createLinearGradient(-4, 0, 4, 0); g.addColorStop(0, 'rgba(120,200,110,0.95)'); g.addColorStop(1, 'rgba(30,70,30,0.95)');
        shape(ctx, rrect(-4, -2, 8, 14, 3), g, { line: 1.6 });
        shape(ctx, rrect(-2, -7, 4, 6, 1), '#c9a050', { line: 1.2 });
        ctx.restore();
        emi.fillStyle = 'rgba(80,140,50,0.7)'; emi.beginPath(); emi.arc(46, 40 + bob, 3, 0, 7); emi.fill();
        handAt(ctx, 49, 42 + bob, skin);
    }
}

function drawHeadFront(ctx, emi, a, up, bob, { SK, skin, hair }) {
    const cx = 40, cy = 30;
    // ears
    if (!SK) { shape(ctx, ell(24.5, 32, 3, 4), rad(ctx, 24.5, 32, 3, skin), { line: 1.6 }); shape(ctx, ell(55.5, 32, 3, 4), rad(ctx, 55.5, 32, 3, skin), { line: 1.6 }); }
    // long hair / dreads behind head
    if (!SK && (a.hair === 1 || a.hair === 5)) {
        if (a.hair === 1) shape(ctx, c => { c.moveTo(23, 22); c.quadraticCurveTo(18, 48, 26, 62); c.lineTo(54, 62); c.quadraticCurveTo(62, 48, 57, 22); }, lin(ctx, 20, 20, 60, 62, hair, 1.2, 0.5));
        else for (let k = 0; k < 8; k++) { const x = 22 + k * 5.2; limb(ctx, x, 22, 34 + (k % 3) * 3, 4.6, (x - 40) * -0.004, lin(ctx, x - 2, 22, x + 2, 60, hair, 1.25, 0.55), 1.4); }
    }
    // head
    shape(ctx, ell(cx, cy, 15.5, 16.5), rad(ctx, cx, cy, 16.5, skin, 1.3, 0.62));
    if (SK) {
        // skull: sockets, nose, teeth, cracks, glowing eyes
        if (!up) {
            shape(ctx, ell(34, 31, 4.6, 5), '#1a1014', { line: 1 }); shape(ctx, ell(46, 31, 4.6, 5), '#1a1014', { line: 1 });
            for (const x of [34, 46]) {
                const g = ctx.createRadialGradient(x, 31.5, 0, x, 31.5, 3); g.addColorStop(0, '#ffd0b0'); g.addColorStop(0.4, '#ff3020'); g.addColorStop(1, 'rgba(120,0,0,0)');
                ctx.fillStyle = g; ctx.beginPath(); ctx.arc(x, 31.5, 3, 0, 7); ctx.fill();
                emi.fillStyle = g; emi.beginPath(); emi.arc(x, 31.5 + bob, 3.5, 0, 7); emi.fill();
            }
            shape(ctx, poly(40, 35, 37.5, 40, 42.5, 40), '#1a1014', { line: 0.8 });
            shape(ctx, rrect(32, 42, 16, 6, 2), tone(BONE, 0.85), { line: 1.4 });
            for (let k = 0; k < 5; k++) strokeLine(ctx, [34.5 + k * 2.8, 42.5, 34.5 + k * 2.8, 47.5], '#2a1e1c', 0.9);
            strokeLine(ctx, [44, 16, 47, 21, 45, 24], tone(BONE, 0.55), 1);
        } else strokeLine(ctx, [36, 18, 40, 24, 38, 30], tone(BONE, 0.55), 1);
    } else if (!up) {
        // eyes
        for (const x of [34, 46]) {
            shape(ctx, ell(x, 32, 3.2, 2.4), '#f2ece0', { line: 1.1 });
            shape(ctx, ell(x + 0.3, 32.3, 1.7, 1.9), '#2a2018', { line: 0 });
            ctx.fillStyle = '#ffffff'; ctx.fillRect(x - 0.6, 31, 1.2, 1.2);
        }
        strokeLine(ctx, [30.5, 27.5, 37, 27], tone(a.hair === 4 ? skin : hair, 0.8), 1.7);
        strokeLine(ctx, [43, 27, 49.5, 27.5], tone(a.hair === 4 ? skin : hair, 0.8), 1.7);
        // nose, mouth, cheeks
        ctx.fillStyle = tone(skin, 0.7, 0.8); ctx.beginPath(); ctx.ellipse(40.5, 37.5, 2.2, 1.5, 0, 0, 7); ctx.fill();
        strokeLine(ctx, [36.5, 41.5, 40, 42.3, 43.5, 41.5], tone(skin, 0.45), 1.3);
        ctx.fillStyle = 'rgba(200,80,70,0.18)'; ctx.beginPath(); ctx.arc(31, 37, 3, 0, 7); ctx.arc(49, 37, 3, 0, 7); ctx.fill();
        if (a.beard === 1) { ctx.fillStyle = tone(hair, 0.9, 0.45); for (let k = 0; k < 40; k++) { const t = k / 40; const x = 29 + t * 22, y = 39 + Math.sin(t * Math.PI) * 6 + (k % 3); ctx.fillRect(x, y, 1, 1); } }
        if (a.beard === 2) shape(ctx, c => { c.moveTo(25, 32); c.quadraticCurveTo(26, 52, 40, 54); c.quadraticCurveTo(54, 52, 55, 32); c.quadraticCurveTo(52, 40, 44, 40.5); c.quadraticCurveTo(40, 44, 36, 40.5); c.quadraticCurveTo(28, 40, 25, 32); }, lin(ctx, 25, 32, 55, 54, hair, 1.25, 0.55), { line: 1.8 });
        if (a.beard === 3) shape(ctx, c => { c.moveTo(36, 44); c.quadraticCurveTo(40, 54, 44, 44); }, lin(ctx, 36, 44, 44, 54, hair, 1.2, 0.6), { line: 1.4 });
        if (a.patch) {
            strokeLine(ctx, [25, 24, 55, 36], '#1c1614', 1.6);
            shape(ctx, ell(34, 32, 4.4, 3.6), lin(ctx, 30, 28, 38, 36, '#2a2220', 1.4, 0.6), { line: 1.4 });
        }
        if (a.scar) strokeLine(ctx, [47, 34, 49.5, 41], '#a4483e', 1.4);
    }
    // hair on top
    if (!SK) {
        const hg = lin(ctx, 24, 12, 56, 30, hair, 1.35, 0.55);
        if (a.hair === 4) {
            ctx.fillStyle = 'rgba(255,255,255,0.25)'; ctx.beginPath(); ctx.ellipse(35, 20, 5, 3, -0.4, 0, 7); ctx.fill();
        } else if (up) {
            shape(ctx, ell(cx, cy - 1, 16.3, 16.8), lin(ctx, 24, 12, 56, 48, hair, 1.3, 0.5));
            for (let k = 0; k < 7; k++) strokeLine(ctx, [28 + k * 4, 16, 27 + k * 4.4, 42], tone(hair, 0.65), 0.9);
            if (a.hair === 2) { for (let k = 0; k < 5; k++) shape(ctx, ell(40, 47 + k * 4.6, 3.6 - k * 0.3, 3), rad(ctx, 40, 47 + k * 4.6, 3.6, hair), { line: 1.3 }); }
            if (a.hair === 3) for (let k = 0; k < 9; k++) shape(ctx, ell(27 + (k % 5) * 6.5, 18 + Math.floor(k / 5) * 12, 5, 5), rad(ctx, 27 + (k % 5) * 6.5, 18 + Math.floor(k / 5) * 12, 5, hair), { line: 1.2 });
        } else if (a.hair === 3) {
            for (let k = 0; k < 9; k++) { const ang = Math.PI + k / 8 * Math.PI; const x = cx + Math.cos(ang) * 14, y = 26 + Math.sin(ang) * 13; shape(ctx, ell(x, y, 5.2, 5.2), rad(ctx, x, y, 5.2, hair), { line: 1.3 }); }
        } else {
            // fringe with a jagged lower edge
            shape(ctx, c => {
                c.moveTo(24, 30); c.quadraticCurveTo(22, 11, 40, 11.5); c.quadraticCurveTo(58, 11, 56, 30);
                c.lineTo(53, 24); c.lineTo(50, 27); c.lineTo(47, 21.5); c.lineTo(43, 25); c.lineTo(40, 20.5); c.lineTo(36, 25); c.lineTo(32, 21); c.lineTo(29, 26); c.lineTo(27, 23);
            }, hg);
            for (let k = 0; k < 6; k++) strokeLine(ctx, [29 + k * 4.4, 13.5, 28 + k * 4.6, 21], tone(hair, 1.45), 0.8);
        }
    }
    // hats
    const hat = SK ? 2 : a.hat;
    if (hat === 1) {
        shape(ctx, c => { c.moveTo(24, 26); c.quadraticCurveTo(23, 10, 40, 10); c.quadraticCurveTo(57, 10, 56, 26); c.quadraticCurveTo(40, 20, 24, 26); }, lin(ctx, 24, 10, 56, 26, a.accHex, 1.35, 0.6));
        for (let k = 0; k < 3; k++) strokeLine(ctx, [30 + k * 8, 13, 28 + k * 9, 22], tone(a.accHex, 0.7), 0.9);
        if (!up) { shape(ctx, ell(56, 24, 3.4, 3), rad(ctx, 56, 24, 3.4, a.accHex)); shape(ctx, poly(56, 26, 60, 27, 63, 37, 58, 35), lin(ctx, 56, 26, 63, 37, a.accHex, 1.1, 0.5), { line: 1.4 }); }
        else { shape(ctx, poly(38, 24, 42, 24, 45, 36, 40, 33, 35, 36), lin(ctx, 35, 24, 45, 36, a.accHex, 1.1, 0.5), { line: 1.4 }); }
    } else if (hat === 2) {
        const hc = SK ? '#2a2226' : '#24201f';
        shape(ctx, c => { c.moveTo(12, 20); c.quadraticCurveTo(40, 2, 68, 20); c.quadraticCurveTo(58, 26, 40, 27); c.quadraticCurveTo(22, 26, 12, 20); }, lin(ctx, 12, 6, 68, 27, hc, 1.5, 0.55));
        shape(ctx, ell(40, 15, 13, 7), lin(ctx, 27, 8, 53, 22, hc, 1.7, 0.7), { line: 1.8 });
        if (!SK) strokeLine(ctx, [13, 20, 24, 24.5, 40, 26, 56, 24.5, 67, 20], '#c9a24a', 1.2);
        else strokeLine(ctx, [20, 17, 26, 20], 'rgba(0,0,0,0.6)', 1.6);
    } else if (hat === 3) {
        shape(ctx, c => { c.moveTo(24, 22); c.quadraticCurveTo(24, 8, 40, 8); c.quadraticCurveTo(56, 8, 56, 22); }, lin(ctx, 24, 8, 56, 22, '#1e2440', 1.6, 0.6));
        shape(ctx, c => { c.moveTo(21, 22); c.quadraticCurveTo(40, 17, 59, 22); c.quadraticCurveTo(40, 30, 21, 22); }, lin(ctx, 21, 18, 59, 28, '#14182a', 1.5, 0.6), { line: 1.8 });
        if (!up) shape(ctx, ell(40, 15, 3, 2.6), rad(ctx, 40, 15, 3, '#d4ad4e'), { line: 1 });
    }
}

function drawSide(ctx, emi, pose, a, { SK, skin, hair, coat, acc, scarf, bob, leg, arm, atk }) {
    const pants = SK ? BONE : acc;
    // back leg, front leg (stride)
    const st = leg * 0.38;
    const legFill = (x0) => SK ? BONE : lin(ctx, x0 - 5, 74, x0 + 5, 96, pants, 1.1, 0.5);
    limb(ctx, 39, 72, 20, SK ? 4.5 : 9.5, st, SK ? tone(BONE, 0.8) : lin(ctx, 34, 72, 44, 96, pants, 0.95, 0.45));
    const bx = 39 - Math.sin(st) * 20, by = 72 + Math.cos(st) * 20;
    shape(ctx, rrect(bx - 4, by - 4, 13, 8, 3.5), lin(ctx, bx - 4, by - 4, bx + 9, by + 4, BOOT, 1.2, 0.5));
    limb(ctx, 42, 72, 20, SK ? 4.5 : 9.5, -st, legFill(42));
    const fx = 42 + Math.sin(st) * 20, fy = 72 + Math.cos(st) * 20;
    shape(ctx, rrect(fx - 4, fy - 4, 13, 8.5, 3.5), lin(ctx, fx - 4, fy - 4, fx + 9, fy + 4, BOOT, 1.5, 0.6));
    ctx.save(); ctx.translate(0, bob);
    // coat tail behind
    shape(ctx, poly(30, 62, 44, 64, 40, 84, 26, 86, 22, 80), lin(ctx, 22, 62, 44, 86, coat, 0.95, 0.45));
    // back arm (sword arm when attacking is the front one)
    // torso
    shape(ctx, c => { c.moveTo(30, 47); c.quadraticCurveTo(41, 42, 51, 48); c.lineTo(50, 75); c.quadraticCurveTo(40, 78, 31, 75); }, lin(ctx, 30, 44, 51, 78, coat, 1.25, 0.55));
    if (SK) { for (let k = 0; k < 4; k++) strokeLine(ctx, [36, 51 + k * 5, 47, 53 + k * 5], BONE, 2.2); }
    else {
        shape(ctx, rrect(31, 69, 19, 5.5, 1.5), lin(ctx, 0, 69, 0, 75, '#4a2c1a', 1.3, 0.6), { line: 1.4 });
        shape(ctx, rrect(45, 68.5, 5, 6.5, 1.2), lin(ctx, 45, 68, 50, 75, '#d4ad4e', 1.4, 0.6), { line: 1.2 });
        shape(ctx, poly(46, 46, 52, 48, 48, 60), lin(ctx, 46, 46, 52, 60, '#e6dcc8', 1.1, 0.75), { line: 1.2 });
    }
    // scarf with tail flowing back
    if (!SK) {
        shape(ctx, c => { c.moveTo(33, 43); c.quadraticCurveTo(43, 46, 50, 43); c.lineTo(50, 48); c.quadraticCurveTo(42, 51, 33, 48); }, lin(ctx, 33, 43, 50, 51, scarf, 1.3, 0.6), { line: 1.6 });
        shape(ctx, poly(34, 45, 26, 50, 20, 57, 25, 58, 31, 51), lin(ctx, 20, 45, 34, 58, scarf, 1.1, 0.55), { line: 1.4 });
    }
    // head (profile, facing right)
    if (!SK && (a.hair === 1 || a.hair === 5)) shape(ctx, c => { c.moveTo(30, 18); c.quadraticCurveTo(24, 40, 30, 58); c.lineTo(38, 56); c.lineTo(40, 24); }, lin(ctx, 24, 18, 40, 58, hair, 1.2, 0.5));
    if (!SK && a.hair === 2) for (let k = 0; k < 5; k++) shape(ctx, ell(28 - k * 0.6, 34 + k * 4.4, 3.4 - k * 0.3, 2.9), rad(ctx, 28, 34 + k * 4.4, 3.4, hair), { line: 1.2 });
    shape(ctx, c => { c.ellipse(41, 30, 13.5, 16.5, 0, 0, Math.PI * 2); }, rad(ctx, 41, 30, 16.5, skin, 1.3, 0.62));
    shape(ctx, c => { c.moveTo(53, 29); c.quadraticCurveTo(58.5, 33, 54, 36.5); c.lineTo(52, 35); }, rad(ctx, 55, 33, 4, skin), { line: 1.6 }); // nose
    if (SK) {
        shape(ctx, ell(48, 30, 4, 4.6), '#1a1014', { line: 1 });
        const g = ctx.createRadialGradient(48.5, 30.5, 0, 48.5, 30.5, 3); g.addColorStop(0, '#ffd0b0'); g.addColorStop(0.4, '#ff3020'); g.addColorStop(1, 'rgba(120,0,0,0)');
        ctx.fillStyle = g; ctx.beginPath(); ctx.arc(48.5, 30.5, 3, 0, 7); ctx.fill();
        emi.fillStyle = g; emi.beginPath(); emi.arc(48.5, 30.5 + bob, 3.5, 0, 7); emi.fill();
        shape(ctx, rrect(44, 41, 12, 5.5, 2), tone(BONE, 0.85), { line: 1.3 });
        for (let k = 0; k < 4; k++) strokeLine(ctx, [46 + k * 2.8, 41.5, 46 + k * 2.8, 46], '#2a1e1c', 0.9);
    } else {
        shape(ctx, ell(48.5, 31, 2.6, 2.3), '#f2ece0', { line: 1 });
        shape(ctx, ell(49.6, 31.3, 1.4, 1.7), '#2a2018', { line: 0 });
        strokeLine(ctx, [45, 26.5, 51, 26.5], tone(a.hair === 4 ? skin : hair, 0.8), 1.6);
        strokeLine(ctx, [49, 41.5, 53, 41], tone(skin, 0.45), 1.3);
        shape(ctx, ell(35, 32, 3, 4), rad(ctx, 35, 32, 3, skin), { line: 1.5 }); // ear
        if (a.beard === 2) shape(ctx, c => { c.moveTo(36, 34); c.quadraticCurveTo(38, 52, 52, 49); c.quadraticCurveTo(56, 44, 54, 39); c.quadraticCurveTo(48, 41, 44, 37); }, lin(ctx, 36, 34, 56, 52, hair, 1.25, 0.55), { line: 1.6 });
        if (a.beard === 3) shape(ctx, c => { c.moveTo(48, 43); c.quadraticCurveTo(52, 52, 55, 42); }, lin(ctx, 48, 43, 55, 52, hair, 1.2, 0.6), { line: 1.3 });
        if (a.beard === 1) { ctx.fillStyle = tone(hair, 0.9, 0.45); for (let k = 0; k < 24; k++) ctx.fillRect(40 + (k * 7) % 14, 38 + (k * 3) % 8, 1, 1); }
        if (a.patch) { strokeLine(ctx, [30, 24, 54, 28], '#1c1614', 1.6); shape(ctx, ell(48.5, 31, 3.8, 3.2), '#2a2220', { line: 1.3 }); }
        if (a.scar) strokeLine(ctx, [43, 33, 45, 40], '#a4483e', 1.4);
        // hair cap from the forehead over the back of the head
        if (a.hair === 3) for (let k = 0; k < 7; k++) { const ang = Math.PI * 0.9 + k / 6 * Math.PI * 0.75; const x = 39 + Math.cos(ang) * 13, y = 27 + Math.sin(ang) * 13; shape(ctx, ell(x, y, 5, 5), rad(ctx, x, y, 5, hair), { line: 1.2 }); }
        else if (a.hair !== 4) shape(ctx, c => { c.moveTo(52, 20); c.quadraticCurveTo(46, 9, 34, 13); c.quadraticCurveTo(25, 18, 28, 36); c.quadraticCurveTo(31, 40, 35, 38); c.quadraticCurveTo(33, 28, 39, 24); c.lineTo(44, 25); c.lineTo(47, 21); }, lin(ctx, 26, 10, 52, 40, hair, 1.35, 0.55));
        else { ctx.fillStyle = 'rgba(255,255,255,0.25)'; ctx.beginPath(); ctx.ellipse(40, 19, 5, 3, -0.3, 0, 7); ctx.fill(); }
    }
    const hat = SK ? 2 : a.hat;
    if (hat === 1) {
        shape(ctx, c => { c.moveTo(28, 26); c.quadraticCurveTo(28, 10, 42, 10); c.quadraticCurveTo(55, 11, 54, 22); c.quadraticCurveTo(42, 18, 28, 26); }, lin(ctx, 28, 10, 54, 26, a.accHex, 1.35, 0.6));
        shape(ctx, poly(29, 22, 22, 26, 17, 36, 22, 35, 28, 27), lin(ctx, 17, 22, 29, 36, a.accHex, 1.1, 0.5), { line: 1.4 });
    } else if (hat === 2) {
        const hc = SK ? '#2a2226' : '#24201f';
        shape(ctx, c => { c.moveTo(18, 22); c.quadraticCurveTo(26, 15, 34, 16); c.quadraticCurveTo(44, 6, 54, 16); c.quadraticCurveTo(62, 15, 66, 21); c.quadraticCurveTo(42, 26, 18, 22); }, lin(ctx, 18, 6, 66, 26, hc, 1.5, 0.55));
        if (!SK) strokeLine(ctx, [19, 22, 42, 25, 65, 21], '#c9a24a', 1.1);
    } else if (hat === 3) {
        shape(ctx, c => { c.moveTo(28, 22); c.quadraticCurveTo(28, 9, 42, 9); c.quadraticCurveTo(54, 9, 54, 20); }, lin(ctx, 28, 9, 54, 22, '#1e2440', 1.6, 0.6));
        shape(ctx, c => { c.moveTo(27, 22); c.lineTo(62, 20); c.quadraticCurveTo(62, 25, 52, 25); c.lineTo(27, 25); }, lin(ctx, 27, 20, 62, 25, '#14182a', 1.5, 0.6), { line: 1.6 });
    }
    // front arm + sword
    let hx, hy;
    const sleeve = SK ? BONE : lin(ctx, 36, 48, 48, 72, coat, 1.2, 0.55);
    if (atk === 1) { limb(ctx, 40, 49, 18, SK ? 4.2 : 9, Math.PI - 0.9, sleeve); hx = 33; hy = 36; cutlass(ctx, hx, hy, Math.PI + 0.75, 28, SK); }
    else if (atk === 2) { limb(ctx, 40, 50, 20, SK ? 4.2 : 9, -1.45, sleeve); hx = 60; hy = 54; cutlass(ctx, hx, hy, -Math.PI / 2 - 0.05, 26, SK); }
    else { const sw = (arm || 0) * 0.3; limb(ctx, 41, 49, 21, SK ? 4.2 : 9, -0.1 - sw, sleeve); hx = 43 + Math.sin(0.1 + sw) * 21; hy = 49 + Math.cos(0.1 + sw) * 21; cutlass(ctx, hx, hy, -0.7, 24, SK); }
    handAt(ctx, hx, hy, skin);
    ctx.restore();
}

function drawRoll(ctx, a, k) {
    const coat = a.skeleton ? '#3a3340' : a.coatHex;
    const cx = 40, cy = 72, ang = k * Math.PI / 2;
    shape(ctx, ell(cx, cy, 20, 18), rad(ctx, cx, cy, 20, coat, 1.3, 0.5));
    const hx = cx + Math.cos(ang) * 12, hy = cy + Math.sin(ang) * 11;
    shape(ctx, ell(hx, hy, 10, 10), rad(ctx, hx, hy, 10, a.hair === 4 ? a.skinHex : a.hairHex, 1.3, 0.55));
    shape(ctx, ell(cx - Math.cos(ang) * 14, cy - Math.sin(ang) * 12, 6, 5), rad(ctx, cx, cy, 6, BOOT, 1.5, 0.6));
    if (!a.skeleton) shape(ctx, ell(cx + Math.sin(ang) * 15, cy - Math.cos(ang) * 13, 5, 4), rad(ctx, cx, cy, 5, a.scarfHex), { line: 1.4 });
    // motion arc
    ctx.beginPath(); ctx.arc(cx, cy, 24, ang + 2.4, ang + 3.6); ctx.lineWidth = 2; ctx.strokeStyle = 'rgba(230,225,210,0.35)'; ctx.stroke();
}

function drawPile(ctx, emi) {
    const bones = [[20, 92, 44, 86], [30, 96, 58, 92], [46, 86, 64, 96], [24, 84, 36, 96], [52, 98, 68, 90]];
    for (const [x0, y0, x1, y1] of bones) {
        strokeLine(ctx, [x0, y0, x1, y1], OUTLINE, 5.5); strokeLine(ctx, [x0, y0, x1, y1], BONE, 3.2);
        for (const [x, y] of [[x0, y0], [x1, y1]]) shape(ctx, ell(x, y, 2.6, 2.6), rad(ctx, x, y, 2.6, BONE), { line: 1.2 });
    }
    shape(ctx, ell(40, 82, 11, 9.5), rad(ctx, 40, 82, 11, BONE, 1.3, 0.6));
    for (const x of [36, 44]) {
        shape(ctx, ell(x, 82, 2.8, 3), '#1a1014', { line: 0.8 });
        ctx.fillStyle = 'rgba(255,60,40,0.65)'; ctx.beginPath(); ctx.arc(x, 82.5, 1.3, 0, 7); ctx.fill();
        emi.fillStyle = 'rgba(160,30,20,0.9)'; emi.beginPath(); emi.arc(x, 82.5, 2, 0, 7); emi.fill();
    }
    shape(ctx, rrect(35, 87, 10, 3.5, 1), tone(BONE, 0.85), { line: 1 });
}

// ---------------------------------------------------------------------------- sheet assembly
function newCanvas(w, h) { const c = document.createElement('canvas'); c.width = w; c.height = h; return c; }

// canvas pair -> Pix with crisp alpha (alpha-tested in the game) and auto heights for normal maps
export function canvasToPix(col, emi, { dome = 1, detail = 0.25, R = 18 } = {}) {
    const w = col.width, h = col.height;
    const p = new Pix(w, h);
    const cd = col.getContext('2d').getImageData(0, 0, w, h).data;
    const ed = emi ? emi.getContext('2d').getImageData(0, 0, w, h).data : null;
    for (let i = 0; i < w * h; i++) {
        const a = cd[i * 4 + 3];
        if (a < 110) continue;
        const k = a / 255; // un-premultiply soft edges against black
        p.col[i * 4] = Math.min(255, cd[i * 4] / Math.max(k, 0.5));
        p.col[i * 4 + 1] = Math.min(255, cd[i * 4 + 1] / Math.max(k, 0.5));
        p.col[i * 4 + 2] = Math.min(255, cd[i * 4 + 2] / Math.max(k, 0.5));
        p.col[i * 4 + 3] = 255;
        if (ed && ed[i * 4 + 3] > 8) {
            const ea = ed[i * 4 + 3] / 255;
            p.emi[i * 4] = ed[i * 4] * ea; p.emi[i * 4 + 1] = ed[i * 4 + 1] * ea; p.emi[i * 4 + 2] = ed[i * 4 + 2] * ea; p.emi[i * 4 + 3] = 255;
        }
    }
    p.autoHeight(dome, detail, R);
    p.native = NATIVE;
    return p;
}

const POSES = {
    stand: [{ leg: 0, arm: 0 }],
    walk: [{ leg: 1, arm: 1, bob: -0.5 }, { leg: 0, arm: 0 }, { leg: -1, arm: -1, bob: -0.5 }, { leg: 0, arm: 0 }],
    windup: [{ leg: 0, arm: 0, atk: 1 }],
    strike: [{ leg: 1, arm: 0, atk: 2 }],
    hurt: [{ leg: 0, arm: 0, bob: 1 }]
};

export function paintCharacterSheetHi(look) {
    const a = look.skeleton ? look : resolveLook(look);
    const cols = 12, rows = 4;
    const col = newCanvas(FW * cols, FH * rows), emi = newCanvas(FW * cols, FH * rows);
    const cc = col.getContext('2d'), ec = emi.getContext('2d');
    const frames = {};
    const add = (name, c, r) => { (frames[name] = frames[name] || []).push({ x: c * 20, y: r * 26, w: 20, h: 26 }); };
    const cell = (c, r, fn) => {
        cc.save(); ec.save();
        cc.beginPath(); cc.rect(c * FW, r * FH, FW, FH); cc.clip();
        ec.beginPath(); ec.rect(c * FW, r * FH, FW, FH); ec.clip();
        cc.translate(c * FW, r * FH); ec.translate(c * FW, r * FH);
        fn(cc, ec);
        cc.restore(); ec.restore();
    };
    ['down', 'up', 'side'].forEach((dir, row) => {
        let c = 0;
        for (const [name, poses] of Object.entries(POSES)) for (const pose of poses) { cell(c, row, (x, e) => drawFigure(x, e, dir, pose, a)); add(`${name}-${dir}`, c, row); c++; }
    });
    let c = 0;
    for (let k = 0; k < 4; k++) { cell(c, 3, x => drawRoll(x, a, k)); add('roll', c, 3); c++; }
    // dead: the side pose, lying down
    const tmp = newCanvas(FW, FH), tmpE = newCanvas(FW, FH);
    drawFigure(tmp.getContext('2d'), tmpE.getContext('2d'), 'side', { leg: 0, arm: 0 }, a);
    cell(c, 3, (x) => { x.translate(6, 98); x.rotate(-Math.PI / 2); x.scale(0.74, 0.74); x.drawImage(tmp, 0, 0); });
    add('dead', c, 3); c++;
    if (!a.skeleton) { cell(c, 3, (x, e) => drawFigure(x, e, 'down', { leg: 0, arm: 0, drink: true }, a)); add('drink', c, 3); c++; }
    else { cell(c, 3, (x, e) => drawPile(x, e)); add('pile', c, 3); c++; }
    return { pix: canvasToPix(col, emi, { dome: 1.1, detail: 0.25, R: 14 }), frames, canvas: col };
}

// ---------------------------------------------------------------------------- saber fire
export function saberFireHi() {
    const W = 18 * S, H = 30 * S; // 72 × 120
    const col = newCanvas(W, H), emi = newCanvas(W, H);
    const c = col.getContext('2d'), e = emi.getContext('2d');
    // ash bed with embers
    shape(c, ell(36, 104, 32, 11), (() => { const g = c.createRadialGradient(36, 104, 2, 36, 104, 32); g.addColorStop(0, '#3a1e14'); g.addColorStop(0.5, '#231a1a'); g.addColorStop(1, '#141012'); return g; })(), { line: 2 });
    for (let k = 0; k < 26; k++) {
        const ang = k * 2.39, r = 4 + (k * 7) % 20, x = 36 + Math.cos(ang) * r, y = 104 + Math.sin(ang) * r * 0.32;
        const hot = k % 3 === 0;
        c.fillStyle = hot ? '#ff8c3a' : '#5a2a1a'; c.fillRect(x, y, 2, 1.5);
        if (hot) { e.fillStyle = 'rgba(255,120,40,0.9)'; e.fillRect(x - 0.5, y - 0.5, 3, 2.5); }
    }
    // stone ring
    for (let k = 0; k < 11; k++) {
        const ang = k / 11 * Math.PI * 2, x = 36 + Math.cos(ang) * 29, y = 104 + Math.sin(ang) * 9.5, r = 5 + (k % 3);
        shape(c, ell(x, y, r, r * 0.8), rad(c, x, y, r, '#6a6670', 1.35, 0.5), { line: 1.8 });
        c.fillStyle = 'rgba(80,110,60,0.6)'; c.fillRect(x - 2, y - r * 0.6, 3, 1.5);
    }
    // the curved cutlass rammed into the ground
    c.save(); c.translate(36, 46);
    const g = c.createLinearGradient(-6, 0, 7, 0); g.addColorStop(0, '#eef4f6'); g.addColorStop(0.45, '#aeb8bf'); g.addColorStop(1, '#4e565c');
    shape(c, cc => { cc.moveTo(-4, 4); cc.bezierCurveTo(-8, 22, -6, 42, -2, 58); cc.lineTo(4, 58); cc.bezierCurveTo(4, 40, 6, 22, 4, 4); }, g, { line: 2.2 });
    strokeLine(c, [-1.5, 8, -3.5, 30, -1, 52], 'rgba(255,255,255,0.55)', 1);
    for (const y of [20, 33, 45]) { c.fillStyle = '#2a2a30'; c.fillRect(3, y, 2, 2.5); } // nicks
    c.fillStyle = 'rgba(30,20,20,0.6)'; c.fillRect(-6, 52, 12, 6); // sunk into the ash
    // guard with curls
    shape(c, cc => { cc.moveTo(-15, 2); cc.quadraticCurveTo(-18, -4, -12, -4); cc.lineTo(12, -4); cc.quadraticCurveTo(18, -4, 15, 3); cc.quadraticCurveTo(12, 0, 8, 2); cc.lineTo(-8, 2); cc.quadraticCurveTo(-12, 0, -15, 2); }, lin(c, -16, -5, 16, 3, '#c9a24a', 1.5, 0.5), { line: 1.8 });
    // grip with leather wrap
    shape(c, rrect(-3.5, -22, 7, 18, 2.5), lin(c, -4, -22, 4, -4, '#4a2a1a', 1.3, 0.6), { line: 1.6 });
    for (let k = 0; k < 5; k++) strokeLine(c, [-3.5, -20 + k * 3.6, 3.5, -18 + k * 3.6], 'rgba(20,10,6,0.7)', 1);
    c.restore();
    // skull on the hilt
    shape(c, cc => { cc.ellipse(36, 13, 13, 12, 0, Math.PI * 0.95, Math.PI * 2.05); cc.quadraticCurveTo(48, 26, 44, 28); cc.lineTo(28, 28); cc.quadraticCurveTo(24, 26, 23.5, 15); }, rad(c, 36, 14, 14, BONE, 1.35, 0.55), { line: 2.2 });
    strokeLine(c, [41, 3, 43, 8, 41, 11], 'rgba(90,80,70,0.8)', 1);
    for (const x of [30.5, 41.5]) {
        shape(c, ell(x, 15, 4, 4.4), '#140c10', { line: 1 });
        const eg = c.createRadialGradient(x, 15.5, 0, x, 15.5, 4); eg.addColorStop(0, '#eafff8'); eg.addColorStop(0.35, '#3ff0c0'); eg.addColorStop(1, 'rgba(20,120,100,0)');
        c.fillStyle = eg; c.beginPath(); c.arc(x, 15.5, 3.2, 0, 7); c.fill();
        e.fillStyle = eg; e.beginPath(); e.arc(x, 15.5, 4.5, 0, 7); e.fill();
    }
    shape(c, poly(36, 19, 33.5, 23, 38.5, 23), '#140c10', { line: 0.8 });
    shape(c, rrect(29, 24, 14, 5, 1.5), tone(BONE, 0.85), { line: 1.4 });
    for (let k = 0; k < 5; k++) strokeLine(c, [30.8 + k * 2.6, 24.5, 30.8 + k * 2.6, 28.5], '#2a1e1c', 0.9);
    return canvasToPix(col, emi, { dome: 0.9, detail: 0.3, R: 12 });
}

// ---------------------------------------------------------------------------- flames
export function flameSheetHi(kind = 'teal', fw = 14, fh = 22, frames = 8) {
    const W = fw * S, H = fh * S;
    const pal = kind === 'teal'
        ? ['rgba(10,70,62,0.0)', '#127a6a', '#2fd4aa', '#a6ffe4', '#f2fffb']
        : ['rgba(90,20,4,0.0)', '#b8380e', '#ff8a2a', '#ffd27a', '#fff6d8'];
    const col = newCanvas(W * frames, H), emi = newCanvas(W * frames, H);
    const c = col.getContext('2d'), e = emi.getContext('2d');
    const rects = [];
    const tongue = (ctx, x, base, w, h, lean) => {
        ctx.beginPath();
        ctx.moveTo(x - w, base);
        ctx.bezierCurveTo(x - w * 1.1, base - h * 0.45, x + lean * 0.5 - w * 0.4, base - h * 0.75, x + lean, base - h);
        ctx.bezierCurveTo(x + lean * 0.4 + w * 0.5, base - h * 0.7, x + w * 1.1, base - h * 0.4, x + w, base);
        ctx.closePath();
    };
    for (let f = 0; f < frames; f++) {
        const ox = f * W;
        const t = f / frames * Math.PI * 2;
        for (const ctx of [c, e]) {
            ctx.save(); ctx.translate(ox, 0);
            const layers = [[1, pal[1], 1.0], [0.72, pal[2], 0.85], [0.48, pal[3], 0.7], [0.26, pal[4], 0.5]];
            for (const [s, color, hs] of layers) {
                for (let k = -1; k <= 1; k++) {
                    const x = W / 2 + k * W * 0.18 * s + Math.sin(t + k * 1.7) * 2;
                    const hgt = H * hs * (0.82 + 0.18 * Math.sin(t * 1 + k * 2.1)) * (k === 0 ? 1 : 0.72);
                    tongue(ctx, x, H - 3, W * 0.22 * s * (k === 0 ? 1.15 : 0.85), hgt, Math.sin(t + k) * 5);
                    ctx.fillStyle = color; ctx.fill();
                }
            }
            // sparks
            for (let k = 0; k < 3; k++) {
                const sx = W / 2 + Math.sin(t * 2 + k * 2.2) * W * 0.3, sy = H * 0.3 - ((f * 7 + k * 11) % 20);
                ctx.fillStyle = pal[3]; ctx.fillRect(sx, sy, 2, 2);
            }
            ctx.restore();
        }
        rects.push({ x: f * fw, y: 0, w: fw, h: fh });
    }
    return { pix: canvasToPix(col, emi, { dome: 0.3, detail: 0.2, R: 8 }), frames: { burn: rects } };
}
