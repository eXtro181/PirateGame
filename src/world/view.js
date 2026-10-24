// Turns the tile map into a lit 3D world: splat-blended terrain with cliffs, water with foam,
// instanced props and scatter, modular ruins, decals, fires and their lights, whirlpools, ambience.
import {
    Entity, Color, Vec2, Mat4, StandardMaterial, MeshInstance, LIGHTFALLOFF_INVERSESQUARED,
    SHADOW_PCF3_32F, SHADOWUPDATE_REALTIME, BLEND_NORMAL, BLEND_ADDITIVE, CULLFACE_NONE
} from 'playcanvas';
import { MeshBuilder, instanced, trs } from '../gfx/mesh.js';
import { NOISE } from '../gfx/noise.js';
import { T, Rng } from './defs.js';
import { RES } from './terrain.js';

const DECAL_UV = { blood: [0, 0], puddle: [0.5, 0], burn: [0, 0.5], moss: [0.5, 0.5] };

export class WorldView {
    constructor(app, map, terrain, M, models, particles) {
        Object.assign(this, { app, map, terrain, M, models, particles });
        this.device = app.graphicsDevice;
        this.root = new Entity('world');
        app.root.addChild(this.root);
        this.time = 0;
        this.flickers = [];
        this.buildTerrain();
        this.buildWater();
        this.buildProps();
        this.buildScatter();
        this.buildDecals();
        this.buildLights();
        this.ambient = particles.ambient(null);
    }

    render(name, meshInstances, parent = this.root) {
        const e = new Entity(name);
        e.addComponent('render', { meshInstances, castShadows: meshInstances.some(m => m.castShadow), receiveShadows: true });
        parent.addChild(e);
        return e;
    }

    // ------------------------------------------------------------------ terrain
    buildTerrain() {
        const tr = this.terrain, W = tr.W, H = tr.H;
        const b = new MeshBuilder();
        for (let j = 0; j < H; j++) for (let i = 0; i < W; i++) {
            const k = j * W + i, x = i / RES, z = j / RES;
            const n = tr.normal(i, j);
            b.color = [tr.w[k * 4], tr.w[k * 4 + 1], tr.w[k * 4 + 2], tr.w[k * 4 + 3]];
            b.v(x, tr.h[k], z, n[0], n[1], n[2], x * 0.32, z * 0.32);
        }
        for (let j = 0; j < H - 1; j++) for (let i = 0; i < W - 1; i++) {
            const a = j * W + i, c = a + W;
            b.idx.push(a, c, a + 1, a + 1, c, c + 1);
        }
        const mesh = b.toMesh(this.device, { colors: true });
        const mi = new MeshInstance(mesh, this.M.terrain());
        mi.castShadow = true; mi.receiveShadow = true;
        this.render('terrain', [mi]);
    }

    // ------------------------------------------------------------------ water + foam
    buildWater() {
        const { map } = this;
        const m = this.M.water();
        m.normalDetailMap = this.M.waterN;
        m.normalDetailMapTiling = new Vec2(43, 43);
        m.normalDetailMapBumpiness = 0.6;
        m.update();
        this.waterMat = m;
        const pad = 60;
        const b = new MeshBuilder();
        b.quad([-pad, 0, map.h + pad], [map.w + pad, 0, map.h + pad], [map.w + pad, 0, -pad], [-pad, 0, -pad], [0, 0, 1, 1]);
        const mi = new MeshInstance(b.toMesh(this.device), m);
        mi.castShadow = false;
        this.render('water', [mi]);

        // foam: a band of quads along the shoreline, alpha from the coast field, noise texture scrolling
        const tr = this.terrain, W = tr.W, H = tr.H;
        const fb = new MeshBuilder();
        const step = 2; // every second terrain vertex
        const vid = new Int32Array(W * H).fill(-1);
        for (let j = 0; j < H; j += step) for (let i = 0; i < W; i += step) {
            const h = tr.h[j * W + i];
            const a = Math.max(0, 1 - Math.abs(h + 0.08) / 0.35); // strongest where the ground meets the waterline
            if (a <= 0.01) continue;
            fb.color = [1, 1, 1, a];
            vid[j * W + i] = fb.v(i / RES, 0.02, j / RES, 0, 1, 0, i / RES * 0.25, j / RES * 0.25);
        }
        for (let j = 0; j < H - step; j += step) for (let i = 0; i < W - step; i += step) {
            const a = vid[j * W + i], bb = vid[j * W + i + step], c = vid[(j + step) * W + i], d = vid[(j + step) * W + i + step];
            if (a < 0 || bb < 0 || c < 0 || d < 0) continue;
            fb.idx.push(a, c, bb, bb, c, d);
        }
        const fm = new StandardMaterial();
        fm.diffuse = new Color(0.55, 0.62, 0.6);
        fm.opacityMap = this.M.noise; fm.opacityMapChannel = 'a';
        fm.opacityVertexColor = true; fm.opacityVertexColorChannel = 'a';
        fm.opacityMapTiling = new Vec2(1, 1);
        fm.blendType = BLEND_NORMAL; fm.depthWrite = false;
        fm.gloss = 0.4; fm.useMetalness = true; fm.metalness = 0; fm.useSkybox = false;
        fm.update();
        this.foamMat = fm;
        if (fb.idx.length) {
            const fmi = new MeshInstance(fb.toMesh(this.device, { colors: true, tangents: false }), fm);
            fmi.castShadow = false;
            this.render('foam', [fmi]);
        }
    }

    // ------------------------------------------------------------------ props
    groundY(x, z) { return this.terrain.height(x, z); }
    buildProps() {
        const { map, models } = this;
        const groups = new Map();
        const rng = new Rng(4711);
        const add = (key, m) => { if (!models[key]) return; if (!groups.has(key)) groups.set(key, []); groups.get(key).push(m); };
        for (const p of map.props) {
            if (p.type === 'saberFire') continue; // built with its fire below
            if (p.type === 'tuft') continue; // grass cards read as black crosses from above – left out
            const ry = ['wall', 'wallV', 'palisade', 'post', 'torch', 'grave'].includes(p.type) ? 0 : rng.range(0, 360);
            const s = ['palm', 'bush', 'boulder', 'boulderBig', 'tuft'].includes(p.type) ? rng.range(0.85, 1.2) : 1;
            add(`${p.type}:${p.v}`, trs(p.x, this.groundY(p.x, p.z), p.z, ry, s));
        }
        // docks: planks over the DOCK tiles, posts at the outer edge
        const db = new MeshBuilder();
        for (let y = 0; y < map.h; y++) for (let x = 0; x < map.w; x++) {
            if (map.get(x, y) !== T.DOCK) continue;
            db.at(x + 0.5, 0.22, y + 0.5).box(1.0, 0.1, 1.0, 1);
            if (map.get(x + 1, y) !== T.DOCK || map.get(x, y + 1) !== T.DOCK) add('post:0', trs(x + 0.9, -0.3, y + 0.9));
        }
        if (db.idx.length) {
            const mi = new MeshInstance(db.toMesh(this.device), this.M.pbr('wood', { tiling: 1 }));
            mi.castShadow = true; this.render('dock', [mi]);
        }
        // the sea wreck
        const wf = map.fires.find(f => f.kind === 'sea');
        if (wf) add('wreck:0', trs(wf.x, -0.4, wf.z, 35, 1.2, 0, -12));
        for (const [key, mats] of groups) this.instances(key, mats);
    }
    instances(key, mats, castShadow = true) {
        const mis = this.models[key].map(part => instanced(this.device, part.mesh, part.mat, mats, { castShadow }));
        return this.render(key, mis);
    }

    // scatter: grass tufts on green ground, pebbles and driftwood on beaches
    buildScatter() {
        const { map } = this;
        const rng = new Rng(91);
        const tufts = [[], [], []], pebbles = [], drift = [];
        for (let y = 1; y < map.h - 1; y++) for (let x = 1; x < map.w - 1; x++) {
            const t = map.get(x, y);
            if (t === T.GRASS || t === T.DARK) {
                const n = rng.chance(0.45) ? 1 + Math.floor(rng.next() * 2) : 0;
                for (let k = 0; k < n; k++) {
                    const px = x + rng.next(), pz = y + rng.next();
                    tufts[rng.int(0, 2)].push(trs(px, this.groundY(px, pz), pz, rng.range(0, 360), rng.range(0.5, 1.0)));
                }
            } else if (t === T.SAND && rng.chance(0.18)) {
                const px = x + rng.next(), pz = y + rng.next();
                pebbles.push(trs(px, this.groundY(px, pz), pz, rng.range(0, 360), rng.range(0.7, 1.4)));
            }
            if (t === T.SAND && map.dist && map.dist[map.idx(x, y)] < 1.6 && rng.chance(0.03)) {
                const px = x + rng.next(), pz = y + rng.next();
                drift.push(trs(px, this.groundY(px, pz), pz, rng.range(0, 360), rng.range(0.8, 1.3)));
            }
        }
        void tufts;
        if (pebbles.length) this.instances('pebble', pebbles);
        if (drift.length) this.instances('drift', drift);
    }

    // ------------------------------------------------------------------ decals
    buildDecals() {
        const { map } = this;
        const rng = new Rng(5);
        const statics = [];
        for (let y = 1; y < map.h - 1; y++) for (let x = 1; x < map.w - 1; x++) {
            const t = map.get(x, y);
            if (t === T.DIRT && rng.chance(0.12)) statics.push(['puddle', x + rng.next(), y + rng.next(), rng.range(0.6, 1.4), rng.range(0, 6.3)]);
            if (t === T.STONE && rng.chance(0.08)) statics.push(['moss', x + rng.next(), y + rng.next(), rng.range(0.6, 1.2), rng.range(0, 6.3)]);
        }
        for (const p of map.props) if (p.type === 'wall' || p.type === 'wallV') for (let k = 0; k < 2; k++) statics.push(['moss', p.x + rng.range(-0.8, 0.8), p.z + rng.range(-0.4, 0.4), rng.range(0.7, 1.1), rng.range(0, 6.3)]);
        for (const f of map.fires) if (f.kind === 'land') statics.push(['burn', f.x, f.z, 2.2, 0]);
        if (statics.length) this.render('decals', [this.decalMesh(statics)]);
        this.blood = [];
        this.bloodEntity = null;
    }
    decalMesh(list) {
        const b = new MeshBuilder();
        for (const [kind, x, z, s, r] of list) {
            const [u0, v0] = DECAL_UV[kind];
            const c = Math.cos(r) * s / 2, sn = Math.sin(r) * s / 2;
            const y = (px, pz) => this.terrain.ground(px, pz) + 0.025;
            const P = (dx, dz) => [x + dx, y(x + dx, z + dz), z + dz];
            b.quad(P(-c + sn, -sn - c), P(c + sn, sn - c), P(c - sn, sn + c), P(-c - sn, -sn + c), [u0, v0, u0 + 0.5, v0 + 0.5]);
        }
        const mi = new MeshInstance(b.toMesh(this.device), this.M.decal());
        mi.castShadow = false;
        return mi;
    }
    // persistent blood pools (cleared when resting at a fire)
    addBlood(x, z, s = 0.9) {
        this.blood.push(['blood', x, z, s, Math.random() * 6.3]);
        if (this.blood.length > 60) this.blood.shift();
        this.bloodEntity?.destroy();
        this.bloodEntity = this.render('blood', [this.decalMesh(this.blood)]);
    }
    clearBlood() { this.blood = []; this.bloodEntity?.destroy(); this.bloodEntity = null; }

    // ------------------------------------------------------------------ lights
    addOmni(x, y, z, color, intensity, range, { shadows = false, flicker = 0, parent = null, res = 512 } = {}) {
        const e = new Entity('light');
        e.addComponent('light', {
            type: 'omni', color: new Color(color[0], color[1], color[2]), intensity, range,
            falloffMode: LIGHTFALLOFF_INVERSESQUARED, castShadows: shadows, shadowType: SHADOW_PCF3_32F,
            shadowResolution: res, shadowBias: 0.15, normalOffsetBias: 0.04, shadowUpdateMode: SHADOWUPDATE_REALTIME
        });
        e.setLocalPosition(x, y, z);
        (parent || this.root).addChild(e);
        if (flicker) this.flickers.push({ e, base: intensity, amt: flicker, seed: Math.random() * 100, x, y, z });
        return e;
    }
    buildLights() {
        const { map, app, particles, models } = this;
        this.fireLights = {};
        for (const f of map.fires) {
            const root = new Entity('fire-' + f.id);
            this.root.addChild(root);
            if (f.kind === 'land') {
                const gy = this.groundY(f.x, f.z);
                root.setPosition(f.x, gy, f.z);
                this.render('saber', models['saberFire:0'].map(p => { const mi = new MeshInstance(p.mesh, p.mat); mi.castShadow = true; return mi; }), root);
                const fx = new Entity('flames'); fx.setLocalPosition(0, 0.12, 0); root.addChild(fx);
                particles.fire(fx, 'teal', 0.9);
                this.fireLights[f.id] = { root, light: this.addOmni(f.x, gy + 1.3, f.z, [0.35, 1.0, 0.85], 3.5, 11, { shadows: true, flicker: 0.22, res: 1024 }) };
            } else {
                root.setPosition(f.x, 0, f.z);
                const a = new Entity(); a.setLocalPosition(0.3, 0.9, -0.4); root.addChild(a); particles.fire(a, 'orange', 1.4);
                const b = new Entity(); b.setLocalPosition(-0.6, 0.5, 1.0); root.addChild(b); particles.fire(b, 'orange', 0.9);
                this.fireLights[f.id] = { root, light: this.addOmni(f.x, 2.0, f.z, [1.0, 0.5, 0.18], 22, 18, { shadows: true, flicker: 0.3, res: 1024 }) };
            }
        }
        // torches: no shadows, but every one is a real light
        for (const l of map.lights) {
            if (l.kind !== 'torch') continue;
            const gy = this.groundY(l.x, l.z + 0.4);
            const e = new Entity('torch-fire'); e.setPosition(l.x, gy + 1.72, l.z + 0.4); this.root.addChild(e);
            particles.fire(e, 'orange', 0.35);
            this.addOmni(l.x, gy + 1.9, l.z + 0.4, [1.0, 0.55, 0.22], 4.5, 8, { flicker: 0.35 });
        }
        // fireflies: one emitter per cluster
        const ff = map.lights.filter(l => l.kind === 'firefly');
        for (let i = 0; i < ff.length; i += 5) {
            const g = ff.slice(i, i + 5);
            const cx = g.reduce((s, l) => s + l.x, 0) / g.length, cz = g.reduce((s, l) => s + l.z, 0) / g.length;
            const e = new Entity('fireflies'); e.setPosition(cx, this.groundY(cx, cz) + 0.9, cz); this.root.addChild(e);
            particles.fireflies(e, [3, 0.6, 3]);
            this.addOmni(cx, this.groundY(cx, cz) + 1, cz, [0.6, 1.0, 0.35], 0.6, 4, { flicker: 0.5 });
        }
        // whirlpools: a sinking funnel with scrolling foam, faint teal glow at the core
        this.whirls = [];
        for (const w of map.whirlpools) {
            const e = new Entity('whirl'); e.setPosition(w.x, 0.01, w.z); this.root.addChild(e);
            const b = new MeshBuilder();
            const prof = [];
            for (let i = 0; i <= 10; i++) { const t = i / 10; prof.push([w.r * (1 - t * 0.95), -Math.pow(t, 2.2) * 3.2]); }
            b.lathe(prof.reverse(), 40, { caps: false, uvU: 6, uvV: 0.5 });
            const m = new StandardMaterial();
            m.diffuse = new Color(0.01, 0.03, 0.04);
            m.emissive = new Color(0.25, 0.55, 0.55); m.emissiveMap = this.M.noise; m.emissiveIntensity = 0.5;
            m.opacityMap = this.M.noise; m.opacityMapChannel = 'a'; m.opacity = 0.9;
            m.blendType = BLEND_NORMAL; m.depthWrite = false; m.cull = CULLFACE_NONE;
            m.gloss = 0.9; m.useMetalness = true; m.useSkybox = false;
            m.update();
            const mi = new MeshInstance(b.toMesh(this.device), m);
            this.render('funnel', [mi], e);
            this.whirls.push({ e, m });
            this.addOmni(w.x, 1.5, w.z, [0.3, 0.75, 0.8], 2.5, w.r * 1.4);
        }
        void app;
    }

    update(dt, cam) {
        this.time += dt;
        const t = this.time;
        const m = this.waterMat;
        m.normalMapOffset = new Vec2((t * 0.006) % 1, (t * 0.004) % 1);
        m.normalDetailMapOffset = new Vec2((-t * 0.009) % 1, (t * 0.007) % 1);
        m.update();
        this.foamMat.opacityMapOffset = new Vec2((t * 0.02) % 1, (t * 0.013) % 1);
        this.foamMat.update();
        for (const f of this.flickers) {
            // Perlin noise, not random: smooth, organic flicker of intensity and position
            const n = NOISE.n1(t * 3.1 + f.seed) * 0.7 + NOISE.n1(t * 9.3 + f.seed * 2) * 0.3;
            f.e.light.intensity = f.base * (1 + n * f.amt);
            f.e.setLocalPosition(f.x + NOISE.n1(t * 2.2 + f.seed + 5) * 0.06, f.y + NOISE.n1(t * 2.7 + f.seed + 9) * 0.05, f.z);
        }
        for (const w of this.whirls) {
            w.e.setEulerAngles(0, -t * 60, 0);
            w.m.emissiveMapOffset = new Vec2(0, (t * 0.15) % 1); w.m.opacityMapOffset = w.m.emissiveMapOffset; w.m.update();
        }
        if (cam) this.ambient.setPosition(cam.x, 2, cam.z);
    }
}
export { Mat4, BLEND_ADDITIVE };
