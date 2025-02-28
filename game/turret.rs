use raylib_wasm::PINK;

use crate::vec2::Vector2;
use crate::vec2::Vector2Ext;

use crate::webhacks;
use crate::State;

use crate::webhacks::Bool;

use crate::ACTIVE_RADIUS;
use crate::ALPHA_BEIGE;
use crate::TURRET_RADIUS;

#[derive(Clone, Debug)]
pub struct Turret {
    pub position: Vector2, // position along the path in pixels
    pub dead: Bool,
    pub hover: Bool,
}

impl Turret {
    pub fn new(position: Vector2) -> Turret {
        Turret {
            position,
            dead: Bool::False(),
            hover: Bool::False(),
        }
    }

    pub fn update(&mut self, state: &State) {
        let mouse_distance = self.position.dist(&state.mouse_pos);
        if mouse_distance < TURRET_RADIUS {
            self.hover = Bool::True();
        } else if mouse_distance < 1.5 * TURRET_RADIUS {
            //
        } else {
            self.hover = Bool::False();
        }
        if self.hover.bool() && state.mouse_btn_pressed.bool() {
            // despawn the turret
            self.dead = Bool::True();
        }
    }

    pub fn draw_background(&self, _index: usize, _state: &State) {
        webhacks::draw_circle(self.position, ACTIVE_RADIUS, ALPHA_BEIGE);
    }

    pub fn draw_foreground(&self, _index: usize, _state: &State) {
        let radius = if self.hover.bool() {
            TURRET_RADIUS * 1.5
        } else {
            TURRET_RADIUS
        };
        webhacks::draw_circle(self.position, radius, PINK);
    }
}
