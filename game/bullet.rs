use raylib_wasm::GREEN;
use raylib_wasm::PINK;

use crate::entity_manager::{EntityId, HasId, NO_ID};
use crate::vec2::Vector2;

use crate::enemy::Enemy;
use crate::turret::Turret;
use crate::u32_bool::Bool;
use crate::webhacks;
use crate::State;
use std::cell::RefCell;

use crate::ACTIVE_RADIUS;
use crate::ALPHA_BEIGE;
use crate::TURRET_RADIUS;
use crate::WINDOW_HEIGHT;

pub struct BulletUpdate {
    pub id: EntityId,
    pub dead: bool,
    pub position: Vector2,
}

impl From<&Bullet> for BulletUpdate {
    fn from(bullet: &Bullet) -> Self {
        BulletUpdate {
            id: bullet.id,
            dead: bullet.dead.into(),
            position: bullet.position,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Bullet {
    pub position: Vector2,
    pub source: EntityId,
    pub target: EntityId,
    pub dead: Bool,
    pub id: EntityId,
}

impl Bullet {
    pub fn new(position: Vector2, source: Option<&Turret>, target: Option<&Enemy>) -> Bullet {
        Bullet {
            position,
            source: match source {
                Some(turret) => turret.id,
                None => 0,
            },
            target: match target {
                Some(enemy) => enemy.id,
                None => 0,
            },
            dead: false.into(),
            id: NO_ID,
        }
    }

    pub fn update(&self, state: &State) -> BulletUpdate {
        let dt = state.dt();

        let mut update = BulletUpdate::from(self);

        // for now just move vertically down the screen
        update.position.y += 200.0 * dt;

        // despawn if off screen
        if update.position.y > WINDOW_HEIGHT as f32
            || update.position.y < 0.0
            || update.position.x > WINDOW_HEIGHT as f32
            || update.position.x < 0.0
        {
            update.dead = true;
        }
        update
    }

    pub fn apply(&mut self, update: &BulletUpdate) {
        debug_assert_eq!(self.id, update.id);
        self.position = update.position;
        self.dead = update.dead.into();
    }

    pub fn draw_background(&self, _index: usize, _state: &State) {
        // webhacks::draw_circle(self.position, ACTIVE_RADIUS, ALPHA_BEIGE);
    }

    pub fn draw_foreground(&self, _index: usize, _state: &State) {
        // let radius = if self.hover.into() {
        //     TURRET_RADIUS * 1.5
        // } else {
        //     TURRET_RADIUS
        // };
        webhacks::draw_circle(self.position, 5.0, GREEN);
    }
}

impl HasId for Bullet {
    fn id(&self) -> EntityId {
        self.id
    }

    fn set_id(&mut self, id: EntityId) {
        self.id = id;
    }
}
