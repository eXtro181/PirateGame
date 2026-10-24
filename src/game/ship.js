// Sailing: the player's ship (16 headings, sail levels, broadsides), ghost ships that hunt the
// player at sea, bone sharks, cannonballs and the whirlpool to the End of the World.
import { dist } from './actors.js';
import { FXCOL } from './fx.js';

const TAU = Math.PI * 2;
const wrap = a => ((a % TAU) + TAU) % TAU;

class Hull {
    constructor(game, x, y, h, sheet) { this.game = game; this.x = x; this.y = y; this.h = h; this.v = 0; this.sheet = sheet; this.bob = Math.random() * 6; this.hurtR = 16; this.shadow = 0; this.flash = 0; }
    rect() { return this.sheet.frames[Math.round(wrap(this.h) / TAU * 16) % 16]; }
    draw(r, cam) {
        const rect = this.rect(), b = Math.round(Math.sin(this.bob) * 1);
        r.sprite(rect, Math.round(this.x - this.sheet.ax - cam[0]), Math.round(this.y - this.sheet.ay + b - (this.rise ?? 0) * 0 - cam[1]), { flash: this.flash > 0, color: this.alpha !== undefined ? [1, 1, 1, this.alpha] : undefined, outline: this.game.focus?.hover === this });
    }
    drawGlow(r, cam) {
        const rect = this.rect(), b = Math.round(Math.sin(this.bob) * 1);
        r.sprite(rect, Math.round(this.x - this.sheet.ax - cam[0]), Math.round(this.y - this.sheet.ay + b - cam[1]), { tex: rect.gtex });
    }
    wake(dt) {
        const g = this.game;
        if (this.v > 8 && Math.random() < this.v / 60) {
            const fx = Math.sin(this.h), fy = -Math.cos(this.h) * 0.75;
            for (const s of [-1, 1]) g.fx.p(this.x - fx * 16 + Math.cos(this.h) * s * 5, this.y - fy * 16 + Math.sin(this.h) * s * 4, { vx: -fx * 10 + Math.cos(this.h) * s * 12, vy: -fy * 10, life: 0.9, col: [0.66, 0.78, 0.9, 0.8], drag: 1.5 });
        }
        this.bob += dt * 2;
    }
    broadside(side, dmg, owner) { // side: -1 port (left), +1 starboard (right)
        const g = this.game;
        const fx = Math.sin(this.h), fy = -Math.cos(this.h);
        const nx = Math.cos(this.h) * side, ny = Math.sin(this.h) * side;
        for (let i = -1; i <= 1; i++) {
            const sx = this.x + fx * i * 8 + nx * 8, sy = this.y + fy * i * 6 * 0.75 + ny * 6;
            g.spawnShotRaw({ owner, kind: 'cannon', x: sx, y: sy, z: 6, vx: nx * 170 + fx * this.v + (Math.random() - 0.5) * 20, vy: (ny * 170 + fy * this.v) * 0.8, dmg, r: 4, life: 0.9, sea: true });
            g.fx.burst('smoke', sx, sy, 5, { speed: 30, up: 10, g: -10, life: 0.9, col: FXCOL.smoke });
        }
        g.audio.sfx('cannon'); g.shake(3);
    }
}

export class Ship extends Hull {
    constructor(game, x, y, h) { super(game, x, y, h, game.app.art.beasts.ship); this.sail = 0; this.hull = this.maxHull = 120; this.cool = [0, 0]; this.r = 10; this.isShip = true; this.name = 'Dein Schiff'; }
    update(dt, input, sailing) {
        const g = this.game;
        this.flash = Math.max(0, this.flash - dt);
        this.cool[0] = Math.max(0, this.cool[0] - dt); this.cool[1] = Math.max(0, this.cool[1] - dt);
        if (this.rise !== undefined) { this.rise += dt; this.alpha = Math.min(1, this.rise / 1.6); if (Math.random() < 0.7) g.fx.burst('splash', this.x + (Math.random() - 0.5) * 30, this.y, 2, { speed: 40, up: 60 }); if (this.rise > 1.8) { this.rise = undefined; this.alpha = undefined; } }
        if (!sailing) { this.v *= Math.max(0, 1 - dt * 2); this.wake(dt); return; }
        if (input.pressed('up')) { this.sail = Math.min(3, this.sail + 1); g.audio.sfx('board'); }
        if (input.pressed('down')) { this.sail = Math.max(0, this.sail - 1); g.audio.sfx('board'); }
        const ax = input.axis();
        const stickTurn = input.stick && (input.stick.x || input.stick.y) && Math.hypot(input.stick.x, input.stick.y) > 0.5;
        if (stickTurn) { // touch/pad: steer towards the stick direction, sail by stick length
            const want = Math.atan2(input.stick.x, -input.stick.y);
            let d = ((want - this.h + Math.PI * 3) % TAU) - Math.PI;
            this.h = wrap(this.h + Math.max(-1.6 * dt, Math.min(1.6 * dt, d)));
            this.sail = Math.max(this.sail, 2);
        } else this.h = wrap(this.h + ax.x * dt * (0.7 + this.v / 60));
        const target = this.sail * 30 * (g.shantyT > 0 ? 1.35 : 1);
        this.v += (target - this.v) * Math.min(1, dt * 0.8);
        const fx = Math.sin(this.h), fy = -Math.cos(this.h);
        const nx = this.x + fx * this.v * dt, ny = this.y + fy * this.v * dt;
        if (!g.world.blocked(nx, ny, 10, true)) { this.x = nx; this.y = ny; }
        else { this.v *= 0.3; if (this.v > 20) { this.damage(4); } }
        this.wake(dt);
        if (input.pressed('attack') && !this.cool[0]) { this.broadside(-1, 40 * (g.save.origin === 'kanonier' ? 1.3 : 1), 'player'); this.cool[0] = 1.6; }
        if ((input.pressed('heavy') || input.pressed('item')) && !this.cool[1]) { this.broadside(1, 40 * (g.save.origin === 'kanonier' ? 1.3 : 1), 'player'); this.cool[1] = 1.6; }
    }
    damage(n) {
        const g = this.game;
        this.hull -= n; this.flash = 0.12; g.shake(3); g.audio.sfx('crash');
        g.fx.burst('dust', this.x, this.y, 10, { speed: 60, col: [0.4, 0.3, 0.22, 1] });
        if (this.hull <= 0) { this.hull = 0; g.shipSunk(); }
    }
    hurt(n) { this.damage(Math.round(n * 0.5)); return true; }
}

export class GhostShip extends Hull {
    constructor(game, route) {
        const [x, y] = route[0];
        super(game, x * 16, y * 16, 0, game.app.art.beasts.ghostship);
        this.route = route.map(([a, b]) => [a * 16, b * 16]); this.wp = 1;
        this.hp = this.maxHp = 160; this.cool = 2; this.name = 'Geisterschiff'; this.hurtR = 18; this.isGhost = true; this.home = { x: this.x, y: this.y };
    }
    get dead() { return this.hp <= 0; }
    update(dt) {
        const g = this.game, s = g.ship;
        this.flash = Math.max(0, this.flash - dt);
        if (this.dead) { this.sinkT = (this.sinkT || 0) + dt; this.alpha = Math.max(0, 1 - this.sinkT / 2); return; }
        this.cool -= dt;
        const chasing = g.sailing && s && dist(this, s) < 260;
        let tx, ty;
        if (chasing) {
            // keep a broadside distance, circling
            const d = dist(this, s), a = Math.atan2(this.x - s.x, -(this.y - s.y));
            const side = a + Math.PI / 2;
            tx = s.x + Math.sin(side) * 0 + (this.x - s.x) / d * 110; ty = s.y + (this.y - s.y) / d * 110;
            if (d > 140) { tx = s.x; ty = s.y; }
            // fire when the target is to the side
            const rel = Math.atan2(s.x - this.x, -(s.y - this.y)) - this.h;
            const side2 = Math.sin(rel);
            if (this.cool <= 0 && d < 170 && Math.abs(side2) > 0.75) { this.broadside(side2 > 0 ? 1 : -1, 14, 'enemy'); this.cool = 3.2; }
        } else {
            [tx, ty] = this.route[this.wp];
            if (Math.hypot(tx - this.x, ty - this.y) < 30) this.wp = (this.wp + 1) % this.route.length;
        }
        const want = Math.atan2(tx - this.x, -(ty - this.y));
        const d = ((want - this.h + Math.PI * 3) % TAU) - Math.PI;
        this.h = wrap(this.h + Math.max(-0.9 * dt, Math.min(0.9 * dt, d)));
        this.v += ((chasing ? 46 : 30) - this.v) * dt;
        const nx = this.x + Math.sin(this.h) * this.v * dt, ny = this.y - Math.cos(this.h) * this.v * dt;
        if (!g.world.blocked(nx, ny, 12, true)) { this.x = nx; this.y = ny; } else this.h += dt;
        this.wake(dt);
        if (Math.random() < 0.2) g.fx.p(this.x + (Math.random() - 0.5) * 30, this.y - 20 + (Math.random() - 0.5) * 20, { z: 0, vz: 10, life: 1.4, col: FXCOL.soul, glow: 1.2, wobble: 6 });
    }
    hurt(n) {
        const g = this.game;
        if (this.dead) return;
        this.hp -= n; this.flash = 0.1; g.fx.num(this.x, this.y - 30, n);
        g.fx.burst('dust', this.x, this.y - 6, 10, { speed: 60, col: [0.3, 0.45, 0.45, 1] });
        if (this.hp <= 0) { g.audio.sfx('death'); g.fx.rise('soul', this.x, this.y, 40, { w: 40 }); g.gainPearls(700, this); g.ui.banner('Geisterschiff versenkt', '', 'small', 2); }
    }
}

export class Shark {
    constructor(game, x, y) { this.game = game; this.x = x; this.y = y; this.home = { x, y }; this.a = Math.random() * TAU; this.hp = 60; this.sheet = game.app.art.beasts.shark; this.fx = 1; this.cool = 0; this.hurtR = 10; this.name = 'Knochenhai'; this.flash = 0; this.t = 0; }
    get dead() { return this.hp <= 0; }
    update(dt) {
        const g = this.game, s = g.ship;
        this.t += dt; this.flash = Math.max(0, this.flash - dt);
        if (this.dead) return;
        this.cool -= dt;
        let tx = this.home.x + Math.cos(this.a) * 60, ty = this.home.y + Math.sin(this.a) * 40;
        this.a += dt * 0.5;
        if (g.sailing && s && dist(this, s) < 160) { tx = s.x; ty = s.y; }
        const dx = tx - this.x, dy = ty - this.y, l = Math.hypot(dx, dy) || 1;
        const sp = 62;
        const nx = this.x + dx / l * sp * dt, ny = this.y + dy / l * sp * dt;
        if (!g.world.blocked(nx, ny, 6, true) || true) { const w = g.world; const T = (x, y) => w.g(Math.floor(x / 16), Math.floor(y / 16)) === 0; if (T(nx, ny)) { this.x = nx; this.y = ny; } }
        this.fx = dx;
        if (g.sailing && s && dist(this, s) < 18 && this.cool <= 0) { s.damage(8); this.cool = 2.5; g.fx.burst('splash', this.x, this.y, 12, { speed: 60, up: 60 }); }
    }
    hurt(n) { if (this.dead) return; this.hp -= n; this.flash = 0.1; if (this.hp <= 0) { this.game.gainPearls(90, this); this.game.fx.burst('bone', this.x, this.y, 12); } }
    draw(r, cam) {
        if (this.dead) return;
        const fr = this.sheet.anims.walk[Math.floor(this.t * 6) % 4];
        // only the fin and back break the surface: draw the upper half, translucent
        r.sprite({ ...fr, h: Math.floor(fr.h * 0.55) }, Math.round(this.x - this.sheet.ax - cam[0]), Math.round(this.y - 14 - cam[1]), { flip: this.fx < 0, flash: this.flash > 0, color: [1, 1, 1, 0.85] });
    }
    drawGlow(r, cam) { if (this.dead) return; const fr = this.sheet.anims.walk[Math.floor(this.t * 6) % 4]; r.sprite({ ...fr, h: Math.floor(fr.h * 0.55) }, Math.round(this.x - this.sheet.ax - cam[0]), Math.round(this.y - 14 - cam[1]), { tex: fr.gtex, flip: this.fx < 0 }); }
}
