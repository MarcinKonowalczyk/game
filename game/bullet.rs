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

use std::rc::Weak;

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

    pub fn update(&mut self, state: &RefCell<State>) {
        // for now just move vertically down the screen
        self.position.y += 200.0 * { state.borrow().dt() };
        // let man = state.man();
        // let target = man.get(self.target);

        // match target {
        //     Some(target) => {
        //         // println!("target: {:?}", target);
        //         let target: &Enemy = target.downcast_ref();

        //         // let dir = target.position - self.position;
        //         // let dist = dir.mag();
        //         // let speed = 100.0;
        //         // let dt = state.curr_time - state.prev_time;
        //         // let step = speed * dt;
        //         // if dist < step {
        //         //     self.dead = Bool::True();
        //         //     target.health -= 10.0;
        //         // } else {
        //         //     let dir = dir.normalize();
        //         //     self.position += dir * step;
        //         // }
        //     }
        //     None => {
        //         println!("target is None");
        //         // self.dead = true.into();
        //     }

        // let target = target.unwrap();

        // let mouse_distance = self.position.dist(&state.mouse_pos);
        // if mouse_distance < TURRET_RADIUS {
        //     self.hover = Bool::True();
        // } else if mouse_distance < 1.5 * TURRET_RADIUS {
        //     //
        // } else {
        //     self.hover = Bool::False();
        // }
        // if self.hover.into() && state.mouse_btn_pressed.into() {
        //     // despawn the turret
        //     self.dead = Bool::True();
        // }

        if self.position.y > WINDOW_HEIGHT as f32 {
            self.dead = true.into();
        }
    }

    pub fn draw_background(&self, _index: usize, _state: &RefCell<State>) {
        // webhacks::draw_circle(self.position, ACTIVE_RADIUS, ALPHA_BEIGE);
    }

    pub fn draw_foreground(&self, _index: usize, _state: &RefCell<State>) {
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
