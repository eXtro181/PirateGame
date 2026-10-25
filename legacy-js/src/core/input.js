// Keyboard, gamepad, mouse and touch mapped to game actions (2D: screen axes = world axes).
const KEYS = {
    up: ['KeyW', 'ArrowUp'], down: ['KeyS', 'ArrowDown'], left: ['KeyA', 'ArrowLeft'], right: ['KeyD', 'ArrowRight'],
    attack: ['KeyJ'], heavy: ['KeyL'], item: ['KeyK'], roll: ['Space'], parry: ['KeyQ'], lock: ['Tab'],
    drink: ['KeyR'], interact: ['KeyE'], shanty: ['KeyF'], menu: ['Escape'], map: ['KeyM'], legend: ['KeyH'],
    spell: ['KeyC'], swap: ['KeyX'], confirm: ['Enter', 'NumpadEnter'], names: ['AltLeft', 'AltRight'],
    belt1: ['Digit1'], belt2: ['Digit2'], belt3: ['Digit3'], belt4: ['Digit4']
};
const PAD = { roll: 0, item: 1, interact: 2, spell: 3, attack: 5, heavy: 7, parry: 4, lock: 11, drink: 13, shanty: 12, menu: 9, map: 8, swap: 14, confirm: 0 };

export class Input {
    constructor() {
        this.down = new Set(); this.hits = new Set();
        this.padPrev = {}; this.padNow = {};
        this.vHeld = new Set(); this.vHits = new Set();
        this.touchStick = { x: 0, y: 0 };
        this.any = false;
        this.mouse = { x: -1, y: -1, click: false, moved: false };
        this.enabled = true;
        window.addEventListener('keydown', (e) => {
            if (e.target instanceof HTMLInputElement || e.target instanceof HTMLSelectElement || e.target instanceof HTMLTextAreaElement) return;
            if (['Tab', 'Space', 'ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight', 'AltLeft', 'AltRight'].includes(e.code)) e.preventDefault();
            if (!this.down.has(e.code)) this.hits.add(e.shiftKey && e.code === 'KeyJ' ? 'ShiftJ' : e.code);
            this.down.add(e.code);
            this.any = true; this.usingPad = false;
        });
        window.addEventListener('keyup', e => this.down.delete(e.code));
        window.addEventListener('blur', () => this.down.clear());
    }
    poll() {
        this.padPrev = this.padNow; this.padNow = {}; this.stick = { x: 0, y: 0 };
        const pads = navigator.getGamepads ? navigator.getGamepads() : [];
        for (const p of pads) {
            if (!p) continue;
            for (const [a, i] of Object.entries(PAD)) if (p.buttons[i]?.pressed) this.padNow[a] = true;
            if (p.buttons[12]?.pressed) this.padNow.up = true;
            if (p.buttons[13]?.pressed) this.padNow.down = true;
            if (Math.hypot(p.axes[0], p.axes[1]) > 0.25) this.stick = { x: p.axes[0], y: p.axes[1] };
            if (Object.keys(this.padNow).length || this.stick.x || this.stick.y) { this.usingPad = true; if (Object.keys(this.padNow).length) this.any = true; }
            break;
        }
        if (this.touchStick.x || this.touchStick.y) this.stick = { ...this.touchStick };
    }
    held(a) { if (!this.enabled) return false; return KEYS[a]?.some(k => this.down.has(k)) || !!this.padNow[a] || this.vHeld.has(a); }
    pressed(a) {
        if (!this.enabled) return false;
        if (a === 'heavy' && this.hits.has('ShiftJ')) return true;
        if (a === 'attack' && this.hits.has('ShiftJ')) return false;
        return KEYS[a]?.some(k => this.hits.has(k)) || (!!this.padNow[a] && !this.padPrev[a]) || this.vHits.has(a);
    }
    anyKey() { return this.any || this.mouse.click; }
    axis() {
        if (!this.enabled) return { x: 0, y: 0 };
        let x = (this.held('right') ? 1 : 0) - (this.held('left') ? 1 : 0);
        let y = (this.held('down') ? 1 : 0) - (this.held('up') ? 1 : 0);
        if (this.stick && (this.stick.x || this.stick.y)) { x = this.stick.x; y = this.stick.y; }
        const l = Math.hypot(x, y);
        if (l > 1) { x /= l; y /= l; }
        return { x, y };
    }
    vPress(a) { this.vHits.add(a); this.vHeld.add(a); this.any = true; }
    vRelease(a) { this.vHeld.delete(a); }
    endFrame() { this.hits.clear(); this.vHits.clear(); this.any = false; this.mouse.click = false; }
}
