use knukki::*;

fn main() {
    let mut menu = SimpleFlatMenu::new(Some(Color::rgb(100, 200, 50)));
    menu.add_component(
        Box::new(HoverColorCircleComponent::new(
            Color::rgb(50, 0, 0),
            Color::rgb(250, 0, 0),
        )),
        ComponentDomain::between(0.1, 0.1, 0.3, 0.3),
    );
    menu.add_component(
        Box::new(HoverColorCircleComponent::new(
            Color::rgb(0, 50, 0),
            Color::rgb(0, 250, 0),
        )),
        ComponentDomain::between(0.2, 0.5, 0.5, 0.8),
    );
    menu.add_component(
        Box::new(HoverColorCircleComponent::new(
            Color::rgb(0, 0, 50),
            Color::rgb(0, 0, 250),
        )),
        ComponentDomain::between(0.7, 0.4, 0.9, 0.8),
    );
    start(Application::new(Box::new(menu)), "Hover color circle menu");
}
