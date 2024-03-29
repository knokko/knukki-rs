use knukki::*;

pub const EXAMPLE_NAME: &'static str = "text-display";

pub fn create_app() -> Application {
    let mut menu = SimpleFlatMenu::new(Some(Color::rgb(100, 200, 50)));

    menu.add_component(Box::new(SimpleTextComponent::new(
        "ᄍᄎᄏʡʢʣʤମଯରיך⽉⽊⽋כלםמןנឝឞសסעףפ綾菱ץ".to_string(),
        HorizontalTextAlignment::Center,
        VerticalTextAlignment::Center,
        TextStyle {
            font_id: None,
            text_color: Color::rgb(0, 0, 0),
            background_color: Color::rgb(0, 0, 200),
            background_fill_mode: TextBackgroundFillMode::DoNot
        }
    )), ComponentDomain::between(0.0, 0.0, 1.0, 1.0));

    Application::new(Box::new(menu))
}
