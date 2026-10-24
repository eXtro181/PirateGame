// The character selection scene: a camp fire on a dark shore, far away from the playable map.
// Warm fire light from below with shadows, cold rim light from behind, mountains as a matte
// painting, layered fog, depth of field, flying sparks and a slowly "breathing" camera.
import { Entity, Color, Vec2, StandardMaterial, MeshInstance, BLEND_NORMAL, CULLFACE_NONE, Texture, PIXELFORMAT_SRGBA8, FILTER_LINEAR, ADDRESS_CLAMP_TO_EDGE } from 'playcanvas';
import { MeshBuilder, instanced, trs } from '../gfx/mesh.js';
import { Actor, HEROES } from '../game/actors.js';
import { NOISE } from '../gfx/noise.js';
import { Rng } from './defs.js';

export const CAMP = { x: -300, z: -300 };

function mattePainting() {
    const W = 2048, H = 768, c = document.createElement('canvas'); c.width = W; c.height = H;
    const g = c.getContext('2d');
    const sky = g.createLinearGradient(0, 0, 0, H);
    sky.addColorStop(0, '#05070b'); sky.addColorStop(0.55, '#0c141a'); sky.addColorStop(1, '#16222a');
    g.fillStyle = sky; g.fillRect(0, 0, W, H);
    // moon behind clouds
    const mg = g.createRadialGradient(W * 0.68, H * 0.22, 0, W * 0.68, H * 0.22, 260);
    mg.addColorStop(0, 'rgba(200,215,230,0.9)'); mg.addColorStop(0.08, 'rgba(170,190,210,0.55)'); mg.addColorStop(0.3, 'rgba(80,100,120,0.15)'); mg.addColorStop(1, 'rgba(0,0,0,0)');
    g.fillStyle = mg; g.fillRect(0, 0, W, H);
    const ridge = (base, amp, freq, col, seed) => {
        g.fillStyle = col; g.beginPath(); g.moveTo(0, H);
        for (let x = 0; x <= W; x += 6) {
            const n = NOISE.fbm(x * freq + seed, seed, 5) * amp + Math.abs(NOISE.n2(x * freq * 0.4 + seed, 3.3)) * amp * 1.4;
            g.lineTo(x, base - n);
        }
        g.lineTo(W, H); g.fill();
    };
    ridge(H * 0.62, 220, 0.002, '#121b21', 1);
    ridge(H * 0.72, 170, 0.003, '#0d1418', 7);
    ridge(H * 0.84, 120, 0.005, '#080c0f', 13);
    // fog bands
    for (let i = 0; i < 3; i++) {
        const y = H * (0.6 + i * 0.1), fg = g.createLinearGradient(0, y - 60, 0, y + 60);
        fg.addColorStop(0, 'rgba(60,80,85,0)'); fg.addColorStop(0.5, 'rgba(60,80,85,0.22)'); fg.addColorStop(1, 'rgba(60,80,85,0)');
        g.fillStyle = fg; g.fillRect(0, y - 60, W, 120);
    }
    return c;
}

export class Camp {
    constructor(app, stage, M, models, particles, lib) {
        Object.assign(this, { app, stage, M, models, particles, lib });
        this.root = new Entity('camp');
        this.root.setPosition(CAMP.x, 0, CAMP.z);
        app.root.addChild(this.root);
        this.root.enabled = false;
        this.time = 0;
        this.figures = [];
        this.build();
    }
    add(parent, mesh, mat, shadow = true) {
        const mi = new MeshInstance(mesh, mat); mi.castShadow = shadow; mi.receiveShadow = true;
        const e = new Entity(); e.addComponent('render', { meshInstances: [mi], castShadows: shadow }); parent.addChild(e); return e;
    }
    build() {
        const dev = this.app.graphicsDevice, M = this.M, R = this.root;
        const rng = new Rng(8);
        // ground: a bumpy disc of dirt and sand
        const g = new MeshBuilder();
        const N = 48, rings = 16, Rad = 22;
        const hAt = (x, z) => NOISE.fbm(x * 0.15, z * 0.15, 3) * 0.35 + Math.max(0, Math.hypot(x, z) - 10) * 0.08;
        const vid = [];
        for (let r = 0; r <= rings; r++) for (let a = 0; a <= N; a++) {
            const rr = (r / rings) ** 1.4 * Rad, ang = a / N * Math.PI * 2;
            const x = Math.cos(ang) * rr, z = Math.sin(ang) * rr;
            vid.push(g.v(x, hAt(x, z), z, 0, 1, 0, x * 0.35, z * 0.35));
        }
        for (let r = 0; r < rings; r++) for (let a = 0; a < N; a++) {
            const i = r * (N + 1) + a, j = i + N + 1;
            g.idx.push(i, i + 1, j, i + 1, j + 1, j);
        }
        // normals from neighbours
        for (let k = 0; k < g.count; k++) {
            const x = g.pos[k * 3], z = g.pos[k * 3 + 2], e = 0.2;
            const dx = (hAt(x + e, z) - hAt(x - e, z)) / (2 * e), dz = (hAt(x, z + e) - hAt(x, z - e)) / (2 * e);
            const l = Math.hypot(dx, 1, dz); g.nrm[k * 3] = -dx / l; g.nrm[k * 3 + 1] = 1 / l; g.nrm[k * 3 + 2] = -dz / l;
        }
        this.add(R, g.toMesh(dev), M.pbr('dirt', { tiling: 1 }), false);
        this.hAt = hAt;
        // fire: logs, stones, flames
        const fb = new MeshBuilder();
        for (let i = 0; i < 9; i++) { const a = i / 9 * Math.PI * 2; fb.at(Math.cos(a) * 0.75, hAt(0, 0) - 0.05, Math.sin(a) * 0.75, 0, i * 40).rock(0.2, 30 + i, 0.4, 8, 6); }
        this.add(R, fb.toMesh(dev), M.pbr('rock', { tiling: 2 }));
        const lb = new MeshBuilder();
        for (let i = 0; i < 5; i++) lb.at(0, hAt(0, 0) + 0.08, 0, 72, i * 72 + 10, 0).cylinder(0.08, 0.06, 0.9, 7);
        this.add(R, lb.toMesh(dev), M.pbr('bark', { tint: '#4a3a2a' }));
        const fire = new Entity('campfire'); fire.setLocalPosition(0, hAt(0, 0) + 0.15, 0); R.addChild(fire);
        this.particles.fire(fire, 'orange', 1.3);
        this.fireLight = new Entity('fire-light');
        this.fireLight.addComponent('light', { type: 'omni', color: new Color(1, 0.52, 0.2), intensity: 16, range: 14, castShadows: true, shadowResolution: 1024, shadowBias: 0.15, normalOffsetBias: 0.04 });
        this.fireLight.setLocalPosition(0, hAt(0, 0) + 0.9, 0);
        R.addChild(this.fireLight);
        // cold rim light from behind the characters (they sit between fire and mountains)
        this.rim = new Entity('rim');
        this.rim.addComponent('light', { type: 'spot', color: new Color(0.45, 0.6, 1), intensity: 30, range: 26, innerConeAngle: 20, outerConeAngle: 38, castShadows: false });
        this.rim.setLocalPosition(0, 6, -10);
        this.rim.lookAt(CAMP.x, 0.8, CAMP.z + 1.5);
        R.addChild(this.rim);
        this.rim.setLocalPosition(0, 6, -10);
        // spotlight for the selected character
        this.spot = new Entity('select-spot');
        this.spot.addComponent('light', { type: 'spot', color: new Color(1, 0.85, 0.65), intensity: 0, range: 12, innerConeAngle: 12, outerConeAngle: 26, castShadows: true, shadowResolution: 1024 });
        R.addChild(this.spot);
        // dead trees, boulders, a wrecked rowing boat
        const tb = new MeshBuilder();
        for (const [x, z, h] of [[-6, -4, 3.4], [7, -5.5, 4.2], [-9, 2, 2.8]]) {
            tb.at(x, hAt(x, z) - 0.2, z).tube([[0, 0, 0], [0.2, h * 0.5, 0.1], [-0.1, h, -0.2]], [0.22, 0.14, 0.04], 7);
            for (let k = 0; k < 4; k++) { const a = k * 1.7 + x, y = h * (0.45 + k * 0.12); tb.at(x, hAt(x, z) - 0.2, z).tube([[0.05, y, 0], [Math.cos(a) * 0.9, y + 0.5, Math.sin(a) * 0.9], [Math.cos(a) * 1.5, y + 0.6 + rng.range(-0.3, 0.3), Math.sin(a) * 1.5]], [0.07, 0.04, 0.01], 5); }
        }
        this.add(R, tb.toMesh(dev), M.pbr('bark', { tint: '#6a6058' }));
        const rb = new MeshBuilder();
        for (let i = 0; i < 9; i++) { const a = rng.range(0, 6.3), d = rng.range(5, 12); rb.at(Math.cos(a) * d, hAt(Math.cos(a) * d, Math.sin(a) * d) - 0.1, Math.sin(a) * d, 0, rng.range(0, 360)).rock(rng.range(0.4, 1.2), 50 + i, 0.45); }
        this.add(R, rb.toMesh(dev), M.pbr('rock', { tiling: 1 }));
        const boat = new MeshBuilder();
        boat.at(4.2, hAt(4.2, 2) + 0.15, 2.2, 0, 35, 160).lathe([[0, -1.2], [0.45, -0.8], [0.55, 0], [0.45, 0.8], [0.2, 1.2]], 10, { caps: false, squash: 0.6 });
        this.add(R, boat.toMesh(dev), M.pbr('wood', { tiling: 1.5, cull: CULLFACE_NONE }));
        // grass tufts around
        const mats = [];
        for (let i = 0; i < 120; i++) { const a = rng.range(0, 6.3), d = rng.range(2, 14); const x = Math.cos(a) * d, z = Math.sin(a) * d; mats.push(trs(x, hAt(x, z), z, rng.range(0, 360), rng.range(0.7, 1.3))); }
        this.models['tuft:0'].forEach(p => { const mi = instanced(dev, p.mesh, p.mat, mats, { castShadow: false }); const e = new Entity(); e.addComponent('render', { meshInstances: [mi] }); R.addChild(e); });
        // matte painting far behind, unlit
        const tex = new Texture(dev, { width: 2048, height: 768, format: PIXELFORMAT_SRGBA8, mipmaps: false, minFilter: FILTER_LINEAR, magFilter: FILTER_LINEAR, addressU: ADDRESS_CLAMP_TO_EDGE, addressV: ADDRESS_CLAMP_TO_EDGE });
        tex.setSource(mattePainting());
        const mm = new StandardMaterial();
        mm.diffuse = new Color(0, 0, 0); mm.emissiveMap = tex; mm.emissive = new Color(1, 1, 1); mm.emissiveIntensity = 2.6;
        mm.useLighting = false; mm.useFog = false; mm.useSkybox = false; mm.cull = CULLFACE_NONE; mm.update();
        const pb = new MeshBuilder();
        pb.quad([-60, -6, -38], [60, -6, -38], [60, 39, -38], [-60, 39, -38], [0, 0, 1, 1]);
        this.add(R, pb.toMesh(dev, { tangents: false }), mm, false);
        // fog layers between scene and painting
        this.fogMats = [];
        for (let i = 0; i < 3; i++) {
            const fm = new StandardMaterial();
            fm.diffuse = new Color(0, 0, 0); fm.emissive = new Color(0.16, 0.2, 0.21); fm.emissiveMap = this.M.noise;
            fm.opacityMap = this.M.noise; fm.opacityMapChannel = 'a'; fm.opacity = 0.32 - i * 0.06;
            fm.blendType = BLEND_NORMAL; fm.depthWrite = false; fm.useLighting = false; fm.useSkybox = false; fm.cull = CULLFACE_NONE;
            fm.emissiveMapTiling = fm.opacityMapTiling = new Vec2(3, 1);
            fm.update();
            const qb = new MeshBuilder();
            const z = -8 - i * 9, y0 = -0.2 + i * 0.4;
            qb.quad([-40, y0, z], [40, y0, z], [40, y0 + 5 + i * 2, z], [-40, y0 + 5 + i * 2, z], [0, 0, 1, 1]);
            this.add(R, qb.toMesh(dev, { tangents: false }), fm, false);
            this.fogMats.push(fm);
        }
        this.ambient = this.particles.ambient(R);
        this.ambient.setLocalPosition(0, 2, -4);
    }

    actor(p) { return new Actor(this.app, this.lib, this.M, { model: HEROES[p.look?.model || 0] || HEROES[0], tone: p.look?.tone || 0, weapon: p.weapon, off: p.off }); }
    // seats around the fire (local coordinates), facing the fire
    seat(i, n) {
        const a = -Math.PI * 0.5 + (i - (n - 1) / 2) * 0.66; // far side of the fire, facing camera and flames
        return { x: Math.cos(a) * 2.1, z: Math.sin(a) * 2.1 };
    }
    clearFigures() { for (const f of this.figures) f.fig.destroy(); this.figures = []; }
    // profiles as seated figures; null entries are skipped
    setProfiles(list) {
        this.clearFigures();
        const n = Math.min(list.length, 5);
        list.slice(0, n).forEach((p, i) => {
            const fig = this.actor(p);
            const s = this.seat(i, n);
            this.figures.push({ fig, home: s, pos: { ...s }, state: 'sit', t: Math.random() * 3 });
        });
        this.selected = -1;
        this.mode = 'select';
    }
    // creator: one figure standing next to the fire, updated live
    setPreview(spec) {
        this.clearFigures();
        const fig = this.actor(spec);
        this.figures.push({ fig, home: { x: 1.7, z: 0.5 }, pos: { x: 1.7, z: 0.5 }, state: 'idle', t: 0 });
        this.selected = 0;
        this.mode = 'create';
    }
    select(i) {
        this.selected = i;
        this.figures.forEach((f, k) => { f.state = k === i ? 'walk' : 'sit'; f.t = 0; });
    }
    enter() {
        this.root.enabled = true;
        const fr = this.stage.frame;
        fr.dof.enabled = this.stage.quality !== 'low';
        fr.dof.focusDistance = this.stage.portrait ? 13 : 7.2; fr.dof.focusRange = 3.5; fr.dof.blurRadius = 3; fr.dof.nearBlur = false;
        fr.update();
        this.active = true;
    }
    exit() {
        this.root.enabled = false;
        this.clearFigures();
        this.stage.frame.dof.enabled = false;
        this.stage.frame.update();
        this.active = false;
    }
    update(dt) {
        if (!this.active) return;
        this.time += dt;
        const t = this.time;
        // fire flicker (Perlin) and selection spotlight
        this.fireLight.light.intensity = 16 * (1 + NOISE.n1(t * 3) * 0.25 + NOISE.n1(t * 9 + 4) * 0.1);
        for (const fm of this.fogMats) { fm.opacityMapOffset = new Vec2((t * 0.01 * (1 + this.fogMats.indexOf(fm))) % 1, 0); fm.emissiveMapOffset = fm.opacityMapOffset; fm.update(); }
        const sel = this.figures[this.selected];
        const spotI = this.spot.light.intensity;
        this.spot.light.intensity = spotI + ((sel && this.mode === 'select' ? 45 : (this.mode === 'create' ? 25 : 0)) - spotI) * Math.min(1, dt * 3);
        this.figures.forEach((f, k) => {
            f.t += dt;
            let target = f.home;
            if (this.mode === 'select' && k === this.selected) target = { x: f.home.x + (f.home.x >= 0 ? 1.2 : -1.2), z: f.home.z + 1.3 }; // steps forward, past the fire
            const dx = target.x - f.pos.x, dz = target.z - f.pos.z, d = Math.hypot(dx, dz);
            let speed = 0;
            if (d > 0.05 && f.state !== 'sit') { const sp = Math.min(d * 3, 1.6); f.pos.x += dx / d * sp * dt; f.pos.z += dz / d * sp * dt; speed = sp; }
            const standing = this.mode === 'create' || k === this.selected;
            const st = standing ? (speed > 0.2 ? 'walk' : 'idle') : 'sit';
            f.fig.animate(dt, { state: st, t: f.t, speed });
            // seated ones look at the fire, the selected one at the camera
            if (standing && speed < 0.2) f.fig.setYaw(Math.sin(t * 0.3) * 0.3, 1);
            else if (standing) f.fig.setYaw(dx, dz);
            else f.fig.setYaw(-f.pos.x, -f.pos.z);
            f.fig.setPos(CAMP.x + f.pos.x, this.hAt(f.pos.x, f.pos.z), CAMP.z + f.pos.z);
            if (standing) this.spot.setPosition(CAMP.x + f.pos.x, 5.5, CAMP.z + f.pos.z + 2.5), this.spot.lookAt(CAMP.x + f.pos.x, 0.8, CAMP.z + f.pos.z);
        });
        // camera: low, looking over the fire at the group, gently breathing
        const cam = this.stage.camera;
        const ox = this.mode === 'create' ? 1.1 : 0;
        const back = this.stage.portrait ? 15 : 9.2;
        cam.setPosition(CAMP.x + ox + Math.sin(t * 0.21) * 0.15, (this.stage.portrait ? 3.2 : 2.3) + Math.sin(t * 0.33) * 0.06, CAMP.z + back);
        cam.lookAt(CAMP.x + ox + 0.3, 1.05, CAMP.z + 1.2);
    }
}
