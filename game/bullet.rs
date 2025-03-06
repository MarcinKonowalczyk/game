use raylib_wasm::PINK;

use crate::vec2::Vector2;

use crate::enemy::Enemy;
use crate::turret::Turret;
use crate::webhacks;
use crate::State;

use crate::webhacks::Bool;

use crate::ACTIVE_RADIUS;
use crate::ALPHA_BEIGE;
use crate::TURRET_RADIUS;

use std::rc::Weak;

#[derive(Clone, Debug)]
pub struct Bullet {
    pub position: Vector2,
    pub source: Weak<Turret>,
    pub target: Weak<Enemy>,
}

impl Bullet {
    pub fn new(position: Vector2, source: Option<&Turret>, target: Option<&Enemy>) -> Bullet {
        Bullet {
            position,
            source: match source {
                Some(turret) => &turret,
                None => Weak::new(),
            },
        }
    }

    pub fn update(&mut self, state: &State) {
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
        // webhacks::draw_circle(self.position, radius, PINK);
    }
}
