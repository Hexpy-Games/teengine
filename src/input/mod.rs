use glutin::event::VirtualKeyCode;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeyState {
    Pressed,
    Held,
    Released,
    Idle,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputAction {
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    Jump,
    Action,
}

pub struct InputManager {
    key_states: HashMap<VirtualKeyCode, KeyState>,
    action_mappings: HashMap<InputAction, Vec<VirtualKeyCode>>,
    previous_keys: HashMap<VirtualKeyCode, bool>,
}

impl InputManager {
    pub fn new() -> Self {
        let mut action_mappings = HashMap::new();

        // Basic key mappings
        action_mappings.insert(
            InputAction::MoveLeft,
            vec![VirtualKeyCode::A, VirtualKeyCode::Left],
        );
        action_mappings.insert(
            InputAction::MoveRight,
            vec![VirtualKeyCode::D, VirtualKeyCode::Right],
        );
        action_mappings.insert(
            InputAction::MoveUp,
            vec![VirtualKeyCode::W, VirtualKeyCode::Up],
        );
        action_mappings.insert(
            InputAction::MoveDown,
            vec![VirtualKeyCode::S, VirtualKeyCode::Down],
        );
        action_mappings.insert(InputAction::Jump, vec![VirtualKeyCode::Space]);
        action_mappings.insert(InputAction::Action, vec![VirtualKeyCode::E]);

        Self {
            key_states: HashMap::new(),
            action_mappings,
            previous_keys: HashMap::new(),
        }
    }

    pub fn update(&mut self) {
        // Update the current state based on the previous frame's key states
        for (key, state) in self.key_states.iter_mut() {
            let was_pressed = *self.previous_keys.get(key).unwrap_or(&false);
            let is_pressed =
                matches!(state, KeyState::Pressed | KeyState::Held);

            *state = match (was_pressed, is_pressed) {
                (true, true) => KeyState::Held,
                (false, true) => KeyState::Pressed,
                (true, false) => KeyState::Released,
                (false, false) => KeyState::Idle,
            };

            self.previous_keys.insert(*key, is_pressed);
        }
    }

    pub fn process_keyboard_input(
        &mut self,
        keycode: VirtualKeyCode,
        pressed: bool,
    ) {
        let current_state =
            self.key_states.entry(keycode).or_insert(KeyState::Idle);
        *current_state = if pressed {
            match *current_state {
                KeyState::Idle | KeyState::Released => KeyState::Pressed,
                KeyState::Pressed | KeyState::Held => KeyState::Held,
            }
        } else {
            match *current_state {
                KeyState::Pressed | KeyState::Held => KeyState::Released,
                KeyState::Released | KeyState::Idle => KeyState::Idle,
            }
        };
    }

    pub fn is_action_active(
        &self,
        action: InputAction,
    ) -> bool {
        if let Some(keys) = self.action_mappings.get(&action) {
            keys.iter().any(|key| {
                matches!(
                    self.key_states.get(key),
                    Some(KeyState::Pressed) | Some(KeyState::Held)
                )
            })
        } else {
            false
        }
    }

    pub fn is_action_just_pressed(
        &self,
        action: InputAction,
    ) -> bool {
        if let Some(keys) = self.action_mappings.get(&action) {
            keys.iter().any(|key| {
                matches!(self.key_states.get(key), Some(KeyState::Pressed))
            })
        } else {
            false
        }
    }

    pub fn is_action_just_released(
        &self,
        action: InputAction,
    ) -> bool {
        if let Some(keys) = self.action_mappings.get(&action) {
            keys.iter().any(|key| {
                matches!(self.key_states.get(key), Some(KeyState::Released))
            })
        } else {
            false
        }
    }

    pub fn remap_action(
        &mut self,
        action: InputAction,
        keys: Vec<VirtualKeyCode>,
    ) {
        self.action_mappings.insert(action, keys);
    }
}
