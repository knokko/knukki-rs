use crate::*;
use log::info;

use wasm_bindgen::prelude::*;
use wasm_bindgen::{JsCast, Clamped};
use web_sys::{window, CanvasRenderingContext2d, Document, Element, Window, HtmlCanvasElement};
use unicode_segmentation::{Graphemes, UnicodeSegmentation};

pub struct WebFont {
    buffer_canvas: HtmlCanvasElement,
    pre_font: String,
    post_font: String,

    max_ascent: f32,
    max_descent: f32,
    whitespace_width: f32,
}

impl WebFont {
    pub fn from_strings(pre_font: String, post_font: String) -> Self {
        let the_window: Window = window().expect("There should be a window");
        let the_document: Document = the_window.document().expect("The window should have a document");
        let buffer_canvas_element: Element = the_document.create_element("canvas")
            .expect("Should be able to create canvas");
        let buffer_canvas: HtmlCanvasElement = buffer_canvas_element.dyn_into::<HtmlCanvasElement>()
            .expect("The canvas should be a canvas");

        // Determine max ascent and descent
        let font = format!("{} {}px {}", pre_font, 100, post_font);
        let high_ascent_string = "Ǘ";
        let high_descent_string = "̘";
        let whitespace_string = "n";

        let high_ascent = compute_metrics(high_ascent_string, &font).actual_ascent();
        let whitespace_metrics = compute_metrics(whitespace_string, &font);
        let whitespace_width = whitespace_metrics.actual_left() + whitespace_metrics.actual_right();
        let high_descent = compute_metrics(high_descent_string, &font).actual_descent();

        Self {
            buffer_canvas, pre_font, post_font,
            max_descent: high_descent as f32 / 100.0,
            max_ascent: high_ascent as f32 / 100.0,
            whitespace_width: whitespace_width as f32 / 100.0
        }
    }

    pub fn from_strs(pre_font: &str, post_font: &str) -> Self {
        Self::from_strings(String::from(pre_font), String::from(post_font))
    }
}

impl Font for WebFont {
    fn draw_grapheme(&self, grapheme: &str, point_size: f32) -> Option<CharTexture> {

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

        let width = (metrics.actual_right() + metrics.actual_left() + 1) as u32;
        let height = (metrics.actual_ascent() + metrics.actual_descent() + 1) as u32;

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

        let offset_y = (self.get_max_descent(point_size) as i32 - metrics.actual_descent()).max(0);
        Some(CharTexture { texture, offset_y: offset_y as u32 })
    }

    fn get_max_descent(&self, point_size: f32) -> f32 {
        self.max_descent * point_size
    }

    fn get_max_ascent(&self, point_size: f32) -> f32 {
        self.max_ascent * point_size
    }

    fn get_whitespace_width(&self, point_size: f32) -> f32 {
        self.whitespace_width * point_size
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