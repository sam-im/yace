mod utils;

use std::{cell::RefCell, rc::Rc, sync::mpsc};
use chip8::Chip8;
use wasm_bindgen::prelude::*;
use web_sys::{js_sys::Date, Element};

const SCALE: f64 = 1.;
const TIME_PER_CYCLE: f64 = 0.16;   // amounts to approx. 60hz

#[wasm_bindgen(start)]
fn main() -> Result<(), JsValue> {
    utils::set_panic_hook();
    let (tx, rx) = std::sync::mpsc::channel::<Event>();
    init_html(tx)?;
    let mut chip8 = init_chip8()?;

    let f = Rc::new(RefCell::new(None));
    let g = Rc::clone(&f);

    let mut time = Date::now();
    let mut acc = 0.;

    *g.borrow_mut() = Some(Closure::new(move || {
        while let Ok(msg) = rx.try_recv() {
            handle_event(&msg, &mut chip8);
        }

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
    // workaround that loads a rom into the compiled binary,
    // TODO use fetch instead
    let rom = include_bytes!("../../roms/1-chip8-logo.ch8");
    let mut chip8 = Chip8::new();
    chip8.load_rom(rom);
    Ok(chip8)
}


fn init_html(event_tx: mpsc::Sender<Event>) -> Result<(), JsValue> {
    let document = utils::document();

    let canvas = document.get_element_by_id("canvas").unwrap();
    let (width, height) = ((640. * SCALE) as usize, (320. * SCALE) as usize);
    canvas.set_attribute("width", &width.to_string())?;
    canvas.set_attribute("height", &height.to_string())?;

    let button_ids: [&str; 16] = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "A", "B", "C", "D", "E", "F"];
    let mut buttons: Vec<Element> = button_ids
        .iter()
        .map(|c| document.get_element_by_id(c).unwrap())
        .collect();

    // Register global event handlers onto keys
    buttons.iter_mut().enumerate().for_each(|(i, b)| {
        let tx = event_tx.clone();
        let keydown_cb = Closure::<dyn Fn()>::new(move || {
            tx.send(Event::Keypad(KeypadEvent::KeyDown(i))).unwrap();
            utils::log(format!("keypad event: {i} down").as_str());
        });

        let tx = event_tx.clone();
        let keyup_cb = Closure::<dyn Fn()>::new(move || {
            tx.send(Event::Keypad(KeypadEvent::KeyUp(i))).unwrap();
            utils::log(format!("keypad event: {i} up").as_str());
        });

        b.add_event_listener_with_callback("mousedown", keydown_cb.as_ref().unchecked_ref()).unwrap();
        b.add_event_listener_with_callback("mouseup", keyup_cb.as_ref().unchecked_ref()).unwrap();

        // prevent the compiler from dropping these closures
        keydown_cb.forget();
        keyup_cb.forget();
    });

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

enum Event {
    Keypad(KeypadEvent),
}

enum KeypadEvent {
    KeyDown(usize),
    KeyUp(usize),
}

fn handle_event(event: &Event, chip8: &mut Chip8) {
    match event {
        Event::Keypad(e) => {
            match e {
                KeypadEvent::KeyDown(i) => {
                    if let Some(key_status) = chip8.keypad.get_mut(*i) {
                        if ! *key_status {
                            *key_status = true;
                        }
                    }
                },
                KeypadEvent::KeyUp(i) => {
                    if let Some(key_status) = chip8.keypad.get_mut(*i) {
                        if *key_status {
                            *key_status = false;
                        }
                    }
                },
            }
        }
    }
}
