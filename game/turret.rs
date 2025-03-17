use raylib_wasm::PINK;

use crate::vec2::Vector2;
// use crate::vec2::Vector2Ext;

use crate::anim;
use crate::entity_manager::{EntityId, HasId};
use crate::webhacks;
use crate::State;

use crate::u32_bool::Bool;

use crate::ACTIVE_RADIUS;
use crate::ALPHA_BEIGE;

const FIRE_COOLDOWN: f32 = 0.5; // seconds

#[derive(Clone, Debug)]
pub struct Turret {
    pub position: Vector2,
    pub dead: Bool,
    pub hover: Bool,
    pub fire_cooldown: f32,
    pub id: EntityId,
    pub facing: Vector2,
    pub radius: f32,
    pub anim: Option<anim::Anim>,
}
pub struct TurretUpdate {
    pub id: EntityId, // to match up with the turret
    pub dead: bool,
    pub fire_cooldown: f32,
    pub hover: bool,
    pub bullet_request: Option<BulletRequest>,
    pub facing: Vector2,
}

impl From<&Turret> for TurretUpdate {
    fn from(turret: &Turret) -> Self {
        TurretUpdate {
            id: turret.id,
            dead: turret.dead.into(),
            fire_cooldown: turret.fire_cooldown,
            hover: turret.hover.into(),
            bullet_request: None,
            facing: turret.facing,
        }
    }
}

pub struct BulletRequest {
    pub position: Vector2,
    pub source: EntityId,
    pub target: Option<EntityId>,
}

impl Turret {
    pub fn new(position: Vector2) -> Turret {
        Turret {
            position,
            dead: false.into(),
            hover: false.into(),
            fire_cooldown: FIRE_COOLDOWN,
            id: 0,
            facing: Vector2::new(1.0, 0.0), // facing right
            radius: 20.0,
            anim: None,
        }
    }

    pub fn update(&self, state: &State) -> TurretUpdate {
        let mouse_pos = state.mouse_pos;
        let mouse_btn_pressed = state.mouse_btn_pressed;
        let dt = state.dt();
        let mouse_distance = self.position.dist(&mouse_pos);

        let mut update = TurretUpdate::from(self);

        if mouse_distance < self.radius {
            update.hover = true;
        } else if mouse_distance < 1.5 * self.radius {
            // no change
        } else {
            update.hover = false;
        }

        if update.hover && mouse_btn_pressed.into() {
            // despawn the turret
            update.dead = true;
        }

        update.fire_cooldown -= dt;
        if let Some(enemy) = state.man.closest_enemy(self.position) {
            if self.position.dist(&enemy.position.into()) < ACTIVE_RADIUS {
                update.facing = enemy.position.xy - self.position;
                if update.fire_cooldown <= 0.0 {
                    update.bullet_request = Some(BulletRequest {
                        position: self.position,
                        source: self.id,
                        target: Some(enemy.id),
                    });
                    update.fire_cooldown = FIRE_COOLDOWN;
                }
            }
        }

        update
    }

    pub fn apply(&mut self, update: &TurretUpdate) {
        debug_assert_eq!(self.id, update.id);
        self.dead = update.dead.into();
        self.fire_cooldown = update.fire_cooldown;
        self.hover = update.hover.into();
        self.facing = update.facing;
    }

    pub fn draw_debug(&self, _state: &State) {
        webhacks::draw_circle(self.position, ACTIVE_RADIUS, ALPHA_BEIGE);
    }

    pub fn draw_foreground(&self, state: &State) {
        let radius = if self.hover.into() {
            self.radius * 1.5
        } else {
            self.radius
        };
        match self.anim {
            Some(ref anim) => {
                let scale = (2.0 * radius) / (anim.meta.avg_width).max(anim.meta.avg_height);
                let rotation = self.facing.angle();
                anim.draw(
                    self.position,
                    scale,
                    crate::anim::Anchor::Center,
                    rotation,
                    state.curr_time,
                );
            }
            None => {
                webhacks::draw_circle(self.position, radius, PINK);
            }
        }
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
