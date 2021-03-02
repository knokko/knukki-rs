use knukki::*;

pub fn create_app() -> Application {
    let mut menu = SimpleFlatMenu::new(Some(Color::rgb(150, 150, 250)));
    for x in 0..50 {
        for y in 0..50 {
            let min_x = 0.1 + 0.015 * x as f32;
            let min_y = 0.1 + 0.015 * y as f32;
            let max_x = 0.1 + 0.015 * (x + 1) as f32;
            let max_y = 0.1 + 0.015 * (y + 1) as f32;
            let base_color = Color::rgb(100, 3 * x, 3 * y);
            let hover_color = Color::rgb(200, 5 * x, 5 * y);
            menu.add_component(
                Box::new(HoverColorCircleComponent::new(base_color, hover_color)),
                ComponentDomain::between(min_x, min_y, max_x, max_y),
            );
        }
    }

    Application::new(Box::new(menu))
}