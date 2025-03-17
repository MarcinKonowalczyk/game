pub type EntityId = u32;

pub const NO_ID: EntityId = 0;

use crate::bullet::Bullet;
use crate::enemy::Enemy;
use crate::turret::Turret;
use crate::vec2::Vector2;
use crate::webhacks;
use std::collections::HashSet;

// #[derive(Clone, Debug)]
pub struct EntityManager {
    pub turrets: Vec<Turret>,
    pub enemies: Vec<Enemy>,
    pub bullets: Vec<Bullet>,
    pub ids: HashSet<EntityId>,
}

use std::fmt::Display;

impl Display for EntityManager {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("EntityManager")
            .field("turrets", &self.turrets)
            .field("enemies", &self.enemies)
            .field("bullets", &self.bullets)
            .finish()
    }
}

fn random_id() -> EntityId {
    webhacks::get_random_value(0, 1000000) as EntityId
}

pub trait HasId {
    fn id(&self) -> EntityId;
    fn set_id(&mut self, id: EntityId);
}

#[derive(Debug)]
pub enum Entity {
    Turret(Turret),
    Enemy(Enemy),
    Bullet(Bullet),
}

impl HasId for Entity {
    fn id(&self) -> EntityId {
        match self {
            Entity::Turret(turret) => turret.id,
            Entity::Enemy(enemy) => enemy.id,
            Entity::Bullet(bullet) => bullet.id,
        }
    }

    fn set_id(&mut self, id: EntityId) {
        match self {
            Entity::Turret(turret) => turret.id = id,
            Entity::Enemy(enemy) => enemy.id = id,
            Entity::Bullet(bullet) => bullet.id = id,
        }
    }
}

impl From<Turret> for Entity {
    fn from(turret: Turret) -> Entity {
        Entity::Turret(turret)
    }
}

impl From<Enemy> for Entity {
    fn from(enemy: Enemy) -> Entity {
        Entity::Enemy(enemy)
    }
}

impl From<Bullet> for Entity {
    fn from(bullet: Bullet) -> Entity {
        Entity::Bullet(bullet)
    }
}

impl<'a> From<&'a Turret> for &'a Entity {
    fn from(turret: &Turret) -> &Entity {
        unsafe { std::mem::transmute(turret) }
    }
}

impl<'a> From<&'a Enemy> for &'a Entity {
    fn from(enemy: &Enemy) -> &Entity {
        unsafe { std::mem::transmute(enemy) }
    }
}

impl EntityManager {
    pub fn new() -> EntityManager {
        EntityManager {
            // turrets_n: 0,
            // turrets_arr: std::ptr::null_mut(),
            // enemies_n: 0,
            // enemies_arr: std::ptr::null_mut(),
            // bullets_n: 0,
            // bullets_arr: std::ptr::null_mut(),
            turrets: Vec::new(),
            enemies: Vec::new(),
            bullets: Vec::new(),
            ids: HashSet::new(),
        }
    }

    pub fn to_state(self) -> Box<[u32]> {
        let mut state = Vec::new();

        // let ptr = self.turrets_arr;
        // let len = self.turrets_n;
        let ptr = self.turrets.as_ptr();
        let len = self.turrets.len() as u32;
        let cap = self.turrets.capacity() as u32;
        let size = std::mem::size_of::<Turret>();
        debug_assert!(size % 4 == 0);

        state.push(len);
        state.push(cap);
        if len > 0 {
            state.extend_from_slice(unsafe {
                std::slice::from_raw_parts(ptr as *const u32, len as usize * size / 4)
            });
        }

        // let ptr = self.enemies_arr;
        // let len = self.enemies_n;
        let ptr = self.enemies.as_ptr();
        let len = self.enemies.len() as u32;
        let cap = self.enemies.capacity() as u32;
        let size = std::mem::size_of::<Enemy>();
        debug_assert!(size % 4 == 0);

        state.push(len);
        state.push(cap);
        if len > 0 {
            state.extend_from_slice(unsafe {
                std::slice::from_raw_parts(ptr as *const u32, len as usize * size / 4)
            });
        }

        // let ptr = self.bullets_arr;
        // let len = self.bullets_n;
        let ptr = self.bullets.as_ptr();
        let len = self.bullets.len() as u32;
        let cap = self.bullets.capacity() as u32;
        let size = std::mem::size_of::<Bullet>();
        debug_assert!(size % 4 == 0);

        state.push(len);
        state.push(cap);
        if len > 0 {
            state.extend_from_slice(unsafe {
                std::slice::from_raw_parts(ptr as *const u32, len as usize * size / 4)
            });
        }

        // forget about the Vecs so they don't get dropped
        std::mem::forget(self.turrets);
        std::mem::forget(self.enemies);
        std::mem::forget(self.bullets);

        state.into_boxed_slice()
    }
}

impl EntityManager {
    pub fn from_state(state: &[u32]) -> EntityManager {
        let mut em = EntityManager::new();

        let mut offset = 0;
        let len = state[offset + 0] as usize;
        let cap = state[offset + 1] as usize;
        offset += 2;
        let size = std::mem::size_of::<Turret>();
        let data = &state[offset..offset + len * size / 4];
        offset += len * size / 4;

        let ptr = data.as_ptr() as *mut Turret;
        em.turrets = unsafe { Vec::from_raw_parts(ptr, len, cap) };

        for turret in em.turrets.iter() {
            em.ids.insert(turret.id);
        }

        let len = state[offset + 0] as usize;
        let cap = state[offset + 1] as usize;
        offset += 2;
        let size = std::mem::size_of::<Enemy>();
        let data = &state[offset..offset + len * size / 4];
        offset += len * size / 4;

        let ptr = data.as_ptr() as *mut Enemy;
        em.enemies = unsafe { Vec::from_raw_parts(ptr, len, cap) };

        for enemy in em.enemies.iter() {
            em.ids.insert(enemy.id);
        }

        let len = state[offset + 0] as usize;
        let cap = state[offset + 1] as usize;
        offset += 2;

        let size = std::mem::size_of::<Bullet>();
        let data = &state[offset..offset + len * size / 4];

        // #[allow(unused_assignments)]
        // offset += len * size / 4;

        let ptr = data.as_ptr() as *mut Bullet;
        em.bullets = unsafe { Vec::from_raw_parts(ptr, len, cap) };

        for bullet in em.bullets.iter() {
            em.ids.insert(bullet.id);
        }

        em
    }

    fn gen_id(&self) -> EntityId {
        let mut id = random_id();
        while self.ids.contains(&id) {
            id = random_id();
        }
        id
    }

    pub fn add(&mut self, mut entity: Entity) {
        if entity.id() == 0 {
            let id = self.gen_id();
            entity.set_id(id);
        }
        self.ids.insert(entity.id());
        match entity {
            Entity::Turret(turret) => {
                self.turrets.push(turret);
            }
            Entity::Enemy(enemy) => {
                self.enemies.push(enemy);
            }
            Entity::Bullet(bullet) => {
                self.bullets.push(bullet);
            }
        }
    }

    pub fn filter_dead(&mut self) {
        let dead_turrets: Vec<EntityId> = self
            .turrets
            .iter()
            .filter(|turret| turret.dead.into())
            .map(|turret| turret.id)
            .collect();
        let dead_enemies: Vec<EntityId> = self
            .enemies
            .iter()
            .filter(|enemy| enemy.dead.into())
            .map(|enemy| enemy.id)
            .collect();
        let dead_bullets: Vec<EntityId> = self
            .bullets
            .iter()
            .filter(|bullet| bullet.dead.into())
            .map(|bullet| bullet.id)
            .collect();

        self.turrets.retain(|turret| (!turret.dead).into());
        self.enemies.retain(|enemy| (!enemy.dead).into());
        self.bullets.retain(|bullet| (!bullet.dead).into());

        for id in dead_turrets {
            self.ids.remove(&id);
        }
        for id in dead_enemies {
            self.ids.remove(&id);
        }
        for id in dead_bullets {
            self.ids.remove(&id);
        }
    }

    pub fn closest_enemy(&self, position: Vector2) -> Option<&Enemy> {
        let mut closest = None;
        let mut closest_dist = std::f32::MAX;
        for enemy in self.enemies.iter() {
            let dist = enemy.position.xy.dist(&position);
            if dist < closest_dist {
                closest = Some(enemy.into());
                closest_dist = dist;
            }
        }
        closest
    }

    pub fn get_enemy(&self, id: EntityId) -> Option<&Enemy> {
        for enemy in self.enemies.iter() {
            if enemy.id == id {
                return Some(enemy);
            }
        }
        None
    }

    pub fn get_enemy_mut(&mut self, id: EntityId) -> Option<&mut Enemy> {
        for enemy in self.enemies.iter_mut() {
            if enemy.id == id {
                return Some(enemy);
            }
        }
        None
    }
}
