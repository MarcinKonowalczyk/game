// #![deny(unused_results)]

use entity_manager::{Entity, EntityManager};
use raylib::{KeyboardKey as KEY, MouseButton, RAYWHITE};
use raylib_wasm::{self as raylib, Color, BLUE};
use u32_bool::Bool;
use webhacks::MusicStatus;

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
const SPEED_ENEMY: f32 = 210.0;
const SPEED_BULLET: f32 = SPEED_ENEMY + 50.0;
// const SPEED_ENEMY: f32 = 1340.0;

const TURRET_RADIUS: f32 = 10.0;
const ACTIVE_RADIUS: f32 = 150.0;

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
    pub slime_anim: anim::Anim,
    pub bullet_anim: anim::Anim,
    pub turret_anim: anim::Anim,
    pub path_n: u32,
    pub path_arr: *const Vector2,
    pub path_length: f32,
    pub mute: Bool,
    pub debug: Bool,
    pub life: u32,
    pub man: EntityManager,
}

impl State {
    fn get_path(&self) -> &[Vector2] {
        if self.path_arr.is_null() {
            return &[];
        } else {
            unsafe { std::slice::from_raw_parts(self.path_arr, self.path_n as usize) }
        }
    }

    fn dt(&self) -> f32 {
        self.curr_time - self.prev_time
    }
}

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

fn make_initial_turrets(man: &mut EntityManager) {
    let t1 = Turret::new(Vector2::new(200.0, 150.0));
    let t2 = Turret::new(Vector2::new(400.0, 180.0));

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

    let music = webhacks::load_music_stream("assets_private/hello_03.wav");
    let font = webhacks::load_font("assets_private/Kavoon-Regular.ttf");

    let (path_points, path_length) = make_path_points();
    let (path_n, path_arr) = clone_to_malloced(&path_points);

    let mut man = EntityManager::new();

    make_initial_turrets(&mut man);

    let slime_anim = anim::Anim::new(webhacks::load_image("assets/slime_green-mag.png"));
    let bullet_anim = anim::Anim::new(webhacks::load_image("assets/bullet-mag.png"));
    let turret_anim = anim::Anim::new(webhacks::load_image("assets/turret-mag.png"));

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
        slime_anim: slime_anim,
        bullet_anim: bullet_anim,
        turret_anim: turret_anim,
        path_n: path_n,
        path_arr: path_arr,
        path_length: path_length,
        mute: true.into(),
        debug: true.into(),
        life: 20,
        man: man,
    }
}

fn clone_to_malloced<T: Clone>(arr: &[T]) -> (u32, *mut T) {
    match arr.len() {
        0 => (0, std::ptr::null_mut()),
        n => {
            let mem_size = std::mem::size_of::<T>() * n;
            match std::alloc::Layout::from_size_align(mem_size, 4) {
                Ok(layout) => {
                    let ptr = unsafe { std::alloc::alloc(layout) as *mut T };
                    for (i, item) in arr.iter().enumerate() {
                        unsafe {
                            *ptr.offset(i as isize) = item.clone();
                        }
                    }
                    (n as u32, ptr)
                }
                Err(_) => {
                    panic!("failed to clone_to_malloced");
                }
            }
        }
    }
}

#[allow(unused)]
fn free_malloced<T>(len: u32, ptr: *mut T) {
    match ptr {
        _ if ptr.is_null() => {}
        _ => {
            let size = std::mem::size_of::<T>() * len as usize;
            match std::alloc::Layout::from_size_align(size, 4) {
                Ok(layout) => {
                    let _ptr = ptr as *mut u8;
                    unsafe { std::alloc::dealloc(_ptr, layout) }
                }
                Err(_) => {}
            }
        }
    }
}

pub type GameLoad = fn(state: *mut State);

#[no_mangle]
pub fn game_load(_state: *mut State) {
    let mut state = unsafe { std::ptr::read(_state) };
    state.prev_time = state.curr_time;
    state.curr_time = webhacks::get_time() as f32;

    if state.all_loaded.into() {
        return;
    }

    let mut any_not_loaded = false;

    // check if the music is loaded
    if !webhacks::is_music_loaded(state.music) {
        println!("music not loaded: {:?}", state.music);
        any_not_loaded = true;
    }

    // check if the font is loaded
    if !webhacks::is_font_loaded(state.font) {
        any_not_loaded = true;
    }

    if !state.slime_anim.is_image_loaded() {
        any_not_loaded = true;
    } else {
        // slime_anim image is loaded! let's load the texture
        // state.texture = webhacks::load_texture_from_image(state.image);

        state.slime_anim.load_texture();

        if !state.slime_anim.is_texture_loaded() {
            any_not_loaded = true;
        }
    }

    if !state.bullet_anim.is_image_loaded() {
        any_not_loaded = true;
    } else {
        // bullet_anim image is loaded! let's load the texture
        // state.texture = webhacks::load_texture_from_image(state.image);

        state.bullet_anim.load_texture();

        if !state.bullet_anim.is_texture_loaded() {
            any_not_loaded = true;
        }
    }

    if !state.turret_anim.is_image_loaded() {
        any_not_loaded = true;
    } else {
        // turret_anim image is loaded! let's load the texture
        // state.texture = webhacks::load_texture_from_image(state.image);

        state.turret_anim.load_texture();

        if !state.turret_anim.is_texture_loaded() {
            any_not_loaded = true;
        }
    }

    if !any_not_loaded {
        state.all_loaded = true.into();

        // Once we've determined that init/load is done, we can unload some resources
        state.slime_anim.find_blobs();
        state.slime_anim.unload_image();

        state.bullet_anim.find_blobs();
        state.bullet_anim.unload_image();

        state.turret_anim.find_blobs();
        state.turret_anim.unload_image();

        webhacks::play_music_stream(state.music);

        if state.mute.into() {
            webhacks::set_music_volume(state.music, 0.0);
        } else {
            webhacks::set_music_volume(state.music, 1.0);
        }

        let texture_shape = webhacks::get_texture_shape(state.slime_anim.texture);
        log::info(
            format!(
                "slime texture shape: [{}, {}]",
                texture_shape.x, texture_shape.y
            )
            .as_str(),
        );

        for turret in state.man.turrets.iter_mut() {
            turret.anim = Some(state.turret_anim.clone());
        }
    }

    // wrtie back the state
    unsafe {
        std::ptr::write(_state, state);
    }
}

struct HandleKeysUpdate {
    slime_pos: Vector2,
    mute: bool,
    debug: bool,
}

impl From<&State> for HandleKeysUpdate {
    fn from(state: &State) -> Self {
        HandleKeysUpdate {
            slime_pos: state.slime_pos,
            mute: state.mute.into(),
            debug: state.debug.into(),
        }
    }
}

fn handle_keys(state: &State) -> HandleKeysUpdate {
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

    let mut update = HandleKeysUpdate::from(state);

    update.slime_pos.y -= dt * speed * (w as i32 as f32);
    update.slime_pos.y += dt * speed * (s as i32 as f32);
    update.slime_pos.x -= dt * speed * (a as i32 as f32);
    update.slime_pos.x += dt * speed * (d as i32 as f32);

    // prevent the rect from wandering off the screen too far
    if update.slime_pos.x < -100.0 {
        update.slime_pos.x = -100.0;
    } else if update.slime_pos.x > WINDOW_WIDTH as f32 {
        update.slime_pos.x = WINDOW_WIDTH as f32;
    }

    if update.slime_pos.y < -100.0 {
        update.slime_pos.y = -100.0;
    } else if update.slime_pos.y > WINDOW_HEIGHT as f32 {
        update.slime_pos.y = WINDOW_HEIGHT as f32;
    }

    // if raylib::IsKeyPressed(KEY::M) {
    if webhacks::is_key_pressed(KEY::M) {
        update.mute = !update.mute;
    }

    if webhacks::is_key_pressed(KEY::P) {
        update.debug = !update.debug;
    }

    update
}

fn apply_keys_update(state: &mut State, update: HandleKeysUpdate) {
    if state.mute != update.mute.into() {
        if update.mute {
            webhacks::set_music_volume(state.music, 0.0);
        } else {
            webhacks::set_music_volume(state.music, 1.0);
        }
    }

    state.slime_pos = update.slime_pos;
    state.mute = update.mute.into();
    state.debug = update.debug.into();
}
struct HandleMouseUpdate {
    mouse_pos: Vector2,
    mouse_btn: bool,
    mouse_btn_pressed: bool,
}

impl From<&State> for HandleMouseUpdate {
    fn from(state: &State) -> Self {
        HandleMouseUpdate {
            mouse_pos: state.mouse_pos,
            mouse_btn: state.mouse_btn.into(),
            mouse_btn_pressed: state.mouse_btn_pressed.into(),
        }
    }
}

fn handle_mouse(state: &State) -> HandleMouseUpdate {
    let mut mouse_pos = webhacks::get_mouse_position();
    let is_outside = mouse_pos.x < 0.0
        || mouse_pos.y < 0.0
        || mouse_pos.x > WINDOW_WIDTH as f32
        || mouse_pos.y > WINDOW_HEIGHT as f32;
    if is_outside {
        mouse_pos = Vector2::new(-1.0, -1.0);
    }

    let mut update = HandleMouseUpdate::from(state);
    update.mouse_pos = mouse_pos;
    update.mouse_btn = webhacks::is_mouse_button_down(MouseButton::Left as i32);
    update.mouse_btn_pressed = webhacks::is_mouse_button_pressed(MouseButton::Left as i32);

    update
}

struct HandleEntitiesUpdate {
    life_lost: u32,
    new_enemies: Vec<Enemy>,
    enemy_updates: Vec<enemy::EnemyUpdate>,
    bullet_updates: Vec<bullet::BulletUpdate>,
    turret_updates: Vec<turret::TurretUpdate>,
    new_bullets: Vec<bullet::Bullet>,
    new_turrets: Vec<turret::Turret>,
}

impl HandleEntitiesUpdate {
    fn new() -> HandleEntitiesUpdate {
        HandleEntitiesUpdate {
            life_lost: 0,
            new_enemies: vec![],
            enemy_updates: vec![],
            bullet_updates: vec![],
            turret_updates: vec![],
            new_bullets: vec![],
            new_turrets: vec![],
        }
    }
}

fn handle_entities(state: &State) -> HandleEntitiesUpdate {
    let mut update = HandleEntitiesUpdate::new();

    {
        if match state.man.enemies.last() {
            Some(Enemy {
                spawn_time: last_spawn_time,
                ..
            }) if state.curr_time - last_spawn_time > SPAWN_INTERVAL => true,
            None => true,
            _ => false,
        } {
            let mut new_enemy = Enemy::new(state.curr_time);
            new_enemy.anim = Some(state.slime_anim.clone());
            update.new_enemies.push(new_enemy.into());
        }
    }

    {
        update.enemy_updates = state
            .man
            .enemies
            .iter()
            .map(|enemy| enemy.update(state))
            .collect::<Vec<_>>();

        // Calculate the total damage done by the enemies
        update.life_lost = update
            .enemy_updates
            .iter()
            .map(|update| update.damage_done)
            .sum::<u32>();
        update.life_lost = std::cmp::min(update.life_lost, state.life);
    }

    {
        update.bullet_updates = state
            .man
            .bullets
            .iter()
            .map(|bullet| bullet.update(state))
            .collect::<Vec<_>>();

        // update.hit_requests = update
        //     .bullet_updates
        //     .iter()
        //     .filter_map(|update| match &update.hit_request {
        //         Some(hit_request) => Some(hit_request.clone()),
        //         None => None,
        //     })
        //     .collect::<Vec<_>>();
    }

    {
        // Get all the turret updates
        update.turret_updates = state
            .man
            .turrets
            .iter()
            .map(|turret| turret.update(state))
            .collect::<Vec<_>>();

        update.new_bullets = update
            .turret_updates
            .iter()
            .filter_map(|update| match &update.bullet_request {
                Some(bullet_request) => {
                    let mut bullet = bullet::Bullet::new(
                        bullet_request.position,
                        bullet_request.source,
                        bullet_request.target,
                    );
                    bullet.anim = Some(state.bullet_anim.clone());
                    Some(bullet.into())
                }
                None => None,
            })
            .collect::<Vec<_>>();

        // TODO: Should handle this better

        // Check whether any turrets are dead
        let any_dead = update
            .turret_updates
            .iter()
            .any(|update| update.dead.into());

        if !any_dead && { state.mouse_btn_pressed.into() } {
            let mut new_turret = Turret::new(state.mouse_pos);
            new_turret.anim = Some(state.turret_anim.clone());
            update.new_turrets.push(new_turret.into());
        }
    }

    update
}

fn apply_entities_update(state: &mut State, update: HandleEntitiesUpdate) {
    state.life -= update.life_lost;

    // Apply self updates to all entities
    std::iter::Iterator::zip(state.man.enemies.iter_mut(), update.enemy_updates.iter())
        .for_each(|(enemy, update)| enemy.apply(update));

    std::iter::Iterator::zip(state.man.bullets.iter_mut(), update.bullet_updates.iter())
        .for_each(|(bullet, update)| bullet.apply(update));

    std::iter::Iterator::zip(state.man.turrets.iter_mut(), update.turret_updates.iter())
        .for_each(|(turret, update)| turret.apply(update));

    // Handle interactions between entities
    let hit_requests = update.bullet_updates.iter().filter_map(|update| {
        if let Some(hit_request) = &update.hit_request {
            Some(hit_request)
        } else {
            None
        }
    });

    for hit_request in hit_requests {
        let target = state.man.get_enemy_mut(hit_request.target);
        if let Some(target) = target {
            target.hit(hit_request.damage);
        }
    }

    // Spawn new entities
    update
        .new_enemies
        .into_iter()
        .for_each(|enemy| state.man.add(enemy.into()));

    update
        .new_bullets
        .into_iter()
        .for_each(|bullet| state.man.add(bullet.into()));

    update
        .new_turrets
        .into_iter()
        .for_each(|turret| state.man.add(turret.into()));

    state.man.filter_dead();
}

fn draw_entities_debug(state: &State) {
    // draw lines from enemies to turrets if they are within range
    for enemy in state.man.enemies.iter() {
        for turret in state.man.turrets.iter() {
            let distance = enemy.position.dist(&turret.position);
            if distance < ACTIVE_RADIUS {
                let enemy_pos = enemy.position;
                webhacks::draw_line_ex(enemy_pos, turret.position, 2.0, RAYWHITE);
            }
        }
    }

    // draw line to mouse if it's within range
    for enemy in state.man.enemies.iter() {
        let distance = enemy.position.dist(&state.mouse_pos);
        if distance < ACTIVE_RADIUS {
            let enemy_pos = enemy.position;
            webhacks::draw_line_ex(enemy_pos, state.mouse_pos, 2.0, RAYWHITE);
        }
    }

    for enemy in state.man.enemies.iter() {
        enemy.draw_debug(state);
    }
    for turret in state.man.turrets.iter() {
        turret.draw_debug(state);
    }

    for bullet in state.man.bullets.iter() {
        bullet.draw_debug(state);
    }
}

fn draw_entities_foreground(state: &State) {
    for enemy in state.man.enemies.iter() {
        enemy.draw_foreground(state);
    }
    for turret in state.man.turrets.iter() {
        turret.draw_foreground(state);
    }
    for bullet in state.man.bullets.iter() {
        bullet.draw_foreground(state);
    }
}

fn draw_mouse(_state: &State) {
    // let color = if state.mouse_btn.into() {
    //     RED
    // } else {
    //     RAYWHITE
    // };
    // webhacks::draw_circle(state.mouse_pos, ACTIVE_RADIUS, BEIGE);
    // webhacks::draw_circle(state.mouse_pos, 2.0, color);
}

fn draw_path(state: &State) {
    // Draw the path
    let path = unsafe { std::slice::from_raw_parts(state.path_arr, state.path_n as usize) };
    for i in 1..path.len() {
        let p1 = path[i - 1];
        let p2 = path[i];
        webhacks::draw_line_ex(p1, p2, 2.0, RAYWHITE);
        // unsafe { raylib::DrawLineEx(p1, p2, 2.0, RAYWHITE) }
    }
}

fn draw_text(state: &State) {
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
    let last = path[path.len() - 1];
    let pos = last + Vector2::new(-text_size.x as f32 / 2.0, 20.0);
    webhacks::draw_text(state.font, &life_text, pos, font_size, 2.0, RAYWHITE);
}

pub type GameFrame = fn(state: *mut State);

#[no_mangle]
pub fn game_frame(state_ptr: *mut State) {
    let mut state = unsafe { std::ptr::read(state_ptr) };
    state.prev_time = state.curr_time;
    state.curr_time = webhacks::get_time() as f32;

    let update = handle_entities(&state);
    apply_entities_update(&mut state, update);

    let game_over = state.life == 0;

    let update = handle_keys(&state);
    apply_keys_update(&mut state, update);

    let update = handle_mouse(&state);
    state.mouse_pos = update.mouse_pos;
    state.mouse_btn = update.mouse_btn.into();
    state.mouse_btn_pressed = update.mouse_btn_pressed.into();

    unsafe { raylib::BeginDrawing() };

    {
        unsafe { raylib::ClearBackground(BLUE) };

        state.slime_anim.draw(
            state.slime_pos,
            5.0,
            anim::Anchor::Center,
            45.0_f32.to_radians(),
            state.curr_time,
        );

        draw_text(&state);
        if state.debug.into() {
            draw_entities_debug(&state);
        }
        draw_path(&state);
        draw_entities_foreground(&state);

        draw_mouse(&state);

        if game_over {
            // draw a shaded rectangle over the screen
            unsafe {
                raylib::DrawRectangle(0, 0, WINDOW_WIDTH, WINDOW_HEIGHT, ALPHA_BLACK);
            }

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
        webhacks::update_music_stream(state.music);

        // Update the frame count
        state.frame_count += 1;
    }

    // Write back the state
    unsafe {
        std::ptr::write(state_ptr, state);
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
