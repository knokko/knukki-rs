use knukki::*;

pub const EXAMPLE_NAME: &'static str = "text-display";

pub fn create_app() -> Application {
    let mut menu = SimpleFlatMenu::new(Some(Color::rgb(100, 200, 50)));

    menu.add_component(Box::new(SimpleTextComponent::new(
        "Hello, knukki!".to_string(),
        HorizontalTextAlignment::Center,
        VerticalTextAlignment::Center,
        None
    )), ComponentDomain::between(0.0, 0.2, 1.0, 0.6));

    Application::new(Box::new(menu))
}
