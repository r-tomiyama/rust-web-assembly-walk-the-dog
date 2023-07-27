use anyhow::{anyhow, Result};
use web_sys::{Window, Document, HtmlCanvasElement, CanvasRenderingContext2d, Response, HtmlImageElement};
use wasm_bindgen::{JsCast, JsValue, closure::{WasmClosureFnOnce, WasmClosure}, prelude::Closure};
use wasm_bindgen_futures::JsFuture;

macro_rules! log {
    ($($t:tt)*) => {
        web_sys::console::log_1(&format!($($t)*).into());
    }
}

pub fn window() -> Result<Window> {
    web_sys::window().ok_or_else(|| anyhow!("no window found"))
}

pub fn document() -> Result<Document> {
    window()?.document().ok_or_else(|| anyhow!("no document found"))
}

pub fn canvas() -> Result<HtmlCanvasElement> {
    document()?
        .get_element_by_id("canvas")
        .ok_or_else(|| anyhow!("no canvas found"))?
        .dyn_into::<HtmlCanvasElement>()
        .map_err(|element| anyhow!("Error converting {:#?} to HtmlCanvasElement", element))
}

pub fn context() -> Result<CanvasRenderingContext2d> {
    canvas()?
        .get_context("2d")
        .map_err(|js_value| anyhow!("Error getting 2d context: {:#?}", js_value))?
        .ok_or_else(|| anyhow!("no 2d context found"))?
        .dyn_into::<CanvasRenderingContext2d>()
        .map_err(|element| {
            anyhow!("Error converting {:#?} to CanvasRenderingContext2d", element)
        })
}

pub fn spawn_local<F>(future: F)
where
    F: futures::Future<Output = ()> + 'static,
{
    wasm_bindgen_futures::spawn_local(future);
}

pub async fn fetch_with_str(resource: &str) -> Result<JsValue> {
    JsFuture::from(window()?.fetch_with_str(resource))
        .await
        .map_err(|err| anyhow!("error fetching {:#?}", err))
}

pub async fn fetch_json(json_path: &str) -> Result<JsValue> {
    let resp_value = fetch_with_str(json_path).await?;
    let resp: Response = resp_value
        .dyn_into()
        .map_err(|element| anyhow!("Error converting {:#?} to Response", element))?;

    JsFuture::from(
        resp.json()
            .map_err(|err| anyhow!("error getting json {:#?}", err))?
        )
        .await
        .map_err(|err| anyhow!("error getting json {:#?}", err))
}

pub fn new_image() -> Result<HtmlImageElement> {
    HtmlImageElement::new()
        .map_err(|err| anyhow!("error creating image {:#?}", err))
}

pub fn closure_once<F, A, R>(fn_once: F) -> Closure<F::FnMut>
where
    F: 'static + WasmClosureFnOnce<A, R>,
{
    Closure::once(fn_once)
}

pub type LoopClosure = Closure<dyn FnMut(f64)>;
pub fn request_animation_frame(callback: &LoopClosure) -> Result<i32> {
    window()?
        .request_animation_frame(callback.as_ref().unchecked_ref())
        .map_err(|err| anyhow!("error requesting animation frame {:#?}", err))
}

pub fn create_raf_closure(f: impl FnMut(f64) + 'static) -> LoopClosure {
    closure_wrap(Box::new(f))
}


pub fn closure_wrap<T: WasmClosure + ?Sized>(data: Box<T>) -> Closure<T> {
    Closure::wrap(data)
}

pub fn now() -> Result<f64> {
    Ok(window()?
        .performance()
        .ok_or_else(|| anyhow!("no performance found"))?
        .now())
}
