// App bootstrap, device detection, resolution (up to 4K), top-down orthographic camera,
// moonlight and post-processing.
import {
    Application, Entity, Color, Keyboard, Mouse, CameraFrame,
    FILLMODE_NONE, RESOLUTION_FIXED, PROJECTION_ORTHOGRAPHIC, TONEMAP_ACES2,
    SHADOW_PCF3_32F, SHADOWUPDATE_REALTIME
} from 'playcanvas';

// world framing: on 16:9 the view is 13.5 tiles tall (as in A Link to the Past);
// in portrait the narrow side shows about 10.5 tiles
const LANDSCAPE_HALF_H = 6.75;
const PORTRAIT_W = 10.5;

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

export function createStage(canvas) {
    const device = detectDevice();
    const app = new Application(canvas, {
        keyboard: new Keyboard(window),
        mouse: new Mouse(canvas),
        graphicsDeviceOptions: { antialias: false, alpha: false, powerPreference: 'high-performance' }
    });
    app.setCanvasFillMode(FILLMODE_NONE);
    const params = new URLSearchParams(location.search);
    // quality: 'ultra' renders up to 3840 × 2160, 'high' at the screen's own resolution, 'low' at half, without shadows and post
    let quality = params.has('lowfx') ? 'low' : (store.get('sw-quality') || (device.detectedTouch ? 'high' : 'ultra'));
    if (!['ultra', 'high', 'low'].includes(quality)) quality = 'high';

    const scene = app.scene;
    scene.ambientLight = new Color(0.09, 0.11, 0.17);
    scene.lighting.shadowsEnabled = true;
    scene.lighting.shadowAtlasResolution = device.detectedTouch ? 2048 : 4096;
    scene.lighting.maxLightsPerCell = 16;

    const camera = new Entity('camera');
    camera.addComponent('camera', {
        projection: PROJECTION_ORTHOGRAPHIC,
        orthoHeight: LANDSCAPE_HALF_H,
        nearClip: 1,
        farClip: 80,
        clearColor: new Color(0.01, 0.02, 0.035)
    });
    camera.setEulerAngles(-90, 0, 0);
    camera.setPosition(0, 40, 0);
    app.root.addChild(camera);

    // moonlight: cold, low, casting long shadows
    const moon = new Entity('moon');
    moon.addComponent('light', {
        type: 'directional',
        color: new Color(0.55, 0.66, 1.0),
        intensity: 1.1,
        castShadows: true,
        shadowType: SHADOW_PCF3_32F,
        shadowResolution: device.detectedTouch ? 2048 : 4096,
        shadowDistance: 60,
        shadowBias: 0.25,
        normalOffsetBias: 0.1,
        shadowIntensity: 0.85,
        shadowUpdateMode: SHADOWUPDATE_REALTIME
    });
    moon.setEulerAngles(-48, -35, 0);
    app.root.addChild(moon);

    const frame = new CameraFrame(app, camera.camera);
    frame.rendering.toneMapping = TONEMAP_ACES2;
    frame.bloom.intensity = 0.035;
    frame.vignette.intensity = 0.55;
    frame.vignette.inner = 0.35;
    frame.vignette.outer = 1.15;
    frame.vignette.curvature = 0.6;
    frame.grading.enabled = true;
    frame.grading.saturation = 0.92;
    frame.grading.contrast = 1.08;
    frame.grading.brightness = 1.05;
    frame.grading.tint = new Color(0.97, 0.98, 1.04);

    const stage = { app, camera, moon, frame, device, ortho: LANDSCAPE_HALF_H, portrait: false, quality, pixels: [0, 0] };

    const fit = () => {
        const ww = window.innerWidth, wh = window.innerHeight;
        const portrait = wh > ww;
        let cw, ch;
        if (!device.touch && !portrait) {
            // desktop: largest 16:9 area, letterboxed
            const s = Math.min(ww / 16, wh / 9);
            cw = Math.floor(16 * s); ch = Math.floor(9 * s);
        } else {
            // phone / tablet: use the whole screen in either orientation
            cw = ww; ch = wh;
        }
        canvas.style.width = `${cw}px`;
        canvas.style.height = `${ch}px`;
        const dpr = window.devicePixelRatio || 1;
        // longest side capped at 3840 (4K), shortest at 2160
        const cap = Math.min(3840 / Math.max(cw, ch), 2160 / Math.min(cw, ch));
        let ratio;
        if (stage.quality === 'ultra') ratio = cap; // supersample up to 4K
        else if (stage.quality === 'high') ratio = Math.min(dpr, device.detectedTouch ? 2 : dpr, cap);
        else ratio = Math.min(dpr, cap) * 0.5;
        const w = Math.max(320, Math.round(cw * ratio)), h = Math.max(180, Math.round(ch * ratio));
        app.setCanvasResolution(RESOLUTION_FIXED, w, h);
        stage.pixels = [w, h];
        const aspect = cw / ch;
        stage.portrait = portrait;
        stage.ortho = aspect >= 1 ? LANDSCAPE_HALF_H : PORTRAIT_W / (2 * aspect);
        camera.camera.orthoHeight = stage.ortho;
        frame.bloom.blurLevel = h > 1500 ? 12 : 8;
        frame.update();
        document.body.classList.toggle('portrait', portrait);
        document.body.classList.toggle('touch', device.touch);
        stage.onResize?.();
    };
    window.addEventListener('resize', fit);
    window.addEventListener('orientationchange', () => setTimeout(fit, 150));
    window.visualViewport?.addEventListener('resize', fit);

    stage.setQuality = (q) => {
        stage.quality = q;
        const low = q === 'low';
        scene.lighting.shadowsEnabled = !low;
        moon.light.castShadows = !low;
        frame.enabled = !low;
        store.set('sw-quality', q);
        fit();
    };
    stage.isLow = () => stage.quality === 'low';
    stage.setTouch = (on) => {
        device.touch = on;
        store.set('sw-controls', on ? 'touch' : 'keys');
        fit();
    };
    stage.fit = fit;
    stage.setQuality(quality);
    return stage;
}
