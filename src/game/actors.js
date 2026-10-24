// Rigged, animated characters from the KayKit packs (CC0, Kay Lousberg): the pirate (5 bodies with
// recolourable clothes) and skeleton pirates. Same interface as the procedural figures:
// animate(dt, {state, t, speed, atk, combo}), setPos, setYaw, setVisible, destroy, joints.head.
import { Entity, Texture, PIXELFORMAT_SRGBA8, FILTER_LINEAR_MIPMAP_LINEAR, FILTER_LINEAR, MeshInstance, StandardMaterial, Color } from 'playcanvas';
import { MeshBuilder } from '../gfx/mesh.js';

export const HEROES = ['Rogue', 'Rogue_Hooded', 'Barbarian', 'Knight', 'Mage'];
export const HERO_NAMES = ['Freibeuter', 'Kapuzenschmuggler', 'Barbar', 'Ritter', 'Seehexer'];
export const SKELETONS = ['Skeleton_Warrior', 'Skeleton_Minion', 'Skeleton_Rogue'];
const WEAPON_MODEL = { cutlass: 'sword_1handed', hammer: 'axe_2handed', rapier: 'sword_1handed', dagger: 'dagger', fetish: 'wand', club: 'axe_1handed' };
// clothing colours: replace the hue of saturated, non-skin pixels in the texture palette
export const CLOTH_TONES = [
    { name: 'Original' }, { name: 'Blutrot', hue: 0 }, { name: 'Giftgrün', hue: 105 }, { name: 'Tiefsee', hue: 205 },
    { name: 'Purpur', hue: 285 }, { name: 'Asche', grey: true }
];
const HEIGHT = 1.68; // world units, about one and a half tiles
const LOOP = new Set(['Idle', 'Idle_Combat', 'Walking_A', 'Walking_D_Skeletons', 'Running_A', 'Sit_Floor_Idle', 'Blocking', '2H_Melee_Idle']);

function loadContainer(app, url) {
    return new Promise((resolve, reject) => {
        app.assets.loadFromUrl(url, 'container', (err, asset) => (err ? reject(new Error(`${url}: ${err}`)) : resolve(asset.resource)));
    });
}

export class ModelLib {
    constructor(app) { this.app = app; this.res = {}; this.scale = {}; this.tinted = new Map(); }
    async load(names, onProgress) {
        let done = 0;
        await Promise.all(names.map(async (n) => {
            const ext = /^[a-z]/.test(n) || n.startsWith('Skeleton_Blade') || n.startsWith('Skeleton_Axe') ? 'gltf' : 'glb';
            this.res[n] = await loadContainer(this.app, `models/${n}.${ext}`);
            onProgress?.(++done, names.length);
        }));
    }
    // uniform scale that makes a character HEIGHT tall (measured once in bind pose)
    heightScale(name) {
        if (this.scale[name]) return this.scale[name];
        const e = this.res[name].instantiateRenderEntity();
        let top = 0;
        for (const r of e.findComponents('render')) for (const mi of r.meshInstances) { const a = mi.mesh.aabb; top = Math.max(top, a.center.y + a.halfExtents.y); }
        e.destroy();
        this.scale[name] = top > 0.1 ? HEIGHT / top : 0.62;
        return this.scale[name];
    }
    // material with recoloured clothes (cached per model and tone)
    tintedMaterial(name, base, toneIdx) {
        const tone = CLOTH_TONES[toneIdx];
        if (!tone || (tone.hue === undefined && !tone.grey)) return base;
        const key = `${name}|${toneIdx}`;
        if (this.tinted.has(key)) return this.tinted.get(key);
        const src = base.diffuseMap;
        const img = src?.getSource?.();
        if (!img) return base;
        const c = document.createElement('canvas'); c.width = img.width; c.height = img.height;
        const g = c.getContext('2d'); g.drawImage(img, 0, 0);
        const id = g.getImageData(0, 0, c.width, c.height), d = id.data;
        for (let i = 0; i < d.length; i += 4) {
            const r = d[i] / 255, gg = d[i + 1] / 255, b = d[i + 2] / 255;
            const mx = Math.max(r, gg, b), mn = Math.min(r, gg, b), l = (mx + mn) / 2, dl = mx - mn;
            if (dl < 0.08) continue; // greys and metals stay
            const s = dl / (1 - Math.abs(2 * l - 1));
            let h = mx === r ? ((gg - b) / dl) % 6 : mx === gg ? (b - r) / dl + 2 : (r - gg) / dl + 4;
            h *= 60; if (h < 0) h += 360;
            if (h > 8 && h < 45 && s > 0.2 && s < 0.75 && l > 0.42) continue; // skin tones stay
            let nh = tone.hue ?? h, ns = tone.grey ? s * 0.12 : Math.min(1, s * 0.9 + 0.05), nl = tone.grey ? l * 0.85 : l;
            const C = (1 - Math.abs(2 * nl - 1)) * ns, X = C * (1 - Math.abs((nh / 60) % 2 - 1)), m = nl - C / 2;
            const [r1, g1, b1] = nh < 60 ? [C, X, 0] : nh < 120 ? [X, C, 0] : nh < 180 ? [0, C, X] : nh < 240 ? [0, X, C] : nh < 300 ? [X, 0, C] : [C, 0, X];
            d[i] = (r1 + m) * 255; d[i + 1] = (g1 + m) * 255; d[i + 2] = (b1 + m) * 255;
        }
        g.putImageData(id, 0, 0);
        const t = new Texture(this.app.graphicsDevice, { width: c.width, height: c.height, format: PIXELFORMAT_SRGBA8, mipmaps: true, minFilter: FILTER_LINEAR_MIPMAP_LINEAR, magFilter: FILTER_LINEAR });
        t.setSource(c);
        const m = base.clone();
        m.diffuseMap = t;
        m.update();
        this.tinted.set(key, m);
        return m;
    }
}

// shared tweak for KayKit materials: darker, less glossy, fits the dark look
const tuned = new WeakSet();
function tune(mat) {
    if (tuned.has(mat)) return;
    tuned.add(mat);
    mat.diffuse = new Color(0.82, 0.8, 0.78);
    if ('gloss' in mat) mat.gloss = Math.min(mat.gloss, 0.38);
    mat.useSkybox = false;
    mat.update();
}

export class Actor {
    // spec: { model, tone, weapon, off, skeleton }
    constructor(app, lib, M, spec) {
        this.app = app; this.lib = lib; this.M = M;
        this.skeleton = !!spec.skeleton;
        this.modelName = spec.model;
        this.root = new Entity('actor');
        app.root.addChild(this.root);
        this.vis = new Entity('vis');
        this.vis.setLocalScale(lib.heightScale(spec.model) * (spec.scale || 1), lib.heightScale(spec.model) * (spec.scale || 1), lib.heightScale(spec.model) * (spec.scale || 1));
        this.root.addChild(this.vis);
        const res = lib.res[spec.model];
        this.model = res.instantiateRenderEntity();
        this.vis.addChild(this.model);
        for (const r of this.model.findComponents('render')) {
            r.castShadows = true;
            for (const mi of r.meshInstances) {
                tune(mi.material);
                if (spec.tone) mi.material = lib.tintedMaterial(spec.model, mi.material, spec.tone);
            }
        }
        // hide the weapons the pack puts in the hands, then attach ours
        this.handR = this.model.findByName('handslot.r');
        this.handL = this.model.findByName('handslot.l');
        for (const h of [this.handR, this.handL]) if (h) for (const c of [...h.children]) c.enabled = false;
        this.attachWeapon(spec);
        // animations
        this.model.addComponent('anim', { activate: true });
        this.tracks = {};
        for (const a of res.animations) {
            const tr = a.resource;
            this.tracks[tr.name] = tr;
        }
        const first = this.skeleton ? 'Idle_Combat' : 'Idle';
        this.model.anim.assignAnimation(first, this.tracks[first] || Object.values(this.tracks)[0], undefined, 1, true);
        for (const [n, tr] of Object.entries(this.tracks)) if (n !== first) this.model.anim.assignAnimation(n, tr, undefined, 1, LOOP.has(n));
        this.cur = first;
        this.joints = { head: { e: this.model.findByName('head') || this.model } };
        this.time = 0;
    }
    attachWeapon(spec) {
        const lib = this.lib;
        const wname = this.skeleton ? (spec.model === 'Skeleton_Minion' ? 'Skeleton_Axe' : 'Skeleton_Blade') : WEAPON_MODEL[spec.weapon || 'cutlass'];
        if (wname && lib.res[wname] && this.handR) {
            const w = lib.res[wname].instantiateRenderEntity();
            for (const r of w.findComponents('render')) { r.castShadows = true; r.meshInstances.forEach(mi => tune(mi.material)); }
            this.handR.addChild(w);
            this.weapon = w;
        }
        if (!this.skeleton && this.handL) {
            if (spec.off === 'pistol') this.handL.addChild(this.pistol());
            if (spec.off === 'tide') this.handL.addChild(this.shell());
        }
        // rum bottle (mug) shown while drinking
        if (!this.skeleton && lib.res.mug_full && this.handL) {
            this.mug = lib.res.mug_full.instantiateRenderEntity();
            this.handL.addChild(this.mug);
            this.mug.enabled = false;
        }
    }
    pistol() {
        const b = new MeshBuilder();
        b.at(0, 0.05, 0.02, 90, 0, 0).cylinder(0.035, 0.03, 0.42, 8);
        b.at(0, -0.05, -0.06, 20, 0, 0).box(0.08, 0.22, 0.1, 4);
        const e = new Entity('pistol');
        const mi = new MeshInstance(b.toMesh(this.app.graphicsDevice), this.M.pbr('metal', { metal: 0.8, tint: '#8a7a68' }));
        e.addComponent('render', { meshInstances: [mi], castShadows: true });
        e.setLocalScale(1 / this.vis.getLocalScale().x * 0.62, 1 / this.vis.getLocalScale().x * 0.62, 1 / this.vis.getLocalScale().x * 0.62);
        return e;
    }
    shell() {
        const b = new MeshBuilder();
        b.at(0, 0.08, 0, 0, 0, 0, 1, 1.4, 0.7).sphere(0.09, 10, 8);
        const e = new Entity('shell');
        e.addComponent('render', { meshInstances: [new MeshInstance(b.toMesh(this.app.graphicsDevice), this.M.plain('#b8d0d0', { emissive: '#5fd8ff', emissiveIntensity: 3 }))] });
        const s = 1 / this.vis.getLocalScale().x * 0.62;
        e.setLocalScale(s, s, s);
        return e;
    }
    play(name, fade = 0.15, speed = 1, restart = false) {
        if (!this.tracks[name]) return;
        const anim = this.model.anim;
        anim.speed = speed;
        if (this.cur !== name) { anim.baseLayer.transition(name, fade); this.cur = name; }
        else if (restart) { anim.baseLayer.activeStateCurrentTime = 0; }
    }
    dur(name) { return this.tracks[name]?.duration || 1; }
    setPos(x, y, z) { this.root.setPosition(x, y, z); }
    setYaw(fx, fz) { this.root.setEulerAngles(0, Math.atan2(fx, fz) * 180 / Math.PI, 0); }
    setVisible(v) { this.root.enabled = v; }
    destroy() { this.root.destroy(); }

    // game state → animation clip
    animate(dt, s) {
        this.time += dt;
        const st = s.state, t = s.t || 0, sk = this.skeleton;
        const key = `${st}|${s.combo ?? ''}|${s.atkId ?? ''}`;
        const fresh = key !== this.lastKey; this.lastKey = key;
        if (this.mug) this.mug.enabled = st === 'drink';
        switch (st) {
            case 'walk': case 'chase': case 'return': {
                const v = s.speed ?? 3;
                if (v < 0.3) { this.play(sk ? 'Idle_Combat' : 'Idle'); break; }
                if (v > 2.6) this.play('Running_A', 0.15, v / 4.2);
                else this.play(sk ? 'Walking_D_Skeletons' : 'Walking_A', 0.15, Math.max(0.6, v / 2.0));
                break;
            }
            case 'roll': this.play('Dodge_Forward', 0.06, this.dur('Dodge_Forward') / 0.5, fresh); break;
            case 'attack': {
                const a = s.atk || { wind: 0.2, act: 0.1, rec: 0.3 };
                const clip = a.heavy ? '2H_Melee_Attack_Chop' : ['1H_Melee_Attack_Slice_Diagonal', '1H_Melee_Attack_Slice_Horizontal', '1H_Melee_Attack_Chop'][(s.combo || 0) % 3];
                this.play(clip, 0.06, this.dur(clip) / (a.wind + a.act + a.rec + 0.05), fresh);
                break;
            }
            case 'windup': { // skeleton: the chop starts with the wind-up and lands at the strike
                const a = s.atk || { wind: 0.5 };
                const clip = s.combo ? '1H_Melee_Attack_Slice_Diagonal' : '1H_Melee_Attack_Chop';
                this.play(clip, 0.1, this.dur(clip) / (a.wind + 0.55), fresh);
                break;
            }
            case 'strike': break; // keeps playing the clip started in the wind-up
            case 'parry': this.play('Block', 0.05, 1.6, fresh); break;
            case 'cast': this.play('Spellcast_Shoot', 0.08, this.dur('Spellcast_Shoot') / 0.65, fresh); break;
            case 'drink': this.play('Use_Item', 0.12, this.dur('Use_Item') / 1.05, fresh); break;
            case 'hurt': case 'stagger': this.play('Hit_A', 0.05, 1.3, fresh); break;
            case 'dead': this.play(sk ? 'Death_C_Skeletons' : 'Death_A', 0.1, 1); break;
            case 'pile': case 'gone': this.play('Death_C_Skeletons', 0.08, 1.2); break;
            case 'rise': this.play('Death_C_Skeletons_Resurrect', 0.05, this.dur('Death_C_Skeletons_Resurrect') / 0.7, fresh); break;
            case 'sit': this.play(t < 0.9 ? 'Sit_Floor_Down' : 'Sit_Floor_Idle', 0.25, 1); break;
            default: this.play(sk ? 'Idle_Combat' : 'Idle', 0.2, 1);
        }
    }
}
export { StandardMaterial };
