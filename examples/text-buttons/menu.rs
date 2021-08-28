use knukki::*;

pub const EXAMPLE_NAME: &'static str = "text-buttons";

pub fn create_app() -> Application {
    let mut menu = SimpleFlatMenu::new(Some(Color::rgb(100, 200, 50)));

    menu.add_component(
        Box::new(TextButton::new("Without border", TextButtonStyle {
            font_id: None,
            base_text_color: Color::rgb(200, 200, 200),
            base_background_color: Color::rgb(0, 150, 200),
            hover_text_color: Color::rgb(255, 255, 255),
            hover_background_color: Color::rgb(0, 200, 250),
            margin: 0.15,
            border_style: TextButtonBorderStyle::None
        })), ComponentDomain::between(0.1, 0.1, 0.4, 0.4)
    );

    Application::new(Box::new(menu))
}