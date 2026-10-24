// Actors in world pixels: the player, enemies, NPCs, ground items and projectiles.
import { WEAPONS, OFFHAND, SPELLS, ENEMIES, ITEMS, RARITY, derived } from './data.js';
import { FXCOL } from './fx.js';

export const dist = (a, b) => Math.hypot(a.x - b.x, a.y - b.y);
const clamp = (v, a, b) => Math.max(a, Math.min(b, v));
const BLACK = [0, 0, 0, 1];

// ------------------------------------------------------------------ base
export class Actor {
    constructor(game, x, y) {
        this.game = game; this.x = x; this.y = y;
        this.fx = 0; this.fy = 1; // facing
        this.anim = 'idle'; this.frame = 0; this.at = 0;
        this.flash = 0; this.r = 5; this.hurtR = 7;
        this.alpha = 1;
    }
    face(dx, dy) { const l = Math.hypot(dx, dy); if (l > 0.01) { this.fx = dx / l; this.fy = dy / l; } }
    get dir() { return Math.abs(this.fx) > Math.abs(this.fy) * 1.1 ? 'side' : this.fy > 0 ? 'down' : 'up'; }
    get flip() { return this.dir === 'side' && this.fx < 0; }
    rect() {
        const s = this.sheet;
        if (!s) return null;
        if (s.beast) { const a = s.anims[this.anim] || s.anims.idle; return a[Math.min(a.length - 1, this.frame | 0) % a.length]; }
        const d = s.dirs[this.dir]; const a = d[this.anim] || d.idle;
        return a[(this.frame | 0) % a.length];
    }
    drawPos(rect) { const s = this.sheet; return [Math.round(this.x - s.ax) , Math.round(this.y - s.ay - (this.z || 0))]; }
    beastFlip() { return this.sheet.beast ? this.fx < -0.05 : this.flip; }
    draw(r, cam) {
        const rect = this.rect(); if (!rect) return;
        const [x, y] = this.drawPos(rect);
        r.sprite(rect, x - cam[0], y - cam[1], { flip: this.beastFlip(), flash: this.flash > 0, outline: this.game.focus?.hover === this, color: this.alpha < 1 ? [1, 1, 1, this.alpha] : (this.tint || undefined) });
    }
    drawGlow(r, cam) {
        const rect = this.rect(); if (!rect) return;
        const [x, y] = this.drawPos(rect);
        if (rect.glow) r.sprite(rect, x - cam[0], y - cam[1], { tex: rect.gtex, flip: this.beastFlip() });
        else r.sprite(rect, x - cam[0], y - cam[1], { flip: this.beastFlip(), color: BLACK });
    }
    move(dx, dy) { const [x, y] = this.game.world.move(this.x, this.y, dx, dy, this.r); this.x = x; this.y = y; }
    // within an arc in front of me?
    inArc(t, range, half = 1.1) {
        const dx = t.x - this.x, dy = (t.y - this.y) * 1.15, d = Math.hypot(dx, dy);
        if (d > range + (t.hurtR || 6)) return false;
        if (d < 8) return true;
        const a = Math.acos(clamp((dx * this.fx + dy * this.fy) / d, -1, 1));
        return a < half;
    }
}

// ------------------------------------------------------------------ player
export class Player extends Actor {
    constructor(game, save) {
        super(game, 0, 0);
        this.save = save; this.r = 4.5; this.hurtR = 6;
        this.state = 'free'; this.st_t = 0; this.combo = 0; this.queued = null;
        this.iframes = 0; this.stDelay = 0; this.parryT = 0; this.healLeft = 0;
        this.refresh(true);
    }
    refresh(full = false) {
        const s = this.save;
        this.d = derived(s);
        this.w = WEAPONS[s.weapon];
        if (!this.sheet || this.sheetKey !== JSON.stringify([s.look, s.weapon])) { this.sheet = this.game.app.art.playerSheet(s.look, s.weapon === 'fetish' ? 'fetish' : s.weapon); this.sheetKey = JSON.stringify([s.look, s.weapon]); }
        if (full) { this.hp = this.d.maxHp; this.st = this.d.maxSt; this.mp = this.d.maxMp; this.flasks = s.flasksMax; this.phials = s.phialsMax; }
        this.hp = Math.min(this.hp, this.d.maxHp); this.mp = Math.min(this.mp, this.d.maxMp);
    }
    get busy() { return this.state !== 'free'; }
    useSt(n) { if (this.st <= 0) return false; this.st -= n; this.stDelay = 0.7; return true; }
    update(dt, input) {
        const g = this.game;
        this.st_t += dt; this.at += dt; this.flash = Math.max(0, this.flash - dt);
        this.iframes = Math.max(0, this.iframes - dt);
        this.parryT = Math.max(0, this.parryT - dt);
        // stamina regen
        this.stDelay -= dt;
        if (this.stDelay <= 0 && this.state !== 'roll') this.st = Math.min(this.d.maxSt, this.st + 42 * this.d.stRegen * dt * (this.state === 'free' ? 1 : 0.4));
        if (this.healLeft > 0) { const h = Math.min(this.healLeft, this.d.maxHp * 0.9 * dt); this.hp = Math.min(this.d.maxHp, this.hp + h); this.healLeft -= h; }
        if (this.state === 'dead' || this.state === 'ship') return;
        const ax = input.axis();
        const lockT = g.lock && !g.lock.dead ? g.lock : null;
        switch (this.state) {
            case 'free': {
                const moving = Math.hypot(ax.x, ax.y) > 0.15;
                const sp = (60 + this.save.stats.dex * 0.8) * (g.slowT > 0 ? 0.6 : 1);
                if (moving) { this.move(ax.x * sp * dt, ax.y * sp * dt); this.face(ax.x, ax.y); this.anim = 'walk'; this.frame = this.at * 10; }
                else { this.anim = 'idle'; this.frame = this.at * 1.6; }
                if (lockT) this.face(lockT.x - this.x, lockT.y - this.y);
                if (g.talkTo) break;
                if (input.pressed('roll') && this.useSt(18)) { this.state = 'roll'; this.st_t = 0; if (moving) this.face(ax.x, ax.y); this.rollDir = [this.fx, this.fy]; g.audio.sfx('roll'); break; }
                if (input.pressed('attack') && this.useSt(this.w.stam)) { this.startAttack(false); break; }
                if (input.pressed('heavy') && this.useSt(this.w.stam * 1.6)) { this.startAttack(true); break; }
                if (input.pressed('parry') && this.useSt(10)) { this.state = 'parry'; this.st_t = 0; this.parryT = 0.2 * (this.w.parry || 1); g.audio.sfx('parry'); break; }
                if (input.pressed('drink')) { if (this.flasks > 0) { this.flasks--; this.state = 'drink'; this.st_t = 0; g.audio.sfx('drink'); } else g.ui.toast('Keine Rumflasche mehr.'); break; }
                if (input.pressed('belt2')) { if (this.phials > 0 && this.mp < this.d.maxMp) { this.phials--; this.state = 'drink'; this.st_t = 0; this.drinkMp = true; g.audio.sfx('drink'); } break; }
                if (input.pressed('spell')) { this.tryCast(); break; }
                if (input.pressed('swap') && this.save.spells.length > 1) { this.save.spell = (this.save.spell + 1) % this.save.spells.length; g.audio.sfx('click'); g.ui.toast(SPELLS[this.save.spells[this.save.spell]].name); break; }
                if (input.pressed('item')) { this.tryShoot(); break; }
                if (input.pressed('belt1')) { this.tryBomb(); break; }
                break;
            }
            case 'roll': {
                const T = 0.42;
                const k = this.st_t / T;
                if (this.st_t < 0.32) this.iframes = Math.max(this.iframes, 0.02);
                const sp = 150 * (1 - k * 0.6);
                this.move(this.rollDir[0] * sp * dt, this.rollDir[1] * sp * dt);
                this.anim = 'roll'; this.frame = Math.min(3, k * 4);
                if (this.st_t > 0.1 && Math.random() < 0.4) g.fx.burst('dust', this.x, this.y, 1, { speed: 20, up: 10, g: 40, life: 0.4 });
                if (this.st_t >= T) this.state = 'free';
                break;
            }
            case 'attack': {
                const T = this.atkT;
                const k = this.st_t / T;
                this.anim = 'atk';
                this.frame = k < 0.32 ? 0 : k < 0.5 ? 1 : k < 0.72 ? 2 : 3;
                if (k < 0.3) { if (lockT) this.face(lockT.x - this.x, lockT.y - this.y); else if (Math.hypot(ax.x, ax.y) > 0.3) this.face(ax.x, ax.y); }
                if (k >= 0.33 && !this.hitDone) { this.hitDone = true; this.move(this.fx * 5, this.fy * 5); g.playerStrike(this, this.heavy); }
                if (k > 0.55 && (input.pressed('attack') || input.pressed('heavy'))) this.queued = input.pressed('heavy') ? 'heavy' : 'attack';
                if (k >= 1) {
                    if (this.queued && this.useSt(this.w.stam * (this.queued === 'heavy' ? 1.6 : 1))) { const h = this.queued === 'heavy'; this.combo = (this.combo + 1) % 3; this.startAttack(h, true); }
                    else { this.state = 'free'; this.combo = 0; }
                }
                break;
            }
            case 'parry': { this.anim = 'atk'; this.frame = 0; if (this.st_t > 0.45) this.state = 'free'; break; }
            case 'hurt': { this.anim = 'hurt'; this.frame = 0; if (this.st_t > 0.35) this.state = 'free'; break; }
            case 'drink': {
                this.anim = 'drink'; this.frame = this.st_t > 0.35 ? 1 : 0;
                if (Math.hypot(ax.x, ax.y) > 0.2) { this.move(ax.x * 20 * dt, ax.y * 20 * dt); }
                if (this.st_t > 0.5 && !this.drank) {
                    this.drank = true;
                    if (this.drinkMp) { this.mp = this.d.maxMp; g.fx.rise('soul', this.x, this.y, 14); }
                    else { this.healLeft = this.d.maxHp * 0.45; g.fx.rise('heal', this.x, this.y, 12, { col: FXCOL.heal }); }
                    g.audio.sfx('heal');
                }
                if (this.st_t > 0.95) { this.state = 'free'; this.drank = false; this.drinkMp = false; }
                break;
            }
            case 'cast': {
                this.anim = 'cast'; this.frame = this.st_t > 0.3 ? 1 : 0;
                if (this.st_t > 0.3 && !this.castDone) { this.castDone = true; g.castSpell(this, this.castSpell); }
                if (this.st_t > 0.65) this.state = 'free';
                break;
            }
            case 'shoot': {
                this.anim = this.save.off === 'tide' ? 'cast' : 'shoot'; this.frame = 1;
                if (this.st_t > 0.18 && !this.shot) { this.shot = true; g.playerShoot(this); }
                if (this.st_t > 0.5) this.state = 'free';
                break;
            }
            case 'throw': {
                this.anim = 'atk'; this.frame = this.st_t > 0.2 ? 2 : 0;
                if (this.st_t > 0.2 && !this.shot) { this.shot = true; g.throwBomb(this); }
                if (this.st_t > 0.45) this.state = 'free';
                break;
            }
            case 'sit': this.anim = 'sit'; this.frame = 0; break;
            case 'talk': this.anim = 'idle'; this.frame = this.at * 1.6; break;
            case 'stagger': this.anim = 'hurt'; this.frame = 0; if (this.st_t > 0.9) this.state = 'free'; break;
        }
    }
    startAttack(heavy, chained = false) {
        this.state = 'attack'; this.st_t = 0; this.heavy = heavy; this.hitDone = false; this.queued = null;
        if (!chained) this.combo = 0;
        this.atkT = (heavy ? 0.85 : 0.48) / this.w.speed;
        this.game.audio.sfx(heavy ? 'heavy' : 'swing');
    }
    tryCast() {
        const sp = this.save.spells[this.save.spell];
        if (!sp) return;
        const S = SPELLS[sp];
        if (this.mp < S.mp) { this.game.ui.toast('Zu wenig Geist.'); return; }
        this.mp -= S.mp; this.state = 'cast'; this.st_t = 0; this.castDone = false; this.castSpell = sp;
        this.game.audio.sfx('charge');
    }
    tryShoot() {
        const off = this.save.off;
        if (off === 'none') { this.game.ui.toast('Keine Zweithand.'); return; }
        if (off === 'pistol' && this.save.bullets <= 0) { this.game.ui.toast('Keine Kugeln mehr.'); return; }
        if (!this.useSt(OFFHAND[off].stam)) return;
        if (off === 'pistol') this.save.bullets--;
        this.state = 'shoot'; this.st_t = 0; this.shot = false;
    }
    tryBomb() {
        if (this.save.bombs <= 0) { this.game.ui.toast('Keine Bomben.'); return; }
        this.save.bombs--; this.state = 'throw'; this.st_t = 0; this.shot = false;
    }
    hurt(dmg, from, o = {}) {
        const g = this.game;
        if (this.state === 'dead' || this.state === 'ship') return false;
        if (this.iframes > 0) return false;
        if (this.parryT > 0 && from && !o.unparryable && from.parryable !== false) {
            g.onParry(from); this.parryT = 0; return false;
        }
        dmg = Math.round(dmg * this.d.armor);
        this.hp -= dmg; this.flash = 0.12; this.iframes = 0.45;
        g.fx.burst('blood', this.x, this.y - 10, 8, { speed: 50 });
        g.fx.num(this.x, this.y - 14, dmg, [0.9, 0.3, 0.25, 1]);
        g.shake(o.heavy ? 4 : 2);
        g.audio.sfx('hurt');
        if (o.push) this.move(o.push[0], o.push[1]);
        if (this.hp <= 0) { this.hp = 0; g.playerDied(); return true; }
        if (this.state !== 'drink' || o.heavy) { this.state = 'hurt'; this.st_t = 0; }
        return true;
    }
}

// ------------------------------------------------------------------ enemies
export class Enemy extends Actor {
    constructor(game, spawn) {
        super(game, spawn.x, spawn.y);
        this.spawn = spawn; this.type = spawn.type;
        this.def = ENEMIES[this.type];
        this.sheet = this.def.beast ? game.app.art.beasts[this.def.beast] : game.app.art.chars[this.def.sheet];
        this.maxHp = this.hp = this.def.hp; this.state = 'idle'; this.st_t = Math.random(); this.home = { x: spawn.x, y: spawn.y };
        this.r = this.def.beast === 'crabWarrior' ? 8 : 5; this.hurtR = this.r + 3;
        this.poise = this.def.poise; this.name = this.def.name;
        this.fx = Math.random() - 0.5; this.fy = 1;
        this.shadow = this.def.beast ? this.sheet.shadow[0] : 6;
    }
    get dead() { return this.state === 'dead'; }
    update(dt) {
        const g = this.game, p = g.player, D = this.def;
        this.st_t += dt; this.at += dt; this.flash = Math.max(0, this.flash - dt);
        if (this.state === 'dead') { this.anim = 'dead'; this.frame = 0; return; }
        const dp = dist(this, p);
        const canSee = p.state !== 'dead' && p.state !== 'ship' && dp < D.aggro && g.world.lineClear(this.x, this.y, p.x, p.y);
        const leash = dist(this, this.home) > 260;
        const walk = (tx, ty, sp) => {
            const dx = tx - this.x, dy = ty - this.y, l = Math.hypot(dx, dy);
            if (l < 2) return false;
            // separation from other enemies
            let sx = 0, sy = 0;
            for (const e of g.enemies) if (e !== this && !e.dead) { const d = dist(this, e); if (d < 14 && d > 0.01) { sx += (this.x - e.x) / d; sy += (this.y - e.y) / d; } }
            this.move((dx / l + sx * 0.6) * sp * dt, (dy / l + sy * 0.6) * sp * dt);
            this.face(dx, dy); this.anim = 'walk'; this.frame = this.at * (this.sheet.beast ? 8 : 9);
            return true;
        };
        switch (this.state) {
            case 'idle':
                this.anim = 'idle'; this.frame = this.at * 1.5;
                if (canSee) { this.state = 'chase'; this.st_t = 0; }
                else if (dist(this, this.home) > 6) walk(this.home.x, this.home.y, D.speed * 0.6);
                break;
            case 'chase': {
                if (leash || (!canSee && this.st_t > 3)) { this.state = 'idle'; break; }
                if (canSee) this.st_t = 0;
                const reach = D.reach;
                if (dp < reach + 4 && this.cool <= 0) { this.state = 'windup'; this.st_t = 0; this.face(p.x - this.x, p.y - this.y); this.telegraph(); break; }
                if (D.ranged && dp < reach && dp > 40 && (this.cool || 0) <= 0) { this.state = 'windup'; this.st_t = 0; this.face(p.x - this.x, p.y - this.y); break; }
                if (D.ranged && dp < 60) walk(this.x - (p.x - this.x), this.y - (p.y - this.y), D.speed * 0.8);
                else if (dp > reach * 0.7) walk(p.x, p.y, D.speed);
                else { this.anim = 'idle'; this.face(p.x - this.x, p.y - this.y); }
                break;
            }
            case 'windup':
                this.anim = this.def.ranged ? (this.sheet.beast ? 'atk' : 'shoot') : 'atk'; this.frame = 0;
                if (!D.charge) this.face(p.x - this.x, p.y - this.y);
                if (this.st_t > D.windup) { this.state = 'strike'; this.st_t = 0; this.hitDone = false; }
                break;
            case 'strike': {
                this.anim = D.ranged && !this.sheet.beast ? 'shoot' : 'atk'; this.frame = 1;
                if (D.charge) {
                    this.move(this.fx * 130 * dt, this.fy * 130 * dt);
                    if (!this.hitDone && dist(this, p) < 14) { this.hitDone = true; p.hurt(D.dmg, this, { push: [this.fx * 10, this.fy * 10] }); }
                    if (this.st_t > 0.5) this.endStrike();
                    break;
                }
                if (!this.hitDone) {
                    this.hitDone = true;
                    if (D.ranged) { g.spawnShot(this, 'bullet', p.x, p.y - 8, D.dmg); g.audio.sfx('pistol'); }
                    else if (D.aoe) {
                        const ix = this.x + this.fx * 16, iy = this.y + this.fy * 12;
                        g.fx.ring(ix, iy, 4, D.aoe, 0.35, [0.8, 0.75, 0.6, 1]); g.fx.burst('dust', ix, iy, 14, { speed: 70 }); g.shake(3); g.audio.sfx('crush');
                        if (Math.hypot(p.x - ix, (p.y - iy) * 1.3) < D.aoe) p.hurt(D.dmg, this, { heavy: true, push: [this.fx * 14, this.fy * 14] });
                    } else {
                        g.fx.slash(this.x + this.fx * 8, this.y - 10 + this.fy * 6, Math.atan2(this.fy, this.fx), D.reach * 0.8, D.burn ? [1, 0.6, 0.3, 1] : [0.85, 0.85, 0.9, 1]);
                        g.audio.sfx('swing');
                        if (this.inArc(p, D.reach, 1.0)) { if (p.hurt(D.dmg, this) && D.chill) g.slowT = 2; if (D.burn) g.fx.burst('ember', p.x, p.y - 10, 6); }
                    }
                }
                if (this.st_t > 0.18) this.endStrike();
                break;
            }
            case 'recover': this.anim = 'idle'; this.frame = 0; if (this.st_t > this.def.recover) { this.state = 'chase'; this.st_t = 0; } break;
            case 'hurt': this.anim = 'hurt'; this.frame = 0; if (this.st_t > 0.3) { this.state = 'chase'; this.st_t = 0; } break;
            case 'stagger': this.anim = 'hurt'; this.frame = 0; this.tint = [1, 1, 1, 1]; if (this.st_t > 1.4) { this.state = 'chase'; this.st_t = 0; this.tint = null; } break;
        }
        this.cool = Math.max(0, (this.cool || 0) - dt);
    }
    telegraph() {
        const D = this.def;
        if (D.aoe) this.game.fx.mark({ kind: 'circle', x: this.x + this.fx * 16, y: this.y + this.fy * 12, r: D.aoe, dur: D.windup });
        if (D.charge) this.game.fx.mark({ kind: 'line', x: this.x, y: this.y, ang: Math.atan2(this.fy, this.fx), r: 70, w: 5, dur: D.windup });
    }
    endStrike() { this.state = 'recover'; this.st_t = 0; this.cool = 0.4 + Math.random() * 0.6; }
    hurt(dmg, src, o = {}) {
        if (this.dead) return;
        const g = this.game;
        const crit = this.state === 'stagger';
        if (crit) dmg = Math.round(dmg * 2.6);
        this.hp -= dmg; this.flash = 0.1;
        g.fx.num(this.x, this.y - (this.sheet.h * 0.7), dmg, crit ? [1, 0.75, 0.3, 1] : undefined);
        g.fx.burst(this.def.sheet?.startsWith('skel') || this.def.sheet === 'ashSkeleton' ? 'bone' : 'blood', this.x, this.y - 10, 7, { speed: 55 });
        g.audio.sfx(crit ? 'crit' : 'hit');
        if (this.state === 'idle') { this.state = 'chase'; this.st_t = 0; }
        if (this.hp <= 0) { this.die(); return; }
        this.poise -= o.poise ?? dmg;
        if (this.poise <= 0 || crit) { this.poise = this.def.poise; if (this.state !== 'stagger') { this.state = 'hurt'; this.st_t = 0; } }
        if (o.push && this.def.poise < 50) this.move(o.push[0], o.push[1]);
    }
    die() {
        const g = this.game;
        this.state = 'dead'; this.hp = 0; this.anim = 'dead';
        g.audio.sfx(this.def.sheet?.includes('kel') ? 'bones' : 'crush');
        g.gainPearls(Math.round(this.def.pearls * (g.save.origin === 'schmuggler' ? 1.2 : 1) * (g.save.gift === 'coin' ? 1.15 : 1)), this);
        if (g.lock === this) g.lock = null;
    }
    stagger() { if (this.dead) return; this.state = 'stagger'; this.st_t = 0; this.flash = 0.2; }
    // hp bar above the head: 16×2 blood red
    drawUI(r, cam, white) {
        if (this.dead || this.hp >= this.maxHp || this.state === 'idle') return;
        const x = Math.round(this.x - 8 - cam[0]), y = Math.round(this.y - this.sheet.h + (this.sheet.beast ? 0 : 2) - 4 - cam[1]);
        r.rect(white.tex, x - 1, y - 1, 18, 4, [0.04, 0.06, 0.09, 0.85]);
        r.rect(white.tex, x, y, Math.max(1, Math.round(16 * this.hp / this.maxHp)), 2, [0.48, 0.1, 0.13, 1]);
        r.rect(white.tex, x, y, Math.max(1, Math.round(16 * this.hp / this.maxHp)), 1, [0.76, 0.31, 0.18, 1]);
    }
}

// ------------------------------------------------------------------ NPCs
export class NPC extends Actor {
    constructor(game, def) {
        super(game, def.x, def.y);
        this.def = def; this.id = def.id; this.name = def.name;
        this.sheet = game.app.art.chars[def.preset];
        this.shadow = 6; this.r = 5;
        this.anim = def.sit ? 'sit' : 'idle';
        this.collider = { kind: 'circle', x: this.x, y: this.y - 2, r: 5, p: this };
    }
    update(dt) {
        this.at += dt;
        const p = this.game.player;
        if (!this.def.sit && dist(this, p) < 50) this.face(p.x - this.x, p.y - this.y);
        this.frame = this.def.sit ? 0 : this.at * 1.4;
        this.anim = this.def.sit ? 'sit' : 'idle';
    }
}

// ------------------------------------------------------------------ ground items
export class GroundItem {
    constructor(game, def) { this.game = game; this.def = def; this.x = def.x; this.y = def.y; this.item = ITEMS[def.item]; this.t = Math.random() * 6; this.shadow = 4; this.name = this.item.name; this.hurtR = 6; }
    update(dt) { this.t += dt; }
    get icon() { return this.game.app.art.ui['i_' + this.item.icon]; }
    draw(r, cam) {
        const ic = this.icon, bob = Math.round(Math.sin(this.t * 2.5) * 1.5);
        r.sprite(ic, Math.round(this.x - 8 - cam[0]), Math.round(this.y - 18 + bob - cam[1]), { outline: this.game.focus?.hover === this });
    }
    drawGlow(r, cam) {
        const ic = this.icon, bob = Math.round(Math.sin(this.t * 2.5) * 1.5);
        const k = 0.5 + 0.5 * Math.sin(this.t * 3);
        const col = RARITY[this.item.rarity];
        if (ic.glow) r.sprite(ic, Math.round(this.x - 8 - cam[0]), Math.round(this.y - 18 + bob - cam[1]), { tex: ic.gtex });
        else r.sprite(ic, Math.round(this.x - 8 - cam[0]), Math.round(this.y - 18 + bob - cam[1]), { color: BLACK });
        r.rect(this.game.app.art.atlas.white.tex, Math.round(this.x - cam[0]), Math.round(this.y - 22 + bob - cam[1]), 1, 1, [col[0] * 1.5 * k, col[1] * 1.5 * k, col[2] * 1.5 * k, 1]);
    }
}

// ------------------------------------------------------------------ projectiles
export class Shot {
    constructor(game, o) { Object.assign(this, { z: 8, r: 4, life: 2, t: 0, hit: new Set() }, o); this.game = game; }
    update(dt) {
        const g = this.game;
        this.t += dt; this.life -= dt;
        this.x += this.vx * dt; this.y += this.vy * dt;
        if (this.vz !== undefined) { this.vz -= (this.grav || 0) * dt; this.z += this.vz * dt; if (this.z <= 0 && this.grav) { this.z = 0; this.life = 0; this.onLand?.(this); return; } }
        if (this.kind === 'fireball' && Math.random() < 0.6) g.fx.p(this.x, this.y, { z: this.z, vz: 8, life: 0.4, col: FXCOL.ember, glow: 1.4 });
        if (this.kind === 'shard' && Math.random() < 0.4) g.fx.p(this.x, this.y, { z: this.z, life: 0.3, col: FXCOL.ice, glow: 1 });
        if (this.kind === 'cannon' && Math.random() < 0.5) g.fx.p(this.x, this.y, { z: this.z, life: 0.5, col: FXCOL.smoke, size: 2 });
        // wall hits
        const tx = Math.floor(this.x / 16), ty = Math.floor(this.y / 16);
        if (!this.sea && g.world.inb(tx, ty) && g.world.cliff[ty * 240 + tx]) { this.life = 0; this.onHitWall?.(this); }
        if (this.life <= 0) { this.onExpire?.(this); return; }
        const targets = this.owner === 'player' ? g.hittables() : this.owner === 'enemy' ? (g.sailing ? [g.ship] : [g.player]) : [];
        for (const t of targets) {
            if (this.hit.has(t) || t.dead || t.hidden) continue;
            const R = (t.hurtR || 6) + this.r;
            if (Math.hypot(t.x - this.x, (t.y - (t.bodyY || 8)) - (this.y - this.z) ) < R + 4 || Math.hypot(t.x - this.x, t.y - this.y) < R) {
                this.hit.add(t);
                this.onHit ? this.onHit(this, t) : t.hurt(this.dmg, this, { push: [this.vx * 0.05, this.vy * 0.05] });
                if (!this.pierce) { this.life = 0; break; }
            }
        }
    }
    draw(r, cam) {
        const w = this.game.app.art.atlas.white, x = Math.round(this.x - cam[0]), y = Math.round(this.y - this.z - cam[1]);
        const c = this.col || [0.2, 0.2, 0.24, 1];
        switch (this.kind) {
            case 'bullet': r.rect(w.tex, x - 1, y - 1, 2, 2, [0.25, 0.27, 0.3, 1]); r.rect(w.tex, x - 1, y - 1, 1, 1, [0.8, 0.85, 0.9, 1]); break;
            case 'cannon': case 'rock': case 'bomb': r.rect(w.tex, x - 2, y - 2, 4, 4, this.kind === 'rock' ? [0.35, 0.32, 0.26, 1] : [0.12, 0.13, 0.16, 1]); r.rect(w.tex, x - 1, y - 2, 1, 1, [0.6, 0.62, 0.66, 1]); break;
            default: r.rect(w.tex, x - 2, y - 2, 4, 4, c); r.rect(w.tex, x - 1, y - 3, 2, 6, c); r.rect(w.tex, x - 3, y - 1, 6, 2, c);
        }
    }
    drawGlow(r, cam) {
        const w = this.game.app.art.atlas.white, x = Math.round(this.x - cam[0]), y = Math.round(this.y - this.z - cam[1]);
        if (this.glow) { const c = this.glow; r.rect(w.tex, x - 2, y - 2, 4, 4, c); r.rect(w.tex, x - 1, y - 3, 2, 6, c); r.rect(w.tex, x - 3, y - 1, 6, 2, c); }
        else if (this.kind === 'bomb') { const k = Math.sin(this.t * 30) > 0 ? 1.5 : 0.3; r.rect(w.tex, x, y - 3, 1, 1, [k, k * 0.6, 0.1, 1]); }
        else r.rect(w.tex, x - 2, y - 2, 4, 4, BLACK);
    }
}
