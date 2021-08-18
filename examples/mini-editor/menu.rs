use knukki::*;

pub const EXAMPLE_NAME: &'static str = "mini-editor";

pub fn create_app() -> Application {
    Application::new(create_main_menu())
}

pub fn create_base_style() -> TextStyle {
    TextStyle {
        font_id: None,
        text_color: Color::rgb(0, 0, 0),
        background_color: Color::rgb(200, 200, 200),
        background_fill_mode: TextBackgroundFillMode::DrawnRegion
    }
}

pub fn create_exit_style() -> TextStyle {
    TextStyle {
        font_id: None,
        text_color: Color::rgb(0, 0, 0),
        background_color: Color::rgb(200, 200, 170),
        background_fill_mode: TextBackgroundFillMode::DrawnRegion
    }
}

fn create_main_menu() -> Box<dyn Component> {
    let mut menu = SimpleFlatMenu::new(Some(Color::rgb(170, 170, 200)));

    menu.add_component(Box::new(SimpleTextComponent::new(
        "New item set", HorizontalTextAlignment::Center, VerticalTextAlignment::Center,
        create_base_style()
    )), ComponentDomain::between(0.3, 0.8, 0.7, 0.95));
    menu.add_component(Box::new(SimpleTextComponent::new(
        "Edit item set", HorizontalTextAlignment::Center, VerticalTextAlignment::Center,
        create_base_style()
    )), ComponentDomain::between(0.3, 0.6, 0.7, 0.75));
    menu.add_component(Box::new(SimpleTextComponent::new(
        "Combine item sets", HorizontalTextAlignment::Center, VerticalTextAlignment::Center,
        create_base_style()
    )), ComponentDomain::between(0.3, 0.4, 0.7, 0.55));
    menu.add_component(Box::new(SimpleTextComponent::new(
        "Exit editor", HorizontalTextAlignment::Center, VerticalTextAlignment::Center,
        create_exit_style()
    )), ComponentDomain::between(0.3, 0.15, 0.7, 0.3));

    Box::new(menu)
}