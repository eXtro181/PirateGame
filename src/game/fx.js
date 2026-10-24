// Pooled visual effects and short light flashes.
import { Sprite } from '../core/sprite.js';

export class Fx {
    constructor(game) {
        this.game = game;
        this.pool = [];
        this.live = [];
        this.flashes = [];
        for (let i = 0; i < 4; i++) {
            const e = game.view.addOmni(0, 1, 0, [1, 0.8, 0.5], 0, 5);
            this.flashes.push({ e, t: 0, dur: 0.1, base: 0 });
        }
    }
    get(sheet) {
        let s = this.pool.find(p => p.sheet === sheet && !p.busy);
        if (!s) {
            s = { sheet, sprite: new Sprite(this.game.app, sheet, { y: 0.08, caster: false, anchor: 0.5 }), busy: false };
            this.pool.push(s);
        }
        s.busy = true;
        s.sprite.setVisible(true);
        s.sprite.entity.setLocalScale(1, 1, 1);
        s.sprite.entity.setEulerAngles(0, 0, 0);
        return s;
    }
    // play an animation once at (x, z)
    play(sheet, anim, x, z, { fps = 16, angle = 0, dur = null, scale = 1, y = null, vx = 0, vz = 0, fade = false } = {}) {
        const s = this.get(sheet);
        s.sprite.set(anim, true);
        s.sprite.setPos(x, z);
        s.sprite.entity.setEulerAngles(0, -angle * 180 / Math.PI, 0);
        s.sprite.entity.setLocalScale(scale, 1, scale);
        const frames = sheet.frames[anim].length;
        this.live.push({ s, anim, x, z, fps, t: 0, dur: dur ?? frames / fps, vx, vz, fade });
        return s;
    }
    flash(x, z, color, intensity = 3, dur = 0.12, range = 6) {
        const f = this.flashes.reduce((a, b) => (a.t / a.dur > b.t / b.dur ? a : b));
        f.e.light.color.set(color[0], color[1], color[2]);
        f.e.light.range = range;
        f.e.setLocalPosition(x, 1.0, z);
        f.t = 0; f.dur = dur; f.base = intensity;
        f.e.light.intensity = intensity;
    }
    update(dt) {
        for (let i = this.live.length - 1; i >= 0; i--) {
            const l = this.live[i];
            l.t += dt;
            l.x += l.vx * dt; l.z += l.vz * dt;
            l.s.sprite.update(dt, l.fps, false);
            l.s.sprite.setPos(l.x, l.z);
            if (l.fade) l.s.sprite.setVisible(l.t < l.dur * 0.6 || Math.floor(l.t * 16) % 2 === 0);
            if (l.t >= l.dur) { l.s.busy = false; l.s.sprite.setVisible(false); this.live.splice(i, 1); }
        }
        for (const f of this.flashes) {
            if (f.t < f.dur) { f.t += dt; f.e.light.intensity = f.base * Math.max(0, 1 - f.t / f.dur); }
            else f.e.light.intensity = 0;
        }
    }
}
