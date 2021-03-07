use golem::*;
use knukki::*;

pub const EXAMPLE_NAME: &'static str = "circle-dragging";

pub fn create_app() -> Application {
    let mut menu = SimpleFlatMenu::new(Some(Color::rgb(100, 200, 50)));

    menu.add_component(Box::new(CircleDraggingComponent {
        base_color: Color::rgb(50, 50, 50),
        drag_color: Color::rgb(200, 50, 50),
        background_color: Color::rgb(150, 150, 200),
        radius: 0.1,
        max_center_distance: 0.4,

        circle_position: Point::new(0.5, 0.5),
        last_radius_x: None,
        last_radius_y: None,
    }), ComponentDomain::between(0.1, 0.3, 0.4, 0.7));

    menu.add_component(Box::new(CircleDraggingComponent {
        base_color: Color::rgb(50, 250, 50),
        drag_color: Color::rgb(50, 50, 50),
        background_color: Color::rgb(150, 0, 200),
        radius: 0.15,
        max_center_distance: 0.3,

        circle_position: Point::new(0.8, 0.2),
        last_radius_x: None,
        last_radius_y: None,
    }), ComponentDomain::between(0.5, 0.5, 0.9, 1.0));

    Application::new(Box::new(menu))
}

struct CircleDraggingComponent {
    base_color: Color,
    drag_color: Color,
    background_color: Color,
    radius: f32,
    max_center_distance: f32,

    circle_position: Point,
    last_radius_x: Option<f32>,
    last_radius_y: Option<f32>,
}

#[rustfmt::skip]
fn create_shader(golem: &Context) -> Result<ShaderProgram, GolemError> {
    let description = ShaderDescription {
        vertex_input: &[
            Attribute::new("position", AttributeType::Vector(Dimension::D2))
        ],
        fragment_input: &[
            Attribute::new("knukkiPosition", AttributeType::Vector(Dimension::D2))
        ],
        uniforms: &[
            Uniform::new("color", UniformType::Vector(NumberType::Float, Dimension::D3)),
            Uniform::new("center", UniformType::Vector(NumberType::Float, Dimension::D2)),
            Uniform::new("radius", UniformType::Vector(NumberType::Float, Dimension::D2)),
        ],
        vertex_shader: "
            void main() {
                gl_Position = vec4(position, 0.0, 1.0);
                knukkiPosition = position * 0.5 + 0.5;
            }",
        fragment_shader: "
            void main() {
                float dx = (knukkiPosition.x - center.x) / radius.x;
                float dy = (knukkiPosition.y - center.y) / radius.y;
                if (dx * dx + dy * dy <= 1.0) {
                    gl_FragColor = vec4(color, 1.0);
                } else {
                    discard;
                }
            }",
    };

    ShaderProgram::new(golem, description)
}

impl CircleDraggingComponent {
    fn is_inside(&self, mouse: Point) -> bool {
        if let Some(radius_x) = self.last_radius_x {
            if let Some(radius_y) = self.last_radius_y {
                let dx = (mouse.get_x() - self.circle_position.get_x()) / radius_x;
                let dy = (mouse.get_y() - self.circle_position.get_y()) / radius_y;
                return dx * dx + dy * dy <= 1.0;
            }
        }

        // Always return false before the first render
        false
    }
}

impl Component for CircleDraggingComponent {
    fn on_attach(&mut self, buddy: &mut dyn ComponentBuddy) {
        buddy.subscribe_mouse_release();
        buddy.subscribe_mouse_press();
        buddy.subscribe_mouse_enter();
        buddy.subscribe_mouse_leave();
        buddy.subscribe_mouse_move();
    }

    fn render(&mut self, renderer: &Renderer, buddy: &mut dyn ComponentBuddy, _force: bool) -> RenderResult {
        renderer.clear(self.background_color);

        let is_dragging = buddy.get_local_mouses().into_iter().any(|mouse| {
            if let Some(position) = buddy.get_mouse_position(mouse) {
                if self.is_inside(position) {
                    if let Some(pressed_buttons) = buddy.get_pressed_mouse_buttons(mouse) {
                        if !pressed_buttons.is_empty() {
                            return true;
                        }
                    }
                }
            }

            false
        });

        let color = match is_dragging {
            true => self.drag_color,
            false => self.base_color
        };

        let aspect_ratio = renderer.get_viewport().get_aspect_ratio();
        let radius_x;
        let radius_y;
        if aspect_ratio > 1.0 {
            radius_x = self.radius / aspect_ratio;
            radius_y = self.radius;
        } else {
            radius_x = self.radius;
            radius_y = self.radius * aspect_ratio;
        }

        self.last_radius_x = Some(radius_x);
        self.last_radius_y = Some(radius_y);

        renderer.use_cached_shader(
            &ShaderId::from_strs("knukki", "ExampleCircleDragging"),
            create_shader, |shader| {
                shader.set_uniform("color", UniformValue::Vector3([
                    color.get_red_float(), color.get_green_float(), color.get_blue_float()
                ]))?;
                shader.set_uniform("center", UniformValue::Vector2(
                    [self.circle_position.get_x(), self.circle_position.get_y()]
                ))?;
                shader.set_uniform("radius", UniformValue::Vector2(
                    [radius_x, radius_y]
                ))?;

                unsafe {
                    shader.draw(
                        renderer.get_quad_vertices(),
                        renderer.get_quad_indices(),
                        0..renderer.get_num_quad_indices(),
                        GeometryMode::Triangles
                    )
                }
            }
        )?;

        // Note that we also draw the background, so we fill the entire render domain
        entire_render_result()
    }

    fn on_mouse_press(&mut self, event: MousePressEvent, buddy: &mut dyn ComponentBuddy) {
        if self.is_inside(event.get_point()) {
            buddy.request_render();
        }
    }

    fn on_mouse_release(&mut self, event: MouseReleaseEvent, buddy: &mut dyn ComponentBuddy) {
        if self.is_inside(event.get_point()) {
            buddy.request_render();
        }
    }

    fn on_mouse_move(&mut self, event: MouseMoveEvent, buddy: &mut dyn ComponentBuddy) {
        if self.is_inside(event.get_from()) {
            if let Some(pressed_buttons) = buddy.get_pressed_mouse_buttons(event.get_mouse()) {
                if !pressed_buttons.is_empty() {
                    let domain_center = Point::new(0.5, 0.5);
                    let mut circle_position = event.get_to() - event.get_from() + self.circle_position;
                    let distance = domain_center.distance_to(circle_position);
                    if distance > self.max_center_distance {
                        circle_position = domain_center + (circle_position - domain_center) * (self.max_center_distance / distance);
                    }
                    self.circle_position = circle_position;
                    buddy.request_render();
                }
            }
        }
    }

    fn on_mouse_enter(&mut self, event: MouseEnterEvent, buddy: &mut dyn ComponentBuddy) {
        if self.is_inside(event.get_entrance_point()) {
            if let Some(pressed_buttons) = buddy.get_pressed_mouse_buttons(event.get_mouse()) {
                if !pressed_buttons.is_empty() {
                    buddy.request_render();
                }
            }
        }
    }

    fn on_mouse_leave(&mut self, event: MouseLeaveEvent, buddy: &mut dyn ComponentBuddy) {
        if self.is_inside(event.get_exit_point()) {
            if let Some(pressed_buttons) = buddy.get_pressed_mouse_buttons(event.get_mouse()) {
                if !pressed_buttons.is_empty() {
                    buddy.request_render();
                }
            }
        }
    }
}
