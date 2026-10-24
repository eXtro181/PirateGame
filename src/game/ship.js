// The player's sloop "Möwenschrei": a real 3D ship that sails with the wind, fires broadsides,
// fights whirlpools – and rises from the deep whenever its captain reaches a shore.
import { buildShipModel, SHIP_LEN } from './shipmodel.js';

const SAIL_SPEED = [0, 4.8, 9.2];
const SAIL_SHOW = [0.12, 0.6, 1];
export const SHIP_R = 1.25; // collision radius against coasts
const DEPTH = -4.2;

export class Ship {
    constructor(game) {
        this.game = game;
        const model = buildShipModel(game.app);
        this.entity = model.root; this.body = model.body; this.sails = model.sails; this.lanternPos = model.lanternPos;
        game.app.root.addChild(this.entity);
        this.len = SHIP_LEN;
        this.sailShow = 0.12;
        this.state = 'surfaced'; // surfaced | rising | sinking | submerged
        this.riseT = 1;
        this.t = 0;
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
    place(x, z, heading) { this.x = x; this.z = z; this.heading = heading; this.speed = 0; this.sail = 0; this.dx = this.dz = 0; this.state = 'surfaced'; this.riseT = 1; this.sync(); }
    get ready() { return this.state === 'surfaced' && this.hull > 0; }
    // summon from the depths at (x, z)
    rise(x, z, heading) {
        this.place(x, z, heading);
        this.state = 'rising'; this.riseT = 0;
        this.sync();
    }
    dive() { if (this.state === 'surfaced') { this.state = 'sinking'; this.riseT = 1; } }

    update(dt, controlled) {
        const g = this.game, inp = g.input;
        if (controlled && this.hull > 0) {
            if (inp.pressed('up')) this.sail = Math.min(2, this.sail + 1);
            if (inp.pressed('down')) this.sail = Math.max(0, this.sail - 1);
            const turn = (inp.held('right') ? 1 : 0) - (inp.held('left') ? 1 : 0) + (inp.stick ? inp.stick.x : 0);
            const steer = 0.8 + Math.min(1, this.speed / 4) * 1.1;
            const h0 = this.heading;
            this.heading += Math.max(-1, Math.min(1, turn)) * steer * dt;
            if (g.map.blockedSail(this.x, this.z, SHIP_R, this.heading) && !g.map.blockedSail(this.x, this.z, SHIP_R, h0)) this.heading = h0;
            if (inp.pressed('attack')) this.fire(-1);
            if (inp.pressed('item')) this.fire(1);
        }
        // wind factor: best with the wind from behind or across, weak against it
        const wa = g.wind.angle;
        const rel = Math.cos(this.heading - wa);
        const wf = 0.35 + 0.65 * Math.max(0, (rel + 0.35) / 1.35);
        const target = SAIL_SPEED[this.sail] * wf * g.wind.strength;
        this.speed += (target - this.speed) * Math.min(1, dt * 1.4);
        this.t += dt;
        if (this.state === 'rising') { this.riseT = Math.min(1, this.riseT + dt / 2.2); if (this.riseT >= 1) this.state = 'surfaced'; }
        else if (this.state === 'sinking') { this.riseT = Math.max(0, this.riseT - dt / 2.6); if (this.riseT <= 0) this.state = 'submerged'; }
        if (this.state !== 'surfaced' && this.state !== 'rising') { this.speed = 0; this.sync(); return; }
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
        if (!g.map.blockedSail(nx, nz, SHIP_R, this.heading)) { this.x = nx; this.z = nz; }
        else {
            if (this.speed > 3.5) { this.damage(this.speed * 2.5); g.sfx('crash'); g.shake(0.3, 0.2); }
            this.speed = -0.8; this.dx = -this.dx * 0.5; this.dz = -this.dz * 0.5;
            if (!g.map.blockedSail(this.x - f.x * 0.2, this.z - f.z * 0.2, SHIP_R, this.heading)) { this.x -= f.x * 0.2; this.z -= f.z * 0.2; }
        }
        this.sync();
    }

    sync() {
        // sails unfurl smoothly with the sail level
        this.sailShow += (SAIL_SHOW[this.sail] - this.sailShow) * 0.12;
        for (const e of this.sails) e.setLocalScale(1, this.sailShow, 1 + (this.sailShow - 0.5) * 0.3);
        const k = this.riseT, ease = 1 - Math.pow(1 - k, 3);
        const y = DEPTH * (1 - ease);
        const sea = this.inWhirl || 0;
        this.entity.setPosition(this.x, y, this.z);
        this.entity.setEulerAngles(0, -this.heading * 180 / Math.PI, 0);
        // bobbing, rolling with speed and turning, nose-up while breaching
        const t = this.t;
        const bob = Math.sin(t * 1.7) * 0.05 + Math.sin(t * 2.9) * 0.02;
        const roll = Math.sin(t * 1.1) * 2.2 + sea * Math.sin(t * 5) * 6;
        const pitch = Math.sin(t * 1.4 + 1) * 1.2 + (this.state === 'rising' ? (1 - ease) * 22 : 0) - (this.state === 'sinking' ? (1 - ease) * 14 : 0);
        this.body.setLocalPosition(0, bob, 0);
        this.body.setLocalEulerAngles(pitch, 0, roll);
        this.entity.enabled = this.state !== 'submerged';
        const f = this.fwd, lp = this.lanternPos;
        this.lantern.setLocalPosition(this.x - f.x * lp[2], lp[1] + y, this.z - f.z * lp[2]);
        this.lantern.light.enabled = this.state !== 'submerged';
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
            const ox = this.x + f.x * k * 1.1 + px * 0.95, oz = this.z + f.z * k * 1.1 + pz * 0.95;
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
