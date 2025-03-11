use raylib_wasm::PINK;

use crate::bullet::Bullet;
use crate::vec2::Vector2;
// use crate::vec2::Vector2Ext;

use crate::entity_manager::{EntityId, HasId};
use crate::webhacks;
use crate::State;

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
pub struct TurretUpdate {
    pub id: EntityId, // to match up with the turret
    pub dead: bool,
    pub fire_cooldown: f32,
    pub hover: bool,
}

impl From<&Turret> for TurretUpdate {
    fn from(turret: &Turret) -> Self {
        TurretUpdate {
            id: turret.id,
            dead: turret.dead.into(),
            fire_cooldown: turret.fire_cooldown,
            hover: turret.hover.into(),
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

    pub fn update(&self, state: &State) -> (TurretUpdate, Option<Bullet>) {
        let mouse_pos = state.mouse_pos;
        let mouse_btn_pressed = state.mouse_btn_pressed;
        let dt = state.dt();
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
        let bullet = if update.fire_cooldown <= 0.0 {
            update.fire_cooldown = FIRE_COOLDOWN;
            // self.fire(&mut update);
            Some(self.fire(&update))
        } else {
            None
        };

        (update, bullet)
    }

    pub fn apply(&mut self, update: &TurretUpdate) {
        debug_assert_eq!(self.id, update.id);
        self.dead = update.dead.into();
        self.fire_cooldown = update.fire_cooldown;
        self.hover = update.hover.into();
    }

    fn fire(&self, _update: &TurretUpdate) -> Bullet {
        Bullet::new(self.position, Some(self), None)
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
