use std::{collections::HashMap, rc::Rc, sync::Mutex};

use serde::Deserialize;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::console;

#[derive(Deserialize)]
struct Rect {
    x: u16,
    y: u16,
    w: u16,
    h: u16,
}

#[derive(Deserialize)]
struct Cell {
    frame: Rect,
}

#[derive(Deserialize)]
struct Sheet {
    frames: HashMap<String, Cell>,
}

// When the `wee_alloc` feature is enabled, this uses `wee_alloc` as the global
// allocator.
//
// If you don't want to use `wee_alloc`, you can safely delete this.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// This is like the `main` function, except for JavaScript.
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let canvas = document
        .get_element_by_id("canvas")
        .unwrap()
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .unwrap();

    let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();

    wasm_bindgen_futures::spawn_local(async move {
        let json = fetch_json("rhb.json")
            .await
            .expect("Could not fetch rhb.json");
        let sheet: Sheet = json
            .into_serde()
            .expect("Could not convert rhb.json into a Sheet structure");

        let (complete_tx, complete_rx) =
            futures::channel::oneshot::channel::<Result<(), JsValue>>();
        let complete_tx = Rc::new(Mutex::new(Some(complete_tx)));
        let error_tx = complete_tx.clone();
        let image = web_sys::HtmlImageElement::new().unwrap();

        let callback = Closure::once(Box::new(move || {
            if let Some(complete_tx) = complete_tx.lock().ok().and_then(|mut opt| opt.take()) {
                complete_tx.send(Ok(()));
            }
        }));

        let error_callback = Closure::once(Box::new(move |err| {
            if let Some(error_tx) = error_tx.lock().ok().and_then(|mut opt| opt.take()) {
                error_tx.send(Err(err));
            }
        }));

        image.set_onload(Some(callback.as_ref().unchecked_ref()));
        image.set_onload(Some(error_callback.as_ref().unchecked_ref()));
        image.set_src("rhb.png");

        complete_rx.await;

        let mut frame = -1;
        let interval_callback = Closure::wrap(Box::new(move || {
            frame = (frame + 1) % 8;
            let frame_name = format!("Run ({}).png", frame + 1);
            console::log_1(&JsValue::from_str(&frame_name));
            let sprite = sheet.frames.get(&frame_name).expect("Cell not found");

            context.clear_rect(0.0, 0.0, 600.0, 600.0);
            context.draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                &image,
                sprite.frame.x.into(),
                sprite.frame.y.into(),
                sprite.frame.w.into(),
                sprite.frame.h.into(),
                300.0,
                300.0,
                sprite.frame.w.into(),
                sprite.frame.h.into(),
            );
        }) as Box<dyn FnMut()>);

        window.set_interval_with_callback_and_timeout_and_arguments_0(
            interval_callback.as_ref().unchecked_ref(),
            50,
        );
        interval_callback.forget();
    });

    Ok(())
}

fn callback() {}

async fn fetch_json(json_path: &str) -> Result<JsValue, JsValue> {
    let window = web_sys::window().unwrap();
    let resp_value = wasm_bindgen_futures::JsFuture::from(window.fetch_with_str(json_path)).await?;
    let resp: web_sys::Response = resp_value.dyn_into()?;

    wasm_bindgen_futures::JsFuture::from(resp.json()?).await
}
