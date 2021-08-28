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

        renderer.get_text_renderer().draw_text(
            &self.text, &TextStyle {
                font_id: self.style.font_id.clone(),
                text_color,
                background_color,
                background_fill_mode: TextBackgroundFillMode::DoNot
            }, TextDrawPosition {
                min_x: self.style.margin / renderer.get_viewport().get_aspect_ratio(),
                min_y: self.style.margin,
                max_x: 1.0 - self.style.margin / renderer.get_viewport().get_aspect_ratio(),
                max_y: 1.0 - self.style.margin,
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
