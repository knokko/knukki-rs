use crate::*;

pub struct TextButtonStyle {
    pub font_id: Option<String>,
    pub base_text_color: Color,
    pub base_background_color: Color,
    pub hover_text_color: Color,
    pub hover_background_color: Color,
    pub margin: f32,
    pub border_style: TextButtonBorderStyle
}

pub enum TextButtonBorderStyle {
    None,
    Rectangular { color: Color, max_width: f32, max_height: f32 },
    RoundRectangular { color: Color, max_width: f32, max_height: f32 },
}

pub struct TextButton {
    text: String,
    style: TextButtonStyle,
    shader: FragmentOnlyShader,
    // TODO on_click
}

fn shader_description_no_border() -> FragmentOnlyShaderDescription {
    FragmentOnlyShaderDescription {
        source_code: "
            void main() {
                gl_FragColor = color1;
            }
        ".to_string(),
        num_float_matrices: 0,
        num_colors: 1,
        num_float_vectors: 0,
        num_int_vectors: 0,
        num_floats: 0,
        num_ints: 0
    }
}

impl TextButton {
    pub fn new(text: &str, style: TextButtonStyle) -> Self {
        let shader_description = shader_description_no_border();
        let shader = FragmentOnlyShader::new(shader_description);
        Self {
            text: text.to_string(),
            style,
            shader
        }
    }
}

impl Component for TextButton {
    fn on_attach(&mut self, buddy: &mut dyn ComponentBuddy) {
        buddy.subscribe_mouse_click();
        buddy.subscribe_mouse_enter();
        buddy.subscribe_mouse_leave();
    }

    fn render(&mut self, renderer: &Renderer, buddy: &mut dyn ComponentBuddy, _force: bool) -> RenderResult {
        let (text_color, background_color) = match buddy.get_local_mouses().is_empty() {
            true => (self.style.hover_text_color, self.style.hover_background_color),
            false => (self.style.base_text_color, self.style.base_background_color)
        };

        renderer.clear(Color::rgb(200, 0, 150));

        let text_style = TextStyle {
            font_id: self.style.font_id.clone(),
            text_color,
            background_color,
            background_fill_mode: TextBackgroundFillMode::DoNot
        };

        let (text_width, text_height) = renderer.get_text_renderer().get_text_size(
            &self.text, &text_style, renderer
        )?;
        let domain_ratio = renderer.get_viewport().get_aspect_ratio();

        let (reserved_margin_x, reserved_margin_y) = {
            let mut reserved_margin_x = 0.0;
            let mut reserved_margin_y = 0.0;

            // TODO This system is not sound, especially when margin is big (> 0.5)
            for _counter in 0 .. 2 {
                let compute_scales = |test_margin_x: f32, test_margin_y: f32| {
                    let max_scale_x1 = (1.0 - 2.0 * test_margin_x) / text_width as f32;
                    let max_scale_y1 = (1.0 - 2.0 * test_margin_y) / text_height as f32;
                    let max_scale_x2 = max_scale_y1 / domain_ratio;
                    let max_scale_y2 = max_scale_x1 * domain_ratio;
                    if max_scale_x2 <= max_scale_x1 {
                        (max_scale_x2, max_scale_y1)
                    } else {
                        (max_scale_x1, max_scale_y2)
                    }
                };

                let (full_scale_x, full_scale_y) = compute_scales(0.0, 0.0);
                let (_, trimmed_scale_y) = compute_scales(reserved_margin_x, reserved_margin_y);

                let drawn_text_height = trimmed_scale_y * text_height as f32;
                let margin_y = self.style.margin * drawn_text_height;
                let margin_x = margin_y / domain_ratio;

                let scaled_width = text_width as f32 * full_scale_x;
                let scaled_height = text_height as f32 * full_scale_y;

                let limit_x = 1.0 - 2.0 * margin_x;
                let limit_y = 1.0 - 2.0 * margin_y;
                reserved_margin_x = if scaled_width <= limit_x {
                    0.0
                } else {
                    (scaled_width - limit_x) / 2.0
                };
                reserved_margin_y = if scaled_height <= limit_y {
                    0.0
                } else {
                    (scaled_height - limit_y) / 2.0
                };
                println!("margins currently are ({}, {})", reserved_margin_x, reserved_margin_y);
            }
            println!("Finished");
            (reserved_margin_x, reserved_margin_y)
        };

        renderer.get_text_renderer().draw_text(
            &self.text, &text_style, TextDrawPosition {
                min_x: reserved_margin_x,
                min_y: reserved_margin_y,
                max_x: 1.0 - reserved_margin_x,
                max_y: 1.0 - reserved_margin_y,
                horizontal_alignment: HorizontalTextAlignment::Center,
                vertical_alignment: VerticalTextAlignment::Center
            }, renderer, Some(&mut |text_position: DrawnTextPosition| {
                let draw_parameters = FragmentOnlyDrawParameters {
                    colors: &[background_color],
                    ..FragmentOnlyDrawParameters::default()
                };

                let margin_y = self.style.margin * (text_position.max_y - text_position.min_y);
                let margin_x = margin_y / renderer.get_viewport().get_aspect_ratio();
                renderer.apply_fragment_shader(
                    text_position.min_x - margin_x,
                    text_position.min_y - margin_y,
                    text_position.max_x + margin_x,
                    text_position.max_y + margin_y,
                    &self.shader, draw_parameters
                );
            })
        )?;

        entire_render_result()
    }

    fn on_mouse_click(&mut self, _event: MouseClickEvent, _buddy: &mut dyn ComponentBuddy) {
        // TODO Fire event listener
    }

    fn on_mouse_enter(&mut self, _event: MouseEnterEvent, buddy: &mut dyn ComponentBuddy) {
        buddy.request_render();
    }

    fn on_mouse_leave(&mut self, _event: MouseLeaveEvent, buddy: &mut dyn ComponentBuddy) {
        buddy.request_render();
    }
}
