#[macro_use]
mod browser;
mod engine;
mod game;
mod segments;
mod sound;

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;

#[derive(Serialize, Deserialize)]
struct Options {
    width: f32,
    height: f32,
}

#[wasm_bindgen]
extern "C" {
    type PIXI;
}

#[wasm_bindgen]
extern "C" {
    type Application;
    type Container;

    #[wasm_bindgen(method, js_name = "addChild")]
    fn add_child(this: &Container, child: &Sprite);

    #[wasm_bindgen(constructor, js_namespace = PIXI)]
    fn new(dimens: &JsValue) -> Application;

    #[wasm_bindgen(method, getter)]
    fn view(this: &Application) -> HtmlCanvasElement;

    #[wasm_bindgen(method, getter)]
    fn stage(this: &Application) -> Container;

    type Sprite;

    #[wasm_bindgen(static_method_of = Sprite, js_namespace = PIXI)]
    fn from(name: &JsValue) -> Sprite;
}

// This is like the `main` function, except for JavaScript.
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    let app = Application::new(
        &JsValue::from_serde(&Options {
            width: 640.0,
            height: 360.0,
        })
        .unwrap(),
    );

    let body = browser::document().unwrap().body().unwrap();
    body.append_child(&app.view()).unwrap();

    let sprite = Sprite::from(&JsValue::from_str("Stone.png"));

    app.stage().add_child(&sprite);

    console_error_panic_hook::set_once();

    Ok(())
}
