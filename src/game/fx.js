// Visual effects: weapon trails, light flashes, chain lightning, projectiles, particle bursts, decals.
import { Entity, MeshInstance, Mesh, Color, PRIMITIVE_TRIANGLES } from 'playcanvas';
import { MeshBuilder } from '../gfx/mesh.js';

// a dynamic ribbon mesh that can be rebuilt every frame (lightning bolts, slash arcs)
class DynamicRibbon {
    constructor(app, mat, maxPts = 64) {
        this.mesh = new Mesh(app.graphicsDevice);
        this.maxPts = maxPts;
        this.mi = new MeshInstance(this.mesh, mat);
        this.mi.castShadow = false;
        this.e = new Entity('ribbon');
        this.e.addComponent('render', { meshInstances: [this.mi], castShadows: false, receiveShadows: false });
        app.root.addChild(this.e);
        this.e.enabled = false;
    }
    // strips: array of { pts: [[x,y,z]...], w: [..] } – camera-facing ribbons (camera dir given)
    set(strips, camDir) {
        const b = new MeshBuilder();
        for (const s of strips) b.ribbon(s.pts, s.w, camDir);
        if (!b.idx.length) { this.e.enabled = false; return; }
        this.mesh.setPositions(b.pos); this.mesh.setNormals(b.nrm); this.mesh.setUvs(0, b.uv); this.mesh.setIndices(b.idx);
        this.mesh.update(PRIMITIVE_TRIANGLES);
        this.e.enabled = true;
    }
}

export class Fx {
    constructor(game) {
        this.game = game;
        const app = game.app, M = game.M;
        this.app = app; this.M = M;
        this.flashes = [];
        for (let i = 0; i < 6; i++) this.flashes.push({ e: game.view.addOmni(0, 1, 0, [1, 0.8, 0.5], 0, 5), t: 1, dur: 1, base: 0 });
        this.trails = [];
        this.bolts = [];
        this.boltCore = new DynamicRibbon(app, M.glow('#e8f0ff', 10));
        this.boltGlow = new DynamicRibbon(app, M.glow('#3a6cff', 3.2, M.dot));
        this.slashMat = M.glow('#ffe0b0', 1.6, M.dot);
        this.heavyMat = M.glow('#ff9a50', 2.4, M.dot);
        this.balls = [];
        this.ballMesh = (() => { const b = new MeshBuilder(); b.sphere(0.09, 8, 6); return b.toMesh(app.graphicsDevice); })();
    }
    flash(x, y, z, color, intensity = 3, dur = 0.12, range = 6) {
        const f = this.flashes.reduce((a, b) => (a.t / a.dur > b.t / b.dur ? a : b));
        f.e.light.color = new Color(color[0], color[1], color[2]);
        f.e.light.range = range;
        f.e.setLocalPosition(x, y, z);
        f.t = 0; f.dur = dur; f.base = intensity;
        f.e.light.intensity = intensity;
        return f;
    }
    burst(name, x, y, z) { return this.game.particles.burst(name, x, y, z); }

    // weapon arc in front of the attacker: a glowing crescent that fades fast
    slash(x, y, z, yaw, heavy, flip) {
        const r = new DynamicRibbon(this.app, heavy ? this.heavyMat : this.slashMat);
        const pts = [], w = [];
        const a0 = flip ? 1.2 : -1.2, a1 = -a0;
        for (let i = 0; i <= 14; i++) {
            const t = i / 14, a = yaw + a0 + (a1 - a0) * t;
            const rad = heavy ? 1.3 : 1.1;
            pts.push([x + Math.sin(a) * rad, y + (heavy ? 1.4 - t * 1.0 : 0.95 + Math.sin(t * Math.PI) * 0.15), z + Math.cos(a) * rad]);
            w.push(Math.sin(t * Math.PI) * (heavy ? 0.32 : 0.22));
        }
        r.set([{ pts, w }], [0, 1, 0]);
        this.trails.push({ r, t: 0, dur: heavy ? 0.22 : 0.16 });
    }

    // chain lightning between points: jagged path recomputed every frame (midpoint displacement)
    lightning(points, dur = 0.35) {
        this.bolts.push({ points, t: 0, dur });
        for (let i = 1; i < points.length; i++) {
            const p = points[i];
            this.flash(p[0], p[1] + 0.5, p[2], [0.6, 0.75, 1], 40, 0.09, 10); // 1–2 frames, blinding blue-white
            this.burst('zap', p[0], p[1], p[2]);
        }
    }
    jag(a, b, depth, out, amp) {
        if (depth === 0) { out.push(b); return; }
        const m = [(a[0] + b[0]) / 2 + (Math.random() - 0.5) * amp, (a[1] + b[1]) / 2 + (Math.random() - 0.5) * amp * 0.6, (a[2] + b[2]) / 2 + (Math.random() - 0.5) * amp];
        this.jag(a, m, depth - 1, out, amp / 2);
        this.jag(m, b, depth - 1, out, amp / 2);
    }

    // projectile visuals (pistol balls, cannonballs, tide bolts)
    ball(kind) {
        let b = this.balls.find(x => !x.busy && x.kind === kind);
        if (!b) {
            const mat = kind === 'tide' ? this.M.glow('#5fd8ff', 6) : kind === 'pistol' ? this.M.glow('#ffb060', 6) : this.M.pbr('metal', { metal: 0.9 });
            const e = new Entity('ball');
            const mi = new MeshInstance(this.ballMesh, mat); mi.castShadow = kind === 'cannon';
            e.addComponent('render', { meshInstances: [mi] });
            if (kind === 'cannon') e.setLocalScale(1.6, 1.6, 1.6);
            this.app.root.addChild(e);
            b = { e, kind, busy: false };
            this.balls.push(b);
        }
        b.busy = true; b.e.enabled = true;
        return b;
    }
    free(b) { b.busy = false; b.e.enabled = false; }

    update(dt) {
        for (const f of this.flashes) {
            if (f.t >= f.dur) { f.e.light.intensity = 0; continue; }
            f.t += dt;
            f.e.light.intensity = f.base * Math.max(0, 1 - f.t / f.dur);
        }
        for (let i = this.trails.length - 1; i >= 0; i--) {
            const tr = this.trails[i];
            tr.t += dt;
            const k = 1 - tr.t / tr.dur;
            tr.r.e.setLocalScale(1, 1, 1);
            tr.r.mi.material = tr.r.mi.material; // material shared; fade via scale of width
            if (k <= 0) { tr.r.e.destroy(); this.trails.splice(i, 1); }
        }
        // lightning
        const cam = this.game.stage.camera.forward;
        const strips = [], glow = [];
        for (let i = this.bolts.length - 1; i >= 0; i--) {
            const b = this.bolts[i];
            b.t += dt;
            if (b.t > b.dur) { this.bolts.splice(i, 1); continue; }
            for (let k = 0; k < b.points.length - 1; k++) {
                const path = [b.points[k]];
                this.jag(b.points[k], b.points[k + 1], 5, path, 1.1);
                strips.push({ pts: path, w: path.map(() => 0.045) });
                glow.push({ pts: path, w: path.map(() => 0.3) });
                if (Math.random() < 0.35) { // a short side branch
                    const j = 2 + Math.floor(Math.random() * (path.length - 4));
                    const p = path[j], end = [p[0] + (Math.random() - 0.5) * 1.6, p[1] + (Math.random() - 0.5) * 0.8, p[2] + (Math.random() - 0.5) * 1.6];
                    const br = [p]; this.jag(p, end, 3, br, 0.5);
                    strips.push({ pts: br, w: br.map(() => 0.03) });
                }
            }
        }
        const cd = [-cam.x, -cam.y, -cam.z];
        this.boltCore.set(strips, cd);
        this.boltGlow.set(glow, cd);
    }
}
