use knukki::*;

pub const EXAMPLE_NAME: &'static str = "hover-color-circle";

pub fn create_app() -> Application {
    let component = HoverColorCircleComponent::new(Color::rgb(50, 50, 50), Color::rgb(30, 200, 80));

    Application::new(Box::new(component))
}