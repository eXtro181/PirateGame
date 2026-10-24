// Canvas UI in the low-res target: HUD (health bar with anchor hook, stamina pips, soul bar,
// weapon portholes, rum circle, quick slots, pearls), boss bar, enemy names, interaction bubble,
// ground item names, banners, toasts, dialogue box, list menus, the sea chart and the legend.
import { rgb } from '../art/pal.js';
import { WEAPONS, OFFHAND, SPELLS, RARITY } from '../game/data.js';
import { ZONES, MW, MH } from '../world/world.js';
import { G } from '../art/tiles.js';
import { levelEntries, weaponEntries, shopEntries } from '../game/dialog.js';
import { levelCost } from '../game/data.js';

const c4 = (h, a = 1) => { const v = rgb(h); return [v[0] / 255, v[1] / 255, v[2] / 255, a]; };
export const COL = {
    bone: c4('#D8CFB8'), boneDim: c4('#D8CFB8', 0.7), gold: c4('#E8A443'), soul: c4('#5FF2C8'), blood: c4('#7A1A22'), bloodLight: c4('#C2502E'),
    ink: c4('#0B1016'), panel: c4('#0B1016', 0.82), mist: c4('#8FA3A8'), brass: c4('#E8A443'), brassDark: c4('#7A5A3E'), parchment: c4('#D8CFB8'), parchmentDark: c4('#8F8672'), red: [0.9, 0.35, 0.3, 1]
};

export class CanvasUI {
    constructor(app) {
        this.app = app; this.F = app.font; this.U = app.art.ui; this.W = app.art.atlas.white;
        this.banners = []; this.toastT = 0; this.toastText = '';
        this.dialog = null; this.list = null; this.mapOpen = false; this.legendOpen = false;
        this.deathT = -1; this.fade = 0; this.t = 0;
    }
    get modal() { return !!(this.dialog || this.list || this.mapOpen || this.legendOpen); }
    rect(r, x, y, w, h, c) { r.rect(this.W.tex, Math.round(x), Math.round(y), Math.round(w), Math.round(h), c); }
    frame(r, x, y, w, h, fill = COL.panel) { // brass-edged panel
        this.rect(r, x, y, w, h, fill);
        this.rect(r, x, y, w, 1, COL.brass); this.rect(r, x, y + h - 1, w, 1, COL.brassDark);
        this.rect(r, x, y, 1, h, COL.brass); this.rect(r, x + w - 1, y, 1, h, COL.brassDark);
    }
    banner(title, sub = '', kind = 'big', dur = 3) { this.banners.push({ title, sub, kind, dur, t: 0 }); }
    toast(text) { this.toastText = text; this.toastT = 2.4; }

    // ------------------------------------------------------------------ dialogue
    openDialog(name, script) {
        this.dialog = { name, pages: script.pages, choices: script.choices || [], page: 0, chars: 0, sel: 0 };
        this.app.audio.sfx('click');
    }
    openList(title, items, { onBack, desc = true, wide = false } = {}) { this.list = { title, items, sel: Math.max(0, items.findIndex(i => !i.disabled)), onBack, desc, wide }; }
    closeList() { this.list = null; }
    levelUp() { const g = this.app.game; this.openList(`Stufe ${g.save.level} · nächste ${levelCost(g.save.level)} Perlen · du hast ${g.save.pearls}`, levelEntries(g), { onBack: () => g.endTalk(), wide: true }); }
    weaponShop() { const g = this.app.game; this.openList(`Waffen · ${g.save.pearls} Perlen`, weaponEntries(g), { onBack: () => g.endTalk(), wide: true }); }
    shop() { const g = this.app.game; this.openList(`Handeln · ${g.save.pearls} Perlen`, shopEntries(g), { onBack: () => g.endTalk(), wide: true }); }
    update(dt, input) {
        this.t += dt;
        for (const b of this.banners) b.t += dt;
        this.banners = this.banners.filter(b => b.t < b.dur);
        this.toastT -= dt;
        if (this.legendOpen) { if (input.pressed('legend') || input.pressed('menu') || input.pressed('interact') || input.pressed('confirm')) this.legendOpen = false; return true; }
        if (this.mapOpen) { if (input.pressed('map') || input.pressed('menu') || input.pressed('interact')) this.mapOpen = false; return true; }
        const D = this.dialog;
        if (D) {
            const text = D.pages[D.page] || '';
            D.chars = Math.min(text.length, D.chars + dt * 60);
            const atEnd = D.page >= D.pages.length - 1 && D.chars >= text.length;
            const ok = input.pressed('interact') || input.pressed('confirm') || input.pressed('attack');
            if (atEnd && D.choices.length) {
                if (input.pressed('up')) { D.sel = (D.sel + D.choices.length - 1) % D.choices.length; this.app.audio.sfx('click'); }
                if (input.pressed('down')) { D.sel = (D.sel + 1) % D.choices.length; this.app.audio.sfx('click'); }
                if (ok && !D.choices[D.sel].disabled) { const c = D.choices[D.sel]; this.dialog = null; this.app.audio.sfx('click'); c.act(); }
                if (input.pressed('menu')) { this.dialog = null; this.app.game?.endTalk(); }
            } else if (ok || input.pressed('menu')) {
                if (D.chars < text.length) D.chars = text.length;
                else if (D.page < D.pages.length - 1) { D.page++; D.chars = 0; }
                else { this.dialog = null; this.app.game?.endTalk(); }
            }
            return true;
        }
        const L = this.list;
        if (L) {
            const n = L.items.length;
            if (input.pressed('up')) { L.sel = (L.sel + n - 1) % n; this.app.audio.sfx('click'); }
            if (input.pressed('down')) { L.sel = (L.sel + 1) % n; this.app.audio.sfx('click'); }
            if (input.pressed('interact') || input.pressed('confirm') || input.pressed('attack')) { const it = L.items[L.sel]; if (it && !it.disabled) { this.app.audio.sfx('click'); it.act(); } }
            if (input.pressed('menu') || input.pressed('roll')) { const b = L.onBack; this.list = null; b?.(); }
            return true;
        }
        return false;
    }
    // pointer / touch on the canvas UI (low-res coordinates)
    pointer(x, y, click) {
        const D = this.dialog;
        if (D && this.dialogBox) {
            const B = this.dialogBox;
            const text = D.pages[D.page] || '';
            const atEnd = D.page >= D.pages.length - 1 && D.chars >= text.length;
            if (atEnd && D.choices.length && this.choiceRects) {
                this.choiceRects.forEach((cr, i) => { if (x >= cr[0] && x <= cr[0] + cr[2] && y >= cr[1] && y <= cr[1] + cr[3]) { D.sel = i; if (click && !D.choices[i].disabled) { const c = D.choices[i]; this.dialog = null; c.act(); } } });
                return true;
            }
            if (click && x >= B[0] && y >= B[1] - 30) { if (D.chars < text.length) D.chars = text.length; else if (D.page < D.pages.length - 1) { D.page++; D.chars = 0; } else { this.dialog = null; this.app.game?.endTalk(); } }
            return true;
        }
        const L = this.list;
        if (L && this.listRects) {
            let hit = false;
            this.listRects.forEach((cr, i) => { if (x >= cr[0] && x <= cr[0] + cr[2] && y >= cr[1] && y <= cr[1] + cr[3]) { hit = true; L.sel = i; if (click && !L.items[i].disabled) L.items[i].act(); } });
            if (click && !hit && this.listBox) { const b = this.listBox; if (x < b[0] || x > b[0] + b[2] || y < b[1] || y > b[1] + b[3]) { const back = L.onBack; this.list = null; back?.(); } }
            return true;
        }
        if (click && (this.mapOpen || this.legendOpen)) { this.mapOpen = false; this.legendOpen = false; return true; }
        return false;
    }

    // ------------------------------------------------------------------ drawing
    draw(r, game) {
        const W = r.W, H = r.H, F = this.F;
        r.spriteBegin('alpha');
        if (game) this.drawWorldLabels(r, game);
        if (game && !game.hideHud) this.drawHud(r, game);
        // banners
        for (const b of this.banners) {
            const k = b.t / b.dur, a = Math.min(1, b.t * 3, (b.dur - b.t) * 2);
            if (b.kind === 'death') {
                this.rect(r, 0, H * 0.42 - 14, W, 36, [0.04, 0.02, 0.03, 0.7 * a]);
                this.bigText(r, b.title, W / 2, H * 0.42 - 4, [0.62, 0.12, 0.14, a], 2);
            } else if (b.kind === 'small') {
                F.draw(r, b.title, W / 2, H * 0.24, [COL.gold[0], COL.gold[1], COL.gold[2], a], { align: 'center' });
            } else {
                this.rect(r, 0, H * 0.3 - 12, W, b.sub ? 40 : 30, [0.04, 0.06, 0.09, 0.55 * a]);
                this.rect(r, W * 0.2, H * 0.3 - 12, W * 0.6, 1, [0.91, 0.64, 0.26, 0.5 * a]);
                this.bigText(r, b.title, W / 2, H * 0.3 - 4, b.kind === 'boss' ? [0.86, 0.35, 0.28, a] : [0.91, 0.75, 0.45, a], b.title.length > 26 ? 1 : 2);
                if (b.sub) F.draw(r, b.sub, W / 2, H * 0.3 + 14, [0.85, 0.81, 0.72, a * 0.9], { align: 'center' });
            }
            void k;
        }
        if (this.toastT > 0) { const a = Math.min(1, this.toastT * 2); const w = F.width(this.toastText) + 12; this.rect(r, W / 2 - w / 2, H - 64, w, 13, [0.04, 0.06, 0.09, 0.7 * a]); F.draw(r, this.toastText, W / 2, H - 61, [0.85, 0.81, 0.72, a], { align: 'center' }); }
        if (this.dialog) this.drawDialog(r);
        if (this.list) this.drawList(r);
        if (this.mapOpen && game) this.drawMap(r, game);
        if (this.legendOpen) this.drawLegend(r);
        if (this.fade > 0) this.rect(r, 0, 0, W, H, [0, 0, 0, this.fade]);
        r.end();
    }
    bigText(r, text, x, y, col, scale = 2) { // 2× bitmap text for banners: drawn as doubled glyph pixels
        if (scale === 1) { this.F.draw(r, text, x, y, col, { align: 'center' }); return; }
        const F = this.F, w = F.width(text) * 2;
        let cx = Math.round(x - w / 2);
        for (const ch of text) {
            const g = F.glyphs[ch] || F.glyphs['?'];
            if (g.rect) { r.sprite(g.rect, cx + 2, y + 2, { sx: 2, sy: 2, color: [0, 0, 0, col[3] * 0.8] }); r.sprite(g.rect, cx, y, { sx: 2, sy: 2, color: col }); }
            cx += (g.w + 1) * 2;
        }
    }
    drawWorldLabels(r, game) {
        const cam = game.cam, F = this.F;
        const W = this.W;
        // enemy hp bars
        for (const e of game.enemies) e.drawUI?.(r, cam, W);
        // ground item names: near or with Alt
        const showAll = game.input.held('names');
        for (const it of game.items) {
            const near = Math.hypot(it.x - game.player.x, it.y - game.player.y) < 44;
            if (!showAll && !near && game.focus?.hover !== it) continue;
            const c = RARITY[it.item.rarity];
            const x = it.x - cam[0], y = it.y - cam[1] - 32;
            const w = F.width(it.name) + 6;
            this.rect(r, x - w / 2, y - 2, w, 11, [0.04, 0.06, 0.09, 0.7]);
            F.draw(r, it.name, x, y, c, { align: 'center' });
        }
        // interaction bubble (parchment)
        const f = game.focus;
        if (f && f.label && !this.modal && game.player.state !== 'dead') {
            const text = (game.input.usingTouch ? '' : 'E  ') + f.label;
            const w = F.width(text) + 10;
            const x = Math.round(f.x - cam[0] - w / 2), y = Math.round(f.y - cam[1] - (f.h || 34) - 16);
            this.rect(r, x, y, w, 13, COL.parchment);
            this.rect(r, x, y + 12, w, 1, COL.parchmentDark); this.rect(r, x + w - 1, y, 1, 13, COL.parchmentDark);
            this.rect(r, x + w / 2 - 1, y + 13, 3, 1, COL.parchment); this.rect(r, x + w / 2, y + 14, 1, 1, COL.parchment);
            F.draw(r, text, x + 5, y + 3, [0.1, 0.08, 0.07, 1], { shadow: false });
            if (!game.input.usingTouch) this.rect(r, x + 4, y + 2, 7, 9, [0.48, 0.1, 0.13, 0.25]);
        }
        // target name over a locked or hovered enemy
        const t = game.lock || (game.focus?.hover?.hp !== undefined && !game.focus.hover.isBoss ? game.focus.hover : null);
        if (t && !t.dead && !t.isBoss && t.name) {
            const x = t.x - cam[0], y = t.y - cam[1] - (t.sheet?.h || 30) - 14;
            F.draw(r, t.name, x, y, [0.85, 0.81, 0.72, 0.9], { align: 'center' });
            if (game.lock === t) { const k = Math.floor(this.t * 4) % 2; this.rect(r, x - 1, t.y - cam[1] - (t.sheet?.h || 30) * 0.5 - 1 - k, 3, 3, COL.soul); }
        }
    }
    drawHud(r, game) {
        const F = this.F, U = this.U, p = game.player, s = game.save, W = r.W, H = r.H;
        if (game.sailing) return this.drawShipHud(r, game);
        // health bar 140×8 at (8, 6) with the anchor hook
        r.sprite(U.hpFrame, 8 - 12 + 2, 4);
        const hpW = Math.round(140 * Math.max(0, p.hp) / p.d.maxHp);
        this.rect(r, 16, 7, 140, 8, [0.06, 0.04, 0.05, 0.9]);
        if (game.hpGhost > p.hp) this.rect(r, 16, 7, Math.round(140 * game.hpGhost / p.d.maxHp), 8, [0.75, 0.6, 0.45, 0.8]);
        this.rect(r, 16, 7, hpW, 8, COL.blood);
        this.rect(r, 16, 7, hpW, 2, COL.bloodLight);
        this.rect(r, 16, 13, hpW, 2, [0.3, 0.05, 0.08, 1]);
        // stamina: 4 pips of 8×3 below
        const seg = p.d.maxSt / 4;
        for (let i = 0; i < 4; i++) {
            const f = Math.max(0, Math.min(1, (p.st - i * seg) / seg));
            const x = 16 + i * 10, y = 18;
            this.rect(r, x, y, 8, 3, [0.06, 0.08, 0.1, 0.9]);
            this.rect(r, x, y, Math.round(8 * f), 3, f >= 1 ? [0.55, 0.75, 0.45, 1] : [0.38, 0.5, 0.32, 1]);
            this.rect(r, x, y, Math.round(8 * f), 1, [0.75, 0.9, 0.6, 1]);
        }
        // soul (mp) bar
        const mpW = Math.round(54 * p.mp / p.d.maxMp);
        this.rect(r, 58, 18, 54, 3, [0.06, 0.08, 0.1, 0.9]);
        this.rect(r, 58, 18, mpW, 3, [0.22, 0.6, 0.52, 1]); this.rect(r, 58, 18, mpW, 1, COL.soul);
        // two overlapping weapon portholes (main + off-hand) at top-left under the bars
        const wx = 10, wy = 26;
        r.sprite(U.porthole, wx + 14, wy + 6); r.sprite(U['i_' + (OFFHAND[s.off]?.icon || 'bullets')], wx + 17, wy + 9, { color: s.off === 'none' ? [1, 1, 1, 0.25] : undefined });
        r.sprite(U.porthole, wx, wy); r.sprite(U['i_' + WEAPONS[s.weapon].icon], wx + 3, wy + 3);
        if (s.weaponLevel) F.draw(r, '+' + s.weaponLevel, wx + 17, wy + 1, COL.gold);
        if (s.off === 'pistol') F.draw(r, String(s.bullets), wx + 34, wy + 20, COL.bone);
        // active slot glow ring is drawn in the glow pass (see drawHudGlow)
        // consumable bottom-left: rum circle 24 px + counter, two quick slots of 16 px above
        const cx = 8, cy = H - 34;
        r.sprite(U.porthole26, cx, cy);
        r.sprite(U.i_rum, cx + 5, cy + 5, { color: p.flasks ? undefined : [1, 1, 1, 0.3] });
        F.draw(r, String(p.flasks), cx + 22, cy + 18, COL.bone);
        const q1 = U.porthole16;
        r.sprite(q1, cx, cy - 20); r.sprite(U.i_bomb, cx, cy - 20, { color: s.bombs ? undefined : [1, 1, 1, 0.3] }); F.draw(r, String(s.bombs), cx + 13, cy - 10, COL.boneDim);
        r.sprite(q1, cx + 18, cy - 20); r.sprite(U.i_phial, cx + 18, cy - 20, { color: p.phials ? undefined : [1, 1, 1, 0.3] }); F.draw(r, String(p.phials), cx + 31, cy - 10, COL.boneDim);
        // spell slot
        const sp = s.spells[s.spell];
        if (sp) { r.sprite(q1, cx + 36, cy - 20); r.sprite(U['i_' + SPELLS[sp].icon], cx + 36, cy - 20, { color: p.mp >= SPELLS[sp].mp ? undefined : [1, 1, 1, 0.35] }); }
        if (!game.input.usingTouch) { F.draw(r, '1', cx + 1, cy - 28, [0.56, 0.64, 0.66, 0.7]); F.draw(r, '2', cx + 19, cy - 28, [0.56, 0.64, 0.66, 0.7]); F.draw(r, 'C', cx + 37, cy - 28, [0.56, 0.64, 0.66, 0.7]); F.draw(r, 'R', cx + 30, cy + 2, [0.56, 0.64, 0.66, 0.7]); }
        // pearls bottom-right
        const pt = String(game.pearlShown | 0);
        r.sprite(U.i_pearl, W - 22 - F.width(pt), H - 22);
        F.draw(r, pt, W - 8, H - 17, COL.bone, { align: 'right' });
        if (game.pearlGain > 0) F.draw(r, '+' + game.pearlGain, W - 8, H - 28, [0.37, 0.95, 0.78, Math.min(1, game.pearlGainT)], { align: 'right' });
        // zone name fades in on arrival
        if (game.zoneT > 0) { const a = Math.min(1, game.zoneT, 4 - game.zoneT); F.draw(r, ZONES[game.zoneKey].name, W / 2, 12, [0.85, 0.81, 0.72, a], { align: 'center' }); }
        // boss bar
        const b = game.boss;
        if (b && !b.dead && b.state !== 'dormant') {
            const bw = Math.min(W - 60, 300), bx = Math.round(W / 2 - bw / 2), by = H - 26;
            F.draw(r, b.name, bx, by - 11, COL.bone);
            this.rect(r, bx - 1, by - 1, bw + 2, 7, COL.brassDark); this.rect(r, bx, by, bw, 5, [0.06, 0.04, 0.05, 1]);
            const k = Math.max(0, b.hp / b.maxHp);
            if (game.bossGhost > b.hp) this.rect(r, bx, by, Math.round(bw * game.bossGhost / b.maxHp), 5, [0.75, 0.6, 0.45, 0.8]);
            this.rect(r, bx, by, Math.round(bw * k), 5, COL.blood); this.rect(r, bx, by, Math.round(bw * k), 1, COL.bloodLight);
            if (b.phase === 1 && !b.mega) this.rect(r, bx + bw / 2, by - 1, 1, 7, COL.brass);
        }
        if (game.slowT > 0) F.draw(r, 'Unterkühlt', 16, 46 + 14, [0.6, 0.85, 1, 1]);
    }
    drawShipHud(r, game) {
        const F = this.F, sh = game.ship, W = r.W, H = r.H;
        F.draw(r, 'Rumpf', 8, 6, COL.boneDim);
        this.rect(r, 40, 7, 100, 6, [0.06, 0.04, 0.05, 0.9]);
        this.rect(r, 40, 7, Math.round(100 * sh.hull / sh.maxHull), 6, [0.48, 0.34, 0.22, 1]); this.rect(r, 40, 7, Math.round(100 * sh.hull / sh.maxHull), 1, [0.75, 0.55, 0.35, 1]);
        F.draw(r, 'Segel', 8, 18, COL.boneDim);
        for (let i = 0; i < 3; i++) this.rect(r, 40 + i * 12, 19, 10, 5, i < sh.sail ? COL.bone : [0.2, 0.22, 0.26, 0.8]);
        F.draw(r, `Backbord ${sh.cool[0] > 0 ? '…' : 'bereit'}`, 8, 30, sh.cool[0] > 0 ? COL.boneDim : COL.gold);
        F.draw(r, `Steuerbord ${sh.cool[1] > 0 ? '…' : 'bereit'}`, 8, 40, sh.cool[1] > 0 ? COL.boneDim : COL.gold);
        if (game.shantyT > 0) F.draw(r, `♪ ${game.shantyName}`, W / 2, 10, COL.soul, { align: 'center' });
        const pt = String(game.pearlShown | 0);
        r.sprite(this.U.i_pearl, W - 22 - F.width(pt), H - 22); F.draw(r, pt, W - 8, H - 17, COL.bone, { align: 'right' });
        if (!game.input.usingTouch) F.draw(r, '←→ steuern · ↑↓ Segel · J/L Breitseite · F Shanty · E anlegen', W / 2, H - 12, [0.56, 0.64, 0.66, 0.8], { align: 'center' });
    }
    drawHudGlow(r, game) { // emissive HUD parts: active weapon slot cyan glow
        if (!game || game.sailing || game.hideHud) return;
        r.spriteBegin('alpha');
        const k = 0.6 + 0.4 * Math.sin(this.t * 3);
        const x = 10, y = 26;
        for (let a = 0; a < Math.PI * 2; a += 0.12) this.rect(r, x + 11 + Math.cos(a) * 11.5, y + 11 + Math.sin(a) * 11.5, 1, 1, [0.37 * k, 0.95 * k, 0.78 * k, 1]);
        r.end();
    }
    drawDialog(r) {
        const D = this.dialog, F = this.F, W = r.W, H = r.H;
        const bw = Math.min(W - 16, 420), bh = 62, bx = Math.round(W / 2 - bw / 2), by = H - bh - 8;
        this.dialogBox = [bx, by, bw, bh];
        this.frame(r, bx, by, bw, bh);
        F.draw(r, D.name, bx + 8, by - 10, COL.gold);
        const text = (D.pages[D.page] || '').slice(0, Math.floor(D.chars));
        F.wrap(text, bw - 16).forEach((l, i) => F.draw(r, l, bx + 8, by + 8 + i * 10, COL.bone));
        const full = (D.pages[D.page] || '').length <= D.chars;
        this.choiceRects = null;
        if (full && D.page >= D.pages.length - 1 && D.choices.length) {
            const cw = Math.max(...D.choices.map(c => F.width(c.label))) + 24, ch = D.choices.length * 12 + 6;
            const cx = bx + bw - cw, cy = by - ch - 4;
            this.frame(r, cx, cy, cw, ch);
            this.choiceRects = D.choices.map((c, i) => [cx, cy + 3 + i * 12, cw, 12]);
            D.choices.forEach((c, i) => {
                if (i === D.sel) r.sprite(this.U.compass[Math.floor(this.t * 8) % 4], cx + 4, cy + 4 + i * 12);
                F.draw(r, c.label, cx + 16, cy + 5 + i * 12, c.disabled ? [0.6, 0.6, 0.6, 0.5] : i === D.sel ? COL.gold : COL.boneDim);
            });
        } else if (full && Math.floor(this.t * 2) % 2) this.rect(r, bx + bw - 10, by + bh - 8, 4, 3, COL.gold);
    }
    drawList(r) {
        const L = this.list, F = this.F, W = r.W, H = r.H;
        const lw = Math.min(W - 16, L.wide ? 380 : 300);
        const ih = 12, lh = L.items.length * ih + 30 + (L.desc ? 22 : 0);
        const lx = Math.round(W / 2 - lw / 2), ly = Math.round(Math.max(8, H / 2 - lh / 2));
        this.listBox = [lx, ly, lw, lh];
        this.frame(r, lx, ly, lw, lh);
        F.draw(r, L.title, lx + 8, ly + 6, COL.gold);
        this.rect(r, lx + 6, ly + 17, lw - 12, 1, [0.91, 0.64, 0.26, 0.35]);
        this.listRects = L.items.map((it, i) => [lx, ly + 21 + i * ih, lw, ih]);
        L.items.forEach((it, i) => {
            const y = ly + 23 + i * ih;
            if (i === L.sel) { this.rect(r, lx + 2, y - 2, lw - 4, ih, [0.91, 0.64, 0.26, 0.1]); r.sprite(this.U.compass[Math.floor(this.t * 8) % 4], lx + 5, y - 1); }
            const c = it.disabled ? [0.6, 0.6, 0.6, 0.5] : i === L.sel ? COL.gold : COL.bone;
            F.draw(r, it.label, lx + 17 + (i === L.sel ? 2 : 0), y, c);
            if (it.right) F.draw(r, it.right, lx + lw - 8, y, it.disabled ? [0.6, 0.6, 0.6, 0.5] : COL.boneDim, { align: 'right' });
        });
        const cur = L.items[L.sel];
        if (L.desc && cur?.desc) F.wrap(cur.desc, lw - 16).slice(0, 2).forEach((l, i) => F.draw(r, l, lx + 8, ly + lh - 22 + i * 10, [0.56, 0.64, 0.66, 1]));
        if (L.footer) F.draw(r, L.footer, lx + 8, ly + lh + 4, COL.boneDim);
    }
    // ------------------------------------------------------------------ sea chart
    buildMap(world) {
        const c = document.createElement('canvas'); c.width = MW; c.height = MH;
        const g = c.getContext('2d'), img = g.createImageData(MW, MH);
        const col = { [G.WATER]: [14, 33, 64], [G.SAND]: [110, 100, 80], [G.GRASS]: [52, 71, 58], [G.DIRT]: [74, 51, 38], [G.STONE]: [43, 58, 64], [G.FLOOR]: [58, 63, 69], [G.BRIDGE]: [122, 90, 62], [G.HEX]: [58, 30, 34], [G.ASH]: [90, 80, 80], [G.SNOW]: [164, 180, 186], [G.ICE]: [126, 158, 172], [G.MUD]: [46, 50, 38] };
        for (let i = 0; i < MW * MH; i++) {
            let c3 = col[world.ground[i]] || [40, 40, 40];
            if (world.cliff[i]) c3 = [30, 32, 36];
            if (world.ground[i] === G.WATER && world.shoreDist[i] < 2) c3 = [34, 80, 138];
            img.data.set([c3[0], c3[1], c3[2], 255], i * 4);
        }
        g.putImageData(img, 0, 0);
        this.mapTex = this.app.renderer.texture(c);
    }
    drawMap(r, game) {
        const W = r.W, H = r.H, F = this.F;
        const sc = Math.max(1, Math.floor(Math.min((W - 20) / MW, (H - 40) / MH)));
        const mw = MW * sc, mh = MH * sc, mx = Math.round(W / 2 - mw / 2), my = Math.round(H / 2 - mh / 2) + 6;
        this.rect(r, 0, 0, W, H, [0.02, 0.03, 0.05, 0.85]);
        this.frame(r, mx - 3, my - 3, mw + 6, mh + 6, [0.85, 0.81, 0.72, 0.1]);
        r.sprite({ x: 0, y: 0, w: MW, h: MH, tex: this.mapTex }, mx, my, { sx: sc, sy: sc });
        // fog over unexplored islands
        for (const [k, z] of Object.entries(ZONES)) {
            if (game.save.explored.includes(k)) { F.draw(r, z.name, mx + z.cx * sc, my + (z.cy - z.ry - 4) * sc, COL.bone, { align: 'center' }); continue; }
            if (z.sealed) { this.rect(r, mx + (z.cx - z.rx - 6) * sc, my + (z.cy - z.ry - 6) * sc, (z.rx * 2 + 12) * sc, (z.ry * 2 + 12) * sc, [0.02, 0.03, 0.05, 1]); continue; }
            this.rect(r, mx + (z.cx - z.rx - 3) * sc, my + (z.cy - z.ry - 3) * sc, (z.rx * 2 + 6) * sc, (z.ry * 2 + 6) * sc, [0.1, 0.12, 0.15, 0.92]);
            F.draw(r, '?', mx + z.cx * sc, my + z.cy * sc - 3, COL.boneDim, { align: 'center' });
        }
        for (const f of game.world.fires) if (game.save.litFires.includes(f.id)) this.rect(r, mx + f.x / 16 * sc - 1, my + f.y / 16 * sc - 1, 3, 3, COL.soul);
        const pp = game.sailing ? game.ship : game.player;
        const k = Math.floor(this.t * 3) % 2;
        this.rect(r, mx + pp.x / 16 * sc - 2, my + pp.y / 16 * sc - 2, 5, 5, k ? COL.gold : COL.bloodLight);
        if (game.whirlpoolOpen) this.rect(r, mx + game.world.whirlpool.x / 16 * sc - 2, my + game.world.whirlpool.y / 16 * sc - 2, 5, 5, COL.soul);
        F.draw(r, 'Seekarte', W / 2, 6, COL.gold, { align: 'center' });
        F.draw(r, 'M schließen', W / 2, H - 12, COL.boneDim, { align: 'center' });
    }
    drawLegend(r) {
        const F = this.F, W = r.W, H = r.H;
        const rows = [['WASD / Pfeile', 'Laufen'], ['J', 'Hieb (Shift+J / L: schwer)'], ['Leertaste', 'Rolle'], ['Q', 'Parieren'], ['R', 'Rum trinken'], ['2', 'Seelenphiole'], ['1', 'Bombe werfen'], ['K', 'Pistole / Gezeitenstoß'], ['C / X', 'Zauber wirken / wechseln'], ['Tab', 'Gegner fixieren'], ['E', 'Interagieren, rasten, an Bord'], ['Alt', 'Namen aller Gegenstände'], ['M', 'Seekarte'], ['F', 'Shanty auf See'], ['Esc', 'Menü']];
        const lw = 260, lh = rows.length * 11 + 26, lx = Math.round(W / 2 - lw / 2), ly = Math.round(H / 2 - lh / 2);
        this.frame(r, lx, ly, lw, lh);
        F.draw(r, 'Steuerung', lx + 8, ly + 6, COL.gold);
        rows.forEach(([k, v], i) => { F.draw(r, k, lx + 8, ly + 20 + i * 11, COL.soul); F.draw(r, v, lx + 90, ly + 20 + i * 11, COL.bone); });
    }
}
