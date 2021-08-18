use knukki::*;

pub const EXAMPLE_NAME: &'static str = "mini-editor";

pub fn create_app() -> Application {
    let mut menu = SimpleFlatMenu::new(Some(Color::rgb(100, 200, 50)));

    // TODO Add the components to the menu (or replace the menu with a component)

    Application::new(Box::new(menu))
}