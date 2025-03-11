use std::cell::RefCell;

use entity_manager::{Entity, EntityManager};
use raylib::{KeyboardKey as KEY, MouseButton, Rectangle, RAYWHITE};
use raylib_wasm::{self as raylib, Color, BLUE};
use u32_bool::Bool;

mod log;

mod anim;
mod bullet;
mod defer;
mod enemy;
mod entity_manager;
mod turret;
mod u32_bool;
mod vec2;
mod webhacks;

use crate::enemy::Enemy;
use crate::turret::Turret;
use crate::vec2::Vector2;
// use crate::vec2::Vector2Ext;

const WINDOW_WIDTH: i32 = 800;
const WINDOW_HEIGHT: i32 = 600;

const SPEED_DEFAULT: f32 = 850.0;
const SPEED_BOOSTED: f32 = 1550.0;

const SPAWN_INTERVAL: f32 = 1.0;
const SPEED_ENEMY: f32 = 340.0;
// const SPEED_ENEMY: f32 = 1340.0;

const TURRET_RADIUS: f32 = 10.0;
const ACTIVE_RADIUS: f32 = 100.0;

const ALPHA_BEIGE: Color = Color {
    r: 211,
    g: 176,
    b: 131,
    a: 100,
};

const ALPHA_BLACK: Color = Color {
    r: 0,
    g: 0,
    b: 0,
    a: 200,
};

#[repr(C, align(4))]
// #[derive(Clone)]
pub struct State {
    pub all_loaded: Bool,
    pub curr_time: f32,
    pub prev_time: f32,
    pub frame_count: u32,
    pub slime_pos: Vector2,
    pub mouse_pos: Vector2,
    pub mouse_btn: Bool,
    pub mouse_btn_pressed: Bool,
    pub music: webhacks::Music,
    pub font: webhacks::Font,
    pub image: webhacks::Image,
    pub texture: webhacks::Texture,
    pub anim_blobs_n: u32,
    pub anim_blobs_arr: *const anim::Blob,
    pub path_n: u32,
    pub path_arr: *const Vector2,
    pub path_length: f32,
    pub mute: Bool,
    pub life: u32,
    // pub man_state_n: u32,
    // pub man_state_arr: *mut u32,
    pub man: RefCell<EntityManager>,
}

// fn state_to_man(state: std::rc::Rc<RefCell<State>>) -> EntityManager {
//     let man_state_n = { state.borrow().man_state_n };
//     let man: EntityManager = match man_state_n {
//         0 => panic!("State has no EntityManager"),
//         _ => {
//             let state_rc = Rc::downgrade(&state);
//             let man = EntityManager::from_state(
//                 unsafe {
//                     std::slice::from_raw_parts(self.man_state_arr, self.man_state_n as usize)
//                 },
//                 state_rc,
//             );
//             man
//         }
//     };
//     man
// }

impl State {
    fn get_path(&self) -> &[Vector2] {
        if self.path_arr.is_null() {
            return &[];
        } else {
            unsafe { std::slice::from_raw_parts(self.path_arr, self.path_n as usize) }
        }
    }

    // fn save_man(&mut self, man: EntityManager) {
    //     let man_state = man.to_state();
    //     let mut man_state = std::mem::ManuallyDrop::new(man_state);
    //     self.man_state_n = man_state.len() as u32;
    //     self.man_state_arr = man_state.as_mut_ptr();
    // }

    fn dt(&self) -> f32 {
        self.curr_time - self.prev_time
    }
}

// pub type StateRef<'a> = std::rc::Rc<RefCell<State<'a>>>;
// pub type StateWeakRef<'a> = std::rc::Weak<RefCell<State<'a>>>;

// trait StateCellExt {
//     fn man(&self) -> EntityManager;
// }

// impl StateCellExt for RefCell<&mut State> {
//     fn man(&self) -> EntityManager {
//         let man_state_n = { self.borrow().man_state_n };
//         let man_state_arr = { self.borrow().man_state_arr };
//         let man: EntityManager = match man_state_n {
//             0 => EntityManager::new(self),
//             _ => EntityManager::from_state(
//                 unsafe { std::slice::from_raw_parts(man_state_arr, man_state_n as usize) },
//                 self,
//             ),
//         };
//         man
//     }
// }

// impl<'a> StateRefExt<'a> for StateRef<'a> {
//     fn man<'b: 'a>(self: &'b StateRef<'a>) -> EntityManager<'b> {
//         let man_state_n = { self.borrow().man_state_n };
//         let man_state_arr = { self.borrow().man_state_arr };
//         let man: EntityManager = match man_state_n {
//             0 => EntityManager::new(self.clone()),
//             _ => EntityManager::from_state(
//                 unsafe { std::slice::from_raw_parts(man_state_arr, man_state_n as usize) },
//                 self.clone(),
//             ),
//         };
//         man
//     }
// }

// fn state_to_man(state: StateRef) -> RefCell<EntityManager> {
//     let man_state_n = { state.borrow().man_state_n };
//     let man_state_arr = { state.borrow().man_state_arr };
//     let state_ref = Rc::downgrade(&state);
//     let man: EntityManager = match man_state_n {
//         0 => EntityManager::new(state_ref),
//         _ => EntityManager::from_state(
//             unsafe { std::slice::from_raw_parts(man_state_arr, man_state_n as usize) },
//             state_ref,
//         ),
//     };
//     man
// }
// statically check that the State struct is the same size as the C struct
// this is important because we're going to be passing this struct back and forth between Rust and C

#[no_mangle]
pub fn get_state_size() -> usize {
    std::mem::size_of::<State>()
}

fn make_path_points() -> (Vec<Vector2>, f32) {
    let w = WINDOW_WIDTH as f32;
    let h = WINDOW_HEIGHT as f32;
    let p = 80.0;
    let d = 150.0;

    let path_points = vec![
        Vector2::new(0.0, p),
        Vector2::new(p, p),
        Vector2::new(p, p + d),
        Vector2::new(w - p - d, p + d),
        Vector2::new(w - p - d, p),
        Vector2::new(w - p, p),
        Vector2::new(w - p, h - p),
        Vector2::new(p + d, h - p),
        Vector2::new(p + d, h - p - d),
        Vector2::new(p, h - p - d),
        Vector2::new(p, h - p),
    ];

    let path_length = path_points
        .iter()
        .fold((0.0, path_points[0]), |(acc, prev), &p| {
            (acc + prev.dist(&p), p)
        })
        .0;

    (path_points, path_length)
}

fn make_initial_turrets(man: &RefCell<EntityManager>) {
    let mut man = man.borrow_mut();
    let t1 = Turret::new(Vector2::new(200.0, 150.0));
    let t2 = Turret::new(Vector2::new(400.0, 150.0));
    man.add(Entity::Turret(t1));
    man.add(Entity::Turret(t2));
}

pub type GameInit = fn() -> State;

#[no_mangle]
pub fn game_init() -> State {
    // We do not cap the framerate, since it leads to sluggish mouse input, since raylib cannot detect mouse input
    // between the frames and we don't really want to dig down to the GLFW layer and poll for events ourselves.
    // See: https://github.com/raysan5/raylib/issues/3354
    // Apparently this is known and solutions are unplanned. I guess it's not that much of a problem from C.
    // SetTargetFPS(300);

    // webhacks::log("game_init".to_string());

    log::set_trace_log_callback();
    log::set_log_level(log::INFO);
    log::trace("game_init");
    log::warning("im a warning");

    raylib::init_window(WINDOW_WIDTH, WINDOW_HEIGHT, "game");

    webhacks::init_audio_device();
    webhacks::set_random_seed(42);

    let music = webhacks::load_music_stream("assets/hello_03.wav");

    // SetMusicVolume(music, 1.0);

    webhacks::play_music_stream(music);

    let font = webhacks::load_font("assets/Kavoon-Regular.ttf");
    let image = webhacks::load_image("assets/Blue_Slime-Idle-mag.png");

    let (path_points, path_length) = make_path_points();
    let (path_n, path_arr) = clone_to_malloced(&path_points);

    // let mut state = State {
    //     all_loaded: false.into(),
    //     curr_time: webhacks::get_time() as f32,
    //     prev_time: 0.0,
    //     frame_count: 99,
    //     slime_pos: Vector2::new(WINDOW_WIDTH as f32 / 2.0, WINDOW_HEIGHT as f32 / 2.0 + 50.0),
    //     mouse_pos: Vector2::new(0.0, 0.0),
    //     mouse_btn: false.into(),
    //     mouse_btn_pressed: false.into(),
    //     music: music,
    //     font: font,
    //     image: image,
    //     texture: webhacks::null_texture(),
    //     anim_blobs_n: 0,
    //     anim_blobs_arr: std::ptr::null(),
    //     path_n: path_n,
    //     path_arr: path_arr,
    //     path_length: path_length,
    //     // enemies_n: 0,
    //     // enemies_arr: std::ptr::null_mut(),
    //     mute: true.into(),
    //     // turrets_n: turrets_n,
    //     // turrets_arr: turrets_arr,
    //     life: 20,
    //     man: Rc::new(RefCell::new(EntityManager::new(
    // };
    let man = RefCell::new(EntityManager::new());

    make_initial_turrets(&man);

    State {
        all_loaded: false.into(),
        curr_time: webhacks::get_time() as f32,
        prev_time: 0.0,
        frame_count: 99,
        slime_pos: Vector2::new(WINDOW_WIDTH as f32 / 2.0, WINDOW_HEIGHT as f32 / 2.0 + 50.0),
        mouse_pos: Vector2::new(0.0, 0.0),
        mouse_btn: false.into(),
        mouse_btn_pressed: false.into(),
        music: music,
        font: font,
        image: image,
        texture: webhacks::null_texture(),
        anim_blobs_n: 0,
        anim_blobs_arr: std::ptr::null(),
        path_n: path_n,
        path_arr: path_arr,
        path_length: path_length,
        mute: true.into(),
        life: 20,
        man: man,
    }
}

// // deref state at the end of the frame
// fn deref_state(state: StateRef) -> State {
//     match Rc::try_unwrap(state) {
//         Ok(state) => state,
//         Err(_) => panic!("state is still referenced"),
//     }
//     .into_inner()
// }

fn clone_to_malloced<T: Clone>(arr: &[T]) -> (u32, *mut T) {
    let n = arr.len().try_into().unwrap();

    if n == 0 {
        return (0, std::ptr::null_mut());
    }

    let mem_size = std::mem::size_of::<T>() * arr.len();
    let layout = std::alloc::Layout::from_size_align(mem_size, 4).unwrap();
    let ptr = unsafe { std::alloc::alloc(layout) as *mut T };

    for (i, item) in arr.iter().enumerate() {
        unsafe {
            *ptr.offset(i as isize) = item.clone();
        }
    }

    (n, ptr)
}

#[allow(unused)]
fn free_malloced<T>(len: u32, ptr: *mut T) {
    if ptr.is_null() {
        return;
    }
    let size = std::mem::size_of::<T>() * len as usize;
    let layout = std::alloc::Layout::from_size_align(size, 4).unwrap();
    let _ptr = ptr as *mut u8;
    unsafe { std::alloc::dealloc(_ptr, layout) }
}

pub type GameLoad = fn(state: *mut State);

#[no_mangle]
pub fn game_load(_state: *mut State) {
    let state = RefCell::new(unsafe { std::ptr::read(_state) });

    {
        let mut state = state.borrow_mut();
        state.prev_time = state.curr_time;
        state.curr_time = webhacks::get_time() as f32;
    }

    {
        let state = state.borrow_mut();
        if state.all_loaded.into() {
            return;
        }
    }

    let mut any_not_loaded = false;

    // check if the music is loaded
    {
        let mut state = state.borrow_mut();
        if !webhacks::is_music_loaded(state.music) {
            any_not_loaded = true;
        }

        // check if the font is loaded
        if !webhacks::is_font_loaded(state.font) {
            any_not_loaded = true;
        }

        // check if the image is loaded
        if !webhacks::is_image_loaded(state.image) {
            any_not_loaded = true;
        } else {
            // image is loaded! let's load the texture
            // state.texture = webhacks::load_texture_from_image(state.image);

            if !webhacks::is_texture_loaded(state.texture) {
                state.texture = webhacks::load_texture_from_image(state.image);
            }

            if !webhacks::is_texture_loaded(state.texture) {
                any_not_loaded = true;
            }
        }
    }

    {
        let mut state = state.borrow_mut();
        if !any_not_loaded {
            state.all_loaded = true.into();

            // Once we've determined that init/load is done, we can unload some resources

            let blobs = anim::find_blobs(state.image);
            let (anim_blobs_n, anim_blobs_arr) = clone_to_malloced(&blobs);

            state.anim_blobs_arr = anim_blobs_arr;
            state.anim_blobs_n = anim_blobs_n;

            webhacks::unload_image(state.image); // we don't need the image anymore

            if state.mute.into() {
                webhacks::set_music_volume(state.music, 0.0);
            } else {
                webhacks::set_music_volume(state.music, 1.0);
            }

            let texture_shape = webhacks::get_texture_shape(state.texture);
            webhacks::log(
                -1,
                format!("texture shape: [{}, {}]", texture_shape.x, texture_shape.y).as_str(),
            );
        }
    }
    println!("end of game_load");

    // wrtie back the state
    unsafe {
        std::ptr::write(_state, state.into_inner());
    }
}

fn time_to_anim_frame(time: f32, frame_duration: f32, n_frames: u32) -> u32 {
    let frame = (time / frame_duration) as u32 % n_frames;
    frame
}

fn handle_keys(state: &RefCell<State>) {
    let speed = unsafe {
        if raylib::IsKeyDown(KEY::Space) {
            SPEED_BOOSTED
        } else {
            SPEED_DEFAULT
        }
    };

    let dt = unsafe { raylib::GetFrameTime() };

    let (w, s, a, d);
    unsafe {
        w = raylib::IsKeyDown(KEY::W);
        s = raylib::IsKeyDown(KEY::S);
        a = raylib::IsKeyDown(KEY::A);
        d = raylib::IsKeyDown(KEY::D);
    }
    {
        let mut state = state.borrow_mut();
        state.slime_pos.y -= dt * speed * (w as i32 as f32);
        state.slime_pos.y += dt * speed * (s as i32 as f32);
        state.slime_pos.x -= dt * speed * (a as i32 as f32);
        state.slime_pos.x += dt * speed * (d as i32 as f32);

        // prevent the rect from wandering off the screen too far
        if state.slime_pos.x < -100.0 {
            state.slime_pos.x = -100.0;
        } else if state.slime_pos.x > WINDOW_WIDTH as f32 {
            state.slime_pos.x = WINDOW_WIDTH as f32;
        }

        if state.slime_pos.y < -100.0 {
            state.slime_pos.y = -100.0;
        } else if state.slime_pos.y > WINDOW_HEIGHT as f32 {
            state.slime_pos.y = WINDOW_HEIGHT as f32;
        }
    }

    {
        let mut state = state.borrow_mut();
        // if raylib::IsKeyPressed(KEY::M) {
        if webhacks::is_key_pressed(KEY::M) {
            state.mute = !state.mute;
            webhacks::set_music_volume(state.music, if state.mute.into() { 0.0 } else { 1.0 });
        }
    }
}

fn handle_mouse(state: &RefCell<State>) {
    let mut mouse_pos = webhacks::get_mouse_position();
    let is_outside = mouse_pos.x < 0.0
        || mouse_pos.y < 0.0
        || mouse_pos.x > WINDOW_WIDTH as f32
        || mouse_pos.y > WINDOW_HEIGHT as f32;
    if is_outside {
        mouse_pos = Vector2::new(-1.0, -1.0);
    }
    let mut state = state.borrow_mut();
    state.mouse_pos = mouse_pos;
    state.mouse_btn = Bool {
        value: webhacks::is_mouse_button_down(MouseButton::Left as i32) as u32,
    };
    state.mouse_btn_pressed = Bool {
        value: webhacks::is_mouse_button_pressed(MouseButton::Left as i32) as u32,
    };
}

fn draw_slime_at_pos(
    position: Vector2,
    anim_blobs: &[anim::Blob],
    texture: webhacks::Texture,
    time: f32,
) {
    let scale = 5.0;
    let i = time_to_anim_frame(time, 0.1, anim_blobs.len() as u32);

    let blob = anim_blobs[i as usize];
    let source = blob.to_rect();
    let width = blob.width() as f32 * scale;
    let height = blob.height() as f32 * scale;
    webhacks::draw_texture_pro(
        texture,
        source,
        Rectangle {
            x: position.x - width / 2.0,
            y: position.y - height,
            width: width,
            height: height,
        },
    );
    // webhacks::draw_circle(position, 5.0, RAYWHITE); // debug circle
}

fn update_entities(state: &RefCell<State>) {
    {
        // let mut man = state_to_man(state);
        let curr_time = { state.borrow().curr_time };

        let last_enemy = {
            let state = state.borrow();
            let man = state.man.borrow();
            man.enemies.last().cloned()
        };

        match last_enemy {
            Some(Enemy {
                spawn_time: last_spawn_time,
                ..
            }) if curr_time - last_spawn_time > SPAWN_INTERVAL => {
                // let state = state.borrow();
                // let mut man = state.man.borrow_mut();
                // man.add(Enemy::new(curr_time).into());
                state
                    .borrow()
                    .man
                    .borrow_mut()
                    .add(Enemy::new(curr_time).into());
            }
            None => {
                // no enemies
                state
                    .borrow()
                    .man
                    .borrow_mut()
                    .add(Enemy::new(curr_time).into());
            }
            _ => {}
        }

        let updates = state
            .borrow()
            .man
            .borrow()
            .enemies
            .iter()
            .map(|enemy| enemy.update(state))
            .collect::<Vec<_>>();

        // Calculate the total damage done by the enemies
        let life_lost = updates.iter().map(|update| update.damage_done).sum::<u32>();

        // Apply the updates
        for (enemy, update) in std::iter::Iterator::zip(
            state.borrow().man.borrow_mut().enemies.iter_mut(),
            updates.iter(),
        ) {
            enemy.apply(update);
        }

        // println!("life_lost: {}", life_lost);
        {
            let mut state = state.borrow_mut();
            state.life -= std::cmp::min(life_lost, state.life);
        }

        // state.save_man(man);
    }
    {
        // Get all the bullet updates
        let updates = state
            .borrow()
            .man
            .borrow()
            .bullets
            .iter()
            .map(|bullet| bullet.update(state))
            .collect::<Vec<_>>();

        // Apply all the updates
        for (bullet, update) in std::iter::Iterator::zip(
            state.borrow().man.borrow_mut().bullets.iter_mut(),
            updates.iter(),
        ) {
            bullet.apply(update);
        }
    }
    {
        // Get all the turret updates
        let updates = state
            .borrow()
            .man
            .borrow()
            .turrets
            .iter()
            .map(|turret| turret.update(state))
            .collect::<Vec<_>>();

        // Check whether any turrets are dead
        let any_dead = updates.iter().any(|update| update.dead.into());

        // Apply all the updates
        for (turret, update) in std::iter::Iterator::zip(
            state.borrow().man.borrow_mut().turrets.iter_mut(),
            updates.iter(),
        ) {
            turret.apply(update);
        }

        // Get all the new bullets
        let new_bullets = updates
            .iter()
            .filter_map(|update| update.new_bullet.clone())
            .collect::<Vec<_>>();

        // Spawn new bullets
        {
            let state = state.borrow();
            let mut man = state.man.borrow_mut();
            for bullet in new_bullets {
                man.add(bullet.into());
            }
        }

        if !any_dead && { state.borrow().mouse_btn_pressed.into() } {
            state
                .borrow()
                .man
                .borrow_mut()
                .add(Turret::new(state.borrow().mouse_pos).into());
        }
    }

    // filter dead entities
    state.borrow().man.borrow_mut().filter_dead();

    // let mut sorted_ids = man.ids.iter().copied().collect::<Vec<_>>();
    // sorted_ids.sort();
    // println!("{:?}", sorted_ids);

    // save the man state
    // let man_state = man.to_state();
    // let mut man_state = std::mem::ManuallyDrop::new(man_state);
    // state.man_state_n = man_state.len() as u32;
    // state.man_state_arr = man_state.as_mut_ptr();

    // state.save_man(man);
}

fn draw_entities_background(state: &RefCell<State>) {
    let state_rc = state.borrow();
    let man = state_rc.man.borrow();

    // draw lines from enemies to turrets if they are within range
    for enemy in man.enemies.iter() {
        for turret in man.turrets.iter() {
            let distance = enemy
                .screen_position(state_rc.get_path())
                .dist(&turret.position);
            if distance < ACTIVE_RADIUS {
                let enemy_pos = enemy.screen_position(state_rc.get_path());
                webhacks::draw_line_ex(enemy_pos, turret.position, 2.0, RAYWHITE);
            }
        }
    }

    // draw line to mouse if it's within range
    for enemy in man.enemies.iter() {
        let distance = enemy
            .screen_position(state_rc.get_path())
            .dist(&state_rc.mouse_pos);
        if distance < ACTIVE_RADIUS {
            let enemy_pos = enemy.screen_position(state_rc.get_path());
            webhacks::draw_line_ex(enemy_pos, state_rc.mouse_pos, 2.0, RAYWHITE);
        }
    }

    for (i, enemy) in man.enemies.iter().enumerate() {
        enemy.draw_background(i, state);
    }
    for (i, turret) in man.turrets.iter().enumerate() {
        turret.draw_background(i, state);
    }

    for (i, bullet) in man.bullets.iter().enumerate() {
        bullet.draw_background(i, state);
    }
}

fn draw_entities_foreground(state: &RefCell<State>) {
    let state_rc = state.borrow();
    let man = state_rc.man.borrow();
    // let enemies = man.enemies;
    // let turrets = man.turrets;
    // let bullets = man.bullets;

    for (i, enemy) in man.enemies.iter().enumerate() {
        enemy.draw_foreground(i, state);
    }
    for (i, turret) in man.turrets.iter().enumerate() {
        turret.draw_foreground(i, state);
    }
    for (i, bullet) in man.bullets.iter().enumerate() {
        bullet.draw_foreground(i, state);
    }
}

fn draw_mouse(_state: &RefCell<State>) {
    // let color = if state.mouse_btn.into() {
    //     RED
    // } else {
    //     RAYWHITE
    // };
    // webhacks::draw_circle(state.mouse_pos, ACTIVE_RADIUS, BEIGE);
    // webhacks::draw_circle(state.mouse_pos, 2.0, color);
}

fn draw_path(state: &RefCell<State>) {
    let state = state.borrow();
    // Draw the path
    let path = unsafe { std::slice::from_raw_parts(state.path_arr, state.path_n as usize) };
    for i in 1..path.len() {
        let p1 = path[i - 1];
        let p2 = path[i];
        webhacks::draw_line_ex(p1, p2, 2.0, RAYWHITE);
        // unsafe { raylib::DrawLineEx(p1, p2, 2.0, RAYWHITE) }
    }
}

fn draw_text(state: &RefCell<State>) {
    let state = state.borrow();
    let slime_pos_text = format! {
        "slime: [{x}, {y}]",
        x = state.slime_pos.x.round(),
        y = state.slime_pos.y.round()
    };
    webhacks::draw_text(
        state.font,
        &slime_pos_text,
        Vector2::new(10.0, 10.0),
        20,
        2.0,
        RAYWHITE,
    );

    let mouse_pos = format! {
        "mouse: [{x}, {y}]",
        x = state.mouse_pos.x.round(),
        y = state.mouse_pos.y.round()
    };
    webhacks::draw_text(
        state.font,
        &mouse_pos,
        Vector2::new(10.0, 30.0),
        20,
        2.0,
        RAYWHITE,
    );

    // Draw the music indicator in the top right corner
    webhacks::draw_text(
        state.font,
        if state.mute.into() {
            "sound: off"
        } else {
            "sound: on"
        },
        Vector2::new(WINDOW_WIDTH as f32 - 105.0, 10.0),
        20,
        2.0,
        RAYWHITE,
    );

    // draw life
    let path = state.get_path();
    let life_text = format!("life: {}", state.life);
    let font_size = 30;
    let text_size = webhacks::measure_text(state.font, &life_text, font_size, 2.0);
    let mut last = path[path.len() - 1].clone();
    let pos = last.add(&Vector2::new(-text_size.x as f32 / 2.0, 20.0));
    webhacks::draw_text(state.font, &life_text, *pos, font_size, 2.0, RAYWHITE);
}

pub type GameFrame = fn(state: *mut State);

#[no_mangle]
pub fn game_frame(_state: *mut State) {
    let state = RefCell::new(unsafe { std::ptr::read(_state) });
    {
        let mut state = state.borrow_mut();
        state.prev_time = state.curr_time;
        state.curr_time = webhacks::get_time() as f32;
    }

    update_entities(&state);

    let game_over = { state.borrow().life == 0 };

    if !game_over {
        handle_keys(&state);
        handle_mouse(&state);
    }

    unsafe { raylib::BeginDrawing() };

    {
        unsafe { raylib::ClearBackground(BLUE) };

        {
            let state = state.borrow();
            let anim_blobs = unsafe {
                std::slice::from_raw_parts(state.anim_blobs_arr, state.anim_blobs_n as usize)
            };
            draw_slime_at_pos(state.slime_pos, anim_blobs, state.texture, state.curr_time);
        }

        draw_text(&state);
        draw_entities_background(&state);
        draw_path(&state);
        draw_entities_foreground(&state);

        draw_mouse(&state);

        if game_over {
            // draw a shaded rectangle over the screen
            unsafe {
                raylib::DrawRectangle(0, 0, WINDOW_WIDTH, WINDOW_HEIGHT, ALPHA_BLACK);
            }
            let state = state.borrow();

            // draw the game over text
            let text = "Game Over!";
            let font_size = 50;
            let text_size = webhacks::measure_text(state.font, text, font_size, 2.0);
            let position = Vector2::new(
                ((WINDOW_WIDTH - text_size.x as i32) / 2) as f32,
                ((WINDOW_HEIGHT - font_size) / 2) as f32,
            );
            webhacks::draw_text(state.font, text, position, font_size, 2.0, RAYWHITE);

            let text = "Press R to restart";
            let font_size = 20;
            let text_size = webhacks::measure_text(state.font, text, font_size, 1.0);
            let position = Vector2::new(
                ((WINDOW_WIDTH - text_size.x as i32) / 2) as f32,
                ((WINDOW_HEIGHT - font_size) / 2 + 50) as f32,
            );
            webhacks::draw_text(state.font, text, position, font_size, 2.0, RAYWHITE);
        }
    }

    unsafe { raylib::EndDrawing() };

    {
        // Update the music stream
        let mut state = state.borrow_mut();
        webhacks::update_music_stream(state.music);

        // Update the frame count
        state.frame_count += 1;
    }

    // Write back the state
    unsafe {
        std::ptr::write(_state, state.into_inner());
    }
}

#[no_mangle]
pub unsafe fn game_over() {
    raylib::CloseWindow();
}

// CAREFUL!
// 1) we need these only from we web version
// 2) if these are called 'malloc' and 'free' they will clash with the ones from stdlib
// 3) if we have a fmt in malloc, we overflow the stack ue to inf recursion

#[cfg(feature = "web")]
#[no_mangle]
pub fn from_js_malloc(size: usize) -> *mut u8 {
    let layout = std::alloc::Layout::from_size_align(size, 4).unwrap();
    let ptr = unsafe { std::alloc::alloc(layout) };
    log::trace(format!("[from_js_malloc] size: {}, ptr: {:?}", size, ptr).as_str());
    ptr
}

#[cfg(feature = "web")]
#[no_mangle]
pub fn from_js_free(ptr: *mut u8, size: usize) {
    let layout = std::alloc::Layout::from_size_align(size, 4).unwrap();
    unsafe { std::alloc::dealloc(ptr, layout) }
    log::trace(format!("[from_js_free] size: {}, ptr: {:?}", size, ptr).as_str());
}
