// Procedural geometry: a mesh builder with transforms, primitives (box, tube, lathe, sphere,
// displaced rock, ribbons) and helpers for GPU instancing.
import {
    Mesh, MeshInstance, Mat4, Vec3, Quat, calculateTangents, SEMANTIC_TANGENT, PRIMITIVE_TRIANGLES,
    VertexBuffer, VertexFormat, BUFFER_STATIC
} from 'playcanvas';
import { Noise } from './noise.js';

const _v = new Vec3(), _n = new Vec3();
const RN = new Noise(99);

export class MeshBuilder {
    constructor() {
        this.pos = []; this.nrm = []; this.uv = []; this.col = []; this.idx = [];
        this.m = new Mat4(); this.nm = new Mat4();
        this.color = [1, 1, 1, 1];
    }
    get count() { return this.pos.length / 3; }
    // set the transform applied to following primitives (position, euler degrees, scale)
    at(x = 0, y = 0, z = 0, rx = 0, ry = 0, rz = 0, sx = 1, sy = sx, sz = sx) {
        const q = new Quat().setFromEulerAngles(rx, ry, rz);
        this.m.setTRS(new Vec3(x, y, z), q, new Vec3(sx, sy, sz));
        this.nm.copy(this.m).invert().transpose();
        return this;
    }
    setMatrix(m) { this.m.copy(m); this.nm.copy(m).invert().transpose(); return this; }
    tint(r, g, b, a = 1) { this.color = [r, g, b, a]; return this; }
    v(x, y, z, nx, ny, nz, u, w) {
        _v.set(x, y, z); this.m.transformPoint(_v, _v);
        _n.set(nx, ny, nz); this.nm.transformVector(_n, _n); _n.normalize();
        this.pos.push(_v.x, _v.y, _v.z); this.nrm.push(_n.x, _n.y, _n.z); this.uv.push(u, w);
        this.col.push(...this.color);
        return this.count - 1;
    }
    tri(a, b, c) { this.idx.push(a, b, c); }
    // flat quad from 4 corners (counter-clockwise seen from the front), normal computed
    quad(p0, p1, p2, p3, uv = [0, 0, 1, 1]) {
        const ux = p1[0] - p0[0], uy = p1[1] - p0[1], uz = p1[2] - p0[2];
        const wx = p3[0] - p0[0], wy = p3[1] - p0[1], wz = p3[2] - p0[2];
        const nx = uy * wz - uz * wy, ny = uz * wx - ux * wz, nz = ux * wy - uy * wx;
        const l = Math.hypot(nx, ny, nz) || 1;
        const a = this.v(...p0, nx / l, ny / l, nz / l, uv[0], uv[3]);
        const b = this.v(...p1, nx / l, ny / l, nz / l, uv[2], uv[3]);
        const c = this.v(...p2, nx / l, ny / l, nz / l, uv[2], uv[1]);
        const d = this.v(...p3, nx / l, ny / l, nz / l, uv[0], uv[1]);
        this.idx.push(a, b, c, a, c, d);
    }
    // axis-aligned box centred on x/z, bottom at y0. uvScale = texture repeats per unit
    box(w, h, d, uvScale = 1, y0 = 0) {
        const x0 = -w / 2, x1 = w / 2, z0 = -d / 2, z1 = d / 2, y1 = y0 + h;
        const U = uvScale;
        this.quad([x0, y0, z1], [x1, y0, z1], [x1, y1, z1], [x0, y1, z1], [0, 0, w * U, h * U]); // front +z
        this.quad([x1, y0, z0], [x0, y0, z0], [x0, y1, z0], [x1, y1, z0], [0, 0, w * U, h * U]); // back
        this.quad([x1, y0, z1], [x1, y0, z0], [x1, y1, z0], [x1, y1, z1], [0, 0, d * U, h * U]); // +x
        this.quad([x0, y0, z0], [x0, y0, z1], [x0, y1, z1], [x0, y1, z0], [0, 0, d * U, h * U]); // -x
        this.quad([x0, y1, z1], [x1, y1, z1], [x1, y1, z0], [x0, y1, z0], [0, 0, w * U, d * U]); // top
        this.quad([x0, y0, z0], [x1, y0, z0], [x1, y0, z1], [x0, y0, z1], [0, 0, w * U, d * U]); // bottom
        return this;
    }
    // surface of revolution around y. profile: [[radius, y], ...] bottom to top
    lathe(profile, seg = 12, { uvU = 1, uvV = 1, caps = true, phase = 0, squash = 1 } = {}) {
        const base = this.count;
        const rows = profile.length;
        let vAcc = [0];
        for (let i = 1; i < rows; i++) vAcc.push(vAcc[i - 1] + Math.hypot(profile[i][0] - profile[i - 1][0], profile[i][1] - profile[i - 1][1]));
        for (let i = 0; i < rows; i++) {
            const [r, y] = profile[i];
            const p = profile[Math.max(0, i - 1)], q = profile[Math.min(rows - 1, i + 1)];
            let dr = q[0] - p[0], dy = q[1] - p[1];
            const l = Math.hypot(dr, dy) || 1; dr /= l; dy /= l;
            for (let s = 0; s <= seg; s++) {
                const a = s / seg * Math.PI * 2 + phase, ca = Math.cos(a), sa = Math.sin(a);
                this.v(ca * r, y, sa * r * squash, ca * dy, -dr, sa * dy, s / seg * uvU, vAcc[i] * uvV);
            }
        }
        for (let i = 0; i < rows - 1; i++) for (let s = 0; s < seg; s++) {
            const a = base + i * (seg + 1) + s, b = a + seg + 1;
            this.idx.push(a, b, a + 1, a + 1, b, b + 1);
        }
        if (caps) {
            for (const [ri, ny] of [[0, -1], [rows - 1, 1]]) {
                const [r, y] = profile[ri];
                if (r < 1e-4) continue;
                const c = this.v(0, y, 0, 0, ny, 0, 0.5, 0.5);
                const first = this.count;
                for (let s = 0; s <= seg; s++) { const a = s / seg * Math.PI * 2 + phase; this.v(Math.cos(a) * r, y, Math.sin(a) * r * squash, 0, ny, 0, 0.5 + Math.cos(a) * 0.5, 0.5 + Math.sin(a) * 0.5); }
                for (let s = 0; s < seg; s++) ny > 0 ? this.idx.push(c, first + s + 1, first + s) : this.idx.push(c, first + s, first + s + 1);
            }
        }
        return this;
    }
    cylinder(r0, r1, h, seg = 10, opts) { return this.lathe([[r0, 0], [r1, h]], seg, opts); }
    sphere(r, seg = 12, rings = 8, opts) {
        const prof = [];
        for (let i = 0; i <= rings; i++) { const a = -Math.PI / 2 + i / rings * Math.PI; prof.push([Math.cos(a) * r, Math.sin(a) * r]); }
        prof[0][0] = 0; prof[rings][0] = 0;
        return this.lathe(prof, seg, { caps: false, ...opts });
    }
    // irregular rock: a sphere displaced by noise, flattened at the bottom
    rock(r, seed = 1, rough = 0.35, seg = 14, rings = 10, flatten = 0.75) {
        const base = this.count;
        this.sphere(r, seg, rings, { uvU: 2, uvV: 1.2 });
        const p = this.pos;
        // displace the vertices just written (they are already transformed, so displace in place)
        for (let i = base; i < this.count; i++) {
            const x = p[i * 3], y = p[i * 3 + 1], z = p[i * 3 + 2];
            const k = 1 + RN.fbm(x * 1.3 + seed * 3.1, z * 1.3 + y * 0.7 + seed, 3) * rough;
            p[i * 3] = x * k; p[i * 3 + 2] = z * k;
            p[i * 3 + 1] = y >= 0 ? y * k * flatten : y * 0.25;
        }
        // seam vertices of a lathe share positions, so recompute smooth normals for this part
        recomputeNormals(this, base);
        return this;
    }
    // tube along a polyline with per-point radius (trunks, bones, ropes, horns)
    tube(points, radii, seg = 8, { uvU = 1, uvV = 1, capEnd = true } = {}) {
        const base = this.count;
        let px = [1, 0, 0];
        let vAcc = 0;
        for (let i = 0; i < points.length; i++) {
            const p = points[i], q = points[Math.min(points.length - 1, i + 1)], o = points[Math.max(0, i - 1)];
            let tx = q[0] - o[0], ty = q[1] - o[1], tz = q[2] - o[2];
            const tl = Math.hypot(tx, ty, tz) || 1; tx /= tl; ty /= tl; tz /= tl;
            // side vector: previous side projected away from the tangent (parallel transport)
            const d = px[0] * tx + px[1] * ty + px[2] * tz;
            let sx = px[0] - tx * d, sy = px[1] - ty * d, sz = px[2] - tz * d;
            let sl = Math.hypot(sx, sy, sz);
            if (sl < 1e-4) { sx = 0; sy = 0; sz = 1; sl = 1; }
            sx /= sl; sy /= sl; sz /= sl; px = [sx, sy, sz];
            const bx = ty * sz - tz * sy, by = tz * sx - tx * sz, bz = tx * sy - ty * sx;
            if (i > 0) vAcc += Math.hypot(p[0] - points[i - 1][0], p[1] - points[i - 1][1], p[2] - points[i - 1][2]);
            for (let s = 0; s <= seg; s++) {
                const a = s / seg * Math.PI * 2, ca = Math.cos(a), sa = Math.sin(a);
                const nx = sx * ca + bx * sa, ny = sy * ca + by * sa, nz = sz * ca + bz * sa;
                const r = radii[i];
                this.v(p[0] + nx * r, p[1] + ny * r, p[2] + nz * r, nx, ny, nz, s / seg * uvU, vAcc * uvV);
            }
        }
        for (let i = 0; i < points.length - 1; i++) for (let s = 0; s < seg; s++) {
            const a = base + i * (seg + 1) + s, b = a + seg + 1;
            this.idx.push(a, a + 1, b, a + 1, b + 1, b);
        }
        if (capEnd && radii[radii.length - 1] > 1e-3) {
            const last = points[points.length - 1], prev = points[points.length - 2];
            const tx = last[0] - prev[0], ty = last[1] - prev[1], tz = last[2] - prev[2], l = Math.hypot(tx, ty, tz) || 1;
            const c = this.v(last[0] + tx / l * radii[radii.length - 1] * 0.5, last[1] + ty / l * radii[radii.length - 1] * 0.5, last[2] + tz / l * radii[radii.length - 1] * 0.5, tx / l, ty / l, tz / l, 0.5, vAcc * uvV);
            const ring = base + (points.length - 1) * (seg + 1);
            for (let s = 0; s < seg; s++) this.idx.push(ring + s, ring + s + 1, c);
        }
        return this;
    }
    // ribbon along a polyline (for lightning, slash arcs, frond strips); width per point, facing normal n
    ribbon(points, widths, up = [0, 1, 0]) {
        const base = this.count;
        for (let i = 0; i < points.length; i++) {
            const p = points[i], q = points[Math.min(points.length - 1, i + 1)], o = points[Math.max(0, i - 1)];
            let tx = q[0] - o[0], ty = q[1] - o[1], tz = q[2] - o[2];
            let sx = ty * up[2] - tz * up[1], sy = tz * up[0] - tx * up[2], sz = tx * up[1] - ty * up[0];
            const l = Math.hypot(sx, sy, sz) || 1; sx /= l; sy /= l; sz /= l;
            const w = widths[i] / 2, v = i / (points.length - 1);
            this.v(p[0] - sx * w, p[1] - sy * w, p[2] - sz * w, up[0], up[1], up[2], 0, v);
            this.v(p[0] + sx * w, p[1] + sy * w, p[2] + sz * w, up[0], up[1], up[2], 1, v);
        }
        for (let i = 0; i < points.length - 1; i++) { const a = base + i * 2; this.idx.push(a, a + 2, a + 1, a + 1, a + 2, a + 3); }
        return this;
    }
    toMesh(device, { tangents = true, colors = false } = {}) {
        const mesh = new Mesh(device);
        mesh.setPositions(this.pos);
        mesh.setNormals(this.nrm);
        mesh.setUvs(0, this.uv);
        if (colors) mesh.setColors(this.col);
        mesh.setIndices(this.idx);
        if (tangents && this.idx.length) mesh.setVertexStream(SEMANTIC_TANGENT, calculateTangents(this.pos, this.nrm, this.uv, this.idx), 4);
        mesh.update(PRIMITIVE_TRIANGLES);
        return mesh;
    }
}

// smooth normals for the vertices from `base` on, welding equal positions
export function recomputeNormals(b, base = 0) {
    const n = new Float32Array(b.pos.length);
    const key = i => `${b.pos[i * 3].toFixed(4)},${b.pos[i * 3 + 1].toFixed(4)},${b.pos[i * 3 + 2].toFixed(4)}`;
    const groups = new Map();
    for (let i = base; i < b.count; i++) { const k = key(i); if (!groups.has(k)) groups.set(k, []); groups.get(k).push(i); }
    for (let t = 0; t < b.idx.length; t += 3) {
        const [i0, i1, i2] = [b.idx[t], b.idx[t + 1], b.idx[t + 2]];
        if (i0 < base) continue;
        const ax = b.pos[i1 * 3] - b.pos[i0 * 3], ay = b.pos[i1 * 3 + 1] - b.pos[i0 * 3 + 1], az = b.pos[i1 * 3 + 2] - b.pos[i0 * 3 + 2];
        const cx = b.pos[i2 * 3] - b.pos[i0 * 3], cy = b.pos[i2 * 3 + 1] - b.pos[i0 * 3 + 1], cz = b.pos[i2 * 3 + 2] - b.pos[i0 * 3 + 2];
        const nx = ay * cz - az * cy, ny = az * cx - ax * cz, nz = ax * cy - ay * cx;
        for (const i of [i0, i1, i2]) { n[i * 3] += nx; n[i * 3 + 1] += ny; n[i * 3 + 2] += nz; }
    }
    for (const g of groups.values()) {
        let x = 0, y = 0, z = 0;
        for (const i of g) { x += n[i * 3]; y += n[i * 3 + 1]; z += n[i * 3 + 2]; }
        const l = Math.hypot(x, y, z) || 1;
        for (const i of g) { b.nrm[i * 3] = x / l; b.nrm[i * 3 + 1] = y / l; b.nrm[i * 3 + 2] = z / l; }
    }
}

// one draw call for many copies of a mesh. matrices: array of Mat4
export function instanced(device, mesh, material, matrices, { castShadow = true } = {}) {
    const data = new Float32Array(matrices.length * 16);
    matrices.forEach((m, i) => data.set(m.data, i * 16));
    const vb = new VertexBuffer(device, VertexFormat.getDefaultInstancingFormat(device), matrices.length, { usage: BUFFER_STATIC, data });
    const mi = new MeshInstance(mesh, material);
    mi.setInstancing(vb);
    mi.castShadow = castShadow;
    mi.receiveShadow = true;
    return mi;
}

export function trs(x, y, z, ry = 0, s = 1, rx = 0, rz = 0, sy = s) {
    return new Mat4().setTRS(new Vec3(x, y, z), new Quat().setFromEulerAngles(rx, ry, rz), new Vec3(s, sy, s));
}
