// Schwarzwasser – entry point.
import { createStage } from './core/stage.js';
import { buildArt } from './core/assets.js';
import { Input } from './core/input.js';
import { WorldMap } from './world/map.js';
import { WorldView } from './world/view.js';
import { Game } from './game/game.js';
import { Audio } from './audio/audio.js';
import { ProfileStore } from './save/profiles.js';
import { UI } from './ui/ui.js';

const stage = createStage(document.getElementById('game'));
const art = buildArt(stage.app);
const map = new WorldMap();
const view = new WorldView(stage.app, map, art);
const input = new Input();
const audio = new Audio();
const store = new ProfileStore();
const ui = new UI(document.getElementById('ui'), { store, audio });
const game = new Game({ stage, art, map, view, input, audio, hud: ui.hud, ui, store });
ui.bind(game);
stage.app.start();

const onStart = (profile, save) => {
    ui.showHud();
    audio.unlock();
    game.start(profile, save);
};
game.onLogout = () => ui.title(onStart);
ui.title(onStart);
window.addEventListener('pagehide', () => game.save && game.persist(null));
if (new URLSearchParams(location.search).has('debug')) window.__sw = game;
