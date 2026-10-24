// Enemies: beach crabs, reassembling skeleton pirates and the sea monster "Rostkiefer".
import { Humanoid, CrabFigure, SharkFigure } from './figure.js';

class Enemy {
    constructor(game, spawn, fig) {
        this.game = game;
        this.spawn = spawn;
        this.fig = fig;
        this.reset();
    }
    get sprite() { return this.fig; }
    sync(dt, extra = {}) {
        const g = this.game;
        this.fig.animate(dt, { state: this.state, t: this.t, speed: this.speed || 0, ...extra });
        this.fig.setYaw(this.fx, this.fz);
        this.fig.setPos(this.x, g.terrain.ground(this.x, this.z), this.z);
    }
    reset() {
        const s = this.spawn;
        this.x = s.x; this.z = s.z;
        this.vx = this.vz = 0;
        this.fx = 0; this.fz = 1;
        this.hp = this.maxHp;
        this.alive = true;
        this.state = 'idle';
        this.t = Math.random();
        this.hitBy = null;
        this.fig.setVisible(true);
        this.showBarUntil = 0;
    }
    dist() { const p = this.game.player; return Math.hypot(p.x - this.x, p.z - this.z); }
    toPlayer() { const p = this.game.player; const dx = p.x - this.x, dz = p.z - this.z, l = Math.hypot(dx, dz) || 1; return { x: dx / l, z: dz / l, l }; }
    moveBy(dx, dz, dt = 1 / 60) {
        this.speed = Math.hypot(dx, dz) / Math.max(dt, 1e-3);
        const m = this.game.map;
        const r = m.moveCircle(this.x, this.z, dx, dz, this.radius);
        this.x = r.x; this.z = r.z;
    }
    face(x, z) { const l = Math.hypot(x, z); if (l > 0.01) { this.fx = x / l; this.fz = z / l; } }
    set(s) { this.state = s; this.t = 0; }
    // returns true if the hit landed
    damage(dmg, poise, from) {
        if (!this.alive) return false;
        this.hp -= dmg;
        this.showBarUntil = this.game.time + 4;
        this.lastDmg = dmg; this.lastDmgT = this.game.time;
        const dx = this.x - from.x, dz = this.z - from.z, l = Math.hypot(dx, dz) || 1;
        this.vx = dx / l * 3.5; this.vz = dz / l * 3.5;
        if (this.hp <= 0) { this.die(); return true; }
        if (this.state === 'windup' && Math.random() < this.flinch * poise) this.set('stagger');
        else if (this.state !== 'attack' && this.state !== 'windup') { this.set('stagger'); this.staggerTime = 0.28; }
        return true;
    }
    parried() { this.set('stagger'); this.staggerTime = 1.5; this.game.player.riposteUntil = this.game.time + 1.6; }
}

export class Crab extends Enemy {
    constructor(game, spawn) {
        super(game, spawn, new CrabFigure(game.app, game.M));
        this.radius = 0.27;
    }
    get displayName() { return 'Strandkrabbe'; }
    reset() { this.maxHp = 34; this.flinch = 0.8; super.reset(); }
    get pearls() { return 22; }
    die() { this.alive = false; this.set('dead'); this.game.onEnemyKilled(this); }
    update(dt) {
        this.t += dt;
        const tp = this.toPlayer();
        const p = this.game.player;
        this.vx *= Math.pow(0.01, dt); this.vz *= Math.pow(0.01, dt);
        let mx = this.vx * dt, mz = this.vz * dt;
        switch (this.state) {
            case 'idle':
                if (tp.l < 5 && p.state !== 'dead') this.set('chase');
                else if (this.t > 1.6) { this.t = 0; this.wander = { x: Math.random() * 2 - 1, z: Math.random() * 2 - 1 }; }
                if (this.wander) { mx += this.wander.x * 0.6 * dt; mz += this.wander.z * 0.3 * dt; }
                break;
            case 'chase': {
                if (tp.l > 8 || p.state === 'dead') { this.set('idle'); break; }
                // crabs scuttle sideways
                const side = Math.sin(this.t * 3) * 0.8;
                mx += (tp.x * 1.7 - tp.z * side) * dt; mz += (tp.z * 1.7 + tp.x * side) * dt;
                this.face(tp.x, tp.z);
                if (tp.l < 1.15 && this.t > 0.4 && this.game.attackToken(this)) this.set('windup');
                break;
            }
            case 'windup':
                if (this.t > 0.55) { this.set('attack'); this.game.sfx('pinch'); }
                break;
            case 'attack':
                if (this.t < 0.12) { mx += this.fx * 4 * dt; mz += this.fz * 4 * dt; this.game.enemyHit(this, 1.0, 1.2, 15); }
                if (this.t > 0.7) this.set('chase');
                break;
            case 'stagger':
                if (this.t > (this.staggerTime || 0.4)) this.set('chase');
                break;
            case 'dead': // the corpse stays until the next rest
                return this.sync(dt);
        }
        this.moveBy(mx, mz, dt);
        this.sync(dt);
    }
}

export class Skeleton extends Enemy {
    constructor(game, spawn) {
        super(game, spawn, new Humanoid(game.app, game.M, { skeleton: true }));
        this.radius = 0.25;
    }
    get displayName() { return 'Skelettpirat'; }
    reset() { this.maxHp = 62; this.flinch = 0.35; this.destroyed = false; super.reset(); }
    get pearls() { return 55; }
    die() {
        // collapses into a pile and reassembles unless the skull is smashed
        if (this.state === 'pile') { this.destroy(); return; }
        this.hp = 0;
        this.set('pile');
        this.game.sfx('bones');
        this.game.fx.burst('bone', this.x, 0.4 + this.game.terrain.ground(this.x, this.z), this.z);
        this.game.hint('pile', 'Zerschlage den Schädel, solange er am Boden liegt!');
    }
    destroy() {
        this.alive = false; this.destroyed = true;
        this.set('gone');
        this.game.sfx('crush');
        this.game.onEnemyKilled(this);
    }
    damage(dmg, poise, from) {
        if (this.state === 'pile') { this.destroy(); return true; }
        if (this.state === 'rise') return false;
        return super.damage(dmg, poise, from);
    }
    update(dt) {
        this.t += dt;
        const tp = this.toPlayer();
        const p = this.game.player;
        this.vx *= Math.pow(0.01, dt); this.vz *= Math.pow(0.01, dt);
        let mx = this.vx * dt, mz = this.vz * dt;
        const home = Math.hypot(this.x - this.spawn.x, this.z - this.spawn.z);
        switch (this.state) {
            case 'idle':
                if (tp.l < 6.5 && p.state !== 'dead') this.set('chase');
                break;
            case 'chase': {
                if (tp.l > 10 || home > 14 || p.state === 'dead') { this.set('return'); break; }
                this.face(tp.x, tp.z);
                const sp = tp.l > 1.4 ? 2.1 : 0;
                mx += tp.x * sp * dt; mz += tp.z * sp * dt;
                if (tp.l < 1.45 && this.t > 0.3 && this.game.attackToken(this)) { this.set('windup'); this.windT = 0.5 + Math.random() * 0.35; this.combo = Math.random() < 0.35 ? 1 : 0; }
                break;
            }
            case 'return': {
                const dx = this.spawn.x - this.x, dz = this.spawn.z - this.z, l = Math.hypot(dx, dz);
                if (l < 0.3) { this.set('idle'); this.hp = this.maxHp; break; }
                this.face(dx, dz);
                mx += dx / l * 2 * dt; mz += dz / l * 2 * dt;
                if (tp.l < 5 && home < 10) this.set('chase');
                break;
            }
            case 'windup':
                this.face(tp.x, tp.z);
                if (this.t > this.windT) { this.set('attack'); this.game.sfx('swing'); }
                break;
            case 'attack':
                if (this.t < 0.14) { mx += this.fx * 3 * dt; mz += this.fz * 3 * dt; this.game.enemyHit(this, 1.35, 1.1, 24); }
                if (this.t > 0.6) {
                    if (this.combo > 0) { this.combo--; this.set('windup'); this.windT = 0.3; }
                    else this.set('chase');
                }
                break;
            case 'stagger':
                if (this.t > (this.staggerTime || 0.35)) { this.staggerTime = 0; this.set('chase'); }
                break;
            case 'pile':
                if (this.t > 4.2) { this.set('rise'); this.game.sfx('bones'); }
                break;
            case 'rise':
                if (this.t > 0.7) { this.hp = Math.round(this.maxHp * 0.6); this.set('chase'); }
                break;
            case 'gone': // the bones stay where they fell
                return this.sync(dt);
        }
        this.moveBy(mx, mz, dt);
        const st = this.state === 'attack' ? 'strike' : this.state;
        this.sync(dt, { state: st, atk: { wind: this.windT || 0.5, act: 0.14, rec: 0.3 }, combo: this.combo === 0 ? 1 : 0 });
    }
}

// Sea monster: circles the ship, then rams it.
export class Shark extends Enemy {
    constructor(game, spawn) {
        super(game, spawn, new SharkFigure(game.app, game.M));
        this.radius = 1.2;
        this.sea = true;
    }
    get displayName() { return 'Rostkiefer, Schrecken der Tiefe'; }
    reset() { this.maxHp = 160; this.flinch = 0; super.reset(); this.angle = 0; this.state = 'roam'; this.depth = -0.3; }
    get pearls() { return 420; }
    die() { this.alive = false; this.set('dead'); this.game.onEnemyKilled(this); this.game.hint('shark', 'Rostkiefer ist besiegt. Die Tiefe hat ein Ungeheuer weniger.'); }
    damage(dmg, poise, from) {
        if (!this.alive) return false;
        this.hp -= dmg; this.showBarUntil = this.game.time + 5; this.lastDmg = dmg; this.lastDmgT = this.game.time;
        if (this.hp <= 0) this.die(); else if (this.state === 'charge') this.set('retreat');
        return true;
    }
    update(dt) {
        this.t += dt;
        const ship = this.game.ship;
        const dx = ship.x - this.x, dz = ship.z - this.z, l = Math.hypot(dx, dz) || 1;
        let speed = 2.4, tx = this.x + Math.cos(this.t * 0.3) * 5, tz = this.z + Math.sin(this.t * 0.25) * 5;
        const active = this.game.mode === 'sea' && ship.hull > 0;
        switch (this.state) {
            case 'roam':
                if (active && l < 16) this.set('circle');
                break;
            case 'circle': {
                const a = Math.atan2(this.z - ship.z, this.x - ship.x) + 0.9;
                tx = ship.x + Math.cos(a) * 6; tz = ship.z + Math.sin(a) * 6; speed = 4.2;
                if (!active || l > 24) this.set('roam');
                if (this.t > 3.5 + Math.random()) { this.set('charge'); this.game.sfx('roar'); }
                break;
            }
            case 'charge':
                tx = ship.x; tz = ship.z; speed = 7.5;
                if (l < 1.8) { this.game.ramShip(this, 18); this.set('retreat'); }
                if (this.t > 3) this.set('retreat');
                break;
            case 'retreat':
                tx = this.x - dx; tz = this.z - dz; speed = 5;
                if (this.t > 1.8) this.set('circle');
                break;
            case 'dead': // sinks into the deep
                this.fig.setPos(this.x, -0.3 - this.t * 1.5, this.z);
                this.fig.animate(dt, { state: 'dead' });
                if (this.t > 3) this.fig.setVisible(false);
                return;
        }
        const ddx = tx - this.x, ddz = tz - this.z, dl = Math.hypot(ddx, ddz) || 1;
        const want = Math.atan2(ddz, ddx);
        let diff = want - this.angle;
        while (diff > Math.PI) diff -= Math.PI * 2;
        while (diff < -Math.PI) diff += Math.PI * 2;
        this.angle += Math.max(-3 * dt, Math.min(3 * dt, diff));
        const nx = this.x + Math.cos(this.angle) * speed * dt, nz = this.z + Math.sin(this.angle) * speed * dt;
        if (!this.game.map.blockedSail(nx, nz, 0.6)) { this.x = nx; this.z = nz; } else this.angle += 1.5 * dt;
        // mostly submerged; it surfaces when it charges
        this.depth += ((this.state === 'charge' ? -0.02 : -0.32) - this.depth) * Math.min(1, dt * 2);
        this.fig.animate(dt, { state: this.state });
        this.fig.setPos(this.x, this.depth, this.z);
        this.fig.setYaw(Math.cos(this.angle), Math.sin(this.angle));
    }
}
