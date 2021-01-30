use crate::*;

/// A helper struct to keep track of mouse information (like the position and the pressed buttons).
/// This struct is made to make the implementation of `ComponentBuddy`s easier (and for code reuse)
/// between different implementations.
pub struct MouseStore {
    // I won't use a (Hash)Map because the number of mouses is expected to be very small
    entries: Vec<MouseEntry>,
}

impl MouseStore {
    /// Constructs a new empty `MouseStore`
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Gets the state of the given `Mouse`, if this store has information about it. If not, this
    /// method will return `None`.
    pub fn get_mouse_state(&self, mouse: Mouse) -> Option<&MouseState> {
        for entry in &self.entries {
            if entry.mouse == mouse {
                return Some(&entry.state);
            }
        }

        return None;
    }

    /// Gives the opportunity to update the state of a given `Mouse` (by returning a mutable
    /// reference to it). If this store doesn't have any information about the given `Mouse` yet,
    /// this will return `None` and you should probably use `add_mouse` instead.
    pub fn update_mouse_state(&mut self, mouse: Mouse) -> Option<&mut MouseState> {
        for entry in &mut self.entries {
            if entry.mouse == mouse {
                return Some(&mut entry.state);
            }
        }

        return None;
    }

    /// Removes the given `Mouse` from this store (and any associated state like position and
    /// pressed buttons).
    ///
    /// This should typically be called when the mouse leaves the window.
    pub fn remove_mouse(&mut self, mouse: Mouse) {
        self.entries.drain_filter(|entry| entry.mouse == mouse);
    }

    /// Adds the given `Mouse` to this store and initialises its state to the given `MouseState`.
    /// If this store already had state about the given mouse, that old state will be replaced with
    /// the given `MouseState`.
    ///
    /// This should typically be called when the mouse enters the window.
    pub fn add_mouse(&mut self, mouse: Mouse, initial_state: MouseState) {
        // Make sure we don't get multiple entries with the same mouse
        self.remove_mouse(mouse);
        self.entries.push(MouseEntry {
            mouse,
            state: initial_state,
        });
    }

    /// Creates and returns a `Vec` containing all `Mouse`s that have been added to this store, but
    /// *not* (yet) removed.
    pub fn get_mouses(&self) -> Vec<Mouse> {
        self.entries.iter().map(|entry| entry.mouse).collect()
    }
}

struct MouseEntry {
    mouse: Mouse,
    state: MouseState,
}

/// Represents the state (position, pressed buttons...) of a single `Mouse`.
#[derive(Clone, Debug, PartialEq)]
pub struct MouseState {
    /// The current position of the associated mouse
    pub position: Point,
    pub buttons: PressedMouseButtons,
}

#[derive(Eq, Clone, Debug)]
pub struct PressedMouseButtons {
    button_vec: Vec<MouseButton>,
}

impl PartialEq for PressedMouseButtons {
    fn eq(&self, other: &Self) -> bool {
        'own_outer_loop:
        for own_button in &self.button_vec {
            for other_button in &other.button_vec {
                if own_button == other_button {
                    continue 'own_outer_loop;
                }
            }
            return false;
        }

        'other_outer_loop:
        for other_button in &other.button_vec {
            for own_button in &self.button_vec {
                if own_button == other_button {
                    continue 'other_outer_loop;
                }
            }
            return false;
        }

        true
    }
}

impl PressedMouseButtons {

    pub fn new() -> Self {
        Self { button_vec: Vec::with_capacity(2) }
    }

    pub fn is_pressed(&self, button: MouseButton) -> bool {
        self.button_vec.contains(&button)
    }

    pub fn press(&mut self, button: MouseButton) {
        if !self.is_pressed(button) {
            self.button_vec.push(button);
        }
    }

    pub fn release(&mut self, button: MouseButton) {
        self.button_vec.retain(|pressed_button| *pressed_button != button);
    }
}
#[cfg(test)]
mod tests {

    use crate::*;

    #[test]
    fn test_add_and_remove() {
        let mut store = MouseStore::new();
        let mouse1 = Mouse::new(100);
        let mouse2 = Mouse::new(7);
        let mouse3 = Mouse::new(33);
        let test_state = MouseState {
            position: Point::new(0.4, 0.1),
            buttons: PressedMouseButtons::new(),
        };

        assert!(store.get_mouse_state(mouse1).is_none());
        assert_eq!(Vec::<Mouse>::new(), store.get_mouses());

        store.add_mouse(mouse2, test_state.clone());
        assert!(store.get_mouse_state(mouse1).is_none());
        assert_eq!(vec![mouse2], store.get_mouses());
        store.add_mouse(mouse1, test_state.clone());
        assert_eq!(vec![mouse2, mouse1], store.get_mouses());
        assert!(store.get_mouse_state(mouse1).is_some());
        assert!(store.get_mouse_state(mouse2).is_some());

        // This shouldn't have any effect
        store.remove_mouse(mouse3);
        assert!(store.get_mouse_state(mouse1).is_some());
        assert!(store.get_mouse_state(mouse2).is_some());
        assert_eq!(vec![mouse2, mouse1], store.get_mouses());

        // This should remove only the second mouse
        store.remove_mouse(mouse2);
        assert!(store.get_mouse_state(mouse1).is_some());
        assert!(store.get_mouse_state(mouse2).is_none());
        assert_eq!(vec![mouse1], store.get_mouses());

        // Adding the first mouse again shouldn't have any effect
        store.add_mouse(mouse1, test_state.clone());
        assert!(store.get_mouse_state(mouse1).is_some());
        assert!(store.get_mouse_state(mouse2).is_none());
        assert_eq!(vec![mouse1], store.get_mouses());

        // Adding the second mouse again should be possible
        store.add_mouse(mouse2, test_state.clone());
        assert!(store.get_mouse_state(mouse1).is_some());
        assert!(store.get_mouse_state(mouse2).is_some());
        assert_eq!(vec![mouse1, mouse2], store.get_mouses());

        // Check that we can remove both mouses...
        store.remove_mouse(mouse1);
        assert!(store.get_mouse_state(mouse1).is_none());
        assert!(store.get_mouse_state(mouse2).is_some());
        assert_eq!(vec![mouse2], store.get_mouses());
        store.remove_mouse(mouse2);
        assert!(store.get_mouse_state(mouse1).is_none());
        assert!(store.get_mouse_state(mouse2).is_none());
        assert_eq!(Vec::<Mouse>::new(), store.get_mouses());
    }

    #[test]
    fn test_state_updating() {
        let mouse1 = Mouse::new(0);
        let mouse2 = Mouse::new(200);

        let mut state1 = MouseState {
            position: Point::new(0.0, 0.2),
            buttons: PressedMouseButtons::new(),
        };
        let mut state2 = MouseState {
            position: Point::new(0.3, 0.1),
            buttons: PressedMouseButtons::new(),
        };
        let mut state3 = MouseState {
            position: Point::new(0.6, 0.7),
            buttons: PressedMouseButtons::new(),
        };

        let mut store = MouseStore::new();

        // Test single add
        store.add_mouse(mouse1, state1.clone());
        assert_eq!(Some(&state1), store.get_mouse_state(mouse1));
        assert_eq!(Some(&mut state1), store.update_mouse_state(mouse1));

        // Test overwriting
        store.add_mouse(mouse1, state2.clone());
        assert_eq!(Some(&mut state2), store.update_mouse_state(mouse1));
        assert_eq!(vec![mouse1], store.get_mouses());

        // Check that it's not possible to update a mouse the store doesn't have
        assert!(store.update_mouse_state(mouse2).is_none());
        assert_eq!(vec![mouse1], store.get_mouses());

        // But it should be possible once mouse2 has been added
        store.add_mouse(mouse2, state3.clone());
        assert_eq!(Some(&mut state3), store.update_mouse_state(mouse2));

        // Of course, we should be able to actually mutate it
        store.update_mouse_state(mouse2).unwrap().position = state1.position;
        assert_eq!(vec![mouse1, mouse2], store.get_mouses());

        // This should update the state of mouse2
        assert_eq!(
            state1.position,
            store.get_mouse_state(mouse2).unwrap().position
        );
        // But not the state of mouse1
        assert_eq!(Some(&state2), store.get_mouse_state(mouse1));

        // We shouldn't keep the state of mouse1 after removing it
        store.remove_mouse(mouse1);
        assert!(store.get_mouse_state(mouse1).is_none());
        assert!(store.update_mouse_state(mouse1).is_none());

        // But we should keep the state of mouse2
        assert_eq!(
            state1.position,
            store.get_mouse_state(mouse2).unwrap().position
        );
        assert_eq!(
            state1.position,
            store.update_mouse_state(mouse2).unwrap().position
        );
        assert_eq!(vec![mouse2], store.get_mouses());
    }

    #[test]
    fn test_pressed_buttons_eq() {
        assert_eq!(PressedMouseButtons::new(), PressedMouseButtons::new());
        assert_eq!(PressedMouseButtons {
            button_vec: vec![MouseButton::new(2)]
        }, PressedMouseButtons {
            button_vec: vec![MouseButton::new(2)]
        });
        assert_eq!(PressedMouseButtons {
            button_vec: vec![MouseButton::new(2), MouseButton::new(5)]
        }, PressedMouseButtons {
            button_vec: vec![MouseButton::new(2), MouseButton::new(5)]
        });
        assert_eq!(PressedMouseButtons {
            button_vec: vec![MouseButton::new(5), MouseButton::new(2)]
        }, PressedMouseButtons {
            button_vec: vec![MouseButton::new(2), MouseButton::new(5)]
        });

        assert_ne!(PressedMouseButtons::new(), PressedMouseButtons {
            button_vec: vec![MouseButton::new(0)]
        });
        assert_ne!(PressedMouseButtons {
            button_vec: vec![MouseButton::new(0)]
        }, PressedMouseButtons::new());
        assert_ne!(PressedMouseButtons {
            button_vec: vec![MouseButton::new(1)]
        }, PressedMouseButtons {
            button_vec: vec![MouseButton::new(2)]
        });
        assert_ne!(PressedMouseButtons {
            button_vec: vec![MouseButton::new(1), MouseButton::new(2)]
        }, PressedMouseButtons {
            button_vec: vec![MouseButton::new(2)]
        });
        assert_ne!(PressedMouseButtons {
            button_vec: vec![MouseButton::new(1)]
        }, PressedMouseButtons {
            button_vec: vec![MouseButton::new(2), MouseButton::new(1)]
        });
        assert_ne!(PressedMouseButtons {
            button_vec: vec![MouseButton::new(2), MouseButton::new(3)]
        }, PressedMouseButtons {
            button_vec: vec![MouseButton::new(2), MouseButton::new(1)]
        });
    }

    #[test]
    fn test_pressed_buttons() {
        let mut buttons = PressedMouseButtons::new();
        let button1 = MouseButton::new(0);
        let button2 = MouseButton::new(2);
        let button3 = MouseButton::new(3);

        assert!(!buttons.is_pressed(button1));

        buttons.press(button1);
        assert!(buttons.is_pressed(button1));
        assert!(!buttons.is_pressed(button2));

        buttons.press(button3);
        assert!(buttons.is_pressed(button1));
        assert!(!buttons.is_pressed(button2));
        assert!(buttons.is_pressed(button3));

        buttons.release(button1);
        assert!(!buttons.is_pressed(button1));
        assert!(buttons.is_pressed(button3));

        buttons.press(button2);
        assert!(buttons.is_pressed(button2));
        buttons.press(button2);
        assert!(!buttons.is_pressed(button1));
        assert!(buttons.is_pressed(button2));
        assert!(buttons.is_pressed(button3));
    }
}
