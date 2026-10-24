// App bootstrap, pixel-perfect canvas, top-down orthographic camera, moonlight and post-processing.
import {
    Application, Entity, Color, Keyboard, Mouse, CameraFrame,
    FILLMODE_NONE, RESOLUTION_FIXED, PROJECTION_ORTHOGRAPHIC, TONEMAP_ACES2,
    SHADOW_PCF3_32F, SHADOWUPDATE_REALTIME
} from 'playcanvas';

export const VIEW_W = 384, VIEW_H = 216;
const ORTHO = VIEW_H / 2 / 16;

export function createStage(canvas) {
    const app = new Application(canvas, {
        keyboard: new Keyboard(window),
        mouse: new Mouse(canvas),
        graphicsDeviceOptions: { antialias: false, alpha: false, powerPreference: 'high-performance' }
    });
    app.setCanvasFillMode(FILLMODE_NONE);
    app.setCanvasResolution(RESOLUTION_FIXED, VIEW_W, VIEW_H);
    app.graphicsDevice.maxPixelRatio = 1;

    const fit = () => {
        const s = Math.max(1, Math.min(Math.floor(window.innerWidth / VIEW_W), Math.floor(window.innerHeight / VIEW_H)));
        const fs = Math.min(window.innerWidth / VIEW_W, window.innerHeight / VIEW_H);
        const scale = fs >= 1 ? s : fs; // integer scaling when possible
        canvas.style.width = `${VIEW_W * scale}px`;
        canvas.style.height = `${VIEW_H * scale}px`;
        document.documentElement.style.setProperty('--ui-scale', String(Math.max(1, scale / 3)));
    };
    window.addEventListener('resize', fit);
    fit();

    const scene = app.scene;
    scene.ambientLight = new Color(0.09, 0.11, 0.17);
    scene.lighting.shadowsEnabled = true;
    scene.lighting.shadowAtlasResolution = 2048;
    scene.lighting.maxLightsPerCell = 16;

    const camera = new Entity('camera');
    camera.addComponent('camera', {
        projection: PROJECTION_ORTHOGRAPHIC,
        orthoHeight: ORTHO,
        nearClip: 1,
        farClip: 80,
        clearColor: new Color(0.01, 0.02, 0.035)
    });
    camera.setEulerAngles(-90, 0, 0);
    camera.setPosition(0, 40, 0);
    app.root.addChild(camera);

    // moonlight: cold, low, casting long shadows towards the bottom right
    const moon = new Entity('moon');
    moon.addComponent('light', {
        type: 'directional',
        color: new Color(0.55, 0.66, 1.0),
        intensity: 1.1,
        castShadows: true,
        shadowType: SHADOW_PCF3_32F,
        shadowResolution: 2048,
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
    frame.bloom.blurLevel = 6;
    frame.vignette.intensity = 0.55;
    frame.vignette.inner = 0.35;
    frame.vignette.outer = 1.15;
    frame.vignette.curvature = 0.6;
    frame.grading.enabled = true;
    frame.grading.saturation = 0.92;
    frame.grading.contrast = 1.08;
    frame.grading.brightness = 1.05;
    frame.grading.tint = new Color(0.97, 0.98, 1.04);
    frame.update();

    // low quality mode for weak GPUs: no shadows, no post-processing
    const setQuality = (low) => {
        scene.lighting.shadowsEnabled = !low;
        moon.light.castShadows = !low;
        frame.enabled = !low;
        try { localStorage.setItem('sw-quality', low ? 'low' : 'high'); } catch { /* ignore */ }
    };
    let low = new URLSearchParams(location.search).has('lowfx');
    try { low = low || localStorage.getItem('sw-quality') === 'low'; } catch { /* ignore */ }
    if (low) setQuality(true);
    return { app, camera, moon, frame, ortho: ORTHO, setQuality, isLow: () => !frame.enabled };
}
