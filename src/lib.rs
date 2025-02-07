mod utils;

use std::{cell::RefCell, rc::Rc};
use chip8::Chip8;
use wasm_bindgen::prelude::*;
use web_sys::js_sys::Date;

const SCALE: f64 = 1.;
const TIME_PER_CYCLE: f64 = 0.01;

#[wasm_bindgen(start)]
fn main() -> Result<(), JsValue> {
    utils::set_panic_hook();
    init_html()?;
    let mut chip8 = init_chip8()?;

    let f = Rc::new(RefCell::new(None));
    let g = Rc::clone(&f);

    let mut time = Date::now();
    let mut acc = 0.;

    *g.borrow_mut() = Some(Closure::new(move || {
        let now = Date::now();
        let dt = now - time;
        acc += dt;
        time = now;

        while acc >= TIME_PER_CYCLE {
            chip8.cycle();
            acc -= TIME_PER_CYCLE;
        }

        draw(&chip8.display);
        utils::request_animation_frame(f.borrow().as_ref().unwrap());
    }));
    utils::request_animation_frame(g.borrow().as_ref().unwrap());
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
    let elem = utils::document().create_element("canvas")?;
    elem.set_id("canvas");
    let (width, height) = ((640. * SCALE) as usize, (320. * SCALE) as usize);
    elem.set_attribute("width", &width.to_string())?;
    elem.set_attribute("height", &height.to_string())?;
    utils::body().append_child(&elem)?;
    Ok(())
}

fn draw(display: &[bool]) {
    let ctx = utils::context();
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
