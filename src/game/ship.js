// The player's sloop "Möwenschrei": sails with the wind, fires broadsides, fights whirlpools.
import { Sprite } from '../core/sprite.js';

const SAIL_SPEED = [0, 2.8, 5.4];

export class Ship {
    constructor(game) {
        this.game = game;
        this.sprite = new Sprite(game.app, game.art.ship, { y: 0.025, anchor: 0.5 });
        this.x = 0; this.z = 0;
        this.heading = 0; // radians clockwise from screen-up
        this.speed = 0;
        this.sail = 0;
        this.hull = 100; this.maxHull = 100;
        this.reload = [0, 0];
        this.dx = 0; this.dz = 0; // drift from currents
        this.spin = 0;
        this.lantern = game.view.addOmni(0, 2.4, 0, [1.0, 0.75, 0.4], 0.5, 6, { flicker: 0.1 });
    }
    get fwd() { return { x: Math.sin(this.heading), z: -Math.cos(this.heading) }; }
    place(x, z, heading) { this.x = x; this.z = z; this.heading = heading; this.speed = 0; this.sail = 0; this.dx = this.dz = 0; this.sync(); }

    update(dt, controlled) {
        const g = this.game, inp = g.input;
        if (controlled && this.hull > 0) {
            if (inp.pressed('up')) this.sail = Math.min(2, this.sail + 1);
            if (inp.pressed('down')) this.sail = Math.max(0, this.sail - 1);
            const turn = (inp.held('right') ? 1 : 0) - (inp.held('left') ? 1 : 0) + (inp.stick ? inp.stick.x : 0);
            const steer = 0.5 + Math.min(1, this.speed / 3) * 0.9;
            this.heading += Math.max(-1, Math.min(1, turn)) * steer * dt;
            if (inp.pressed('attack')) this.fire(-1);
            if (inp.pressed('item')) this.fire(1);
        }
        // wind factor: best with the wind from behind or across, weak against it
        const wa = g.wind.angle;
        const rel = Math.cos(this.heading - wa);
        const wf = 0.35 + 0.65 * Math.max(0, (rel + 0.35) / 1.35);
        const target = SAIL_SPEED[this.sail] * wf * g.wind.strength;
        this.speed += (target - this.speed) * Math.min(1, dt * 0.8);
        this.reload[0] = Math.max(0, this.reload[0] - dt);
        this.reload[1] = Math.max(0, this.reload[1] - dt);

        // whirlpools pull and spin the ship
        this.dx *= Math.pow(0.4, dt); this.dz *= Math.pow(0.4, dt);
        this.inWhirl = 0;
        for (const w of g.map.whirlpools) {
            const ddx = w.x - this.x, ddz = w.z - this.z, l = Math.hypot(ddx, ddz);
            if (l < w.r) {
                const k = 1 - l / w.r;
                this.inWhirl = Math.max(this.inWhirl, k);
                const pull = 1.2 + k * k * 6.5;
                this.dx += (ddx / l * pull + (-ddz / l) * pull * 1.1) * dt;
                this.dz += (ddz / l * pull + (ddx / l) * pull * 1.1) * dt;
                this.heading += k * k * 1.6 * dt;
                if (l < 2.6) { this.damage(16 * dt, true); }
            }
        }
        const f = this.fwd;
        const nx = this.x + (f.x * this.speed + this.dx) * dt, nz = this.z + (f.z * this.speed + this.dz) * dt;
        if (!g.map.blockedSail(nx, nz)) { this.x = nx; this.z = nz; }
        else {
            if (this.speed > 2.5) { this.damage(this.speed * 2.5); g.sfx('crash'); g.shake(0.3, 0.2); }
            this.speed = -0.6; this.dx = -this.dx * 0.5; this.dz = -this.dz * 0.5;
            if (!g.map.blockedSail(this.x - f.x * 0.2, this.z - f.z * 0.2)) { this.x -= f.x * 0.2; this.z -= f.z * 0.2; }
        }
        this.sync();
    }

    sync() {
        this.sprite.set(`l${this.sail}`);
        this.sprite.setPos(this.x, this.z);
        this.sprite.entity.setEulerAngles(0, -this.heading * 180 / Math.PI, 0);
        const f = this.fwd;
        this.lantern.setLocalPosition(this.x - f.x * 1.6, 2.4, this.z - f.z * 1.6);
    }

    fire(side) { // side -1 = port (left), 1 = starboard (right)
        const g = this.game;
        const i = side < 0 ? 0 : 1;
        if (this.reload[i] > 0 || this.hull <= 0) return;
        const mul = g.originData.broadside || 1;
        this.reload[i] = 2.6 / mul;
        const f = this.fwd;
        const px = -f.z * side, pz = f.x * side; // perpendicular
        for (let k = -1; k <= 1; k++) {
            const ox = this.x + f.x * k * 0.7 + px * 0.6, oz = this.z + f.z * k * 0.7 + pz * 0.6;
            g.spawnBall(ox, oz, px * 12 + f.x * this.speed + (Math.random() - 0.5), pz * 12 + f.z * this.speed + (Math.random() - 0.5), Math.round(26 * mul), k * 0.06);
            g.smokeFx(ox + px * 0.4, oz + pz * 0.4);
        }
        g.sfx('cannon');
        g.shake(0.18, 0.15);
    }

    damage(n, quiet = false) {
        if (this.hull <= 0) return;
        this.hull = Math.max(0, this.hull - n);
        if (!quiet) this.game.shake(0.35, 0.2);
        if (this.hull <= 0) this.game.onShipSunk();
    }
}
