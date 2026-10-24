// Schwarzwasser – entry point.
import { createStage } from './core/stage.js';
import { Input } from './core/input.js';
import { Materials } from './gfx/materials.js';
import { Particles } from './gfx/particles.js';
import { buildModels } from './world/props3d.js';
import { WorldMap } from './world/map.js';
import { Terrain } from './world/terrain.js';
import { WorldView } from './world/view.js';
import { Camp } from './world/camp.js';
import { Game } from './game/game.js';
import { Audio } from './audio/audio.js';
import { ProfileStore } from './save/profiles.js';
import { UI } from './ui/ui.js';

const loading = document.getElementById('loading');
const step = (t) => { if (loading) loading.textContent = t; return new Promise(r => setTimeout(r, 0)); };

async function boot() {
    const stage = createStage(document.getElementById('game'));
    await step('Texturen werden gewoben …');
    const M = new Materials(stage.app.graphicsDevice, stage.quality);
    await step('Die Welt hebt sich aus dem Meer …');
    const models = buildModels(stage.app.graphicsDevice, M);
    const particles = new Particles(stage.app, M);
    const map = new WorldMap();
    const terrain = new Terrain(map);
    const view = new WorldView(stage.app, map, terrain, M, models, particles);
    const camp = new Camp(stage.app, stage, M, models, particles);
    await step('Die Toten erwachen …');
    const input = new Input();
    const audio = new Audio();
    const store = new ProfileStore();
    const ui = new UI(document.getElementById('ui'), { store, audio });
    const game = new Game({ stage, M, models, particles, map, terrain, view, input, audio, hud: ui.hud, ui, store });
    ui.bind(game, camp);
    game.onAttract = (dt) => camp.update(dt);
    stage.app.start();
    loading?.remove();

    const onStart = (profile, save) => {
        camp.exit();
        ui.showHud();
        audio.unlock();
        game.start(profile, save);
    };
    game.onLogout = () => { ui.title(onStart); };
    ui.title(onStart);
    window.addEventListener('pagehide', () => game.save && game.persist(null));
    // mouse wheel: a little zoom in tight limits
    window.addEventListener('wheel', (e) => { if (!game.save || ui.modalOpen()) return; game.userZoom = Math.max(0.82, Math.min(1.22, (game.userZoom || 1) * (e.deltaY > 0 ? 1.06 : 1 / 1.06))); }, { passive: true });
    if (new URLSearchParams(location.search).has('debug')) window.__sw = game;
}
boot().catch(e => { console.error(e); if (loading) loading.textContent = 'Fehler beim Start: ' + e.message; });
