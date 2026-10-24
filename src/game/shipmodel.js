// Procedural low-poly 3D model of the sloop "Möwenschrei": lofted plank hull, deck, stern castle,
// two masts with yards and billowing sails, glowing cabin windows and a stern lantern.
// Local space: bow towards -z, keel at y = -0.6, waterline at y = 0.
import { Entity, Mesh, MeshInstance, CULLFACE_NONE, PRIMITIVE_TRIANGLES, calculateTangents, SEMANTIC_TANGENT } from 'playcanvas';

export const SHIP_LEN = 4.8;
const W = 1.75; // beam

class Builder {
    constructor() { this.pos = []; this.nrm = []; this.col = []; this.uv = []; this.idx = []; }
    // quad a-b-c-d; normal oriented away from `inside`
    quad(a, b, c, d, color, inside) {
        const ux = b[0] - a[0], uy = b[1] - a[1], uz = b[2] - a[2];
        const vx = d[0] - a[0], vy = d[1] - a[1], vz = d[2] - a[2];
        let nx = uy * vz - uz * vy, ny = uz * vx - ux * vz, nz = ux * vy - uy * vx;
        const l = Math.hypot(nx, ny, nz) || 1; nx /= l; ny /= l; nz /= l;
        if (inside) {
            const cx = (a[0] + b[0] + c[0] + d[0]) / 4 - inside[0], cy = (a[1] + b[1] + c[1] + d[1]) / 4 - inside[1], cz = (a[2] + b[2] + c[2] + d[2]) / 4 - inside[2];
            if (nx * cx + ny * cy + nz * cz < 0) { nx = -nx; ny = -ny; nz = -nz; }
        }
        const base = this.pos.length / 3;
        const ax = Math.abs(nx), ay = Math.abs(ny), az = Math.abs(nz);
        for (const p of [a, b, c, d]) {
            this.pos.push(p[0], p[1], p[2]); this.nrm.push(nx, ny, nz); this.col.push(color[0], color[1], color[2], 1);
            // planar UVs picked by the dominant normal axis (wood planks run along the hull)
            if (ay >= ax && ay >= az) this.uv.push(p[2] * 0.8, p[0] * 0.8); else if (ax >= az) this.uv.push(p[2] * 0.8, p[1] * 0.8); else this.uv.push(p[0] * 0.8, p[1] * 0.8);
        }
        this.idx.push(base, base + 1, base + 2, base, base + 2, base + 3);
    }
    box(x0, y0, z0, x1, y1, z1, color, shadeTop = 1.15) {
        const c = [(x0 + x1) / 2, (y0 + y1) / 2, (z0 + z1) / 2];
        const top = color.map(v => Math.min(1, v * shadeTop));
        this.quad([x0, y1, z0], [x1, y1, z0], [x1, y1, z1], [x0, y1, z1], top, c);
        this.quad([x0, y0, z0], [x1, y0, z0], [x1, y0, z1], [x0, y0, z1], color, c);
        this.quad([x0, y0, z1], [x1, y0, z1], [x1, y1, z1], [x0, y1, z1], color, c);
        this.quad([x0, y0, z0], [x1, y0, z0], [x1, y1, z0], [x0, y1, z0], color, c);
        this.quad([x0, y0, z0], [x0, y0, z1], [x0, y1, z1], [x0, y1, z0], color, c);
        this.quad([x1, y0, z0], [x1, y0, z1], [x1, y1, z1], [x1, y1, z0], color, c);
    }
    // vertical octagonal pole
    pole(x, z, y0, y1, r, color) {
        const n = 6;
        for (let i = 0; i < n; i++) {
            const a0 = i / n * Math.PI * 2, a1 = (i + 1) / n * Math.PI * 2;
            this.quad([x + Math.cos(a0) * r, y0, z + Math.sin(a0) * r], [x + Math.cos(a1) * r, y0, z + Math.sin(a1) * r],
                [x + Math.cos(a1) * r * 0.8, y1, z + Math.sin(a1) * r * 0.8], [x + Math.cos(a0) * r * 0.8, y1, z + Math.sin(a0) * r * 0.8], color, [x, (y0 + y1) / 2, z]);
        }
    }
    mesh(device) {
        const m = new Mesh(device);
        m.setPositions(this.pos);
        m.setNormals(this.nrm);
        m.setColors(this.col);
        m.setUvs(0, this.uv);
        m.setIndices(this.idx);
        m.setVertexStream(SEMANTIC_TANGENT, calculateTangents(this.pos, this.nrm, this.uv, this.idx), 4);
        m.update(PRIMITIVE_TRIANGLES);
        return m;
    }
}

// vertex colours tint the textured materials (1 = texture as is)
const hexc = (h) => [parseInt(h.slice(1, 3), 16) / 255, parseInt(h.slice(3, 5), 16) / 255, parseInt(h.slice(5, 7), 16) / 255].map(v => Math.min(1, v * 2.2));

// hull stations from bow (t = 0) to stern (t = 1)
const ST = [
    { t: 0.0, w: 0.02, deck: 1.15, keel: 0.15 },
    { t: 0.07, w: 0.34, deck: 1.02, keel: -0.2 },
    { t: 0.18, w: 0.62, deck: 0.92, keel: -0.45 },
    { t: 0.34, w: 0.84, deck: 0.86, keel: -0.58 },
    { t: 0.52, w: 0.88, deck: 0.85, keel: -0.6 },
    { t: 0.7, w: 0.82, deck: 0.9, keel: -0.56 },
    { t: 0.86, w: 0.72, deck: 1.0, keel: -0.45 },
    { t: 1.0, w: 0.6, deck: 1.1, keel: -0.25 }
];

function hull(b) {
    const L = SHIP_LEN;
    const planks = [hexc('#5a3824'), hexc('#46291a'), hexc('#6a4329'), hexc('#3b2316')];
    const trim = hexc('#a07a36'), tar = hexc('#1c1512');
    // cross-section ring as fractions: [width factor, height between keel(0) and deck(1)]
    const ring = [[1, 1], [1.02, 0.9], [1.06, 0.72], [0.98, 0.5], [0.8, 0.3], [0.45, 0.1], [0, 0]];
    const pt = (s, k, side) => {
        const [wf, hf] = ring[k];
        return [side * s.w * wf * W / 1.75, s.keel + (s.deck - s.keel) * hf, -L / 2 + s.t * L];
    };
    for (let i = 0; i < ST.length - 1; i++) {
        const a = ST[i], c = ST[i + 1];
        for (const side of [-1, 1]) {
            for (let k = 0; k < ring.length - 1; k++) {
                const col = k === 0 ? trim : k >= 4 ? tar : planks[(i + k * 3 + (side > 0 ? 1 : 0)) % planks.length];
                const inside = [0, (a.deck + a.keel) / 2, -L / 2 + (a.t + c.t) / 2 * L];
                b.quad(pt(a, k, side), pt(c, k, side), pt(c, k + 1, side), pt(a, k + 1, side), col, inside);
            }
        }
        // deck planks (slightly below the rail)
        const dk = hexc(i % 2 ? '#7d6244' : '#8a6d4b');
        const za = -L / 2 + a.t * L, zc = -L / 2 + c.t * L;
        b.quad([-a.w * 0.95, a.deck - 0.12, za], [a.w * 0.95, a.deck - 0.12, za], [c.w * 0.95, c.deck - 0.12, zc], [-c.w * 0.95, c.deck - 0.12, zc], dk, [0, -5, 0]);
    }
    // transom (stern board)
    const s = ST[ST.length - 1], zs = L / 2;
    b.quad([-s.w, s.deck, zs], [s.w, s.deck, zs], [s.w * 0.45, s.keel, zs], [-s.w * 0.45, s.keel, zs], hexc('#2c1b12'), [0, 0.4, 0]);
}

function rigging(b, bm) {
    const L = SHIP_LEN, wood = hexc('#4a3220'), dark = hexc('#2a1b12');
    // stern castle with cabin
    const zc0 = L / 2 - 1.05, zc1 = L / 2 - 0.02;
    b.box(-0.62, 0.95, zc0, 0.62, 1.45, zc1, hexc('#4e321f'));
    b.box(-0.68, 1.45, zc0 - 0.05, 0.68, 1.52, zc1 + 0.03, hexc('#6f5230'));
    // glowing cabin windows (stern and sides)
    for (const x of [-0.38, -0.1, 0.18, 0.44]) bm.box(x - 0.09, 1.08, zc1 + 0.005, x + 0.09, 1.3, zc1 + 0.03, [1, 0.8, 0.45]);
    for (const side of [-1, 1]) bm.box(side * 0.625 - 0.012, 1.1, zc0 + 0.3, side * 0.625 + 0.012, 1.28, zc0 + 0.55, [1, 0.8, 0.45]);
    // stern lantern post
    b.pole(0, zc1 - 0.1, 1.5, 2.1, 0.035, dark);
    bm.box(-0.08, 2.05, zc1 - 0.18, 0.08, 2.25, zc1 - 0.02, [1, 0.75, 0.35]);
    // bowsprit
    b.quad([-0.05, 1.12, -L / 2 + 0.1], [0.05, 1.12, -L / 2 + 0.1], [0.03, 1.45, -L / 2 - 1.0], [-0.03, 1.45, -L / 2 - 1.0], wood, [0, 0, 0]);
    b.quad([0, 1.07, -L / 2 + 0.1], [0, 1.17, -L / 2 + 0.1], [0, 1.48, -L / 2 - 1.0], [0, 1.42, -L / 2 - 1.0], wood, [1, 0, 0]);
    // figurehead-less bow (the Schiffshain took it): a dark stump
    b.box(-0.06, 0.95, -L / 2 - 0.12, 0.06, 1.12, -L / 2 + 0.05, dark);
    // masts
    const masts = [{ z: -L / 2 + L * 0.34, h: 3.7 }, { z: -L / 2 + L * 0.62, h: 3.2 }];
    for (const m of masts) {
        b.pole(0, m.z, 0.8, m.h, 0.07, wood);
        b.box(-0.18, m.h * 0.78, m.z - 0.18, 0.18, m.h * 0.78 + 0.06, m.z + 0.18, dark); // crow's nest floor
        for (const yy of [m.h * 0.92, m.h * 0.45 + 0.6]) b.box(-0.95, yy - 0.03, m.z - 0.03, 0.95, yy + 0.03, m.z + 0.03, wood);
    }
    // shrouds: thin dark quads from the rail to the masthead
    for (const m of masts) for (const side of [-1, 1]) {
        b.quad([side * 0.8, 0.95, m.z - 0.25], [side * 0.8, 0.95, m.z + 0.25], [side * 0.03, m.h * 0.9, m.z + 0.01], [side * 0.03, m.h * 0.9, m.z - 0.01], hexc('#1a1410'), [0, m.h * 0.5, m.z]);
    }
    return masts;
}

// sail: billowing grid between two yards, bottom edge ragged. One entity per sail so it can be furled.
function sailMesh(device, w, h, seed) {
    const b = new Builder();
    const nx = 6, ny = 5;
    const cloth = hexc('#c9bfa6'), cloth2 = hexc('#b3a98f'), patch = hexc('#8a7f68');
    const P = (i, j) => {
        const u = i / nx - 0.5, v = j / ny;
        const belly = Math.sin((u + 0.5) * Math.PI) * Math.sin(v * Math.PI * 0.9 + 0.15) * 0.32;
        let y = -v * h;
        if (j === ny) y -= ((i * 7 + seed) % 3) * 0.06; // torn hem
        return [u * w * (1 - v * 0.08), y, -belly];
    };
    for (let j = 0; j < ny; j++) for (let i = 0; i < nx; i++) {
        const c = ((i * 3 + j * 5 + seed) % 11 === 0) ? patch : (j % 2 ? cloth2 : cloth);
        b.quad(P(i, j), P(i + 1, j), P(i + 1, j + 1), P(i, j + 1), c, [0, -h / 2, 1]);
    }
    return b.mesh(device);
}

export function buildShipModel(app, M) {
    const dev = app.graphicsDevice;
    const root = new Entity('ship');
    const body = new Entity('ship-body'); // bobs and rolls
    root.addChild(body);
    const b = new Builder(), bm = new Builder();
    hull(b);
    const masts = rigging(b, bm);
    const wood = M.pbr('wood', { vertexColor: true, tiling: 1, cull: CULLFACE_NONE });
    const glow = M.plain('#ffffff', { vertexColor: true, emissive: '#ffb860', emissiveIntensity: 6 });
    const add = (mesh, mat, cast = true) => {
        const e = new Entity();
        const mi = new MeshInstance(mesh, mat);
        mi.castShadow = cast; mi.receiveShadow = true;
        e.addComponent('render', { meshInstances: [mi], castShadows: cast, receiveShadows: true });
        return e;
    };
    body.addChild(add(b.mesh(dev), wood));
    body.addChild(add(bm.mesh(dev), glow, false));
    const sailMat = M.pbr('cloth', { vertexColor: true, tiling: 1, cull: CULLFACE_NONE });
    const sails = [];
    masts.forEach((m, i) => {
        for (const [k, yy, w, h] of [[0, m.h * 0.92, 1.8, m.h * 0.92 - (m.h * 0.45 + 0.6) - 0.08], [1, m.h * 0.45 + 0.6, 1.95, m.h * 0.45 - 0.35]]) {
            const e = add(sailMesh(dev, w, h, i * 5 + k * 3), sailMat);
            e.setLocalPosition(0, yy - 0.03, m.z - 0.08);
            body.addChild(e);
            sails.push(e);
        }
    });
    return { root, body, sails, lanternPos: [0, 2.15, SHIP_LEN / 2 - 0.1] };
}
