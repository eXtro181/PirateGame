//! Hintergrundvideo des Hauptmenüs (Nutzerwunsch 21.11.): `menu.mp4` läuft stumm als Schleife in einem
//! unsichtbaren `<video>`-Element; im Titel kopiert `render::prepare` jedes Bild in eine Textur, die der
//! Endbild-Shader statt der Shader-Szene zeigt (`title_video` in post.wgsl). Ohne Video bleibt das
//! Shader-Titelbild „Thron der Ertrunkenen“.

#[cfg(target_arch = "wasm32")]
thread_local! {
    static VIDEO: std::cell::RefCell<Option<(web_sys::HtmlVideoElement, u32)>> = const { std::cell::RefCell::new(None) };
}

/// Sind Menüvideo und -lied schon gestartet (index.html, `__startMenu` nach der Ladeanzeige)?
#[cfg(target_arch = "wasm32")]
pub fn gate_passed() -> bool {
    web_sys::window()
        .and_then(|w| js_sys::Reflect::get(&w, &wasm_bindgen::JsValue::from_str("__gate")).ok())
        .and_then(|v| v.as_bool())
        .unwrap_or(true)
}
#[cfg(not(target_arch = "wasm32"))]
pub fn gate_passed() -> bool {
    true
}

/// Konnte das Video nicht geladen werden? Dann zeigt der Titel das Shader-Bild.
#[cfg(target_arch = "wasm32")]
pub fn failed() -> bool {
    VIDEO.with(|v| v.borrow().as_ref().is_some_and(|(el, _)| el.network_state() == 3))
}

/// Das Videoelement (aus index.html, sonst beim ersten Aufruf angelegt); Some, sobald ein Bild bereitsteht.
#[cfg(target_arch = "wasm32")]
pub fn frame() -> Option<web_sys::HtmlVideoElement> {
    use wasm_bindgen::JsCast;
    VIDEO.with(|v| {
        let mut v = v.borrow_mut();
        if v.is_none() {
            let doc = web_sys::window()?.document()?;
            if let Some(el) = doc.get_element_by_id("menu-video").and_then(|e| e.dyn_into::<web_sys::HtmlVideoElement>().ok()) {
                *v = Some((el, 0));
            }
        }
        if v.is_none() {
            let doc = web_sys::window()?.document()?;
            let el: web_sys::HtmlVideoElement = doc.create_element("video").ok()?.dyn_into().ok()?;
            el.set_muted(true);
            el.set_default_muted(true);
            el.set_loop(true);
            el.set_autoplay(true);
            let _ = el.set_attribute("playsinline", "");
            let _ = el.set_attribute("muted", "");
            let _ = el.set_attribute("style", "position:fixed;left:0;top:0;width:2px;height:2px;opacity:0;pointer-events:none");
            // WebM (VP9), wo der Browser es kann, sonst MP4 (H.264)
            el.set_src(if el.can_play_type("video/webm; codecs=\"vp9\"").is_empty() { "menu.mp4" } else { "menu.webm" });
            if let Some(b) = doc.body() {
                let _ = b.append_child(&el);
            }
            *v = Some((el, 0));
        }
        let (el, n) = v.as_mut()?;
        // angehalten (nach dem Spiel): ab und zu wieder anstoßen – vor dem Start nicht
        if el.paused() && gate_passed() {
            if *n % 60 == 0 {
                let _ = el.play();
            }
            *n += 1;
        }
        (el.ready_state() >= 2 && el.video_width() > 0).then(|| el.clone())
    })
}

/// außerhalb des Titels: Video anhalten (spart Dekodieren)
#[cfg(target_arch = "wasm32")]
pub fn pause() {
    VIDEO.with(|v| {
        if let Some((el, n)) = v.borrow_mut().as_mut() {
            if !el.paused() {
                let _ = el.pause();
            }
            *n = 0;
        }
    });
}
