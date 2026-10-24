// Dynamic, animated pixel sprite lying on the ground, with an optional upright shadow caster.
import { Entity } from 'playcanvas';
import { QuadBatch, meshInstance } from './gfx.js';

const PX = 1 / 16; // world units per pixel

export class Sprite {
    // sheet: { tex, mat, casterMat, frames: {name: [rect,...]}, fw, fh }
    constructor(app, sheet, { y = 0.05, caster = true, parent = null, pivotY = 0, anchor = 0 } = {}) {
        this.anchor = anchor; // 0 = feet at bottom edge, 0.5 = centred
        this.app = app;
        this.sheet = sheet;
        this.y = y;
        this.pivotY = pivotY; // pixels from bottom of frame to the feet
        this.entity = new Entity('sprite');
        this.flipX = false;
        this.anim = null;
        this.frameIdx = -1;
        this.frameRect = null;
        const r = Object.values(sheet.frames)[0][0];
        this.w = r.w * PX; this.h = r.h * PX;
        this.qb = new QuadBatch();
        this.qb.flat(0, this.h * anchor, this.w, this.h, y, r, sheet.tex.w, sheet.tex.h);
        this.mesh = this.qb.toMesh(app.graphicsDevice);
        this.entity.addComponent('render', { meshInstances: [meshInstance(this.mesh, sheet.mat, { receiveShadow: false })], castShadows: false, receiveShadows: false });
        if (caster && sheet.casterMat) {
            const cb = new QuadBatch();
            cb.caster(0, 0, this.w * 0.8, this.h, r, sheet.tex.w, sheet.tex.h);
            this.casterMesh = cb.toMesh(app.graphicsDevice, false);
            const ce = new Entity('caster');
            ce.addComponent('render', { meshInstances: [meshInstance(this.casterMesh, sheet.casterMat, { castShadow: true, receiveShadow: false })], castShadows: true, receiveShadows: false });
            this.entity.addChild(ce);
        }
        (parent || app.root).addChild(this.entity);
        this.time = 0;
        this.set(Object.keys(sheet.frames)[0]);
    }
    set(name, restart = false) {
        if (this.anim === name && !restart) return;
        this.anim = name;
        this.time = 0;
        this.frameIdx = -1;
        this._apply(0);
    }
    update(dt, fps = 8, loop = true) {
        this.time += dt;
        const frames = this.sheet.frames[this.anim];
        let i = Math.floor(this.time * fps);
        i = loop ? i % frames.length : Math.min(i, frames.length - 1);
        this._apply(i);
    }
    get done() { return this.frameIdx >= this.sheet.frames[this.anim].length - 1; }
    setFlip(f) { if (f !== this.flipX) { this.flipX = f; this.frameIdx = -1; this._apply(Math.max(0, Math.floor(this.time * 8)) % this.sheet.frames[this.anim].length); } }
    _apply(i) {
        if (i === this.frameIdx) return;
        this.frameIdx = i;
        const r = this.sheet.frames[this.anim][i];
        const tw = this.sheet.tex.w, th = this.sheet.tex.h;
        let u0 = r.x / tw, u1 = (r.x + r.w) / tw;
        const v0 = r.y / th, v1 = (r.y + r.h) / th;
        if (this.flipX) [u0, u1] = [u1, u0];
        if (r.w * PX !== this.w || r.h * PX !== this.h) {
            this.w = r.w * PX; this.h = r.h * PX;
            const x0 = -this.w / 2, x1 = this.w / 2, z0 = -this.h * (1 - this.anchor), z1 = this.h * this.anchor;
            this.mesh.setPositions([x0, this.y, z0, x1, this.y, z0, x1, this.y, z1, x0, this.y, z1]);
        }
        this.mesh.setUvs(0, [u0, v0, u1, v0, u1, v1, u0, v1]);
        this.mesh.update();
        if (this.casterMesh) {
            const uv = [u0, v0, u1, v0, u1, v1, u0, v1];
            this.casterMesh.setUvs(0, uv.concat(uv));
            this.casterMesh.update();
        }
    }
    setPos(x, z) {
        // snap to the pixel grid so the art stays crisp
        this.entity.setPosition(Math.round(x * 16) / 16, 0, Math.round((z + this.pivotY * PX) * 16) / 16);
    }
    setVisible(v) { this.entity.enabled = v; }
    destroy() { this.entity.destroy(); }
}
