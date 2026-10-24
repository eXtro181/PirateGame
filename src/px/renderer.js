// Pixel renderer (WebGL2) after the graphics spec:
//  scene → low-res target (640×360, nearest) in layers: ground, ground deco, shadows (multiply),
//  y-sorted entities, overhead, fx · light buffer: zone ambient + baked lightmap + dynamic lights ·
//  glow buffer (emissive, unaffected by darkness) · composite → bloom (threshold) → grading per zone,
//  vignette 25 %, film grain 3 % → integer upscale with letterboxing.
import { createGL, program, texture, target, freeTarget } from './gl.js';

const SPRITE_VS = `
layout(location=0) in vec2 aPos;
layout(location=1) in vec2 aUv;
layout(location=2) in vec4 aCol;
layout(location=3) in vec4 aExt; // x: mode (0 normal, 1 outline, 2 flash), y: sway amp px, z: 1 at the top edge, w: world x
uniform vec2 uRes;
uniform float uTime;
out vec2 vUv; out vec4 vCol; flat out float vMode;
void main() {
    vec2 p = aPos;
    if (aExt.y > 0.0) p.x += floor(sin(uTime * 1.5 + aExt.w * 0.1) * aExt.y * aExt.z + 0.5);
    vUv = aUv; vCol = aCol; vMode = aExt.x;
    gl_Position = vec4(p / uRes * 2.0 - 1.0, 0.0, 1.0);
    gl_Position.y = -gl_Position.y;
}`;
const SPRITE_FS = `
uniform sampler2D uTex;
uniform vec2 uTexel;
uniform vec4 uOutline;
in vec2 vUv; in vec4 vCol; flat in float vMode;
out vec4 o;
void main() {
    vec4 c = texture(uTex, vUv);
    if (vMode > 0.5 && vMode < 1.5) { // 1 px outline around the sprite (hover)
        if (c.a < 0.5) {
            float a = texture(uTex, vUv + vec2(uTexel.x, 0.0)).a + texture(uTex, vUv - vec2(uTexel.x, 0.0)).a
                    + texture(uTex, vUv + vec2(0.0, uTexel.y)).a + texture(uTex, vUv - vec2(0.0, uTexel.y)).a;
            if (a > 0.5) { o = uOutline; return; }
            discard;
        }
    }
    if (c.a < 0.5) discard;
    c *= vCol;
    if (vMode > 1.5) c.rgb = mix(c.rgb, vec3(1.0), 0.75); // hit flash
    o = c;
}`;
const QUAD_VS = `
layout(location=0) in vec2 aPos;
out vec2 vUv;
void main() { vUv = aPos * 0.5 + 0.5; gl_Position = vec4(aPos, 0.0, 1.0); }`;
// water: base colour by depth, two counter-scrolling voronoi noises for hexagonal glints, shore foam
const WATER_FS = `
uniform sampler2D uMask;   // r: water, g: distance to shore (0..1 = 0..8 tiles), b: deep
uniform vec2 uMapSize;     // tiles
uniform vec2 uCam;         // top-left world pixel of the view
uniform vec2 uRes;
uniform float uTime;
uniform vec3 uShallow, uDeep, uGlint, uFoam, uSoul;
in vec2 vUv;
out vec4 o;
vec2 hash2(vec2 p) { p = vec2(dot(p, vec2(127.1, 311.7)), dot(p, vec2(269.5, 183.3))); return fract(sin(p) * 43758.5453); }
float voronoi(vec2 x) {
    vec2 n = floor(x), f = fract(x); float m = 8.0;
    for (int j = -1; j <= 1; j++) for (int i = -1; i <= 1; i++) {
        vec2 g = vec2(float(i), float(j)); vec2 r = g + hash2(n + g) - f; m = min(m, dot(r, r));
    }
    return sqrt(m);
}
void main() {
    vec2 px = floor(vec2(vUv.x, 1.0 - vUv.y) * uRes) + uCam; // world pixel
    vec2 tile = px / 16.0;
    vec4 m = texture(uMask, (tile) / uMapSize);
    if (m.r < 0.5) discard;
    float d = m.g;
    vec3 col = mix(uShallow, uDeep, smoothstep(0.05, 0.7, d));
    // hexagonal glints: 1 - voronoi edge distance, threshold 0.85, quantized
    float v1 = voronoi(px / 22.0 + vec2(uTime * 0.12, uTime * 0.05));
    float v2 = voronoi(px / 15.0 - vec2(uTime * 0.08, -uTime * 0.1));
    float g = step(0.85, (1.0 - v1) * (1.0 - v2) * 1.12) * (1.0 - d * 0.5);
    col = mix(col, uGlint, g * 0.8);
    // shore foam along the distance field
    float wave = sin(uTime * 2.0 + px.x * 0.15 + px.y * 0.1) * 0.012;
    float foam = step(d, 0.03 + wave) * step(0.0, d);
    col = mix(col, uFoam, foam * 0.9);
    // soul light reflections
    float s = step(0.997, fract(sin(dot(floor(px / 3.0), vec2(12.9898, 78.233)) + floor(uTime * 2.0)) * 43758.5));
    col += uSoul * s * 0.6 * (0.4 + d);
    float a = mix(0.62, 0.97, smoothstep(0.0, 0.35, d)); // shallows let the ground shine through
    o = vec4(col, a);
}`;
// lights: ambient + baked lightmap (world space) ; dynamic lights are drawn on top additively
const LIGHTMAP_FS = `
uniform sampler2D uLM; uniform vec2 uLMSize; uniform vec2 uCam; uniform vec2 uRes; uniform vec3 uAmbient; uniform float uHasLM;
in vec2 vUv; out vec4 o;
void main() {
    vec2 px = floor(vec2(vUv.x, 1.0 - vUv.y) * uRes) + uCam;
    vec3 lm = uHasLM > 0.5 ? texture(uLM, vec2(px.x / uLMSize.x, 1.0 - px.y / uLMSize.y)).rgb * 2.0 : vec3(0.0);
    o = vec4(uAmbient + lm, 1.0);
}`;
const LIGHT_VS = `
layout(location=0) in vec2 aPos;
layout(location=1) in vec2 aUv;
layout(location=2) in vec4 aCol;
uniform vec2 uRes;
out vec2 vUv; out vec4 vCol;
void main() { vUv = aUv; vCol = aCol; gl_Position = vec4(aPos / uRes * 2.0 - 1.0, 0.0, 1.0); gl_Position.y = -gl_Position.y; }`;
const LIGHT_FS = `
in vec2 vUv; in vec4 vCol; out vec4 o;
void main() {
    float d = length(vUv * 2.0 - 1.0);
    if (d > 1.0) discard;
    float f = 1.0 - d; f = f * f;
    // stepped falloff with an ordered dither between the steps (pixel look, no hard rings)
    ivec2 q = ivec2(gl_FragCoord.xy) & 3;
    float b = float((q.x * 4 + q.y * 9 + (q.x ^ q.y) * 3) & 15) / 16.0;
    f = floor(f * 6.0 + b) / 6.0;
    o = vec4(vCol.rgb * vCol.a * f, 1.0);
}`;
const COMPOSE_FS = `
uniform sampler2D uScene, uLight, uGlow;
in vec2 vUv; out vec4 o;
void main() {
    vec3 s = texture(uScene, vUv).rgb;
    vec3 l = texture(uLight, vUv).rgb;
    vec3 g = texture(uGlow, vUv).rgb;
    o = vec4(s * l + g, 1.0);
}`;
const BRIGHT_FS = `
uniform sampler2D uTex; uniform float uThreshold;
in vec2 vUv; out vec4 o;
void main() { vec3 c = texture(uTex, vUv).rgb; float l = max(c.r, max(c.g, c.b)); o = vec4(c * smoothstep(uThreshold, uThreshold + 0.25, l), 1.0); }`;
const BLUR_FS = `
uniform sampler2D uTex; uniform vec2 uDir;
in vec2 vUv; out vec4 o;
void main() {
    vec3 c = texture(uTex, vUv).rgb * 0.227;
    c += (texture(uTex, vUv + uDir * 1.38).rgb + texture(uTex, vUv - uDir * 1.38).rgb) * 0.316;
    c += (texture(uTex, vUv + uDir * 3.23).rgb + texture(uTex, vUv - uDir * 3.23).rgb) * 0.070;
    o = vec4(c, 1.0);
}`;
const FINAL_FS = `
uniform sampler2D uHdr, uBloom;
uniform float uBloomK, uTime, uVignette, uGrain, uExposure;
uniform vec3 uLift, uGain; uniform float uSat, uContrast;
uniform vec4 uFlash;
in vec2 vUv; out vec4 o;
void main() {
    vec3 c = texture(uHdr, vUv).rgb + texture(uBloom, vUv).rgb * uBloomK;
    c *= uExposure;
    c = c / (1.0 + c * 0.25); // gentle shoulder so glows do not clip hard
    // grading per zone: lift shadows, gain highlights, saturation, contrast
    float l = dot(c, vec3(0.299, 0.587, 0.114));
    c = mix(vec3(l), c, uSat);
    c = (c - 0.5) * uContrast + 0.5;
    c = c * uGain + uLift * (1.0 - c);
    c = mix(c, uFlash.rgb, uFlash.a);
    vec2 q = vUv - 0.5;
    c *= 1.0 - uVignette * smoothstep(0.25, 0.75, length(q * vec2(1.25, 1.0)));
    float n = fract(sin(dot(floor(vUv * vec2(640.0, 360.0)) + floor(uTime * 24.0), vec2(12.9898, 78.233))) * 43758.5453);
    c += (n - 0.5) * uGrain;
    o = vec4(clamp(c, 0.0, 1.0), 1.0);
}`;
const BLIT_FS = `uniform sampler2D uTex; in vec2 vUv; out vec4 o; void main() { o = texture(uTex, vUv); }`;

const STRIDE = 12; // floats per vertex: pos2 uv2 col4 ext4

class Batch {
    constructor(gl, max = 16384) {
        this.gl = gl; this.max = max;
        this.data = new Float32Array(max * 4 * STRIDE);
        this.n = 0; this.tex = null;
        this.vao = gl.createVertexArray();
        gl.bindVertexArray(this.vao);
        this.vbo = gl.createBuffer();
        gl.bindBuffer(gl.ARRAY_BUFFER, this.vbo);
        gl.bufferData(gl.ARRAY_BUFFER, this.data.byteLength, gl.DYNAMIC_DRAW);
        const idx = new Uint32Array(max * 6);
        for (let i = 0; i < max; i++) idx.set([i * 4, i * 4 + 1, i * 4 + 2, i * 4, i * 4 + 2, i * 4 + 3], i * 6);
        this.ibo = gl.createBuffer();
        gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, this.ibo);
        gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, idx, gl.STATIC_DRAW);
        const F = 4;
        gl.enableVertexAttribArray(0); gl.vertexAttribPointer(0, 2, gl.FLOAT, false, STRIDE * F, 0);
        gl.enableVertexAttribArray(1); gl.vertexAttribPointer(1, 2, gl.FLOAT, false, STRIDE * F, 2 * F);
        gl.enableVertexAttribArray(2); gl.vertexAttribPointer(2, 4, gl.FLOAT, false, STRIDE * F, 4 * F);
        gl.enableVertexAttribArray(3); gl.vertexAttribPointer(3, 4, gl.FLOAT, false, STRIDE * F, 8 * F);
        gl.bindVertexArray(null);
    }
    // quad: x,y,w,h in target pixels; u0..v1 normalized; col rgba; mode, sway, worldX
    quad(x, y, w, h, u0, v0, u1, v1, r, g, b, a, mode = 0, sway = 0, wx = 0) {
        if (this.n >= this.max) this.flush();
        const d = this.data, o = this.n * 4 * STRIDE;
        const put = (k, px, py, u, v, top) => {
            const p = o + k * STRIDE;
            d[p] = px; d[p + 1] = py; d[p + 2] = u; d[p + 3] = v;
            d[p + 4] = r; d[p + 5] = g; d[p + 6] = b; d[p + 7] = a;
            d[p + 8] = mode; d[p + 9] = sway; d[p + 10] = top; d[p + 11] = wx;
        };
        put(0, x, y, u0, v0, 1); put(1, x + w, y, u1, v0, 1); put(2, x + w, y + h, u1, v1, 0); put(3, x, y + h, u0, v1, 0);
        this.n++;
    }
    flush() {
        if (!this.n) return;
        const gl = this.gl;
        gl.bindVertexArray(this.vao);
        gl.bindBuffer(gl.ARRAY_BUFFER, this.vbo);
        gl.bufferSubData(gl.ARRAY_BUFFER, 0, this.data, 0, this.n * 4 * STRIDE);
        gl.drawElements(gl.TRIANGLES, this.n * 6, gl.UNSIGNED_INT, 0);
        this.n = 0;
    }
}

export class Renderer {
    constructor(canvas) {
        this.canvas = canvas;
        const gl = this.gl = createGL(canvas);
        this.batch = new Batch(gl);
        this.progs = {
            sprite: program(gl, SPRITE_VS, SPRITE_FS, 'sprite'),
            water: program(gl, QUAD_VS, WATER_FS, 'water'),
            lightmap: program(gl, QUAD_VS, LIGHTMAP_FS, 'lightmap'),
            light: program(gl, LIGHT_VS, LIGHT_FS, 'light'),
            compose: program(gl, QUAD_VS, COMPOSE_FS, 'compose'),
            bright: program(gl, QUAD_VS, BRIGHT_FS, 'bright'),
            blur: program(gl, QUAD_VS, BLUR_FS, 'blur'),
            final: program(gl, QUAD_VS, FINAL_FS, 'final'),
            blit: program(gl, QUAD_VS, BLIT_FS, 'blit')
        };
        this.quadVao = gl.createVertexArray();
        gl.bindVertexArray(this.quadVao);
        const qb = gl.createBuffer();
        gl.bindBuffer(gl.ARRAY_BUFFER, qb);
        gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([-1, -1, 1, -1, 1, 1, -1, -1, 1, 1, -1, 1]), gl.STATIC_DRAW);
        gl.enableVertexAttribArray(0); gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 0, 0);
        gl.bindVertexArray(null);
        this.time = 0;
        this.W = 640; this.H = 360;
        this.grade = { lift: [0.02, 0.03, 0.05], gain: [1, 1, 1], sat: 0.8, contrast: 1.05 };
        this.vignette = 0.25; this.grain = 0.03; this.bloomK = 0.9; this.threshold = 0.75; this.exposure = 1;
        this.flash = [1, 1, 1, 0];
        this.quality = 'high';
        this.resize(640, 360);
    }
    resize(W, H) {
        const gl = this.gl;
        if (this.W === W && this.H === H && this.rt) return;
        this.W = W; this.H = H;
        for (const k of ['scene', 'light', 'glow', 'hdr', 'b1', 'b2']) freeTarget(gl, this.rt?.[k]);
        this.rt = {
            scene: target(gl, W, H),
            light: target(gl, W, H, { hdr: true }),
            glow: target(gl, W, H, { hdr: true }),
            hdr: target(gl, W, H, { hdr: true, linear: true }),
            b1: target(gl, Math.ceil(W / 4), Math.ceil(H / 4), { hdr: true, linear: true }),
            b2: target(gl, Math.ceil(W / 4), Math.ceil(H / 4), { hdr: true, linear: true })
        };
    }
    // ---------------------------------------------------------------- drawing helpers
    use(prog) { this.gl.useProgram(prog.p); this.cur = prog; return prog.u; }
    bindTarget(t) {
        const gl = this.gl;
        gl.bindFramebuffer(gl.FRAMEBUFFER, t ? t.fb : null);
        if (t) gl.viewport(0, 0, t.w, t.h);
    }
    spriteBegin(blend = 'alpha') {
        const gl = this.gl;
        const u = this.use(this.progs.sprite);
        gl.uniform2f(u.uRes, this.W, this.H);
        gl.uniform1f(u.uTime, this.time);
        gl.uniform4f(u.uOutline, 0.37, 0.95, 0.78, 1);
        gl.enable(gl.BLEND);
        if (blend === 'alpha') gl.blendFuncSeparate(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA, gl.ONE, gl.ONE_MINUS_SRC_ALPHA);
        else if (blend === 'add') gl.blendFunc(gl.SRC_ALPHA, gl.ONE);
        else if (blend === 'mul') gl.blendFunc(gl.DST_COLOR, gl.ZERO);
        this.batch.tex = null;
    }
    setTex(tex) {
        if (this.batch.tex === tex) return;
        this.batch.flush();
        const gl = this.gl;
        gl.activeTexture(gl.TEXTURE0);
        gl.bindTexture(gl.TEXTURE_2D, tex);
        gl.uniform1i(this.cur.u.uTex, 0);
        gl.uniform2f(this.cur.u.uTexel, 1 / tex.w, 1 / tex.h);
        this.batch.tex = tex;
    }
    // draw one sprite: s = { tex, x, y, w, h } (atlas rect), at target pixel (x, y) top-left
    sprite(s, x, y, o = {}) {
        this.setTex(o.tex || s.tex);
        const t = this.batch.tex;
        let u0 = s.x / t.w, u1 = (s.x + s.w) / t.w;
        const v0 = s.y / t.h, v1 = (s.y + s.h) / t.h;
        if (o.flip) [u0, u1] = [u1, u0];
        const c = o.color || [1, 1, 1, 1];
        let qx = Math.round(x), qy = Math.round(y), qw = s.w * (o.sx || 1), qh = s.h * (o.sy || 1);
        let uu0 = u0, uu1 = u1, vv0 = v0, vv1 = v1;
        if (o.outline) { // grow the quad by a pixel so the outline fits
            const du = (u1 - u0) / s.w, dv = (v1 - v0) / s.h;
            qx -= 1; qy -= 1; qw += 2; qh += 2; uu0 -= du; uu1 += du; vv0 -= dv; vv1 += dv;
        }
        this.batch.quad(qx, qy, qw, qh, uu0, vv0, uu1, vv1, c[0], c[1], c[2], c[3] ?? 1, o.outline ? 1 : (o.flash ? 2 : 0), o.sway || 0, o.wx || 0);
    }
    rect(tex, x, y, w, h, c) { // solid rectangle using a 1×1 white texel at atlas (0,0)
        this.setTex(tex);
        this.batch.quad(Math.round(x), Math.round(y), w, h, 0.5 / tex.w, 0.5 / tex.h, 0.5 / tex.w, 0.5 / tex.h, c[0], c[1], c[2], c[3] ?? 1);
    }
    end() { this.batch.flush(); }
    fullscreen() { const gl = this.gl; gl.bindVertexArray(this.quadVao); gl.drawArrays(gl.TRIANGLES, 0, 6); }
    texUnit(i, tex, name) { const gl = this.gl; gl.activeTexture(gl.TEXTURE0 + i); gl.bindTexture(gl.TEXTURE_2D, tex); gl.uniform1i(this.cur.u[name], i); }

    // ---------------------------------------------------------------- frame
    // frame = { cam:[x,y], drawScene(r), water:{mask,mapSize,colors}, ambient, lightmap, lights:[{x,y,r,col,i}], drawGlow(r), drawUI(r) }
    render(frame, dt) {
        const gl = this.gl;
        this.time += dt;
        gl.disable(gl.DEPTH_TEST);
        gl.disable(gl.CULL_FACE);
        // 1. scene colour
        this.bindTarget(this.rt.scene);
        gl.clearColor(0.043, 0.063, 0.086, 1);
        gl.clear(gl.COLOR_BUFFER_BIT);
        frame.drawScene?.(this);
        // 2. light buffer
        this.bindTarget(this.rt.light);
        gl.disable(gl.BLEND);
        {
            const u = this.use(this.progs.lightmap);
            const a = frame.ambient || [1, 1, 1];
            gl.uniform3f(u.uAmbient, a[0], a[1], a[2]);
            gl.uniform2f(u.uCam, frame.cam[0], frame.cam[1]);
            gl.uniform2f(u.uRes, this.W, this.H);
            gl.uniform1f(u.uHasLM, frame.lightmap ? 1 : 0);
            if (frame.lightmap) { this.texUnit(0, frame.lightmap.tex, 'uLM'); gl.uniform2f(u.uLMSize, frame.lightmap.w, frame.lightmap.h); }
            this.fullscreen();
        }
        if (frame.lights?.length) {
            const u = this.use(this.progs.light);
            gl.uniform2f(u.uRes, this.W, this.H);
            gl.enable(gl.BLEND); gl.blendFunc(gl.ONE, gl.ONE);
            this.batch.tex = null;
            for (const L of frame.lights) {
                const x = L.x - frame.cam[0], y = L.y - frame.cam[1];
                if (x + L.r < 0 || y + L.r < 0 || x - L.r > this.W || y - L.r > this.H) continue;
                this.batch.quad(Math.round(x - L.r), Math.round(y - L.r), L.r * 2, L.r * 2, 0, 0, 1, 1, L.col[0], L.col[1], L.col[2], L.i ?? 1);
            }
            this.batch.flush();
        }
        // 3. glow buffer (emissive)
        this.bindTarget(this.rt.glow);
        gl.clearColor(0, 0, 0, 1);
        gl.clear(gl.COLOR_BUFFER_BIT);
        frame.drawGlow?.(this);
        // 4. compose
        this.bindTarget(this.rt.hdr);
        gl.disable(gl.BLEND);
        {
            this.use(this.progs.compose);
            this.texUnit(0, this.rt.scene.tex, 'uScene'); this.texUnit(1, this.rt.light.tex, 'uLight'); this.texUnit(2, this.rt.glow.tex, 'uGlow');
            this.fullscreen();
        }
        // UI is drawn into the composed image (unlit, pixel exact)
        if (frame.drawUI) { gl.enable(gl.BLEND); frame.drawUI(this); gl.disable(gl.BLEND); }
        // 5. bloom
        const post = this.quality !== 'low';
        if (post) {
            this.bindTarget(this.rt.b1);
            let u = this.use(this.progs.bright);
            gl.uniform1f(u.uThreshold, this.threshold);
            this.texUnit(0, this.rt.hdr.tex, 'uTex');
            this.fullscreen();
            u = this.use(this.progs.blur);
            for (let k = 0; k < 2; k++) {
                this.bindTarget(this.rt.b2); gl.uniform2f(u.uDir, 1 / this.rt.b1.w, 0); this.texUnit(0, this.rt.b1.tex, 'uTex'); this.fullscreen();
                this.bindTarget(this.rt.b1); gl.uniform2f(u.uDir, 0, 1 / this.rt.b1.h); this.texUnit(0, this.rt.b2.tex, 'uTex'); this.fullscreen();
            }
        }
        // 6. final to the screen: integer scale, letterbox
        gl.bindFramebuffer(gl.FRAMEBUFFER, null);
        const cw = this.canvas.width, ch = this.canvas.height;
        gl.viewport(0, 0, cw, ch);
        gl.clearColor(0, 0, 0, 1); gl.clear(gl.COLOR_BUFFER_BIT);
        const k = this.scale;
        const vw = this.W * k, vh = this.H * k;
        gl.viewport(Math.floor((cw - vw) / 2), Math.floor((ch - vh) / 2), vw, vh);
        const u = this.use(this.progs.final);
        this.texUnit(0, this.rt.hdr.tex, 'uHdr');
        this.texUnit(1, this.rt.b1.tex, 'uBloom');
        gl.uniform1f(u.uBloomK, post ? this.bloomK : 0);
        gl.uniform1f(u.uTime, this.time);
        gl.uniform1f(u.uVignette, this.vignette);
        gl.uniform1f(u.uGrain, post ? this.grain : 0);
        gl.uniform1f(u.uExposure, this.exposure);
        const G = this.grade;
        gl.uniform3fv(u.uLift, G.lift); gl.uniform3fv(u.uGain, G.gain);
        gl.uniform1f(u.uSat, G.sat); gl.uniform1f(u.uContrast, G.contrast);
        gl.uniform4fv(u.uFlash, this.flash);
        // nearest sampling of the hdr target for the upscale (it is linear for bloom reads)
        gl.bindTexture(gl.TEXTURE_2D, this.rt.hdr.tex);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
        this.fullscreen();
        gl.bindTexture(gl.TEXTURE_2D, this.rt.hdr.tex);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
    }
    // water pass, called from drawScene between ground and deco layers
    drawWater(w, cam) {
        const gl = this.gl;
        this.batch.flush();
        const u = this.use(this.progs.water);
        gl.enable(gl.BLEND); gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);
        this.texUnit(0, w.mask, 'uMask');
        gl.uniform2f(u.uMapSize, w.mapW, w.mapH);
        gl.uniform2f(u.uCam, cam[0], cam[1]);
        gl.uniform2f(u.uRes, this.W, this.H);
        gl.uniform1f(u.uTime, this.time);
        const c = w.colors;
        gl.uniform3fv(u.uShallow, c.shallow); gl.uniform3fv(u.uDeep, c.deep); gl.uniform3fv(u.uGlint, c.glint); gl.uniform3fv(u.uFoam, c.foam); gl.uniform3fv(u.uSoul, c.soul);
        this.fullscreen();
    }
    // bake static lights into a world-space lightmap (rendered once per world)
    bakeLightmap(worldW, worldH, lights, scale = 4) {
        const gl = this.gl;
        const w = Math.ceil(worldW / scale), h = Math.ceil(worldH / scale);
        const t = target(gl, w, h, { linear: true });
        this.bindTarget(t);
        gl.clearColor(0, 0, 0, 1); gl.clear(gl.COLOR_BUFFER_BIT);
        const u = this.use(this.progs.light);
        gl.uniform2f(u.uRes, w, h);
        gl.enable(gl.BLEND); gl.blendFunc(gl.ONE, gl.ONE);
        for (const L of lights) {
            const r = L.r / scale;
            this.batch.quad(L.x / scale - r, L.y / scale - r, r * 2, r * 2, 0, 0, 1, 1, L.col[0] * 0.5, L.col[1] * 0.5, L.col[2] * 0.5, L.i ?? 1);
        }
        this.batch.flush();
        gl.disable(gl.BLEND);
        return { tex: t.tex, w: worldW, h: worldH };
    }
    texture(src, opts) { return texture(this.gl, src, opts); }
}
