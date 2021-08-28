use crate::*;

pub struct SimpleTextComponent {
    text: String,
    horizontal_alignment: HorizontalTextAlignment,
    vertical_alignment: VerticalTextAlignment,
    style: TextStyle
}

impl SimpleTextComponent {
    pub fn new(
        text: impl Into<String>,
        horizontal_alignment: HorizontalTextAlignment,
        vertical_alignment: VerticalTextAlignment,
        style: TextStyle
    ) -> Self {
        Self {
            text: text.into(), horizontal_alignment, vertical_alignment, style
        }
    }
}

impl Component for SimpleTextComponent {
    fn on_attach(&mut self, _buddy: &mut dyn ComponentBuddy) {
    }

    fn render(&mut self, renderer: &Renderer, _buddy: &mut dyn ComponentBuddy, _force: bool) -> RenderResult {
        let position = TextDrawPosition {
            min_x: 0.0,
            min_y: 0.0,
            max_x: 1.0,
            max_y: 1.0,
            horizontal_alignment: self.horizontal_alignment,
            vertical_alignment: self.vertical_alignment,
        };

        let region = renderer.get_text_renderer().draw_text(
            &self.text, &self.style, position, renderer, None
        )?;

        Ok(RenderResultStruct {
            drawn_region: Box::new(RectangularDrawnRegion::new(
                region.min_x, region.min_y, region.max_x, region.max_y
            )),
            filter_mouse_actions: false
        })
    }
}
