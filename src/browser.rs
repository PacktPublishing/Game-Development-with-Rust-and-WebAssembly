use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Document, HtmlCanvasElement, Response, Window};

pub fn window() -> Result<Window, JsValue> {
    web_sys::window().ok_or(JsValue::from("Error Creating Window"))
}

pub fn document() -> Result<Document, JsValue> {
    window()?
        .document()
        .ok_or(JsValue::from("Error Creating Document"))
}

pub fn canvas() -> Result<HtmlCanvasElement, JsValue> {
    document()?
        .get_element_by_id("canvas")
        .ok_or(JsValue::from("Could Not FindElement"))?
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|err| JsValue::from(err))
}

pub async fn fetch_with_str(resource: &str) -> Result<JsValue, JsValue> {
    JsFuture::from(window()?.fetch_with_str(resource)).await
}

pub async fn fetch_json(json_path: &str) -> Result<JsValue, JsValue> {
    let resp_value = fetch_with_str(json_path).await?;
    let resp: Response = resp_value.dyn_into()?;

    JsFuture::from(resp.json()?).await
}
