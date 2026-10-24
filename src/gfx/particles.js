// Particle presets on top of the PlayCanvas particle system: fire, embers, smoke, ambient dust,
// fireflies and pooled one-shot bursts (sparks, blood, splash, heal, bone dust).
import { Entity, Curve, CurveSet, Vec3, BLEND_ADDITIVE, BLEND_NORMAL, EMITTERSHAPE_BOX, EMITTERSHAPE_SPHERE } from 'playcanvas';

const C = (...v) => new Curve(v);
const CS = (r, g, b) => new CurveSet([r, g, b]);

function emitter(app, parent, o) {
    const e = new Entity('particles');
    e.addComponent('particlesystem', {
        numParticles: o.n ?? 40,
        lifetime: o.life ?? 1,
        rate: o.rate ?? 0.05, rate2: o.rate2 ?? o.rate ?? 0.05,
        emitterShape: o.sphere ? EMITTERSHAPE_SPHERE : EMITTERSHAPE_BOX,
        emitterExtents: new Vec3(...(o.extents ?? [0.2, 0.1, 0.2])),
        emitterRadius: o.radius ?? 0.2,
        colorMap: o.tex,
        blendType: o.additive === false ? BLEND_NORMAL : BLEND_ADDITIVE,
        intensity: o.intensity ?? 1,
        lighting: !!o.lit,
        depthWrite: false,
        sort: 0,
        loop: o.loop ?? true,
        autoPlay: o.loop ?? true,
        preWarm: !!o.preWarm,
        startAngle: 0, startAngle2: 360,
        localSpace: !!o.local,
        velocityGraph: o.vel ?? new CurveSet([[0, 0], [0, 0.5], [0, 0]]),
        velocityGraph2: o.vel2 ?? o.vel ?? new CurveSet([[0, 0], [0, 0.5], [0, 0]]),
        radialSpeedGraph: o.radial ?? C(0, 0), radialSpeedGraph2: o.radial2 ?? o.radial ?? C(0, 0),
        rotationSpeedGraph: C(0, o.spin ?? 0), rotationSpeedGraph2: C(0, -(o.spin ?? 0)),
        scaleGraph: o.scale ?? C(0, 0.2, 1, 0.05), scaleGraph2: o.scale2 ?? o.scale ?? C(0, 0.2, 1, 0.05),
        colorGraph: o.color ?? CS([0, 1], [0, 1], [0, 1]),
        alphaGraph: o.alpha ?? C(0, 0, 0.15, 1, 1, 0), alphaGraph2: o.alpha2 ?? o.alpha ?? C(0, 0, 0.15, 1, 1, 0)
    });
    (parent || app.root).addChild(e);
    return e;
}

export class Particles {
    constructor(app, M) {
        this.app = app; this.M = M;
        this.pools = {};
    }
    // looping flames; kind 'teal' (saber fire), 'orange' (wreck, torches, campfire)
    fire(parent, kind = 'orange', size = 1) {
        const col = kind === 'teal'
            ? CS([0, 0.45, 0.5, 0.12, 1, 0.05], [0, 0.95, 0.5, 0.75, 1, 0.35], [0, 0.85, 0.5, 0.6, 1, 0.3])
            : CS([0, 1, 0.4, 1, 1, 0.6], [0, 0.85, 0.4, 0.45, 1, 0.12], [0, 0.5, 0.4, 0.12, 1, 0.04]);
        const f = emitter(this.app, parent, {
            n: Math.round(40 * size), life: 0.7, rate: 0.02 / size, tex: this.M.flame, intensity: 1.8,
            extents: [0.35 * size, 0.05, 0.35 * size], color: col,
            vel: new CurveSet([[0, -0.1], [0, 1.1 * size, 1, 1.9 * size], [0, -0.1]]), vel2: new CurveSet([[0, 0.1], [0, 1.6 * size, 1, 2.4 * size], [0, 0.1]]),
            scale: C(0, 0.4 * size, 0.6, 0.28 * size, 1, 0.04), scale2: C(0, 0.52 * size, 0.6, 0.36 * size, 1, 0.06),
            alpha: C(0, 0, 0.12, 0.7, 0.7, 0.35, 1, 0), spin: 40
        });
        const emb = emitter(this.app, parent, {
            n: Math.round(24 * size), life: 2.2, rate: 0.09 / size, tex: this.M.dot, intensity: 5,
            extents: [0.3 * size, 0.2, 0.3 * size], color: kind === 'teal' ? CS([0, 0.5], [0, 1], [0, 0.9]) : CS([0, 1], [0, 0.55], [0, 0.2]),
            vel: new CurveSet([[0, -0.4, 1, -0.6], [0, 1.2, 1, 2.2], [0, -0.4, 1, 0.5]]), vel2: new CurveSet([[0, 0.4, 1, 0.6], [0, 1.8, 1, 3], [0, 0.4, 1, -0.5]]),
            scale: C(0, 0.05, 1, 0.02), alpha: C(0, 0, 0.1, 1, 0.8, 0.8, 1, 0)
        });
        emb.setLocalPosition(0, 0.3, 0);
        const smoke = emitter(this.app, parent, {
            n: Math.round(14 * size), life: 3.2, rate: 0.28, tex: this.M.smoke, additive: false, lit: true,
            extents: [0.25, 0.1, 0.25], color: CS([0, 0.12], [0, 0.12], [0, 0.13]),
            vel: new CurveSet([[0, 0], [0, 0.8, 1, 1.4], [0, 0]]), vel2: new CurveSet([[0, 0.3], [0, 1.1, 1, 1.8], [0, 0.2]]),
            scale: C(0, 0.4 * size, 1, 1.5 * size), alpha: C(0, 0, 0.3, 0.35, 1, 0), spin: 20
        });
        smoke.setLocalPosition(0, 0.9 * size, 0);
        return [f, emb, smoke];
    }
    // slowly drifting dust / ash motes around the camera target
    ambient(parent) {
        return emitter(this.app, parent, {
            n: 160, life: 9, rate: 0.06, tex: this.M.dot, intensity: 0.7, preWarm: true,
            extents: [26, 6, 26], color: CS([0, 0.7], [0, 0.72], [0, 0.65]),
            vel: new CurveSet([[0, -0.15], [0, -0.08], [0, -0.1]]), vel2: new CurveSet([[0, 0.25], [0, 0.12], [0, 0.2]]),
            scale: C(0, 0.04), scale2: C(0, 0.08), alpha: C(0, 0, 0.2, 0.55, 0.8, 0.55, 1, 0)
        });
    }
    fireflies(parent, extents) {
        return emitter(this.app, parent, {
            n: 30, life: 6, rate: 0.25, tex: this.M.dot, intensity: 6, preWarm: true,
            extents, color: CS([0, 0.65], [0, 1], [0, 0.35]),
            vel: new CurveSet([[0, -0.3, 0.5, 0.3, 1, -0.2], [0, -0.05, 1, 0.1], [0, 0.3, 0.5, -0.3, 1, 0.2]]),
            vel2: new CurveSet([[0, 0.3, 0.5, -0.2, 1, 0.3], [0, 0.1, 1, -0.05], [0, -0.3, 0.5, 0.2, 1, -0.3]]),
            scale: C(0, 0.07), alpha: C(0, 0, 0.2, 1, 0.4, 0.2, 0.6, 1, 1, 0)
        });
    }
    // ----------------------------------------------------------------- one-shot bursts (pooled)
    burstDef(name) {
        const M = this.M;
        switch (name) {
            case 'spark': return { n: 26, life: 0.4, rate: 0.001, tex: M.dot, intensity: 9, loop: false, sphere: true, radius: 0.05,
                color: CS([0, 1], [0, 0.8, 1, 0.4], [0, 0.45, 1, 0.1]), radial: C(0, 6, 1, 0.5), radial2: C(0, 10, 1, 1),
                vel: new CurveSet([[0, 0], [0, 1, 1, -4], [0, 0]]), scale: C(0, 0.06, 1, 0.01), alpha: C(0, 1, 1, 0) };
            case 'blood': return { n: 30, life: 0.7, rate: 0.001, tex: M.smoke, additive: false, lit: true, loop: false, sphere: true, radius: 0.08,
                color: CS([0, 0.32], [0, 0.02], [0, 0.02]), radial: C(0, 2.5, 1, 0.2), radial2: C(0, 4.5, 1, 0.3),
                vel: new CurveSet([[0, 0], [0, 1.5, 1, -6], [0, 0]]), scale: C(0, 0.12, 1, 0.06), alpha: C(0, 1, 0.7, 0.9, 1, 0) };
            case 'splash': return { n: 40, life: 0.9, rate: 0.001, tex: M.smoke, additive: false, lit: true, loop: false, extents: [0.8, 0.1, 0.8],
                color: CS([0, 0.6], [0, 0.68], [0, 0.7]), vel: new CurveSet([[0, -1.5], [0, 4, 1, -6], [0, -1.5]]), vel2: new CurveSet([[0, 1.5], [0, 6, 1, -6], [0, 1.5]]),
                scale: C(0, 0.2, 1, 0.45), alpha: C(0, 0.8, 1, 0) };
            case 'smoke': return { n: 18, life: 1.8, rate: 0.001, tex: M.smoke, additive: false, lit: true, loop: false, sphere: true, radius: 0.2,
                color: CS([0, 0.35], [0, 0.34], [0, 0.32]), radial: C(0, 1.2, 1, 0.1), vel: new CurveSet([[0, 0], [0, 0.6], [0, 0]]),
                scale: C(0, 0.4, 1, 1.4), alpha: C(0, 0.6, 1, 0), spin: 30 };
            case 'heal': return { n: 40, life: 1.2, rate: 0.001, tex: M.dot, intensity: 5, loop: false, extents: [0.6, 0.2, 0.6],
                color: CS([0, 1], [0, 0.8], [0, 0.35]), vel: new CurveSet([[0, 0], [0, 1.4, 1, 2.4], [0, 0]]), scale: C(0, 0.09, 1, 0.02), alpha: C(0, 0, 0.2, 1, 1, 0) };
            case 'bone': return { n: 24, life: 1.0, rate: 0.001, tex: M.smoke, additive: false, lit: true, loop: false, sphere: true, radius: 0.3,
                color: CS([0, 0.55], [0, 0.52], [0, 0.45]), radial: C(0, 1.5, 1, 0.1), vel: new CurveSet([[0, 0], [0, 0.6, 1, -1], [0, 0]]),
                scale: C(0, 0.25, 1, 0.6), alpha: C(0, 0.6, 1, 0) };
            case 'pearl': return { n: 30, life: 1.0, rate: 0.001, tex: M.dot, intensity: 6, loop: false, sphere: true, radius: 0.2,
                color: CS([0, 0.5], [0, 0.9], [0, 1]), radial: C(0, 1.5, 1, 0), vel: new CurveSet([[0, 0], [0, 2.5, 1, 1], [0, 0]]), scale: C(0, 0.08, 1, 0.02), alpha: C(0, 1, 1, 0) };
            case 'zap': return { n: 30, life: 0.35, rate: 0.001, tex: M.dot, intensity: 14, loop: false, sphere: true, radius: 0.1,
                color: CS([0, 0.7], [0, 0.85], [0, 1]), radial: C(0, 7, 1, 0.5), vel: new CurveSet([[0, 0], [0, 0, 1, -3], [0, 0]]), scale: C(0, 0.07, 1, 0.01), alpha: C(0, 1, 1, 0) };
            case 'rise': return { n: 60, life: 2.2, rate: 0.001, tex: M.smoke, additive: false, lit: true, loop: false, extents: [2.6, 0.2, 1.2],
                color: CS([0, 0.62], [0, 0.7], [0, 0.72]), vel: new CurveSet([[0, -0.6], [0, 2.5, 1, -1], [0, -0.6]]), vel2: new CurveSet([[0, 0.6], [0, 4, 1, -1], [0, 0.6]]),
                scale: C(0, 0.4, 1, 1.2), alpha: C(0, 0.7, 1, 0) };
        }
        return null;
    }
    burst(name, x, y, z) {
        const pool = this.pools[name] || (this.pools[name] = []);
        let e = pool.find(p => !p.particlesystem.isPlaying?.() && p._t + p._life < performance.now() / 1000);
        if (!e) {
            const d = this.burstDef(name);
            e = emitter(this.app, null, { ...d, loop: false });
            e._life = d.life + 0.1;
            pool.push(e);
        }
        e._t = performance.now() / 1000;
        e.setPosition(x, y, z);
        e.particlesystem.reset();
        e.particlesystem.play();
        return e;
    }
}
