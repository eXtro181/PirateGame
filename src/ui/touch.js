// On-screen controls for phones and tablets: a floating stick on the left, context buttons on the right.
const h = (tag, cls, text) => { const e = document.createElement(tag); if (cls) e.className = cls; if (text) e.textContent = text; return e; };

const LAND = [
    { a: 'attack', label: 'Hieb', cls: 'big main' },
    { a: 'roll', label: 'Rolle', cls: 'big' },
    { a: 'heavy', label: 'Schwer', cls: '' },
    { a: 'parry', label: 'Parieren', cls: '' },
    { a: 'drink', label: 'Rum', cls: 'small' },
    { a: 'lock', label: 'Zielen', cls: 'small' },
    { a: 'item', label: 'Pistole', cls: 'small offhand' }
];
const SEA = [
    { a: 'attack', label: '◀ Breitseite', cls: 'big main' },
    { a: 'item', label: 'Breitseite ▶', cls: 'big main' },
    { a: 'down', label: 'Segel −', cls: '' },
    { a: 'up', label: 'Segel +', cls: '' },
    { a: 'shanty', label: 'Shanty', cls: 'small' }
];

export class TouchControls {
    constructor(input, parent) {
        this.input = input;
        this.el = h('div', 'touch-ui');
        this.stickZone = h('div', 'stick-zone');
        this.base = h('div', 'stick-base');
        this.knob = h('div', 'stick-knob');
        this.base.append(this.knob);
        this.stickZone.append(this.base);
        this.pad = h('div', 'touch-pad');
        this.top = h('div', 'touch-top');
        this.top.append(this.button({ a: 'map', label: 'Karte', cls: 'tiny' }), this.button({ a: 'menu', label: '☰', cls: 'tiny' }));
        this.el.append(this.stickZone, this.pad, this.top);
        parent.append(this.el);
        this.mode = null;
        this.stickId = null;
        this.bindStick();
    }

    button(def) {
        const b = h('button', `tbtn ${def.cls || ''}`, def.label);
        b.dataset.action = def.a;
        const down = (e) => {
            e.preventDefault();
            b.setPointerCapture?.(e.pointerId);
            b.classList.add('on');
            this.input.vPress(def.a);
            navigator.vibrate?.(8);
        };
        const up = (e) => { e.preventDefault(); b.classList.remove('on'); this.input.vRelease(def.a); };
        b.addEventListener('pointerdown', down);
        b.addEventListener('pointerup', up);
        b.addEventListener('pointercancel', up);
        b.addEventListener('contextmenu', e => e.preventDefault());
        return b;
    }

    setMode(mode, offhand) {
        const key = `${mode}:${offhand}`;
        if (key === this.mode) return;
        this.mode = key;
        this.pad.replaceChildren(...(mode === 'sea' ? SEA : LAND)
            .filter(d => d.a !== 'item' || mode === 'sea' || offhand !== 'none')
            .map(d => this.button(d.a === 'item' && mode !== 'sea' ? { ...d, label: offhand === 'tide' ? 'Gezeiten' : 'Pistole' } : d)));
        this.pad.className = `touch-pad ${mode}`;
    }

    bindStick() {
        const z = this.stickZone, R = 56;
        let ox = 0, oy = 0;
        const move = (x, y) => {
            let dx = x - ox, dy = y - oy;
            const l = Math.hypot(dx, dy);
            if (l > R) { dx = dx / l * R; dy = dy / l * R; }
            this.knob.style.transform = `translate(${dx}px, ${dy}px)`;
            const k = Math.min(1, l / R);
            const dead = k < 0.18;
            this.input.touchStick = dead ? { x: 0, z: 0 } : { x: dx / R, z: dy / R };
        };
        z.addEventListener('pointerdown', (e) => {
            if (this.stickId !== null) return;
            e.preventDefault();
            this.stickId = e.pointerId;
            z.setPointerCapture?.(e.pointerId);
            const r = z.getBoundingClientRect();
            ox = e.clientX; oy = e.clientY;
            this.base.style.left = `${ox - r.left}px`;
            this.base.style.top = `${oy - r.top}px`;
            this.base.classList.add('on');
            move(e.clientX, e.clientY);
        });
        z.addEventListener('pointermove', (e) => { if (e.pointerId === this.stickId) { e.preventDefault(); move(e.clientX, e.clientY); } });
        const end = (e) => {
            if (e.pointerId !== this.stickId) return;
            this.stickId = null;
            this.base.classList.remove('on');
            this.knob.style.transform = '';
            this.input.touchStick = { x: 0, z: 0 };
        };
        z.addEventListener('pointerup', end);
        z.addEventListener('pointercancel', end);
    }

    setVisible(v) { this.el.classList.toggle('hidden', !v); if (!v) this.input.touchStick = { x: 0, z: 0 }; }
}
