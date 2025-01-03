mod utils;

use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::prelude::*;
use chip8::Chip8;

#[wasm_bindgen(start)]
fn main() -> Result<(), JsValue> {
    init_html()?;

    // workaround that loads the rom into the compiled binary, use fetch instead
    let rom = include_bytes!("../pkg/roms/1-chip8-logo.ch8");
    let mut chip8 = Chip8::new();
    chip8.load_rom(rom);

    let f = Rc::new(RefCell::new(None));
    let g = Rc::clone(&f);
    *g.borrow_mut() = Some(Closure::new(move || {
        chip8.cycle();
        draw(&chip8.display);
        request_animation_frame(f.borrow().as_ref().unwrap());
    }));

    request_animation_frame(g.borrow().as_ref().unwrap());
    Ok(())
}

fn init_html() -> Result<(), JsValue> {
    let elem = document().create_element("canvas")?;
    elem.set_id("canvas");
    elem.set_attribute("width", "640")?;
    elem.set_attribute("height", "320")?;
    body().append_child(&elem)?;
    Ok(())
}

fn draw(display: &[bool]) {
    let ctx = context();
    display.iter().enumerate().for_each(|(i, val)| {
        let x = (10 * (i % 64)) as f64;
        let y = (10 * (i / 64) as usize) as f64;
        ctx.stroke_rect(x, y, 10., 10.);

        if *val {
            ctx.fill_rect(x, y, 10., 10.);
        }
    });
}

// Utility functions
fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

fn document() -> web_sys::Document {
    window().document().expect("should have a document on window")
}

fn canvas() -> web_sys::HtmlCanvasElement {
    document()
        .get_element_by_id("canvas")
        .expect("should have a canvas in body")
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap()
}
fn context() -> web_sys::CanvasRenderingContext2d {
    canvas()
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap()
}

fn body() -> web_sys::HtmlElement {
    document().body().expect("document should have a body")
}

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}
