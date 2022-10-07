use std::f64;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[wasm_bindgen(start)]
pub fn start() {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>().map_err(|_| ()).unwrap();

    let context = canvas.get_context("2d").unwrap().unwrap().dyn_into::<web_sys::CanvasRenderingContext2d>().unwrap();

    let gradient = context.create_linear_gradient(0.0, 0.0, canvas.width() as f64, 0.0);
    gradient.add_color_stop(0.0, " magenta").ok();
    gradient.add_color_stop(0.5, "blue").ok();
    gradient.add_color_stop(1.0, "red").ok();

    context.set_fill_style(&web_sys::CanvasGradient::from(gradient));
    context.fill_text("Hello, world!", canvas.width() as f64 / 2.0, canvas.height() as f64 / 2.0).ok();
}