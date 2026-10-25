//! Textfelder: Im Browser liegen unsichtbare DOM-Eingabefelder genau über den gezeichneten
//! Feldern. So öffnet ein Tipp auf dem iPhone direkt die Bildschirmtastatur, und Passwörter
//! werden vom Browser wie gewohnt behandelt. Gezeichnet wird alles im Canvas.

#[cfg(target_arch = "wasm32")]
mod imp {
    use std::cell::RefCell;
    use std::collections::HashMap;
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsCast;
    use web_sys::*;

    thread_local! {
        static FIELDS: RefCell<HashMap<u32, HtmlInputElement>> = RefCell::new(HashMap::new());
        static USED: RefCell<Vec<u32>> = const { RefCell::new(Vec::new()) };
        static KEYS: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
    }

    fn make(password: bool) -> Option<HtmlInputElement> {
        let doc = window()?.document()?;
        let el: HtmlInputElement = doc.create_element("input").ok()?.dyn_into().ok()?;
        el.set_type(if password { "password" } else { "text" });
        let _ = el.set_attribute("autocomplete", if password { "current-password" } else { "off" });
        let _ = el.set_attribute("autocapitalize", "off");
        let _ = el.set_attribute("autocorrect", "off");
        let _ = el.set_attribute("spellcheck", "false");
        let _ = el.set_attribute("maxlength", "24");
        let st = el.style();
        for (k, v) in [
            ("position", "fixed"),
            ("opacity", "0.011"),
            ("font-size", "16px"),
            ("border", "0"),
            ("padding", "0"),
            ("margin", "0"),
            ("background", "transparent"),
            ("color", "transparent"),
            ("caret-color", "transparent"),
            ("z-index", "20"),
            ("outline", "none"),
        ] {
            let _ = st.set_property(k, v);
        }
        let cb = Closure::<dyn FnMut(KeyboardEvent)>::new(|e: KeyboardEvent| {
            let k = e.key();
            if matches!(k.as_str(), "Enter" | "Tab" | "Escape" | "ArrowUp" | "ArrowDown") {
                if k == "Tab" || k.starts_with("Arrow") {
                    e.prevent_default();
                }
                e.stop_propagation();
                KEYS.with(|v| v.borrow_mut().push(k));
            } else {
                e.stop_propagation();
            }
        });
        let _ = el.add_event_listener_with_callback("keydown", cb.as_ref().unchecked_ref());
        cb.forget();
        doc.body()?.append_child(&el).ok()?;
        Some(el)
    }

    /// Feld `id` an Position (CSS-Pixel) halten; gibt den aktuellen Wert und ob es fokussiert ist zurück.
    pub fn field(id: u32, x: f32, y: f32, w: f32, h: f32, password: bool, value: &str) -> (String, bool) {
        USED.with(|u| u.borrow_mut().push(id));
        FIELDS.with(|m| {
            let mut m = m.borrow_mut();
            if !m.contains_key(&id) {
                if let Some(el) = make(password) {
                    el.set_value(value);
                    m.insert(id, el);
                }
            }
            let Some(el) = m.get(&id) else { return (value.to_string(), false) };
            let st = el.style();
            let _ = st.set_property("display", "block");
            let _ = st.set_property("left", &format!("{x}px"));
            let _ = st.set_property("top", &format!("{y}px"));
            let _ = st.set_property("width", &format!("{w}px"));
            let _ = st.set_property("height", &format!("{h}px"));
            let focused = window().and_then(|w| w.document()).and_then(|d| d.active_element()).map(|a| a.is_same_node(Some(el.as_ref()))).unwrap_or(false);
            (el.value(), focused)
        })
    }
    pub fn set(id: u32, v: &str) {
        FIELDS.with(|m| {
            if let Some(el) = m.borrow().get(&id) {
                el.set_value(v);
            }
        });
    }
    pub fn focus(id: u32) {
        FIELDS.with(|m| {
            if let Some(el) = m.borrow().get(&id) {
                let _ = el.focus();
            }
        });
    }
    /// Am Ende des Bildes: nicht benutzte Felder ausblenden.
    pub fn end_frame() {
        let used: Vec<u32> = USED.with(|u| std::mem::take(&mut *u.borrow_mut()));
        FIELDS.with(|m| {
            for (id, el) in m.borrow().iter() {
                if !used.contains(id) {
                    let _ = el.blur();
                    let _ = el.style().set_property("display", "none");
                }
            }
        });
    }
    pub fn clear_all() {
        FIELDS.with(|m| {
            for el in m.borrow().values() {
                el.set_value("");
            }
        });
    }
    pub fn keys() -> Vec<String> {
        KEYS.with(|v| std::mem::take(&mut *v.borrow_mut()))
    }
    pub fn dpr() -> f32 {
        window().map(|w| w.device_pixel_ratio() as f32).unwrap_or(1.0)
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod imp {
    pub fn field(_id: u32, _x: f32, _y: f32, _w: f32, _h: f32, _password: bool, value: &str) -> (String, bool) {
        (value.to_string(), false)
    }
    pub fn set(_id: u32, _v: &str) {}
    pub fn focus(_id: u32) {}
    pub fn end_frame() {}
    pub fn clear_all() {}
    pub fn keys() -> Vec<String> {
        vec![]
    }
    pub fn dpr() -> f32 {
        1.0
    }
}

pub use imp::*;
