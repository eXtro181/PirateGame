// Bosses: the four ghost kings (phase 1 as kings, phase 2 transformed into a monster of pirate and
// Greek myth) and Moloch the burning skeleton megalodon, whose element changes at random after
// its transformation. Attacks are data driven: combo, dash, slam, cone, volley, rain, nova, gust, spikes.
import { Actor, dist } from './actors.js';
import { KINGS, MEGALODON, EL_COL } from './data.js';
import { FXCOL } from './fx.js';

const rnd = (a, b) => a + Math.random() * (b - a);
const PART = { fire: 'ember', ice: 'ice', wind: 'wind', earth: 'earth' };
const SHOT = { fire: 'fireball', ice: 'shard', wind: 'feather', earth: 'rock' };

// attack tables: [type, weight, params]
const KING_ATTACKS = {
    fire: [['combo', 3, { n: 2, reach: 34 }], ['dash', 2, { len: 120 }], ['volley', 2, { n: 3, spread: 0.5 }], ['rain', 1, { n: 4, r: 18 }], ['nova', 1, { r: 46 }]],
    ice: [['combo', 3, { n: 3, reach: 34, fast: true }], ['volley', 2, { n: 5, spread: 0.8 }], ['rain', 2, { n: 5, r: 14 }], ['nova', 1, { r: 50, slow: true }]],
    wind: [['combo', 3, { n: 2, reach: 32 }], ['gust', 2, {}], ['rain', 2, { n: 6, r: 12, bolt: true }], ['dash', 2, { len: 140 }]],
    earth: [['slam', 3, { r: 30 }], ['spikes', 2, { n: 7 }], ['combo', 2, { n: 2, reach: 36, heavy: true }], ['volley', 1, { n: 3, spread: 0.4 }]]
};
const BEAST_ATTACKS = {
    fire: [['dash', 3, { len: 150, bite: true }], ['cone', 3, { r: 90, w: 0.45 }], ['nova', 1, { r: 56 }], ['rain', 2, { n: 6, r: 20 }]],
    ice: [['sweep', 3, { reach: 70 }], ['volley', 2, { n: 7, spread: 1.2 }], ['nova', 2, { r: 64, slow: true }], ['rain', 2, { n: 7, r: 14 }]],
    wind: [['dash', 3, { len: 170, dive: true }], ['volley', 3, { n: 7, spread: 1.0 }], ['gust', 2, {}], ['rain', 2, { n: 8, r: 12, bolt: true }]],
    earth: [['slam', 3, { r: 40, double: true }], ['spikes', 3, { n: 9 }], ['volley', 1, { n: 4, spread: 0.6 }], ['nova', 2, { r: 70 }]]
};
const MEGA_ATTACKS = [['dash', 4, { len: 220, bite: true }], ['cone', 3, { r: 110, w: 0.5 }], ['rain', 2, { n: 8, r: 20 }], ['nova', 2, { r: 80 }], ['volley', 2, { n: 6, spread: 1.0 }]];

export class Boss extends Actor {
    constructor(game, key, arena) {
        super(game, arena.x + 40, arena.y - 10);
        this.key = key; this.arena = arena; this.isBoss = true;
        this.mega = key === 'mega';
        this.def = this.mega ? MEGALODON : KINGS[key];
        this.el = this.mega ? 'fire' : this.def.el;
        this.name = this.def.name;
        this.maxHp = this.hp = this.def.hp;
        this.phase = 1; this.state = 'dormant'; this.st_t = 0;
        this.setSheet(this.mega ? game.app.art.beasts.megalodon_fire : game.app.art.chars[this.def.sheet], !this.mega);
        this.fx = -1; this.fy = 0.2;
        this.hidden = true;
        this.home = { x: this.x, y: this.y };
    }
    get dead() { return this.state === 'dead'; }
    setSheet(sheet, human) {
        this.sheet = sheet; this.human = human;
        if (human) { this.r = 7; this.hurtR = 12; this.shadow = 10; this.bodyY = 16; }
        else { const b = sheet.shadow?.[0] || 20; this.r = this.mega ? 18 : 14; this.hurtR = this.mega ? 34 : 24; this.shadow = b; this.bodyY = sheet.h * 0.4; }
    }
    activate() {
        if (this.state !== 'dormant') return;
        this.hidden = false; this.state = 'intro'; this.st_t = 0;
        this.game.fx.rise(PART[this.el], this.x, this.y, 40, { w: 30, col: FXCOL[PART[this.el]] });
        this.game.audio.sfx('roar');
        this.game.bossStarted(this);
    }
    reset() { // player died: back to sleep, full health
        this.hp = this.maxHp; this.phase = 1; this.state = 'dormant'; this.hidden = true; this.x = this.home.x; this.y = this.home.y; this.el = this.mega ? 'fire' : this.def.el;
        this.name = this.def.name;
        this.setSheet(this.mega ? this.game.app.art.beasts.megalodon_fire : this.game.app.art.chars[this.def.sheet], !this.mega);
    }
    pick() {
        const p = this.game.player, d = dist(this, p);
        const table = this.mega ? MEGA_ATTACKS : (this.phase === 1 ? KING_ATTACKS : BEAST_ATTACKS)[this.el];
        const opts = table.filter(([t, , o]) => {
            if (t === 'combo' || t === 'sweep') return d < (o.reach || 40) + 30;
            if (t === 'slam') return d < 90;
            if (t === 'cone') return d < o.r;
            if (t === 'nova') return d < o.r + 20;
            return true;
        });
        const list = opts.length ? opts : table;
        let sum = list.reduce((s, a) => s + a[1], 0), k = Math.random() * sum;
        for (const a of list) { k -= a[1]; if (k <= 0) return a; }
        return list[0];
    }
    update(dt) {
        const g = this.game, p = g.player;
        this.st_t += dt; this.at += dt; this.flash = Math.max(0, this.flash - dt);
        if (this.state === 'dormant' || this.state === 'dead') { if (this.state === 'dead') { this.anim = 'dead'; this.frame = 0; } return; }
        if (this.state === 'intro') { this.anim = this.human ? 'cast' : 'cast'; this.frame = 0; this.face(p.x - this.x, p.y - this.y); if (this.st_t > 1.6) this.toIdle(); return; }
        if (this.state === 'transform') {
            this.anim = this.human ? 'cast' : 'cast'; this.frame = this.st_t > 1 ? 1 : 0;
            if (Math.random() < 0.6) g.fx.rise(PART[this.el], this.x + rnd(-20, 20), this.y, 2, { col: FXCOL[PART[this.el]], speed: 50 });
            if (this.st_t > 1.4 && this.human) {
                // become the monster
                this.setSheet(this.mega ? g.app.art.beasts['megalodon_' + this.el] : g.app.art.beasts[this.def.beast], false);
                this.name = this.def.monster || this.name;
                g.flash([1, 1, 1], 0.9); g.shake(8); g.audio.sfx('roar');
                g.fx.burst(PART[this.el], this.x, this.y - 20, 50, { speed: 140, col: FXCOL[PART[this.el]] });
                g.fx.ring(this.x, this.y, 10, 120, 0.8, EL_COL[this.el]);
                g.ui.banner(this.name, 'Die zweite Gestalt', 'boss', 2.6);
            }
            if (this.st_t > 2.6) { if (this.mega) { this.setSheet(g.app.art.beasts['megalodon_' + this.el], false); g.flash(EL_COL[this.el], 0.6); g.audio.sfx('roar'); g.ui.banner(this.name, 'Das Ende der Welt erwacht', 'boss', 2.4); } this.toIdle(); }
            return;
        }
        if (p.state === 'dead') { this.anim = 'idle'; this.frame = this.at * 2; return; }
        if (this.state === 'idle') {
            // approach / circle the player
            const d = dist(this, p);
            const want = this.human ? 30 : this.mega ? 70 : 50;
            const sp = (this.human ? 46 : this.mega ? 70 : 40) * (this.phase === 2 ? 1.15 : 1);
            if (d > want) { const l = d; this.moveIn((p.x - this.x) / l * sp * dt, (p.y - this.y) / l * sp * dt); this.anim = 'walk'; this.frame = this.at * 8; }
            else { this.anim = 'idle'; this.frame = this.at * 2; }
            this.face(p.x - this.x, p.y - this.y);
            if (this.st_t > this.pause) this.startAttack(this.pick());
            return;
        }
        if (this.state === 'attack') this.runAttack(dt);
        if (this.state === 'stagger') { this.anim = 'hurt'; this.frame = 0; if (this.st_t > 1.1) this.toIdle(); }
    }
    moveIn(dx, dy) {
        const A = this.arena;
        const nx = this.x + dx, ny = this.y + dy;
        if (Math.hypot((nx - A.x) / A.r, (ny - A.y) / A.ry) > 0.92) return;
        const [x, y] = this.game.world.move(this.x, this.y, dx, dy, this.human ? this.r : 4);
        this.x = x; this.y = y;
    }
    toIdle() { this.state = 'idle'; this.st_t = 0; this.pause = (this.phase === 2 ? rnd(0.3, 0.9) : rnd(0.6, 1.4)); }
    startAttack([type, , o]) {
        const g = this.game, p = g.player;
        if (this.mega && this.phase === 2) { // the element changes with every attack
            const els = ['fire', 'ice', 'wind', 'earth'];
            const nel = els[Math.floor(Math.random() * 4)];
            if (nel !== this.el) { this.el = nel; this.setSheet(g.app.art.beasts['megalodon_' + nel], false); g.fx.ring(this.x, this.y - 20, 10, 60, 0.5, EL_COL[nel]); }
        }
        this.state = 'attack'; this.st_t = 0;
        this.atk = { type, o, phase: 'windup', n: 0, t: 0, hits: 0 };
        const A = this.atk, w = this.phase === 2 ? 0.8 : 1;
        this.face(p.x - this.x, p.y - this.y);
        const ang = Math.atan2(p.y - this.y, p.x - this.x);
        const col = [...EL_COL[this.el], 0.42];
        switch (type) {
            case 'combo': A.windup = (o.fast ? 0.35 : 0.55) * w; break;
            case 'sweep': A.windup = 0.8 * w; g.fx.mark({ kind: 'cone', x: this.x, y: this.y, ang, r: o.reach, w: 1.2, dur: A.windup, col }); A.ang = ang; break;
            case 'dash': A.windup = 0.75 * w; A.ang = ang; g.fx.mark({ kind: 'line', x: this.x, y: this.y, ang, r: o.len, w: this.human ? 8 : 16, dur: A.windup, col }); break;
            case 'slam': A.windup = 0.9 * w; A.tx = p.x; A.ty = p.y; g.fx.mark({ kind: 'circle', x: p.x, y: p.y, r: o.r, dur: A.windup, col }); break;
            case 'cone': A.windup = 0.8 * w; A.ang = ang; g.fx.mark({ kind: 'cone', x: this.x, y: this.y - 4, ang, r: o.r, w: o.w, dur: A.windup + 0.9, col }); break;
            case 'volley': A.windup = 0.6 * w; break;
            case 'rain': A.windup = 0.5 * w; A.spots = []; for (let i = 0; i < o.n; i++) { const sx = p.x + rnd(-50, 50) * (i ? 1 : 0), sy = p.y + rnd(-34, 34) * (i ? 1 : 0); A.spots.push([sx, sy, 0.9 + i * 0.12]); g.fx.mark({ kind: 'circle', x: sx, y: sy, r: o.r, dur: A.windup + 0.9 + i * 0.12, col }); } break;
            case 'nova': A.windup = 1.0 * w; g.fx.mark({ kind: 'circle', x: this.x, y: this.y, r: o.r, dur: A.windup, col }); break;
            case 'gust': A.windup = 0.7 * w; A.ang = ang; g.fx.mark({ kind: 'cone', x: this.x, y: this.y, ang, r: 120, w: 0.6, dur: A.windup, col: [0.85, 0.92, 1, 0.3] }); break;
            case 'spikes': A.windup = 0.6 * w; A.ang = ang; A.spots = []; for (let i = 0; i < o.n; i++) { const d = 22 + i * 16, sx = this.x + Math.cos(ang) * d, sy = this.y + Math.sin(ang) * d * 0.8; A.spots.push([sx, sy, 0.5 + i * 0.09]); g.fx.mark({ kind: 'circle', x: sx, y: sy, r: 11, dur: A.windup + 0.5 + i * 0.09, col }); } break;
        }
        g.audio.sfx(type === 'volley' || type === 'rain' || type === 'nova' ? 'charge' : 'swing');
    }
    dmg(mul = 1) { return Math.round(this.def.dmg * mul * (this.phase === 2 ? 1.15 : 1)); }
    runAttack(dt) {
        const g = this.game, p = g.player, A = this.atk, o = A.o;
        A.t += dt;
        const castAnim = ['volley', 'rain', 'nova', 'gust', 'spikes', 'cone'].includes(A.type);
        if (A.phase === 'windup') {
            this.anim = castAnim ? 'cast' : 'atk'; this.frame = 0;
            if (A.type === 'combo') this.face(p.x - this.x, p.y - this.y);
            if (A.t >= A.windup) { A.phase = 'active'; A.t = 0; this.onActive(); }
            return;
        }
        if (A.phase === 'active') {
            this.anim = castAnim ? 'cast' : 'atk'; this.frame = this.human ? (castAnim ? 1 : 1 + (A.t > 0.12 ? 1 : 0)) : 1;
            switch (A.type) {
                case 'combo':
                    if (A.t > 0.28) { A.n++; if (A.n < o.n) { A.phase = 'windup'; A.t = 0; A.windup = (o.fast ? 0.22 : 0.32) * (this.phase === 2 ? 0.8 : 1); } else this.recover(0.7); }
                    break;
                case 'dash': {
                    const sp = (o.len / 0.4);
                    this.moveIn(Math.cos(A.ang) * sp * dt, Math.sin(A.ang) * sp * dt * 0.85);
                    if (Math.random() < 0.8) g.fx.burst(PART[this.el], this.x, this.y - 6, 1, { speed: 20, col: FXCOL[PART[this.el]] });
                    if (!A.hits && dist(this, p) < this.hurtR + 6) { A.hits++; p.hurt(this.dmg(1.2), this, { heavy: true, push: [Math.cos(A.ang) * 16, Math.sin(A.ang) * 16] }); }
                    if (A.t > 0.42) this.recover(0.9);
                    break;
                }
                case 'cone': {
                    // element breath for 0.9 s
                    for (let i = 0; i < 4; i++) { const a = A.ang + rnd(-o.w, o.w), sp = rnd(120, 200); g.fx.p(this.x + Math.cos(A.ang) * (this.human ? 10 : 30), this.y - (this.human ? 14 : 24) + Math.sin(A.ang) * 10, { vx: Math.cos(a) * sp, vy: Math.sin(a) * sp * 0.75, z: 6, life: o.r / sp, col: FXCOL[PART[this.el]], glow: 1.4, size: 2 }); }
                    A.tick = (A.tick || 0) - dt;
                    if (A.tick <= 0) {
                        A.tick = 0.25;
                        const dx = p.x - this.x, dy = (p.y - this.y), d = Math.hypot(dx, dy), a = Math.atan2(dy, dx);
                        let da = Math.abs(((a - A.ang + Math.PI * 3) % (Math.PI * 2)) - Math.PI);
                        if (d < o.r && da < o.w + 0.1) { p.hurt(this.dmg(0.45), this, { unparryable: true }); this.elementHit(p); }
                    }
                    if (A.t > 0.9) this.recover(0.8);
                    break;
                }
                default: if (A.t > 0.3) this.recover(A.type === 'rain' || A.type === 'spikes' ? 1.0 : 0.8);
            }
            return;
        }
        if (A.phase === 'recover') {
            this.anim = 'idle'; this.frame = 0;
            // delayed impacts (rain / spikes)
            if (A.spots) for (const s of A.spots) { if (s[2] !== null && A.t >= s[2] - 0.3) { this.impact(s[0], s[1], A.type === 'spikes' ? 11 : o.r, A.type === 'spikes' ? 0.6 : 0.7, o.bolt); s[2] = null; } }
            if (A.t >= A.rec) this.toIdle();
        }
    }
    recover(t) { const A = this.atk; A.phase = 'recover'; A.rec = t * (this.phase === 2 ? 0.8 : 1) + (A.spots ? 0.7 : 0); A.t = 0; }
    onActive() {
        const g = this.game, p = g.player, A = this.atk, o = A.o;
        const ang = Math.atan2(p.y - this.y, p.x - this.x);
        switch (A.type) {
            case 'combo': {
                this.face(p.x - this.x, p.y - this.y);
                this.moveIn(this.fx * 8, this.fy * 8);
                g.fx.slash(this.x + this.fx * 10, this.y - 14 + this.fy * 6, Math.atan2(this.fy, this.fx), o.reach * 0.8, [...EL_COL[this.el], 1], 2);
                if (this.inArc(p, o.reach, 1.15)) { p.hurt(this.dmg(o.heavy ? 1.2 : 0.8), this, { heavy: o.heavy, push: [this.fx * 8, this.fy * 8] }); this.elementHit(p); }
                g.audio.sfx('swing');
                break;
            }
            case 'sweep': {
                g.fx.slash(this.x, this.y - 20, A.ang, o.reach, [...EL_COL[this.el], 1], 3, 0.25);
                const dx = p.x - this.x, dy = p.y - this.y, d = Math.hypot(dx, dy);
                const da = Math.abs(((Math.atan2(dy, dx) - A.ang + Math.PI * 3) % (Math.PI * 2)) - Math.PI);
                if (d < o.reach && da < 1.25) { p.hurt(this.dmg(1.1), this, { heavy: true, push: [dx / d * 18, dy / d * 18] }); this.elementHit(p); }
                g.audio.sfx('heavy');
                this.recover(0.9);
                break;
            }
            case 'slam': {
                const hit = (x, y) => { g.fx.ring(x, y, 4, o.r, 0.4, EL_COL[this.el]); g.fx.burst(PART[this.el], x, y, 24, { speed: 100, col: FXCOL[PART[this.el]] }); g.fx.burst('dust', x, y, 12, { speed: 70 }); if (Math.hypot(p.x - x, (p.y - y) * 1.3) < o.r) { p.hurt(this.dmg(1.3), this, { heavy: true, unparryable: true }); this.elementHit(p); } };
                hit(A.tx, A.ty); g.shake(6); g.audio.sfx('crush');
                if (o.double) setTimeout(() => { if (this.state === 'attack') { hit(p.x, p.y); g.shake(5); g.audio.sfx('crush'); } }, 450);
                this.recover(o.double ? 1.3 : 1.0);
                break;
            }
            case 'volley': {
                for (let i = 0; i < o.n; i++) {
                    const a = ang + (o.n > 1 ? (i / (o.n - 1) - 0.5) * o.spread * 2 : 0);
                    const sp = this.el === 'wind' ? 150 : 110;
                    g.spawnBossShot(this, SHOT[this.el], a, sp, this.dmg(0.6));
                }
                g.audio.sfx(this.el === 'fire' ? 'cannon' : 'tide');
                break;
            }
            case 'rain': A.t0 = 0; break;
            case 'spikes': A.t0 = 0; break;
            case 'nova': {
                g.fx.ring(this.x, this.y, 6, o.r, 0.5, EL_COL[this.el]); g.fx.burst(PART[this.el], this.x, this.y, 40, { speed: 150, col: FXCOL[PART[this.el]] });
                if (Math.hypot(p.x - this.x, (p.y - this.y) * 1.3) < o.r) { p.hurt(this.dmg(1.0), this, { heavy: true, unparryable: true, push: [(p.x - this.x) * 0.3, (p.y - this.y) * 0.3] }); if (o.slow) g.slowT = 3; this.elementHit(p); }
                g.shake(5); g.audio.sfx('crush');
                break;
            }
            case 'gust': {
                for (let i = 0; i < 40; i++) { const a = A.ang + rnd(-0.6, 0.6), sp = rnd(160, 260); g.fx.p(this.x, this.y - 10, { vx: Math.cos(a) * sp, vy: Math.sin(a) * sp * 0.75, z: rnd(2, 20), life: 0.5, col: FXCOL.wind, size: 1 }); }
                const dx = p.x - this.x, dy = p.y - this.y, d = Math.hypot(dx, dy), da = Math.abs(((Math.atan2(dy, dx) - A.ang + Math.PI * 3) % (Math.PI * 2)) - Math.PI);
                if (d < 120 && da < 0.65) { p.hurt(this.dmg(0.5), this, { unparryable: true }); g.pushPlayer(Math.cos(A.ang) * 90, Math.sin(A.ang) * 90); }
                g.audio.sfx('roll');
                break;
            }
            case 'cone': g.audio.sfx('roar'); break;
        }
    }
    impact(x, y, r, mul, bolt) {
        const g = this.game, p = g.player;
        if (bolt || this.el === 'wind') { g.fx.bolt([[x + rnd(-6, 6), y - 140], [x, y]], [0.8, 0.9, 1, 1], 0.25); g.flash([0.8, 0.9, 1], 0.12); g.audio.sfx('zap'); }
        else if (this.el === 'fire') { g.fx.burst('ember', x, y, 18, { speed: 90 }); g.audio.sfx('cannon'); }
        else if (this.el === 'ice') { g.fx.burst('ice', x, y, 16, { speed: 70, up: 80 }); g.audio.sfx('parried'); }
        else { g.fx.burst('earth', x, y, 16, { speed: 80, up: 90 }); g.fx.burst('dust', x, y, 8, { speed: 50 }); g.audio.sfx('crush'); }
        g.fx.ring(x, y, 2, r, 0.3, EL_COL[this.el]);
        if (Math.hypot(p.x - x, (p.y - y) * 1.3) < r + 3) { p.hurt(this.dmg(mul), this, { unparryable: true }); this.elementHit(p); }
    }
    elementHit(p) {
        const g = this.game;
        if (this.el === 'ice') g.slowT = Math.max(g.slowT, 1.5);
        if (this.el === 'fire') g.fx.burst('ember', p.x, p.y - 10, 6);
        if (this.el === 'earth') p.st = Math.max(0, p.st - 15);
    }
    hurt(dmg, src, o = {}) {
        const g = this.game;
        if (this.state === 'dormant' || this.state === 'dead' || this.state === 'intro' || this.state === 'transform') return;
        if (this.state === 'stagger') dmg = Math.round(dmg * 2.2);
        this.hp -= dmg; this.flash = 0.08;
        g.fx.num(this.x, this.y - (this.human ? 30 : this.sheet.h * 0.6), dmg, this.state === 'stagger' ? [1, 0.75, 0.3, 1] : undefined);
        g.fx.burst(PART[this.el], this.x, this.y - this.bodyY, 6, { speed: 60, col: FXCOL[PART[this.el]] });
        g.audio.sfx('hit');
        if (this.hp <= 0) { this.die(); return; }
        if (this.phase === 1 && this.hp <= this.maxHp * 0.5) {
            this.phase = 2; this.state = 'transform'; this.st_t = 0; this.atk = null;
            this.human = this.human; // keep until the swap
            if (this.mega) this.human = false;
            g.audio.sfx('roar'); g.shake(4);
            g.ui.banner(this.mega ? 'Moloch erwacht' : this.name.split(',')[0] + ' verwandelt sich …', '', 'boss', 1.6);
            if (this.mega) { this.human = true; } // reuse the swap branch for the flash
        }
    }
    stagger() { if (this.state === 'attack' && this.human && this.atk?.type === 'combo') { this.state = 'stagger'; this.st_t = 0; this.atk = null; } }
    die() {
        const g = this.game;
        this.state = 'dead'; this.hp = 0;
        g.fx.burst(PART[this.el], this.x, this.y - 20, 80, { speed: 160, col: FXCOL[PART[this.el]] });
        g.fx.rise('soul', this.x, this.y, 60, { w: 40, speed: 40 });
        g.flash([1, 1, 1], 1); g.shake(10); g.audio.sfx('death');
        g.bossDefeated(this);
    }
    drawPos() { const s = this.sheet; return [Math.round(this.x - s.ax), Math.round(this.y - s.ay)]; }
    get parryable() { return this.human && this.atk?.type === 'combo'; }
}
