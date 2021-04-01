use crate::*;

use wasm_bindgen::prelude::*;
use wasm_bindgen::{JsCast, Clamped};
use web_sys::{window, CanvasRenderingContext2d, Document, Element, Window, HtmlCanvasElement};
use unicode_segmentation::{Graphemes, UnicodeSegmentation};

pub struct WebFont {
    buffer_canvas: HtmlCanvasElement,
    pre_font: String,
    post_font: String,
}

impl WebFont {
    pub fn from_strings(pre_font: String, post_font: String) -> Self {
        let the_window: Window = window().expect("There should be a window");
        let the_document: Document = the_window.document().expect("The window should have a document");
        let buffer_canvas_element: Element = the_document.create_element("canvas")
            .expect("Should be able to create canvas");
        let buffer_canvas: HtmlCanvasElement = buffer_canvas_element.dyn_into::<HtmlCanvasElement>()
            .expect("The canvas should be a canvas");

        Self { buffer_canvas, pre_font, post_font }
    }

    pub fn from_strs(pre_font: &str, post_font: &str) -> Self {
        Self::from_strings(String::from(pre_font), String::from(post_font))
    }
}

impl Font for WebFont {
    fn draw_grapheme(&self, grapheme: &str, point_size: f32) -> Option<Texture> {

        let font = format!("{} {}px {}", self.pre_font, point_size as u32, self.post_font);
        let ctx: CanvasRenderingContext2d = self.buffer_canvas.get_context("2d")
            .expect("Should be able to use canvas.get_context")
            .expect("The canvas should support the 2d context")
            .dyn_into::<CanvasRenderingContext2d>()
            .expect("2d context should be a 2d context");

        ctx.set_font(&font);

        // In order to compute text metrics, I need experimental API's, which are not available in web-sys
        // So I will have to provide some javascript to do the job
        let metrics = compute_metrics(grapheme, &font);

        let width = (metrics.actual_right() + metrics.actual_left()) as u32;
        let height = (metrics.actual_ascent() + metrics.actual_descent()) as u32;

        // Handle whitespace characters
        if width == 0 || height == 0 {
            return None;
        }

        let offset_x = metrics.actual_left();
        let offset_y = -metrics.actual_descent();

        let adjust_width = self.buffer_canvas.width() < width;
        let adjust_height = self.buffer_canvas.height() < height;
        if adjust_width {
            self.buffer_canvas.set_width(width);
        }
        if adjust_height {
            self.buffer_canvas.set_height(height);
        }
        if adjust_width || adjust_height {
            ctx.set_font(&font);
        }

        ctx.set_fill_style(&JsValue::from("black"));
        ctx.fill_rect(0.0, 0.0, width as f64, height as f64);
        ctx.set_fill_style(&JsValue::from("white"));
        ctx.fill_text(grapheme, offset_x as f64, (offset_y + height as i32) as f64)
            .expect("Should be able to draw text");

        let image_data = ctx.get_image_data(0.0, 0.0, width as f64, height as f64)
            .expect("Should be able to read image data");
        let clamped_data: Clamped<Vec<u8>> = image_data.data();

        let mut texture = Texture::new(width, height, Color::rgb(0, 0, 0));

        for color_index in 0 .. width * height {
            let index = color_index as usize * 4;
            let x = color_index % width;
            let y = (color_index / width) as usize;
            let value = clamped_data[index];
            texture[x][height as usize - y - 1] = Color::rgb(value, 0, 0);
        }

        Some(texture)
    }
}

#[wasm_bindgen(module = "/extra-module.js")]
extern "C" {
    type CustomTextMetrics;

    fn compute_metrics(grapheme: &str, font: &str) -> CustomTextMetrics;

    #[wasm_bindgen(method, getter)]
    fn actual_left(this: &CustomTextMetrics) -> i32;

    #[wasm_bindgen(method, getter)]
    fn actual_descent(this: &CustomTextMetrics) -> i32;

    #[wasm_bindgen(method, getter)]
    fn actual_right(this: &CustomTextMetrics) -> i32;

    #[wasm_bindgen(method, getter)]
    fn actual_ascent(this: &CustomTextMetrics) -> i32;
}

pub fn create_default_font() -> WebFont {
    WebFont::from_strs("", "serif")
}