// Turns the WorldMap into lit PlayCanvas geometry: terrain chunks, water, props and lights.
import {
    Entity, Color, StandardMaterial, Vec2, LIGHTFALLOFF_INVERSESQUARED, LIGHTFALLOFF_LINEAR,
    SHADOW_PCF3_32F, SHADOWUPDATE_REALTIME
} from 'playcanvas';
import { QuadBatch, meshInstance, makeTexture, spriteMaterial } from '../core/gfx.js';
import { Pix, hex, shade, mix } from '../art/painter.js';
import { T } from '../art/tiles.js';
import { PROP_INFO } from '../art/props.js';
import { Sprite } from '../core/sprite.js';

const CHUNK = 32;
const PX = 1 / 16;

function waterNormalCanvas() {
    const S = 256;
    const p = new Pix(S, S);
    const h = (x, y) => {
        const a = 2 * Math.PI / S;
        return Math.sin(x * a * 2 + Math.sin(y * a) * 2) * 0.5 + Math.sin(y * a * 3 + x * a) * 0.35 + Math.sin((x + y) * a * 5) * 0.15 + Math.sin(x * a * 17 + y * a * 9) * 0.05 + Math.sin(y * a * 23 - x * a * 7) * 0.04;
    };
    for (let y = 0; y < S; y++) for (let x = 0; x < S; x++) p.px(x, y, [255, 255, 255, 255], h(x, y));
    return p.toCanvas(p.normalData(2.2 * 4));
}

function shallowPix() {
    // two 16x16 dither tiles: dense (next to land) and sparse
    const p = new Pix(32, 16);
    for (let y = 0; y < 16; y++) for (let x = 0; x < 16; x++) {
        if ((x + y) % 2 === 0 || (x % 4 === 1 && y % 4 === 1)) p.px(x, y, hex('#0f3a3e'), 0.2);
        if ((x % 2 === 0 && y % 2 === 0) && ((x + y) % 4 === 0)) p.px(16 + x, y, hex('#0c2f34'), 0.2);
    }
    p.flat.fill(1);
    return p;
}

export class WorldView {
    constructor(app, map, art) {
        this.app = app; this.map = map; this.art = art;
        this.root = new Entity('world');
        app.root.addChild(this.root);
        this.time = 0;
        this.flickers = [];
        this.fireflies = [];
        this.buildWater();
        this.buildTerrain();
        this.buildProps();
        this.buildLights();
    }

    buildWater() {
        const { app, map } = this;
        const dev = app.graphicsDevice;
        const m = new StandardMaterial();
        m.diffuse = new Color(0.06, 0.17, 0.21);
        m.normalMap = makeTexture(dev, waterNormalCanvas(), { srgb: false, repeat: true, name: 'water-n' });
        m.normalMapTiling = new Vec2(map.w / 4, map.h / 4);
        m.bumpiness = 0.45;
        m.specular = new Color(0.07, 0.09, 0.11);
        m.gloss = 0.55;
        m.useSkybox = false;
        m.update();
        this.waterMat = m;
        const qb = new QuadBatch();
        const pad = 40;
        qb.flat(map.w / 2, map.h + pad, map.w + pad * 2, map.h + pad * 2, -0.05, { x: 0, y: 0, w: 1, h: 1 }, 1, 1);
        const e = new Entity('water');
        e.addComponent('render', { meshInstances: [meshInstance(qb.toMesh(dev), m)], castShadows: false });
        this.root.addChild(e);

        // shallow dither band
        const sp = shallowPix();
        const st = { diffuse: makeTexture(dev, sp.toCanvas(), { name: 'shallow' }), normal: makeTexture(dev, sp.toCanvas(sp.normalData()), { srgb: false }), w: 32, h: 16 };
        const sm = spriteMaterial(st, { gloss: 0.6, specular: 0.3 });
        const sb = new QuadBatch();
        for (let y = 0; y < map.h; y++) for (let x = 0; x < map.w; x++) {
            if (map.get(x, y) !== T.WATER) continue;
            let d = 9;
            for (let j = -2; j <= 2; j++) for (let i = -2; i <= 2; i++) if (map.isLand(x + i, y + j)) d = Math.min(d, Math.max(Math.abs(i), Math.abs(j)));
            if (d > 2) continue;
            sb.flat(x + 0.5, y + 1, 1, 1, -0.03, { x: d === 1 ? 0 : 16, y: 0, w: 16, h: 16 }, 32, 16);
        }
        const se = new Entity('shallows');
        se.addComponent('render', { meshInstances: [meshInstance(sb.toMesh(dev), sm)], castShadows: false });
        this.root.addChild(se);
    }

    buildTerrain() {
        const { app, map, art } = this;
        const dev = app.graphicsDevice;
        const tw = art.tiles.tex.w, th = art.tiles.tex.h;
        for (let cy = 0; cy < map.h; cy += CHUNK) {
            for (let cx = 0; cx < map.w; cx += CHUNK) {
                const qb = new QuadBatch();
                const cb = new QuadBatch();
                for (let y = cy; y < Math.min(map.h, cy + CHUNK); y++) {
                    for (let x = cx; x < Math.min(map.w, cx + CHUNK); x++) {
                        const t = map.get(x, y);
                        if (t === T.WATER) continue;
                        const v = map.variant(x, y);
                        let key = `${t}:${v}`;
                        if (t === T.SAND) { const m = map.waterMask(x, y); if (m) key = `coast:${m}:${v}`; }
                        if (t !== T.SAND && t !== T.DOCK && map.waterMask(x, y)) { const m = map.waterMask(x, y); key = `coast:${m}:${v}`; }
                        const r = art.tiles.uv[key];
                        qb.flat(x + 0.5, y + 1, 1, 1, 0, r, tw, th);
                        // cliffs cast short shadows where rock meets open ground
                        if (t === T.ROCK) {
                            const rr = art.tiles.uv[`${T.ROCK}:${v}`];
                            if (map.get(x, y + 1) !== T.ROCK) cb.upright(x + 0.5, y + 1, 1, 0.7, rr, tw, th, 0);
                            if (map.get(x + 1, y) !== T.ROCK) cb.upright(x + 1, y + 0.5, 1, 0.7, rr, tw, th, 1);
                            if (map.get(x - 1, y) !== T.ROCK) cb.upright(x, y + 0.5, 1, 0.7, rr, tw, th, 1);
                            if (map.get(x, y - 1) !== T.ROCK) cb.upright(x + 0.5, y, 1, 0.7, rr, tw, th, 0);
                        }
                    }
                }
                if (!qb.count) continue;
                const e = new Entity('chunk');
                e.addComponent('render', { meshInstances: [meshInstance(qb.toMesh(dev), art.tiles.mat)], castShadows: false, receiveShadows: true });
                this.root.addChild(e);
                if (cb.count) {
                    const ce = new Entity('cliff-casters');
                    ce.addComponent('render', { meshInstances: [meshInstance(cb.toMesh(dev, false), art.tiles.casterMat, { castShadow: true })], castShadows: true, receiveShadows: false });
                    this.root.addChild(ce);
                }
            }
        }
    }

    buildProps() {
        const { app, map, art } = this;
        const dev = app.graphicsDevice;
        const tw = art.props.tex.w, th = art.props.tex.h;
        const qb = new QuadBatch();
        const cb = new QuadBatch();
        // sort by z so overlapping props layer correctly (later = on top, tiny y offset)
        const list = [...map.props].sort((a, b) => a.z - b.z);
        list.forEach((p, i) => {
            if (p.type === 'saberFire') { // painted at 4x as its own sprite
                const s = new Sprite(app, art.saberFire, { y: 0.034, caster: false });
                s.setPos(p.x, p.z);
                return;
            }
            const r = art.props.rects[`${p.type}:${p.v}`];
            const info = PROP_INFO[p.type];
            const w = r.w * PX, h = r.h * PX;
            qb.flat(p.x, p.z, w, h, 0.01 + i * 0.00002, r, tw, th);
            if (info.caster > 0) cb.caster(p.x, p.z - 0.15, w * 0.7, info.caster, r, tw, th);
        });
        // the wreck lies in the water: draw it like a prop
        const wf = map.fires.find(f => f.kind === 'sea');
        if (wf) {
            const r = art.props.rects['wreck:0'];
            qb.flat(wf.x, wf.z + 1.4, r.w * PX, r.h * PX, 0.015, r, tw, th);
            cb.caster(wf.x, wf.z + 0.6, r.w * PX * 0.6, 1.6, r, tw, th);
        }
        const e = new Entity('props');
        e.addComponent('render', { meshInstances: [meshInstance(qb.toMesh(dev), art.props.mat, { receiveShadow: true })], castShadows: false, receiveShadows: true });
        this.root.addChild(e);
        const ce = new Entity('prop-casters');
        ce.addComponent('render', { meshInstances: [meshInstance(cb.toMesh(dev, false), art.props.casterMat, { castShadow: true })], castShadows: true, receiveShadows: false });
        this.root.addChild(ce);
    }

    addOmni(x, y, z, color, intensity, range, { shadows = false, flicker = 0, parent = null } = {}) {
        const e = new Entity('light');
        e.addComponent('light', {
            type: 'omni',
            color: new Color(color[0], color[1], color[2]),
            intensity,
            range,
            falloffMode: LIGHTFALLOFF_INVERSESQUARED,
            castShadows: shadows,
            shadowType: SHADOW_PCF3_32F,
            shadowBias: 0.2,
            normalOffsetBias: 0.05,
            shadowUpdateMode: SHADOWUPDATE_REALTIME
        });
        e.setLocalPosition(x, y, z);
        (parent || this.root).addChild(e);
        if (flicker) this.flickers.push({ light: e.light, base: intensity, amt: flicker, seed: Math.random() * 100 });
        return e;
    }

    buildLights() {
        const { map, art, app } = this;
        // fires: flame sprites + shadow casting lights
        this.fireLights = {};
        for (const f of map.fires) {
            if (f.kind === 'land') {
                const fl = new Sprite(app, art.flameTeal, { y: 0.03, caster: false });
                fl.setPos(f.x, f.z + 0.05);
                fl.entity.setLocalScale(0.9, 1, 0.9);
                this.fireLights[f.id] = { sprite: fl, light: this.addOmni(f.x, 1.1, f.z - 0.2, [0.3, 0.95, 0.9], 2.6, 10, { shadows: true, flicker: 0.25 }) };
            } else {
                const fl = new Sprite(app, art.flameOrange, { y: 0.03, caster: false });
                fl.setPos(f.x + 0.9, f.z + 0.3);
                fl.entity.setLocalScale(1.6, 1, 1.6);
                const fl2 = new Sprite(app, art.flameOrange, { y: 0.03, caster: false });
                fl2.setPos(f.x + 2.2, f.z + 0.6);
                this.fireLights[f.id] = { sprite: fl, sprite2: fl2, light: this.addOmni(f.x + 1, 1.4, f.z, [1.0, 0.55, 0.2], 5, 13, { shadows: true, flicker: 0.3 }) };
            }
        }
        this.whirls = [];
        for (const w of map.whirlpools) {
            const s = new Sprite(app, art.whirl, { y: -0.04, caster: false, anchor: 0.5 });
            s.setPos(w.x, w.z);
            const sc = (w.r * 2) / 7;
            s.entity.setLocalScale(sc, 1, sc);
            const s2 = new Sprite(app, art.whirl, { y: -0.035, caster: false, anchor: 0.5 });
            s2.setPos(w.x, w.z);
            s2.entity.setLocalScale(sc * 0.55, 1, sc * 0.55);
            this.whirls.push({ s, s2, sc });
            this.addOmni(w.x, 3, w.z, [0.35, 0.7, 0.8], 0.45, w.r * 1.2);
        }
        this.torchSprites = [];
        for (const l of map.lights) {
            if (l.kind === 'torch') {
                const s = new Sprite(app, art.flameSmall, { y: 0.035, caster: false });
                s.setPos(l.x, l.z + 0.1);
                this.torchSprites.push(s);
                this.addOmni(l.x, 1.0, l.z + 0.3, [1.0, 0.6, 0.25], 1.6, 6.5, { flicker: 0.35 });
            } else if (l.kind === 'firefly') {
                const s = new Sprite(app, art.fx, { y: 0.4, caster: false });
                s.set('firefly');
                const light = this.addOmni(l.x, 0.6, l.z, [0.7, 1.0, 0.35], 0.25, 1.8);
                this.fireflies.push({ s, light, x: l.x, z: l.z, ph: Math.random() * 10 });
            }
        }
    }

    update(dt) {
        this.time += dt;
        const t = this.time;
        // water drift
        this.waterMat.normalMapOffset = new Vec2((t * 0.004) % 1, (t * 0.0023) % 1);
        this.waterMat.update();
        for (const f of this.flickers) {
            const n = Math.sin(t * 13 + f.seed) * 0.5 + Math.sin(t * 7.3 + f.seed * 2) * 0.3 + Math.sin(t * 23 + f.seed) * 0.2;
            f.light.intensity = f.base * (1 + n * f.amt);
        }
        for (const id in this.fireLights) {
            const fl = this.fireLights[id];
            fl.sprite.update(dt, 10);
            fl.sprite2?.update(dt, 9);
        }
        for (const s of this.torchSprites) s.update(dt, 11);
        for (const w of this.whirls) { w.s.entity.setEulerAngles(0, t * 40, 0); w.s2.entity.setEulerAngles(0, t * 95 + 30, 0); }
        for (const f of this.fireflies) {
            const x = f.x + Math.sin(t * 0.4 + f.ph) * 1.5, z = f.z + Math.cos(t * 0.31 + f.ph * 1.3) * 1.2;
            f.s.setPos(x, z);
            f.light.setLocalPosition(x, 0.6, z);
            const on = Math.max(0, Math.sin(t * 1.3 + f.ph * 3));
            f.light.light.intensity = 0.3 * on;
            f.s.setVisible(on > 0.2);
        }
    }
}
