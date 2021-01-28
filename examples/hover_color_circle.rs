use knukki::*;

fn main() {
    let component = HoverColorCircleComponent::new(Color::rgb(50, 50, 50), Color::rgb(30, 200, 80));
    start(Application::new(Box::new(component)), "Hover color circle");
}
