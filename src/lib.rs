#[macro_use]
mod browser;
mod engine;

use engine::{Game, GameLoop};
use serde::Deserialize;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlImageElement};

#[derive(Deserialize)]
struct Rect {
    x: i16,
    y: i16,
    w: i16,
    h: i16,
}

#[derive(Deserialize)]
struct Cell {
    frame: Rect,
}

#[derive(Deserialize)]
struct Sheet {
    frames: HashMap<String, Cell>,
}

struct WalkTheDog {
    image: HtmlImageElement,
    sheet: Sheet,
    frame_count: u8,
    current_frame: u8,
}

impl WalkTheDog {
    fn new(image: HtmlImageElement, sheet: Sheet) -> Self {
        WalkTheDog {
            image,
            sheet,
            current_frame: 0,
            frame_count: 0,
        }
    }
}

impl Game for WalkTheDog {
    fn update(&mut self) {
        if self.frame_count < 24 {
            self.frame_count += 1;
        } else {
            self.frame_count = 0;
        }
        if (self.frame_count % 3) == 0 {
            self.current_frame = (self.current_frame + 1) % 8;
        }
    }

    fn draw(&self, context: CanvasRenderingContext2d) {
        let frame_name = format!("Run ({}).png", self.current_frame + 1);
        let sprite = self.sheet.frames.get(&frame_name).expect("Cell not found");

        context.clear_rect(0.0, 0.0, 600.0, 600.0);
        context.draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
            &self.image,
            sprite.frame.x.into(),
            sprite.frame.y.into(),
            sprite.frame.w.into(),
            sprite.frame.h.into(),
            300.0,
            300.0,
            sprite.frame.w.into(),
            sprite.frame.h.into(),
        );
    }
}

// This is like the `main` function, except for JavaScript.
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    let context = browser::context()?;

    browser::spawn_local(async move {
        let json = browser::fetch_json("rhb.json")
            .await
            .expect("Could not fetch rhb.json");

        let sheet: Sheet = json
            .into_serde()
            .expect("Could not convert rhb.json into a Sheet structure");

        let image = engine::load_image("rhb.png")
            .await
            .expect("Could not load rhb.png");

        let game = WalkTheDog::new(image, sheet);

        GameLoop::start(game).expect("Could not start game loop");
    });

    Ok(())
}
