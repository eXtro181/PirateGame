// Schwarzwasser – entry point: renderer, generated art, world, scenes, input, main loop.
import { Renderer } from './px/renderer.js';
import { buildArt } from './art/index.js';
import { World, ZONES } from './world/world.js';
import { WorldView } from './world/view.js';
import { Input } from './core/input.js';
import { Audio } from './audio/audio.js';
import { ProfileStore } from './save/profiles.js';
import { MenuScene } from './scenes/menu.js';
import { GameScene } from './game/game.js';
import { CanvasUI } from './ui/hud.js';
import { UI } from './ui/ui.js';
import { Touch } from './ui/touch.js';
import { newSave } from './game/data.js';
import { DEFAULT_LOOK } from './art/chars.js';
import { gradesFor } from './art/pal.js';
import { TS } from './art/tiles.js';
import { loadMonsters } from './art/monsters.js';

const params = new URLSearchParams(location.search);
const DEBUG = params.has('debug');
if (DEBUG) addEventListener('error', e => console.error('STACK', e.error?.stack));
const loading = document.getElementById('loading');
const say = t => { if (loading) loading.textContent = t; };
const nextFrame = () => new Promise(r => requestAnimationFrame(() => setTimeout(r, 0)));
const OPT_KEY = 'schwarzwasser-options';

async function boot() {
    const canvas = document.getElementById('game');
    say('Die Laternen werden entzündet …'); await nextFrame();
    const renderer = new Renderer(canvas);
    say('Pixel für Pixel steigt die Welt aus der See …'); await nextFrame();
    const art = buildArt(renderer);
    art.monsters = await loadMonsters(renderer);
    Object.assign(art.beasts, art.monsters);
    const world = new World(art.props);
    const view = new WorldView(renderer, world, art);
    const input = new Input();
    const audio = new Audio();
    const app = {
        version: 'v0.4 · Pixel', renderer, art, font: art.font, world, view, input, audio, store: new ProfileStore(), hasProfiles: false, game: null,
        options: { fx: 'high', exposure: 1, volume: 0.8, shake: 1, touch: 'auto', look: 'bright' }
    };
    try { Object.assign(app.options, JSON.parse(localStorage.getItem(OPT_KEY) || '{}')); } catch { /* private mode */ }
    app.saveOptions = () => { try { localStorage.setItem(OPT_KEY, JSON.stringify(app.options)); } catch { /* ignore */ } };
    app.cui = new CanvasUI(app);
    app.ui = new UI(document.getElementById('ui'), app);
    const touch = new Touch(input, document.getElementById('ui'));
    let isTouch = matchMedia('(pointer: coarse)').matches;
    const updateTouch = () => { const on = app.options.touch === 'on' || (app.options.touch === 'auto' && isTouch); input.usingTouch = on; touch.show(on && app.scene?.mode === 'world' && !app.ui.busy(), !!app.game?.sailing); };
    app.applyOptions = () => {
        renderer.quality = app.options.fx === 'low' ? 'low' : 'high';
        renderer.exposure = app.options.exposure;
        audio.volume = app.options.volume;
        if (audio.master) audio.master.gain.value = app.options.volume;
        updateTouch();
    };
    app.refreshProfiles = async () => { app.hasProfiles = (await app.store.list().catch(() => [])).length > 0; };
    await app.refreshProfiles();
    const menu = new MenuScene(app);
    app.toTitle = () => { app.game = null; app.scene = menu; menu.enter(); app.cui.list = null; app.cui.dialog = null; app.cui.fade = 0; app.refreshProfiles(); fit(); };
    app.startGame = (profile, save) => {
        const g = new GameScene(app, profile, save);
        app.game = g; app.scene = g; fit();
        app.refreshProfiles();
    };
    if (DEBUG) window.__sw = app;
    // ---------------------------------------------------------------- sizing: integer scale, pixel exact
    const fit = () => {
        const dpr = Math.min(window.devicePixelRatio || 1, 3);
        const sw = Math.round(innerWidth * dpr), sh = Math.round(innerHeight * dpr);
        canvas.width = sw; canvas.height = sh; canvas.style.width = innerWidth + 'px'; canvas.style.height = innerHeight + 'px';
        if (app.scene?.mode === 'world') {
            // the shorter side shows about 360 px of the world (640×360 on 16:9)
            const k = +params.get('k') || Math.max(1, Math.round(Math.min(sw, sh) / 360));
            renderer.scale = k; renderer.resize(Math.ceil(sw / k), Math.ceil(sh / k));
        } else {
            const k = Math.max(1, Math.floor(Math.min(sw / 480, sh / 270)));
            renderer.scale = k; renderer.resize(480, 270);
        }
        app.dpr = dpr;
        updateTouch();
    };
    app.fit = fit;
    addEventListener('resize', fit);
    // ---------------------------------------------------------------- pointer → low-res coordinates
    const toLow = (e) => {
        const k = renderer.scale, cw = canvas.width, ch = canvas.height;
        const ox = Math.floor((cw - renderer.W * k) / 2), oy = Math.floor((ch - renderer.H * k) / 2);
        return [((e.clientX * app.dpr) - ox) / k, ((e.clientY * app.dpr) - oy) / k];
    };
    canvas.addEventListener('pointermove', e => { const [x, y] = toLow(e); input.mouse.x = x; input.mouse.y = y; input.mouse.moved = e.pointerType === 'mouse'; if (app.scene === menu) menu.pointer(x, y, false); else app.cui.pointer(x, y, false); });
    canvas.addEventListener('pointerdown', e => {
        if (e.pointerType === 'touch' && !isTouch) { isTouch = true; updateTouch(); }
        audio.unlock();
        const [x, y] = toLow(e); input.mouse.click = true;
        if (app.scene === menu) menu.pointer(x, y, true);
        else if (!app.cui.pointer(x, y, true) && app.game && e.pointerType === 'mouse' && app.game.focus?.act) app.game.interact();
    });
    addEventListener('keydown', () => { if (isTouch && app.options.touch === 'auto') { isTouch = false; updateTouch(); } }, { capture: true });
    // ---------------------------------------------------------------- start
    if (params.has('view')) app.scene = new PreviewScene(app, params.get('view'));
    else if (params.has('start') && DEBUG) {
        const save = newSave('Testlauf', { ...DEFAULT_LOOK }, params.get('origin') || 'matrose', 'rum');
        const f = params.get('start'); if (f) { save.lastFire = f; save.litFires = [f]; }
        app.startGame(null, save);
    } else { app.scene = menu; menu.enter(); }
    app.applyOptions();
    fit();
    loading?.remove();
    let last = performance.now();
    const loop = (now) => {
        const dt = Math.max(0, Math.min(0.05, (now - last) / 1000)); last = now;
        input.poll();
        try {
            app.scene.update(dt, input);
            audio.update(dt, app.game);
            if (app.game) touch.show(input.usingTouch && !app.ui.busy() && !app.cui.modal, app.game.sailing);
            renderer.render(app.scene.frame(), dt);
        } catch (e) { console.error(e?.stack || e); }
        input.endFrame();
        requestAnimationFrame(loop);
    };
    requestAnimationFrame(loop);
}

// ?view=zone: free camera over the world (art preview)
class PreviewScene {
    constructor(app, zone) {
        this.app = app; this.mode = 'world';
        const z = ZONES[zone] || ZONES.hain;
        this.x = (+params.get('x') || z.cx) * TS; this.y = (+params.get('y') || z.cy) * TS;
        this.zone = zone in ZONES ? zone : 'hain';
    }
    update(dt, input) { const a = input.axis(); this.x += a.x * dt * 160; this.y += a.y * dt * 160; this.app.view.time += dt; }
    frame() {
        const r = this.app.renderer, v = this.app.view;
        const cam = [Math.round(this.x - r.W / 2), Math.round(this.y - r.H / 2)];
        const G = gradesFor(this.app.options.look)[ZONES[this.zone].grade];
        r.grade = G;
        this.app.view.setWater(ZONES[this.zone].biome);
        return { cam, ambient: G.ambient, lightK: G.light, lightmap: v.lightmap, lights: v.lights(cam, []), drawScene: (r) => { v.drawScene(r, cam, [], null); v.drawFog(r, cam, this.zone); }, drawGlow: (r) => v.drawGlow(r, cam, []) };
    }
}

boot().catch(e => { console.error(e); say('Fehler beim Start: ' + e.message); });
