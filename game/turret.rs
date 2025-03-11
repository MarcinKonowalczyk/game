use raylib_wasm::PINK;

use crate::bullet::Bullet;
use crate::vec2::Vector2;
// use crate::vec2::Vector2Ext;

use crate::entity_manager::{EntityId, HasId};
use crate::webhacks;
use crate::State;
use std::cell::RefCell;
use std::str;

use crate::u32_bool::Bool;

use crate::ACTIVE_RADIUS;
use crate::ALPHA_BEIGE;
use crate::TURRET_RADIUS;

const FIRE_COOLDOWN: f32 = 0.5; // seconds

#[derive(Clone, Debug)]
pub struct Turret {
    pub position: Vector2,
    pub dead: Bool,
    pub hover: Bool,
    pub fire_cooldown: f32,
    pub id: EntityId,
}

#[inline]
fn min_f32(a: f32, b: f32) -> f32 {
    if a < b {
        a
    } else {
        b
    }
}

pub struct TurretUpdate {
    pub id: EntityId, // to match up with the turret
    pub dead: bool,
    pub fire_cooldown: f32,
    pub hover: bool,
    pub new_bullet: Option<Bullet>,
}

impl From<&Turret> for TurretUpdate {
    fn from(turret: &Turret) -> Self {
        TurretUpdate {
            id: turret.id,
            dead: turret.dead.into(),
            fire_cooldown: turret.fire_cooldown,
            hover: turret.hover.into(),
            new_bullet: None,
        }
    }
}

impl Turret {
    pub fn new(position: Vector2) -> Turret {
        Turret {
            position,
            dead: false.into(),
            hover: false.into(),
            fire_cooldown: FIRE_COOLDOWN,
            id: 0,
        }
    }

    pub fn update(&self, state: &RefCell<State>) -> TurretUpdate {
        let mouse_pos = { state.borrow().mouse_pos };
        let mouse_btn_pressed = { state.borrow().mouse_btn_pressed };
        let dt = { state.borrow().dt() };
        let mouse_distance = self.position.dist(&mouse_pos);

        let mut update = TurretUpdate::from(self);

        if mouse_distance < TURRET_RADIUS {
            update.hover = true;
        } else if mouse_distance < 1.5 * TURRET_RADIUS {
            // no change
        } else {
            update.hover = false;
        }

        if update.hover && mouse_btn_pressed.into() {
            // despawn the turret
            update.dead = true;
        }

        update.fire_cooldown -= dt;
        if update.fire_cooldown <= 0.0 {
            update.fire_cooldown = FIRE_COOLDOWN;
            self.fire(&mut update);
        }

        update
    }

    pub fn apply(&mut self, update: &TurretUpdate) {
        self.dead = update.dead.into();
        self.fire_cooldown = update.fire_cooldown;
        self.hover = update.hover.into();
    }

    fn fire(&self, update: &mut TurretUpdate) {
        let new_bullet = Bullet::new(self.position, Some(self), None);
        update.new_bullet = Some(new_bullet);
    }

    pub fn draw_background(&self, _index: usize, _state: &RefCell<State>) {
        webhacks::draw_circle(self.position, ACTIVE_RADIUS, ALPHA_BEIGE);
    }

    pub fn draw_foreground(&self, _index: usize, _state: &RefCell<State>) {
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
