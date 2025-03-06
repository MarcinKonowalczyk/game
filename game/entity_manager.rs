pub type EntityId = u32;

use crate::enemy::Enemy;
use crate::turret::Turret;
use crate::webhacks;
use std::collections::HashSet;

// #[derive(Clone, Debug)]
pub struct EntityManager {
    // turrets: Vec<Turret>,
    // enemies: Vec<Enemy>,
    turrets_n: u32,
    turrets_arr: *mut Turret,
    enemies_n: u32,
    enemies_arr: *mut Enemy,
    ids: HashSet<EntityId>,
}

use std::fmt::Display;

impl Display for EntityManager {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("EntityManager")
            .field("turrets_n", &self.turrets_n)
            .field("turrets_arr", &self.turrets_arr)
            .field("enemies_n", &self.enemies_n)
            .field("enemies_arr", &self.enemies_arr)
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

// pub trait HasKind {
//     fn kind(&self) -> EntityKind;
// }

pub enum Entity {
    Turret(Turret),
    Enemy(Enemy),
    #[allow(unused)]
    None,
}

impl HasId for Entity {
    fn id(&self) -> EntityId {
        match self {
            Entity::Turret(turret) => turret.id,
            Entity::Enemy(enemy) => enemy.id,
            Entity::None => 0,
        }
    }

    fn set_id(&mut self, id: EntityId) {
        match self {
            Entity::Turret(turret) => turret.id = id,
            Entity::Enemy(enemy) => enemy.id = id,
            Entity::None => {}
        }
    }
}

// pub enum EntityKind {
//     Turret,
//     Enemy,
// }

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
// impl From<&Turret> for EntityKind {
//     fn from(_turret: &Turret) -> EntityKind {
//         EntityKind::Turret
//     }
// }

// impl From<&Enemy> for EntityKind {
//     fn from(_enemy: &Enemy) -> EntityKind {
//         EntityKind::Enemy
//     }
// }

impl EntityManager {
    pub fn new() -> EntityManager {
        EntityManager {
            turrets_n: 0,
            turrets_arr: std::ptr::null_mut(),
            enemies_n: 0,
            enemies_arr: std::ptr::null_mut(),
            ids: HashSet::new(),
        }
    }

    pub fn to_state(self) -> Box<[u32]> {
        let mut state = Vec::new();

        let ptr = self.turrets_arr;
        let len = self.turrets_n;
        let size = std::mem::size_of::<Turret>();
        debug_assert!(size % 4 == 0);

        state.push(len);
        if len > 0 {
            state.extend_from_slice(unsafe {
                std::slice::from_raw_parts(ptr as *const u32, len as usize * size / 4)
            });
        }

        let ptr = self.enemies_arr;
        let len = self.enemies_n;
        let size = std::mem::size_of::<Enemy>();
        debug_assert!(size % 4 == 0);

        state.push(len);
        if len > 0 {
            state.extend_from_slice(unsafe {
                std::slice::from_raw_parts(ptr as *const u32, len as usize * size / 4)
            });
        }

        // unsafe {
        //     let state_u8 = std::slice::from_raw_parts(state.as_ptr() as *const u8, state.len() * 4);
        //     let turret_1 = self.turrets_arr;
        //     let turret_1_u8 = std::slice::from_raw_parts(
        //         turret_1 as *const Turret as *const u8,
        //         std::mem::size_of::<Turret>(),
        //     );
        //     let turret_2 = self.turrets_arr.add(1);
        //     let turret_2_u8 = std::slice::from_raw_parts(
        //         turret_2 as *const Turret as *const u8,
        //         std::mem::size_of::<Turret>(),
        //     );

        //     let turret_3_u8 = if self.turrets_n > 2 {
        //         let turret_3 = self.turrets_arr.add(2);
        //         Some(std::slice::from_raw_parts(
        //             turret_3 as *const Turret as *const u8,
        //             std::mem::size_of::<Turret>(),
        //         ))
        //     } else {
        //         None
        //     };

        //     let turret_4_u8 = if self.turrets_n > 3 {
        //         let turret_4 = self.turrets_arr.add(3);
        //         Some(std::slice::from_raw_parts(
        //             turret_4 as *const Turret as *const u8,
        //             std::mem::size_of::<Turret>(),
        //         ))
        //     } else {
        //         None
        //     };

        //     println!("> to_state");
        //     println!("  state_u8: {:?}", state_u8);
        //     println!("  turret_1_u8: .........{:?}", turret_1_u8);
        //     println!(
        //         "  turret_2_u8: ...........................................................................{:?}",
        //         turret_2_u8
        //     );
        //     match turret_3_u8 {
        //         Some(turret_3_u8) => {
        //             println!(
        //                 "  turret_3_u8: ..............................................................................................................................................{:?}",
        //                 turret_3_u8
        //             );
        //         }
        //         _ => {}
        //     }
        //     match turret_4_u8 {
        //         Some(turret_4_u8) => {
        //             println!(
        //                 "  turret_4_u8: ..................................................................................................................................................................................................................{:?}",
        //                 turret_4_u8
        //             );
        //         }
        //         _ => {}
        //     }
        // }

        state.into_boxed_slice()
    }

    pub fn from_state(state: &[u32]) -> EntityManager {
        let mut em = EntityManager::new();

        let mut offset = 0;
        let turrets_len = state[0] as usize;
        offset += 1;
        let turret_size = std::mem::size_of::<Turret>();
        let turrets_data = &state[offset..offset + turrets_len * turret_size / 4];
        offset += turrets_len * turret_size / 4;

        let ptr = turrets_data.as_ptr() as *mut Turret;
        let turrets = unsafe { std::slice::from_raw_parts(ptr, turrets_len) };

        // unsafe {
        //     let turret_data_u8 = std::slice::from_raw_parts(
        //         turrets_data.as_ptr() as *const u8,
        //         turrets_data.len() * 4,
        //     );
        //     println!("turret_data_u8: ......{:?}", turret_data_u8);
        //     println!("turrets_len: {}, turrets: {:?}", turrets_len, turrets);
        // }

        // em.turrets = turrets.as_slice();
        em.turrets_n = turrets_len as u32;
        em.turrets_arr = turrets.as_ptr() as *mut Turret;

        for turret in turrets {
            em.ids.insert(turret.id);
        }

        let enemies_len = state[offset] as usize;
        offset += 1;
        let enemy_size = std::mem::size_of::<Enemy>();

        // println!("enemies_len: {}", enemies_len);

        let enemies_data = &state[offset..offset + enemies_len * enemy_size / 4];
        // offset += enemies_len * enemy_size / 4;

        let ptr = enemies_data.as_ptr() as *mut Enemy;
        let enemies = unsafe { std::slice::from_raw_parts(ptr, enemies_len) };

        // unsafe {
        //     let enemy_data_u8 = std::slice::from_raw_parts(
        //         enemies_data.as_ptr() as *const u8,
        //         enemies_data.len() * 4,
        //     );
        //     println!("enemy_data_u8: ......{:?}", enemy_data_u8);
        //     println!(
        //         "enemies_len: {}, enemies_data: {:?}, enemies: {:?}",
        //         enemies_len, enemies_data, enemies
        //     );
        // }

        // em.enemies = enemies.as_slice();
        em.enemies_n = enemies_len as u32;
        em.enemies_arr = enemies.as_ptr() as *mut Enemy;

        for enemy in enemies {
            em.ids.insert(enemy.id);
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
        match entity {
            Entity::Turret(turret) => {
                let mut turrets = match self.turrets_n {
                    0 => Vec::new(),
                    _ => unsafe {
                        std::slice::from_raw_parts(self.turrets_arr, self.turrets_n as usize)
                            .to_vec()
                    },
                };
                turrets.push(turret);
                self.turrets_n = turrets.len() as u32;
                self.turrets_arr = turrets.as_ptr() as *mut Turret;
                std::mem::forget(turrets); // prevent drop
            }
            Entity::Enemy(enemy) => {
                let mut enemies = match self.enemies_n {
                    0 => Vec::new(),
                    _ => unsafe {
                        std::slice::from_raw_parts(self.enemies_arr, self.enemies_n as usize)
                            .to_vec()
                    },
                };
                enemies.push(enemy);
                self.enemies_n = enemies.len() as u32;
                self.enemies_arr = enemies.as_ptr() as *mut Enemy;
                std::mem::forget(enemies); // prevent drop
            }
            Entity::None => {}
        }
    }

    // pub fn filter(&mut self, f: impl Fn(&Entity) -> bool) {
    //     let turrets = self.turrets_mut().unwrap().to_vec();

    //     let new_turrets = turrets
    //         .into_iter()
    //         .filter(|turret| f(&Entity::Turret(turret.clone())))
    //         .collect::<Vec<Turret>>();
    //     let new_turrets = std::mem::ManuallyDrop::new(new_turrets);
    //     self.turrets_n = new_turrets.len() as u32;
    //     self.turrets_arr = new_turrets.as_ptr() as *mut Turret;

    //     let enemies = self.enemies_mut().unwrap().to_vec();
    //     let new_enemies = enemies
    //         .into_iter()
    //         .filter(|enemy| f(&Entity::Enemy(enemy.clone())))
    //         .collect::<Vec<Enemy>>();
    //     let new_enemies = std::mem::ManuallyDrop::new(new_enemies);
    //     self.enemies_n = new_enemies.len() as u32;
    //     self.enemies_arr = new_enemies.as_ptr() as *mut Enemy;
    // }

    pub fn filter_dead(&mut self) {
        let turrets = self.turrets_mut().unwrap().to_vec();
        let new_turrets = turrets
            .into_iter()
            .filter(|turret| (!turret.dead).into())
            .collect::<Vec<Turret>>();
        let new_turrets = std::mem::ManuallyDrop::new(new_turrets);
        self.turrets_n = new_turrets.len() as u32;
        self.turrets_arr = new_turrets.as_ptr() as *mut Turret;

        let enemies = self.enemies_mut().unwrap().to_vec();
        let new_enemies = enemies
            .into_iter()
            .filter(|enemy| (!enemy.dead).into())
            .collect::<Vec<Enemy>>();
        let new_enemies = std::mem::ManuallyDrop::new(new_enemies);
        self.enemies_n = new_enemies.len() as u32;
        self.enemies_arr = new_enemies.as_ptr() as *mut Enemy;
    }

    // pub fn new_default<T>(&mut self) -> T
    // where
    //     T: Default + HasId + HasKind,
    // {
    //     let id = self.gen_id();
    //     let entity = T::default();
    //     self.add_id(id, entity.kind());
    //     entity
    // }

    // pub fn get<'a>(&self, state: &'a State, id: EntityId) -> Entity<'a> {
    //     match state.get_enemies() {
    //         Some(enemies) => {
    //             for enemy in enemies {
    //                 if enemy.id() == id {
    //                     return Entity::Enemy(enemy);
    //                 }
    //             }
    //         }
    //         None => {}
    //     }

    //     match state.get_turrets() {
    //         Some(turrets) => {
    //             for turret in turrets {
    //                 if turret.id() == id {
    //                     return Entity::Turret(turret);
    //                 }
    //             }
    //         }
    //         None => {}
    //     }

    //     Entity::None
    // }

    // pub fn enemies(&'frame self) -> &'frame [Enemy] {
    //     &self.enemies
    // }

    // pub fn turrets(&'frame self) -> &'frame [Turret] {
    //     &self.turrets
    // }

    pub fn turrets(&self) -> Option<&[Turret]> {
        if self.turrets_arr.is_null() {
            None
        } else {
            let slice =
                unsafe { std::slice::from_raw_parts(self.turrets_arr, self.turrets_n as usize) };
            Some(slice)
        }
    }

    // Slice of mutable references to turrets
    pub fn turrets_mut(&mut self) -> Option<&mut [Turret]> {
        if self.turrets_arr.is_null() {
            None
        } else {
            let slice = unsafe {
                std::slice::from_raw_parts_mut(self.turrets_arr, self.turrets_n as usize)
            };
            Some(slice)
        }
    }

    // pub fn set_turrets(&mut self, turrets: &[Turret]) {
    //     free_malloced(self.turrets_n, self.turrets_arr);
    //     let (turrets_n, turrets_arr) = clone_to_malloced(turrets);
    //     self.turrets_n = turrets_n;
    //     self.turrets_arr = turrets_arr;
    // }

    pub fn enemies(&self) -> Option<&[Enemy]> {
        if self.enemies_arr.is_null() {
            None
        } else {
            let slice =
                unsafe { std::slice::from_raw_parts(self.enemies_arr, self.enemies_n as usize) };
            Some(slice)
        }
    }

    // Slice of mutable references to enemies
    pub fn enemies_mut(&mut self) -> Option<&mut [Enemy]> {
        if self.enemies_arr.is_null() {
            None
        } else {
            let slice = unsafe {
                std::slice::from_raw_parts_mut(self.enemies_arr, self.enemies_n as usize)
            };
            Some(slice)
        }
    }

    // pub fn set_enemies(&mut self, enemies: &[Enemy]) {
    //     free_malloced(self.enemies_n, self.enemies_arr);
    //     let (enemies_n, enemies_arr) = clone_to_malloced(&enemies);
    //     self.enemies_n = enemies_n;
    //     self.enemies_arr = enemies_arr;
    // }
}
