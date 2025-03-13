use raylib_wasm::GREEN;
// use raylib_wasm::PINK;

use crate::entity_manager::{EntityId, HasId, NO_ID};
use crate::vec2::Vector2;

use crate::u32_bool::Bool;
use crate::State;
use crate::{webhacks, SPEED_BULLET, WINDOW_WIDTH};

// use crate::ACTIVE_RADIUS;
// use crate::ALPHA_BEIGE;
// use crate::TURRET_RADIUS;
use crate::WINDOW_HEIGHT;

pub struct BulletUpdate {
    pub id: EntityId,
    pub dead: bool,
    pub position: Vector2,
    pub velocity: Vector2,
    pub hit_request: Option<HitRequest>,
}

impl From<&Bullet> for BulletUpdate {
    fn from(bullet: &Bullet) -> Self {
        BulletUpdate {
            id: bullet.id,
            dead: bullet.dead.into(),
            position: bullet.position,
            velocity: bullet.velocity,
            hit_request: None,
        }
    }
}

pub struct HitRequest {
    pub target: EntityId,
    pub damage: u32,
}

impl From<&Bullet> for HitRequest {
    fn from(bullet: &Bullet) -> Self {
        HitRequest {
            target: bullet.target,
            damage: bullet.damage,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Bullet {
    pub position: Vector2,
    pub velocity: Vector2,
    pub source: EntityId,
    pub target: EntityId,
    pub dead: Bool,
    pub id: EntityId,
    pub damage: u32,
}

impl Bullet {
    pub fn new(position: Vector2, source: EntityId, target: Option<EntityId>) -> Bullet {
        Bullet {
            position,
            velocity: Vector2::zero(),
            source: source,
            target: target.unwrap_or(NO_ID),
            dead: false.into(),
            damage: 1,
            id: NO_ID,
        }
    }

    pub fn update(&self, state: &State) -> BulletUpdate {
        let dt = state.dt();

        let mut update = BulletUpdate::from(self);

        let target: Option<(Vector2, f32)> = state.man.get_enemy(self.target).map(|enemy| {
            let direction = enemy.position - self.position;
            (direction, enemy.radius)
        });

        match target {
            Some((direction, radius)) => {
                let velocity = direction.normalize() * SPEED_BULLET;
                update.velocity = velocity;
                update.position += velocity * dt;

                if direction.mag() < radius {
                    update.hit_request = Some(HitRequest::from(self));
                    update.dead = true;
                }
            }
            None => {
                update.dead = true;
            }
        }

        // despawn if off screen
        if update.position.y > WINDOW_HEIGHT as f32
            || update.position.y < 0.0
            || update.position.x > WINDOW_WIDTH as f32
            || update.position.x < 0.0
        {
            update.dead = true;
        }
        update
    }

    pub fn apply(&mut self, update: &BulletUpdate) {
        debug_assert_eq!(self.id, update.id);
        self.position = update.position;
        self.velocity = update.velocity;
        self.dead = update.dead.into();
    }

    pub fn draw_background(&self, state: &State) {
        match state.man.get_enemy(self.target) {
            Some(target) => {
                webhacks::draw_line_ex(self.position, target.position, 2.0, GREEN);
            }
            None => {}
        }
        // webhacks::draw_circle(self.position, ACTIVE_RADIUS, ALPHA_BEIGE);
    }

    pub fn draw_foreground(&self, _state: &State) {
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
