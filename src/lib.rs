mod utils;

use chip8::Chip8;
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::prelude::*;
use web_sys::js_sys::Date;

const SCALE: f64 = 1.;

#[wasm_bindgen(start)]
fn main() -> Result<(), JsValue> {
    utils::set_panic_hook(); // results in bigger size

    init_html()?;
    let mut chip8 = init_chip8()?;

    const TIME_PER_CYCLE: f64 = 0.01;
    let mut current_time = Date::now();
    let mut accumulator = 0.0;

    let f = Rc::new(RefCell::new(None));
    let g = Rc::clone(&f);
    *g.borrow_mut() = Some(Closure::new(move || {
        let now = Date::now();
        let dt = now - current_time;
        current_time = now;
        accumulator += dt;

        while accumulator >= TIME_PER_CYCLE {
            chip8.cycle();
            accumulator = accumulator - TIME_PER_CYCLE;
        }

        draw(&chip8.display);
        request_animation_frame(f.borrow().as_ref().unwrap());
    }));
    request_animation_frame(g.borrow().as_ref().unwrap());

    Ok(())
}

fn init_chip8() -> Result<Chip8, JsValue> {
    // workaround that loads the rom into the compiled binary, use fetch instead
    let rom = include_bytes!("../pkg/roms/1-chip8-logo.ch8");
    let mut chip8 = Chip8::new();
    chip8.load_rom(rom);
    Ok(chip8)
}

fn init_html() -> Result<(), JsValue> {
    let elem = document().create_element("canvas")?;
    elem.set_id("canvas");
    let (width, height) = ((640. * SCALE) as usize, (320. * SCALE) as usize);
    elem.set_attribute("width", &width.to_string())?;
    elem.set_attribute("height", &height.to_string())?;
    body().append_child(&elem)?;
    Ok(())
}

fn draw(display: &[bool]) {
    let ctx = context();
    let size = 10. * SCALE;
    display.iter().enumerate().for_each(|(i, val)| {
        let x = size * (i % 64) as f64;
        let y = size * (i / 64) as f64;
        ctx.stroke_rect(x, y, size, size);

        if *val {
            ctx.fill_rect(x, y, size, size);
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
    window()
        .document()
        .expect("should have a document on window")
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
