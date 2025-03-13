use crate::vec2::Vector2;
// use crate::vec2::Vector2Ext;
use crate::u32_bool::Bool;
use crate::webhacks;

use raylib_wasm::{PINK, RAYWHITE, RED};

use crate::anim::Anim;
use crate::entity_manager::{EntityId, HasId};
use crate::State;
use crate::ACTIVE_RADIUS;
use crate::ALPHA_BEIGE;
use crate::SPEED_ENEMY;

fn path_pos_to_screen_pos(path_pos: f32, path: &[Vector2]) -> Vector2 {
    // walk along the path until we reach the correct position
    let mut current_path_length = 0.0;
    for i in 1..path.len() {
        let p1 = path[i - 1];
        let p2 = path[i];
        let segment_length = p1.dist(&p2);
        if current_path_length + segment_length >= path_pos {
            let segment_pos = (path_pos - current_path_length) / segment_length;
            return p1.lerp(&p2, segment_pos);
        }
        current_path_length += segment_length;
    }

    path[path.len() - 1]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_pos_to_screen_pos() {
        let path = vec![
            Vector2::new(0.0, 0.0),
            Vector2::new(0.0, 100.0),
            Vector2::new(100.0, 100.0),
        ];
        let pos = path_pos_to_screen_pos(0.0, &path);
        assert_eq!(pos, Vector2::new(0.0, 0.0));
        let pos = path_pos_to_screen_pos(50.0, &path);
        assert_eq!(pos, Vector2::new(0.0, 50.0));
        let pos = path_pos_to_screen_pos(150.0, &path);
        assert_eq!(pos, Vector2::new(50.0, 100.0));
    }
}

pub struct EnemyUpdate {
    pub id: EntityId,
    pub position: Vector2,
    pub path_position: f32,
    pub dead: bool,
    pub damage_done: u32,
}

impl From<&Enemy> for EnemyUpdate {
    fn from(enemy: &Enemy) -> Self {
        Self {
            id: enemy.id,
            position: enemy.position,
            path_position: enemy.path_position,
            dead: enemy.dead.into(),
            damage_done: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Enemy {
    pub position: Vector2,
    pub path_position: f32, // position along the path in pixels
    pub health: u32,

    pub spawn_time: f32,

    pub dead: Bool,
    pub id: EntityId,

    pub radius: f32,

    pub anim: Option<Anim>,
}

impl Enemy {
    pub fn new(time: f32) -> Enemy {
        Enemy {
            position: Vector2::zero(), // todo!
            path_position: 0.0,
            health: 3,
            spawn_time: time,
            dead: false.into(),
            id: 0,
            radius: 10.0,
            anim: None,
        }
    }

    pub fn update(&self, state: &State) -> EnemyUpdate {
        let path_length = { state.path_length };

        let mut update = EnemyUpdate::from(self);
        update.path_position += SPEED_ENEMY * state.dt();

        if update.path_position >= path_length {
            update.dead = true;
            update.damage_done += 1;
        }

        // Figure out the position along the path
        let path = state.get_path();
        update.position = path_pos_to_screen_pos(update.path_position, path);
        // update.position.add(&Vector2::new(20.0, -20.0));

        update
    }

    pub fn apply(&mut self, update: &EnemyUpdate) {
        debug_assert_eq!(self.id, update.id);
        self.dead = update.dead.into();
        self.path_position = update.path_position;
        self.position = update.position;
    }

    pub fn draw_background(&self, state: &State) {
        webhacks::draw_circle(self.position, self.radius, RAYWHITE);
        webhacks::draw_circle(self.position, ACTIVE_RADIUS, ALPHA_BEIGE);

        // draw health bar
        let width = self.radius * 2.0 * 1.5;
        let pos = self.position + Vector2::new(-width / 2.0, -(self.radius * 1.5));
        let width = width * (self.health as f32 / 3.0);
        webhacks::draw_line_ex(pos, pos + Vector2::new(width, 0.0), 5.0, RED);

        // draw debug dot at the position along the path
        let path = state.get_path();
        let path_screen_position = path_pos_to_screen_pos(self.path_position, path);
        webhacks::draw_circle(path_screen_position, 0.5, RED);
    }

    pub fn draw_foreground(&self, state: &State) {
        match self.anim {
            Some(ref anim) => {
                // anim.draw(self.position, state.curr_time);
                let scale = (2.0 * self.radius) / (anim.meta.avg_width).max(anim.meta.avg_height);
                anim.draw(
                    self.position,
                    scale,
                    crate::anim::Anchor::Center,
                    state.curr_time,
                );
            }
            None => {
                let distance = self.position.dist(&state.mouse_pos);
                let color = if distance < ACTIVE_RADIUS {
                    PINK
                } else {
                    RAYWHITE
                };
                webhacks::draw_circle(self.position, self.radius, color);
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

// impl HasKind for Enemy {
//     fn kind(&self) -> EntityKind {
//         EntityKind::Enemy
//     }
// }

// use std::default::Default;

// impl Default for Enemy {
//     fn default() -> Self {
//         Enemy::new(0.0)
//     }
// }
