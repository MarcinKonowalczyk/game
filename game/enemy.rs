use std::cell::RefCell;
use std::sync::Arc;

use crate::vec2::Vector2;
// use crate::vec2::Vector2Ext;
use crate::u32_bool::Bool;
use crate::webhacks;

use raylib_wasm::{PINK, RAYWHITE};

use crate::entity_manager::{EntityId, HasId};
use crate::State;
use crate::ACTIVE_RADIUS;
use crate::SPEED_ENEMY;

fn path_pos_to_screen_pos(path_pos: f32, path: &[Vector2]) -> Vector2 {
    // walk along the path until we reach the correct position
    let mut current_path_length = 0.0;
    for i in 1..path.len() {
        let mut p1 = path[i - 1].clone();
        let p2 = path[i];
        let segment_length = p1.dist(&p2);
        if current_path_length + segment_length >= path_pos {
            let segment_pos = (path_pos - current_path_length) / segment_length;
            return *p1.lerp(&p2, segment_pos);
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
    pub position: f32,
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
    pub position: f32, // position along the path in pixels
    pub health: f32,

    #[allow(unused)]
    pub max_health: f32,

    pub spawn_time: f32,

    #[allow(unused)]
    pub last_hit_time: f32,

    pub dead: Bool,
    pub id: EntityId,
}

impl Enemy {
    pub fn new(time: f32) -> Enemy {
        Enemy {
            position: 0.0,
            health: 100.0,
            max_health: 100.0,
            spawn_time: time,
            last_hit_time: -1.0,
            dead: false.into(),
            id: 0,
        }
    }

    pub fn update(&self, state: &State) -> EnemyUpdate {
        let path_length = { state.path_length };

        let mut update = EnemyUpdate::from(self);
        update.position += SPEED_ENEMY * state.dt();

        if update.position >= path_length {
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

    pub fn draw_background(&self, _index: usize, _state: &State) {
        // let path = state.get_path();
        // let pos = self.screen_position(path);
        // webhacks::draw_circle(pos, ACTIVE_RADIUS, ALPHA_BEIGE);
    }

    pub fn draw_foreground(&self, _index: usize, state: &State) {
        let path = state.get_path();
        let pos = self.screen_position(path);
        // let distances = state.get_distances();

        //
        let distance = self
            .screen_position(state.get_path())
            .dist(&state.mouse_pos);
        let color = if distance < ACTIVE_RADIUS {
            PINK
        } else {
            RAYWHITE
        };
        webhacks::draw_circle(pos, 10.0, color);
    }

    pub fn screen_position(&self, path: &[Vector2]) -> Vector2 {
        path_pos_to_screen_pos(self.position, path)
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
