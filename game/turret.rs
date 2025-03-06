use raylib_wasm::PINK;

use crate::vec2::Vector2;
// use crate::vec2::Vector2Ext;

use crate::entity_manager::{EntityId, HasId};
use crate::webhacks;
use crate::State;

use crate::u32_bool::Bool;

use crate::ACTIVE_RADIUS;
use crate::ALPHA_BEIGE;
use crate::TURRET_RADIUS;

#[derive(Clone, Debug)]
pub struct Turret {
    pub position: Vector2,
    pub dead: Bool,
    pub hover: Bool,
    pub id: EntityId,
}

impl Turret {
    pub fn new(position: Vector2) -> Turret {
        Turret {
            position,
            dead: false.into(),
            hover: false.into(),
            id: 0,
        }
    }

    pub fn update(&mut self, state: &State) {
        let mouse_distance = self.position.dist(&state.mouse_pos);
        if mouse_distance < TURRET_RADIUS {
            self.hover = true.into();
        } else if mouse_distance < 1.5 * TURRET_RADIUS {
            //
        } else {
            self.hover = false.into();
        }
        if self.hover.into() && state.mouse_btn_pressed.into() {
            // despawn the turret
            self.dead = true.into();
        }
    }

    pub fn draw_background(&self, _index: usize, _state: &State) {
        webhacks::draw_circle(self.position, ACTIVE_RADIUS, ALPHA_BEIGE);
    }

    pub fn draw_foreground(&self, _index: usize, _state: &State) {
        let radius = if self.hover.into() {
            TURRET_RADIUS * 1.5
        } else {
            TURRET_RADIUS
        };
        webhacks::draw_circle(self.position, radius, PINK);
    }
}

impl HasId for Turret {
    fn id(&self) -> EntityId {
        self.id
    }

    fn set_id(&mut self, id: EntityId) {
        self.id = id;
    }
}

// impl HasKind for Turret {
//     fn kind(&self) -> EntityKind {
//         EntityKind::Turret
//     }
// }

// use std::default::Default;

// impl Default for Turret {
//     fn default() -> Self {
//         Turret::new(Vector2::default())
//     }
// }
