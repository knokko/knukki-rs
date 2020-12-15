pub struct ComponentSubscriptions {
    // Mouse event subscriptions
    pub mouse_click: bool,
    pub mouse_click_out: bool,
    pub mouse_move: bool,
    pub mouse_leave: bool,
    pub mouse_enter: bool,

    // Other subscriptions
    pub char_type: bool,
}

impl ComponentSubscriptions {
    pub fn new() -> Self {
        Self {
            mouse_click: false,
            mouse_click_out: false,
            mouse_move: false,
            mouse_leave: false,
            mouse_enter: false,

            char_type: false,
        }
    }
}
