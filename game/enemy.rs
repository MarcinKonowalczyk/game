use crate::vec2::Vector2;
// use crate::vec2::Vector2Ext;
use crate::u32_bool::Bool;
use crate::webhacks;

use raylib_wasm::{PINK, RAYWHITE, RED};

use crate::anim::Anim;
use crate::entity_manager::{EntityId, HasId};
use crate::path::PathPosition;
use crate::State;
use crate::ACTIVE_RADIUS;
// use crate::ALPHA_BEIGE;
use crate::SPEED_ENEMY;

pub struct EnemyUpdate {
    pub id: EntityId,
    pub position: PathPosition,
    pub dead: bool,
    pub damage_done: u32,
}

impl From<&Enemy> for EnemyUpdate {
    fn from(enemy: &Enemy) -> Self {
        Self {
            id: enemy.id,
            position: enemy.position,
            dead: enemy.dead.into(),
            damage_done: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Enemy {
    pub position: PathPosition,
    pub health: u32,

    pub spawn_time: f32,

    pub dead: Bool,
    pub id: EntityId,

    pub radius: f32,

    pub anim: Option<Anim>,
}

impl Enemy {
    pub fn new(position: PathPosition, time: f32) -> Enemy {
        Enemy {
            position: position,
            health: 3,
            spawn_time: time,
            dead: false.into(),
            id: 0,
            radius: 20.0,
            anim: None,
        }
    }

    pub fn update(&self, state: &State) -> EnemyUpdate {
        let mut update = EnemyUpdate::from(self);
        // update.path_position += SPEED_ENEMY * state.dt();
        update
            .position
            .linear_advance(&state.path, SPEED_ENEMY * state.dt());

        if update.position.linear >= state.path.total_length {
            update.dead = true;
            update.damage_done += 1;
        }

        update
    }

    pub fn apply(&mut self, update: &EnemyUpdate) {
        debug_assert_eq!(self.id, update.id);
        self.dead = update.dead.into();
        self.position = update.position;
    }

    pub fn draw_debug(&self, _state: &State) {
        webhacks::draw_circle(self.position.xy, 0.5, RED);
    }

    pub fn draw_foreground(&self, state: &State) {
        // draw health bar
        let width = self.radius * 2.0 * 1.5;
        let pos = self.position.xy + Vector2::new(-width / 2.0, -(self.radius * 1.5));
        let width = width * (self.health as f32 / 3.0);
        webhacks::draw_line_ex(pos, pos + Vector2::new(width, 0.0), 5.0, RED);

        match self.anim {
            Some(ref anim) => {
                // anim.draw(self.position, state.curr_time);
                let scale = (2.0 * self.radius) / (anim.meta.avg_width).max(anim.meta.avg_height);
                anim.draw(
                    self.position.into(),
                    scale,
                    crate::anim::Anchor::Center,
                    0.0,
                    state.curr_time,
                );
            }
            None => {
                let distance = self.position.xy.dist(&state.mouse_pos);
                let color = if distance < ACTIVE_RADIUS {
                    PINK
                } else {
                    RAYWHITE
                };
                webhacks::draw_circle(self.position.into(), self.radius, color);
                // webhacks::draw_circle(self.position, self.radius, RAYWHITE);
            }
        }
    }

    pub fn hit(&mut self, damage: u32) {
        self.health -= damage;
        if self.health <= 0 {
            self.dead = true.into();
        }
    }
}

impl HasId for Enemy {
    fn id(&self) -> EntityId {
        self.id
    }

    fn set_id(&mut self, id: EntityId) {
        self.id = id;
    }
}
