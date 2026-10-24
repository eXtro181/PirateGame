// Keyboard + gamepad input mapped to game actions.
const KEYS = {
    up: ['KeyW', 'ArrowUp'], down: ['KeyS', 'ArrowDown'], left: ['KeyA', 'ArrowLeft'], right: ['KeyD', 'ArrowRight'],
    attack: ['KeyJ'], heavy: ['KeyL'], item: ['KeyK'], roll: ['Space'], parry: ['KeyQ'], lock: ['Tab'],
    drink: ['KeyR'], interact: ['KeyE'], shanty: ['KeyF'], menu: ['Escape'], map: ['KeyM']
};
// standard gamepad mapping
const PAD = { roll: 0, item: 1, interact: 2, attack: 5, heavy: 7, parry: 4, lock: 11, drink: 13, shanty: 12, menu: 9, map: 8 };

export class Input {
    constructor() {
        this.down = new Set();
        this.hits = new Set();
        this.padPrev = {};
        this.padNow = {};
        this.enabled = true;
        // on-screen touch controls write into these
        this.vHeld = new Set();
        this.vHits = new Set();
        this.touchStick = { x: 0, z: 0 };
        window.addEventListener('keydown', (e) => {
            if (e.target instanceof HTMLInputElement || e.target instanceof HTMLSelectElement) return;
            if (['Tab', 'Space', 'ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight'].includes(e.code)) e.preventDefault();
            if (!this.down.has(e.code)) this.hits.add(e.shiftKey && e.code === 'KeyJ' ? 'ShiftJ' : e.code);
            this.down.add(e.code);
        });
        window.addEventListener('keyup', e => this.down.delete(e.code));
        window.addEventListener('blur', () => this.down.clear());
    }
    poll() {
        this.padPrev = this.padNow;
        this.padNow = {};
        this.stick = { x: 0, z: 0 };
        const pads = navigator.getGamepads ? navigator.getGamepads() : [];
        for (const p of pads) {
            if (!p) continue;
            for (const [a, i] of Object.entries(PAD)) if (p.buttons[i]?.pressed) this.padNow[a] = true;
            if (p.buttons[14]?.pressed) this.padNow.left = true;
            if (p.buttons[15]?.pressed) this.padNow.right = true;
            if (Math.hypot(p.axes[0], p.axes[1]) > 0.25) this.stick = { x: p.axes[0], z: p.axes[1] };
            break;
        }
        if (this.touchStick.x || this.touchStick.z) this.stick = { ...this.touchStick };
    }
    held(action) {
        if (!this.enabled) return false;
        return KEYS[action]?.some(k => this.down.has(k)) || !!this.padNow[action] || this.vHeld.has(action);
    }
    pressed(action) {
        if (!this.enabled) return false;
        if (action === 'heavy' && this.hits.has('ShiftJ')) return true;
        if (action === 'attack' && this.hits.has('ShiftJ')) return false;
        return KEYS[action]?.some(k => this.hits.has(k)) || (!!this.padNow[action] && !this.padPrev[action]) || this.vHits.has(action);
    }
    axis() {
        if (!this.enabled) return { x: 0, z: 0 };
        let x = (this.held('right') ? 1 : 0) - (this.held('left') ? 1 : 0);
        let z = (this.held('down') ? 1 : 0) - (this.held('up') ? 1 : 0);
        if (this.stick && (this.stick.x || this.stick.z)) { x = this.stick.x; z = this.stick.z; }
        const l = Math.hypot(x, z);
        return l > 1 ? { x: x / l, z: z / l } : { x, z };
    }
    // a touch button press: counts as pressed this frame and held until released
    vPress(action) { this.vHits.add(action); this.vHeld.add(action); }
    vRelease(action) { this.vHeld.delete(action); }
    endFrame() { this.hits.clear(); this.vHits.clear(); }
}
