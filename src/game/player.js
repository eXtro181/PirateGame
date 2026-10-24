// The player on foot: Soulslike movement and combat (stamina, roll i-frames, parry, flasks).
import { Humanoid } from './figure.js';
import { WEAPONS } from './data.js';

const WALK = 3.9;
const ROLL_COST = 17, PARRY_COST = 9;

const SPELL_COST = 28;

export class Player {
    constructor(game, save) {
        this.game = game;
        this.makeFigure(save);
        this.x = 0; this.z = 0;
        this.fx = 0; this.fz = 1;
        this.vx = 0; this.vz = 0;
        this.state = 'idle';
        this.t = 0;
        this.hp = 100; this.st = 100;
        this.stDelay = 0;
        this.flasks = 3;
        this.combo = 0;
        this.queued = null;
        this.hitSet = new Set();
        this.riposteUntil = 0;
        this.ammo = 0;
        this.invuln = 0; // brief post-hit invulnerability
        this.radius = 0.24;
        this.mp = 0;
        // the light around the player: only a circle of the world is lit (light radius)
        this.glow = game.view.addOmni(0, 2.4, 0, [1.0, 0.86, 0.68], 4.2, 9, { shadows: true, res: 1024 });
    }
    get weapon() { return WEAPONS[this.game.save.weapon]; }
    get stats() { return this.game.derived; }
    get sprite() { return this.fig; }
    makeFigure(save) {
        this.fig?.destroy();
        this.fig = new Humanoid(this.game.app, this.game.M, { look: save.look, weapon: save.weapon, off: save.off });
    }
    setSave(save) { this.makeFigure(save); }
    place(x, z) { this.x = x; this.z = z; this.vx = this.vz = 0; this.state = 'idle'; this.t = 0; this.fig.setPos(x, this.game.terrain.ground(x, z), z); }
    face(x, z) { const l = Math.hypot(x, z); if (l > 0.01) { this.fx = x / l; this.fz = z / l; } }

    spend(cost) {
        if (this.st <= 0) return false;
        this.st -= cost;
        this.stDelay = 0.55;
        return true;
    }

    canAct() { return this.state === 'idle' || this.state === 'walk'; }

    update(dt) {
        const g = this.game, inp = g.input;
        this.t += dt;
        this.invuln = Math.max(0, this.invuln - dt);
        // stamina regeneration
        this.stDelay -= dt;
        if (this.stDelay <= 0 && this.state !== 'roll' && this.state !== 'attack') {
            const rate = this.state === 'drink' ? 20 : 52;
            this.st = Math.min(this.stats.maxSt, this.st + rate * dt);
        }
        const ax = inp.axis();
        const target = g.lockTarget;
        if (target && (!target.alive || target.state === 'pile' || Math.hypot(target.x - this.x, target.z - this.z) > 11)) g.lockTarget = null;

        switch (this.state) {
            case 'idle':
            case 'walk': {
                const moving = Math.hypot(ax.x, ax.z) > 0.15;
                this.vx = ax.x * WALK; this.vz = ax.z * WALK;
                if (g.lockTarget) this.face(g.lockTarget.x - this.x, g.lockTarget.z - this.z);
                else if (moving) this.face(ax.x, ax.z);
                this.state = moving ? 'walk' : 'idle';
                if (inp.pressed('roll') && this.spend(ROLL_COST)) {
                    if (moving) this.face(ax.x, ax.z);
                    this.state = 'roll'; this.t = 0; g.sfx('roll');
                } else if (inp.pressed('attack') && this.st > 0) this.startAttack(false);
                else if (inp.pressed('heavy') && this.st > 0) this.startAttack(true);
                else if (inp.pressed('parry') && this.spend(PARRY_COST)) { this.state = 'parry'; this.t = 0; g.sfx('parry'); }
                else if (inp.pressed('drink') || inp.pressed('belt1') || inp.pressed('belt2') || inp.pressed('belt3') || inp.pressed('belt4')) {
                    if (this.flasks > 0) { this.state = 'drink'; this.t = 0; this.drank = false; g.sfx('drink'); }
                    else g.toast('Keine Rumflaschen mehr.');
                } else if (inp.pressed('item')) this.useOffhand();
                else if (inp.pressed('spell')) this.castSpell();
                break;
            }
            case 'roll': {
                const d = 0.44;
                const k = Math.max(0, 1 - this.t / d);
                const sp = 8.2 * (0.35 + 0.65 * k);
                this.vx = this.fx * sp; this.vz = this.fz * sp;
                if (this.t > d) this.toIdle();
                break;
            }
            case 'attack': {
                const a = this.atk;
                this.vx *= Math.pow(0.001, dt); this.vz *= Math.pow(0.001, dt);
                if (g.lockTarget && this.t < a.wind) this.face(g.lockTarget.x - this.x, g.lockTarget.z - this.z);
                if (!a.lunged && this.t >= a.wind) {
                    a.lunged = true;
                    this.vx = this.fx * (a.heavy ? 4.5 : 3.2); this.vz = this.fz * (a.heavy ? 4.5 : 3.2);
                    g.slashFx(this, a.heavy);
                    g.sfx(a.heavy ? 'heavy' : 'swing');
                }
                if (this.t >= a.wind && this.t <= a.wind + a.act) {
                    g.meleeHit(this, a);
                }
                if (this.t > a.wind + a.act && (inp.pressed('attack') || inp.pressed('heavy'))) this.queued = inp.pressed('heavy') ? 'heavy' : 'light';
                if (this.t > a.wind + a.act + a.rec * 0.45 && this.queued && this.st > 0) {
                    const h = this.queued === 'heavy';
                    this.queued = null;
                    this.combo = (this.combo + 1) % 3;
                    this.startAttack(h, true);
                    break;
                }
                if (this.t > a.wind + a.act + a.rec) { this.combo = 0; this.toIdle(); }
                break;
            }
            case 'parry': {
                this.vx = this.vz = 0;
                if (this.t > 0.42) this.toIdle();
                break;
            }
            case 'drink': {
                this.vx = ax.x * 1.0; this.vz = ax.z * 1.0;
                if (!this.drank && this.t > 0.6) {
                    this.drank = true;
                    this.flasks--;
                    this.hp = Math.min(this.stats.maxHp, this.hp + 60);
                    g.healFx(this);
                }
                if (this.t > 1.05) this.toIdle();
                break;
            }
            case 'cast': {
                this.vx = this.vz = 0;
                if (!this.cast && this.t > 0.22) { this.cast = true; g.chainLightning(this); }
                if (this.t > 0.6) this.toIdle();
                break;
            }
            case 'hurt': {
                this.vx *= Math.pow(0.002, dt); this.vz *= Math.pow(0.002, dt);
                if (this.t > 0.32) this.toIdle();
                break;
            }
            case 'dead': {
                this.vx = this.vz = 0;
                break;
            }
            case 'frozen': case 'sit': { this.vx = this.vz = 0; break; }
        }
        this.move(dt);
        this.animate(dt);
    }

    toIdle() { this.state = 'idle'; this.t = 0; }

    startAttack(heavy, chained = false) {
        const w = this.weapon;
        const cost = heavy ? w.stam * 1.7 : w.stam;
        if (!this.spend(cost)) return;
        const ax = this.game.input.axis();
        if (!this.game.lockTarget && Math.hypot(ax.x, ax.z) > 0.2) this.face(ax.x, ax.z);
        const sp = w.speed;
        const riposte = this.game.time < this.riposteUntil;
        this.atk = {
            heavy,
            wind: (heavy ? 0.4 : 0.13) / sp,
            act: heavy ? 0.12 : 0.1,
            rec: (heavy ? 0.36 : 0.24) / sp,
            dmg: Math.round(this.stats.dmg * (heavy ? w.heavyMul : 1) * (riposte ? 2.6 : 1) * (1 + this.combo * 0.08)),
            range: w.range * (heavy ? 1.12 : 1),
            arc: heavy ? 1.35 : 1.1,
            poise: heavy ? 2 : 1,
            riposte,
            lunged: false,
            hit: new Set()
        };
        if (riposte) this.riposteUntil = 0;
        this.state = 'attack'; this.t = 0;
        this.vx = this.vz = 0;
    }

    castSpell() {
        const g = this.game;
        if (this.mp < SPELL_COST) { g.toast('Zu wenig Geist für den Kettenblitz. Raste an einem Feuer.'); return; }
        if (!this.spend(12)) return;
        this.mp -= SPELL_COST;
        this.state = 'cast'; this.t = 0; this.cast = false;
        const tgt = g.lockTarget || g.nearestEnemy(this.x, this.z, 7);
        if (tgt) this.face(tgt.x - this.x, tgt.z - this.z);
        g.sfx('charge');
    }

    useOffhand() {
        const g = this.game;
        if (g.save.off === 'none') { g.toast('Keine Zweitwaffe ausgerüstet.'); return; }
        if (this.ammo <= 0) { g.toast(g.save.off === 'pistol' ? 'Kein Pulver mehr. Raste an einem Feuer.' : 'Keine Gezeitenkraft mehr. Raste an einem Feuer.'); return; }
        const cost = g.save.off === 'pistol' ? 10 : 18;
        if (!this.spend(cost)) return;
        this.ammo--;
        g.shoot(this, g.save.off);
        this.state = 'hurt'; this.t = 0.12; // short recoil lock
    }

    move(dt) {
        const g = this.game;
        if (!this.vx && !this.vz) return;
        // sliding movement in small steps so fast rolls cannot tunnel or snag
        const steps = Math.max(1, Math.ceil(Math.hypot(this.vx, this.vz) * dt / 0.12));
        for (let i = 0; i < steps; i++) {
            const r = g.map.moveCircle(this.x, this.z, this.vx * dt / steps, this.vz * dt / steps, this.radius);
            this.x = r.x; this.z = r.z;
        }
    }

    animate(dt) {
        const g = this.game, f = this.fig;
        const st = this.state === 'attack' ? 'attack' : this.state;
        f.animate(dt, { state: st, t: this.t, speed: Math.hypot(this.vx, this.vz), atk: this.atk, combo: this.combo, hipsOffset: this.sitting ? 0 : 0 });
        f.setYaw(this.fx, this.fz);
        const y = g.terrain.ground(this.x, this.z);
        f.setPos(this.x, y, this.z);
        // flicker while invulnerable after a hit
        f.setVisible(g.mode === 'land' && (this.state === 'dead' || this.invuln <= 0 || Math.floor(this.invuln * 20) % 2 === 0));
        this.glow.setLocalPosition(this.x - 0.6, y + 2.4, this.z + 0.6);
        this.glow.enabled = g.mode === 'land';
        // spirit regenerates slowly
        this.mp = Math.min(this.stats.maxMp, this.mp + dt * 1.2);
    }

    isParrying() { return this.state === 'parry' && this.t > 0.02 && this.t < 0.24 * (this.weapon.parry || 1); }
    isInvulnerable() { return (this.state === 'roll' && this.t > 0.03 && this.t < 0.33) || this.invuln > 0 || this.state === 'dead'; }

    hurt(dmg, fromX, fromZ) {
        const g = this.game;
        this.hp -= dmg;
        const dx = this.x - fromX, dz = this.z - fromZ, l = Math.hypot(dx, dz) || 1;
        this.vx = dx / l * 5; this.vz = dz / l * 5;
        this.invuln = 0.5;
        g.shake(0.25, 0.18);
        g.sfx('hurt');
        g.bloodFx(this.x, this.z);
        if (this.hp <= 0) { this.hp = 0; this.state = 'dead'; this.t = 0; g.onPlayerDeath(); }
        else { this.state = 'hurt'; this.t = 0; }
    }
}
