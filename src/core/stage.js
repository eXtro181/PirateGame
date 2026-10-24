// App bootstrap, device detection, Full-HD reference resolution, the fixed isometric camera,
// lighting environment, fog and the post-processing chain (HDR → SSAO → bloom → ACES → grading → vignette → TAA).
import {
    Application, Entity, Color, Vec3, Keyboard, Mouse, CameraFrame,
    FILLMODE_NONE, RESOLUTION_FIXED, PROJECTION_PERSPECTIVE, TONEMAP_ACES, TONEMAP_LINEAR, SSAOTYPE_LIGHTING,
    SHADOW_PCF3_32F, SHADOWUPDATE_REALTIME, FOG_EXP2, FOG_NONE
} from 'playcanvas';

// Iso camera: turned 45°, tilted 55°, small field of view so it reads almost orthographic.
export const CAM = { pitch: 55, yaw: 45, fov: 30, dist: 22 };
// the same slice of the world is visible at every resolution: VIEW units across the short side
const VIEW = 2 * CAM.dist * Math.tan(CAM.fov / 2 * Math.PI / 180);

const store = {
    get(k) { try { return localStorage.getItem(k); } catch { return null; } },
    set(k, v) { try { localStorage.setItem(k, v); } catch { /* ignore */ } }
};

export function detectDevice() {
    const ua = navigator.userAgent || '';
    const iOS = /iPhone|iPad|iPod/.test(ua) || (navigator.platform === 'MacIntel' && navigator.maxTouchPoints > 1);
    const coarse = window.matchMedia?.('(pointer: coarse)').matches;
    const touch = iOS || /Android|Mobile/i.test(ua) || (coarse && navigator.maxTouchPoints > 0);
    const forced = store.get('sw-controls');
    return { iOS, touch: forced ? forced === 'touch' : !!touch, detectedTouch: !!touch };
}

// quality presets: render-height cap, SSAO, volumetric fog, TAA, shadow sizes
export const QUALITY = {
    ultra: { label: 'Ultra (bis 4K)', cap: 2160, ssao: true, vfog: true, taa: true, shadows: true, atlas: 4096, post: true },
    high: { label: 'Hoch (Full HD)', cap: 1080, ssao: true, vfog: true, taa: false, shadows: true, atlas: 4096, post: true },
    medium: { label: 'Mittel', cap: 810, ssao: false, vfog: false, taa: false, shadows: true, atlas: 2048, post: true },
    low: { label: 'Niedrig', cap: 540, ssao: false, vfog: false, taa: false, shadows: false, atlas: 1024, post: false }
};
export const QUALITY_ORDER = ['ultra', 'high', 'medium', 'low'];

export function createStage(canvas) {
    const device = detectDevice();
    const app = new Application(canvas, {
        keyboard: new Keyboard(window),
        mouse: new Mouse(canvas),
        graphicsDeviceOptions: { antialias: false, alpha: false, powerPreference: 'high-performance' }
    });
    app.setCanvasFillMode(FILLMODE_NONE);
    const params = new URLSearchParams(location.search);
    let quality = params.has('lowfx') ? 'low' : (store.get('sw-quality2') || (device.detectedTouch ? 'medium' : 'high'));
    if (!QUALITY[quality]) quality = 'high';
    if (params.get('q') && QUALITY[params.get('q')]) quality = params.get('q');
    const renderScale = Number(params.get('rs')) || 1;

    const scene = app.scene;
    // almost no ambient light: a faint, cold moon glow. Everything else comes from local lights.
    scene.ambientLight = new Color(0.11, 0.13, 0.18);
    scene.exposure = Number(store.get('sw-gamma')) || 1;
    scene.lighting.shadowsEnabled = true;
    scene.lighting.shadowAtlasResolution = 4096;
    scene.lighting.maxLightsPerCell = 20;
    scene.lighting.cells = new Vec3(14, 4, 14);

    const camera = new Entity('camera');
    camera.addComponent('camera', {
        projection: PROJECTION_PERSPECTIVE,
        fov: CAM.fov,
        nearClip: 6,
        farClip: 80,
        clearColor: new Color(0.004, 0.006, 0.01)
    });
    camera.setEulerAngles(-CAM.pitch, CAM.yaw, 0);
    app.root.addChild(camera);

    // the moon: a weak, cold directional light without shadows – it only gives shapes a rim
    const moon = new Entity('moon');
    moon.addComponent('light', {
        type: 'directional', color: new Color(0.45, 0.58, 0.9), intensity: 0.8,
        castShadows: false
    });
    moon.setEulerAngles(-50, 160, 0);
    app.root.addChild(moon);

    const frame = new CameraFrame(app, camera.camera);
    frame.rendering.toneMapping = TONEMAP_ACES;
    frame.rendering.sharpness = 0.25;
    frame.bloom.intensity = 0.045;
    frame.bloom.threshold = 1.0; // only fire, lightning and magic glow
    frame.bloom.blurLevel = 10;
    frame.vignette.intensity = 0.62;
    frame.vignette.inner = 0.32;
    frame.vignette.outer = 1.1;
    frame.vignette.curvature = 0.55;
    // grading: desaturated, shadows pushed to blue-green, highlights amber, blacks lifted a little
    frame.grading.enabled = true;
    frame.grading.saturation = 0.78;
    frame.grading.contrast = 1.12;
    frame.grading.brightness = 1.0;
    frame.grading.tint = new Color(1.0, 0.97, 0.92);
    frame.colorEnhance.enabled = true;
    frame.colorEnhance.shadows = 0.18;
    frame.colorEnhance.highlights = -0.1;
    frame.colorEnhance.midtones = 0.0;
    frame.colorEnhance.dehaze = 0.0;
    frame.ssao.intensity = 0.55;
    frame.ssao.radius = 18;
    frame.ssao.samples = 10;
    frame.ssao.blurEnabled = true;
    const vf = frame.volumetricFog;
    vf.light = moon.light;
    vf.localOmniLights = true;
    vf.localIntensity = 0.25;
    vf.tint = new Color(0.55, 0.62, 0.6);
    vf.density = 0.05;
    vf.heightBase = 0.0;
    vf.heightFalloff = 0.65;
    vf.intensity = 0.25;
    vf.ambientColor = new Color(0.25, 0.32, 0.3);
    vf.ambientIntensity = 0.05;
    vf.maxDistance = 60;
    vf.steps = 20;
    vf.scale = 0.5;

    const stage = { app, camera, moon, frame, device, portrait: false, quality, pixels: [0, 0], cssSize: [1, 1], view: VIEW, fogColor: new Color(0.03, 0.045, 0.045) };

    const fit = () => {
        const ww = window.innerWidth, wh = window.innerHeight;
        const portrait = wh > ww;
        let cw, ch;
        if (!device.touch && !portrait) { // desktop: largest 16:9 area, letterboxed
            const s = Math.min(ww / 16, wh / 9);
            cw = Math.floor(16 * s); ch = Math.floor(9 * s);
        } else { cw = ww; ch = wh; }
        canvas.style.width = `${cw}px`;
        canvas.style.height = `${ch}px`;
        const dpr = window.devicePixelRatio || 1;
        const q = QUALITY[stage.quality];
        const short = Math.min(cw, ch);
        const ratio = Math.min(dpr * 2, q.cap / short) * renderScale; // render at the preset's height (e.g. 1080 lines)
        const w = Math.max(320, Math.round(cw * ratio)), h = Math.max(180, Math.round(ch * ratio));
        app.setCanvasResolution(RESOLUTION_FIXED, w, h);
        stage.pixels = [w, h];
        stage.cssSize = [cw, ch];
        // the HUD covers exactly the canvas (letterboxed on desktop)
        const root = document.documentElement.style;
        root.setProperty('--cv-l', `${(ww - cw) / 2}px`); root.setProperty('--cv-t', `${(wh - ch) / 2}px`);
        root.setProperty('--cv-w', `${cw}px`); root.setProperty('--cv-h', `${ch}px`);
        stage.portrait = portrait;
        // fixed visible radius: the short screen side always spans VIEW units
        if (cw >= ch) { camera.camera.horizontalFov = false; camera.camera.fov = CAM.fov; }
        else { camera.camera.horizontalFov = true; camera.camera.fov = CAM.fov; }
        frame.update();
        document.body.classList.toggle('portrait', portrait);
        document.body.classList.toggle('touch', device.touch);
        stage.onResize?.();
    };
    window.addEventListener('resize', fit);
    window.addEventListener('orientationchange', () => setTimeout(fit, 150));
    window.visualViewport?.addEventListener('resize', fit);

    stage.setQuality = (qname) => {
        stage.quality = qname;
        const q = QUALITY[qname];
        scene.lighting.shadowsEnabled = q.shadows;
        scene.lighting.shadowAtlasResolution = q.atlas;
        frame.enabled = q.post;
        camera.camera.toneMapping = q.post ? TONEMAP_LINEAR : TONEMAP_ACES;
        frame.ssao.type = q.ssao ? SSAOTYPE_LIGHTING : 'none';
        frame.volumetricFog.enabled = q.vfog;
        frame.taa.enabled = q.taa;
        frame.rendering.sharpness = q.taa ? 0.5 : 0.2;
        // without volumetric fog a classic exponential fog keeps the depth haze
        scene.fog.type = q.vfog ? FOG_NONE : FOG_EXP2;
        scene.fog.color = stage.fogColor;
        scene.fog.density = 0.012;
        store.set('sw-quality2', qname);
        stage.onQuality?.(q);
        fit();
    };
    stage.setGamma = (g) => { scene.exposure = g; store.set('sw-gamma', String(g)); };
    stage.isLow = () => stage.quality === 'low';
    stage.setTouch = (on) => { device.touch = on; store.set('sw-controls', on ? 'touch' : 'keys'); fit(); };
    stage.fit = fit;
    // place the camera so it looks at (x, y, z); zoom scales the distance
    stage.lookAt = (x, y, z, zoom = 1) => {
        const d = CAM.dist * zoom, p = CAM.pitch * Math.PI / 180, yw = CAM.yaw * Math.PI / 180;
        camera.setPosition(x + Math.sin(yw) * Math.cos(p) * d, y + Math.sin(p) * d, z + Math.cos(yw) * Math.cos(p) * d);
        camera.setEulerAngles(-CAM.pitch, CAM.yaw, 0);
    };
    stage.setQuality(quality);
    return stage;
}
