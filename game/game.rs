use raylib::{KeyboardKey as KEY, MouseButton, Rectangle, RAYWHITE};
use raylib_wasm::{self as raylib, BLUE};
use raylib_wasm::{cstr, Color};
use webhacks::Bool;

mod anim;
mod array2d;
mod defer;
mod enemy;
mod turret;
mod vec2;
mod webhacks;

use crate::array2d::Array2D;
use crate::enemy::Enemy;
use crate::turret::Turret;
use crate::vec2::Vector2;
use crate::vec2::Vector2Ext;

const WINDOW_WIDTH: i32 = 800;
const WINDOW_HEIGHT: i32 = 600;

const SPEED_DEFAULT: f32 = 850.0;
const SPEED_BOOSTED: f32 = 1550.0;

const SPAWN_INTERVAL: f32 = 1.0;
// const SPEED_ENEMY: f32 = 340.0;
const SPEED_ENEMY: f32 = 1340.0;

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
#[derive(Clone)]
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
    pub enemies_n: u32,
    pub enemies_arr: *mut Enemy,
    pub mute: Bool,
    pub turrets_n: u32,
    pub turrets_arr: *mut Turret,
    pub life: u32,
    pub distances: *mut Array2D,
}

impl State {
    fn get_turrets(&self) -> Option<&[Turret]> {
        if self.turrets_arr.is_null() {
            None
        } else {
            let slice =
                unsafe { std::slice::from_raw_parts(self.turrets_arr, self.turrets_n as usize) };
            Some(slice)
        }
    }

    fn set_turrets(&mut self, turrets: &[Turret]) {
        free_malloced(self.turrets_n, self.turrets_arr);
        let (turrets_n, turrets_arr) = clone_to_malloced(turrets);
        self.turrets_n = turrets_n;
        self.turrets_arr = turrets_arr;
    }

    fn get_enemies(&self) -> Option<&[Enemy]> {
        if self.enemies_arr.is_null() {
            None
        } else {
            let slice =
                unsafe { std::slice::from_raw_parts(self.enemies_arr, self.enemies_n as usize) };
            Some(slice)
        }
    }

    fn set_enemies(&mut self, enemies: &[Enemy]) {
        free_malloced(self.enemies_n, self.enemies_arr);
        let (enemies_n, enemies_arr) = clone_to_malloced(&enemies);
        self.enemies_n = enemies_n;
        self.enemies_arr = enemies_arr;
    }

    fn get_path(&self) -> &[Vector2] {
        if self.path_arr.is_null() {
            return &[];
        } else {
            unsafe { std::slice::from_raw_parts(self.path_arr, self.path_n as usize) }
        }
    }

    fn get_distances(&self) -> &Array2D {
        unsafe { &*self.distances }
    }

    fn set_distances(&mut self, distances: Array2D) {
        let mut distances_box = unsafe { Box::from_raw(self.distances) };
        distances_box.clear();
        *distances_box = distances;
        self.distances = Box::into_raw(distances_box);
    }
}

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

fn make_initial_turrets() -> Vec<Turret> {
    vec![
        Turret::new(Vector2::new(200.0, 150.0)),
        Turret::new(Vector2::new(400.0, 150.0)),
    ]
}

#[no_mangle]
pub fn game_init() -> State {
    // We do not cap the framerate, since it leads to sluggish mouse input, since raylib cannot detect mouse input
    // between the frames and we don't really want to dig down to the GLFW layer and poll for events ourselves.
    // See: https://github.com/raysan5/raylib/issues/3354
    // Apparently this is known and solutions are unplanned. I guess it's not that much of a problem from C.
    // SetTargetFPS(300);

    // webhacks::log("game_init".to_string());

    raylib::init_window(WINDOW_WIDTH, WINDOW_HEIGHT, "game");

    webhacks::init_audio_device();

    let music = webhacks::load_music_stream("assets/hello_03.wav");

    // SetMusicVolume(music, 1.0);

    webhacks::play_music_stream(music);

    let font = webhacks::load_font("assets/Kavoon-Regular.ttf");
    let image = webhacks::load_image("assets/Blue_Slime-Idle-mag.png");

    let (path_points, path_length) = make_path_points();
    let (path_n, path_arr) = clone_to_malloced(&path_points);

    let turrets = make_initial_turrets();
    let (turrets_n, turrets_arr) = clone_to_malloced(&turrets);

    let distances = Array2D::new(0, 0);

    // move to static memory
    let distances_ptr = Box::into_raw(Box::new(distances));

    State {
        all_loaded: Bool::False(),
        curr_time: webhacks::get_time() as f32,
        prev_time: 0.0,
        frame_count: 99,
        slime_pos: Vector2::new(WINDOW_WIDTH as f32 / 2.0, WINDOW_HEIGHT as f32 / 2.0 + 50.0),
        mouse_pos: Vector2::new(0.0, 0.0),
        mouse_btn: Bool::False(),
        mouse_btn_pressed: Bool::False(),
        music: music,
        font: font,
        image: image,
        texture: webhacks::null_texture(),
        anim_blobs_n: 0,
        anim_blobs_arr: std::ptr::null(),
        path_n: path_n,
        path_arr: path_arr,
        path_length: path_length,
        enemies_n: 0,
        enemies_arr: std::ptr::null_mut(),
        mute: Bool::True(),
        turrets_n: turrets_n,
        turrets_arr: turrets_arr,
        distances: distances_ptr,
        life: 2,
    }
}

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

fn free_malloced<T>(len: u32, ptr: *mut T) {
    if ptr.is_null() {
        return;
    }
    let size = std::mem::size_of::<T>() * len as usize;
    let layout = std::alloc::Layout::from_size_align(size, 4).unwrap();
    let _ptr = ptr as *mut u8;
    unsafe { std::alloc::dealloc(_ptr, layout) }
}

pub type GameLoad = fn(state: &mut State);

#[no_mangle]
pub fn game_load(state: &mut State) {
    state.prev_time = state.curr_time;
    state.curr_time = webhacks::get_time() as f32;

    if state.all_loaded.bool() {
        return;
    }

    let mut any_not_loaded = false;

    // check if the music is loaded
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

    if !any_not_loaded {
        state.all_loaded = Bool::True();

        // Once we've determined that init/load is done, we can unload some resources

        let blobs = anim::find_blobs(state.image);
        let (anim_blobs_n, anim_blobs_arr) = clone_to_malloced(&blobs);

        state.anim_blobs_arr = anim_blobs_arr;
        state.anim_blobs_n = anim_blobs_n;

        webhacks::unload_image(state.image); // we don't need the image anymore

        if state.mute.bool() {
            webhacks::set_music_volume(state.music, 0.0);
        } else {
            webhacks::set_music_volume(state.music, 1.0);
        }

        let texture_shape = webhacks::get_texture_shape(state.texture);
        webhacks::log(format!(
            "texture shape: [{}, {}]",
            texture_shape.x, texture_shape.y
        ));
    }
}

fn time_to_anim_frame(time: f32, frame_duration: f32, n_frames: u32) -> u32 {
    let frame = (time / frame_duration) as u32 % n_frames;
    frame
}

fn handle_keys(state: &mut State) {
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

    // if raylib::IsKeyPressed(KEY::M) {
    if webhacks::is_key_pressed(KEY::M) {
        state.mute.toggle();
        webhacks::set_music_volume(state.music, if state.mute.bool() { 0.0 } else { 1.0 });
    }
}

fn handle_mouse(state: &mut State) {
    let mut mouse_pos = webhacks::get_mouse_position();
    let is_outside = mouse_pos.x < 0.0
        || mouse_pos.y < 0.0
        || mouse_pos.x > WINDOW_WIDTH as f32
        || mouse_pos.y > WINDOW_HEIGHT as f32;
    if is_outside {
        mouse_pos = Vector2::new(-1.0, -1.0);
    }
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

fn update_entities(state: &mut State) {
    let mut enemies = state.get_enemies().unwrap_or_default().to_vec();
    let mut turrets = state.get_turrets().unwrap_or_default().to_vec();

    let last_enemy = enemies.last();

    // spawn a new enemy every second
    if last_enemy.is_none() || state.curr_time - last_enemy.unwrap().spawn_time > SPAWN_INTERVAL {
        // spawn a new enemy
        let new_enemy = Enemy::new(state.curr_time);
        enemies.push(new_enemy);
    }

    let mut n_dead = 0;
    for enemy in enemies.iter_mut() {
        enemy.update(state);
        n_dead += enemy.dead.bool() as u32;
    }

    state.life -= std::cmp::min(n_dead, state.life);

    enemies = enemies
        .into_iter()
        .filter(|enemy| !enemy.dead.bool())
        .collect();

    let mut any_dead = false;
    for turret in turrets.iter_mut() {
        turret.update(state);
        any_dead = any_dead || turret.dead.bool();
    }

    if !any_dead && state.mouse_btn_pressed.bool() {
        // check if we've clicked with
        turrets.push(Turret::new(state.mouse_pos));
    }

    turrets = turrets
        .into_iter()
        .filter(|turret| !turret.dead.bool())
        .collect();

    // distances will be a 2D array of size enemies.len() x turret.len() + 1
    // mouse will be tracked as the last row

    let mut distances = Array2D::new(enemies.len(), turrets.len() + 1);

    for (i, enemy) in enemies.iter().enumerate() {
        for (j, turret) in turrets.iter().enumerate() {
            let dist = enemy
                .screen_position(state.get_path())
                .dist(&turret.position);
            distances.set(i, j, dist);
        }
        let mouse_dist = enemy
            .screen_position(state.get_path())
            .dist(&state.mouse_pos);
        distances.set(i, turrets.len(), mouse_dist);
    }

    state.set_distances(distances);
    state.set_enemies(&enemies);
    state.set_turrets(&turrets);
}

fn draw_entities_background(state: &State) {
    let enemies = state.get_enemies().unwrap_or_default();
    let turrets = state.get_turrets().unwrap_or_default();

    let distances = state.get_distances();

    // draw lines from enemies to turrets if they are within range
    for (i, enemy) in enemies.iter().enumerate() {
        for (j, turret) in turrets.iter().enumerate() {
            let distance = distances.get(i, j).clone();
            if distance < ACTIVE_RADIUS {
                let enemy_pos = enemy.screen_position(state.get_path());
                webhacks::draw_line_ex(enemy_pos, turret.position, 2.0, RAYWHITE);
            }
        }
    }

    // draw line to mouse if it's within range
    for (i, enemy) in enemies.iter().enumerate() {
        let distance = distances.get(i, distances.height() - 1).clone();
        if distance < ACTIVE_RADIUS {
            let enemy_pos = enemy.screen_position(state.get_path());
            webhacks::draw_line_ex(enemy_pos, state.mouse_pos, 2.0, RAYWHITE);
        }
    }

    for (i, enemy) in enemies.iter().enumerate() {
        enemy.draw_background(i, state);
    }
    for (i, turret) in turrets.iter().enumerate() {
        turret.draw_background(i, state);
    }
}

fn draw_entities_foreground(state: &State) {
    let enemies = state.get_enemies().unwrap_or_default();
    let turrets = state.get_turrets().unwrap_or_default();

    for (i, enemy) in enemies.iter().enumerate() {
        enemy.draw_foreground(i, state);
    }
    for (i, turret) in turrets.iter().enumerate() {
        turret.draw_foreground(i, state);
    }
}

fn draw_mouse(_state: &State) {
    // let color = if state.mouse_btn.bool() {
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
        if state.mute.bool() {
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

pub type GameFrame = fn(state: &mut State);

#[no_mangle]
pub fn game_frame(state: &mut State) {
    state.prev_time = state.curr_time;
    state.curr_time = webhacks::get_time() as f32;

    update_entities(state);

    let game_over = state.life == 0;

    if !game_over {
        handle_keys(state);
        handle_mouse(state);
    }

    unsafe { raylib::BeginDrawing() };

    {
        unsafe { raylib::ClearBackground(BLUE) };

        let anim_blobs = unsafe {
            std::slice::from_raw_parts(state.anim_blobs_arr, state.anim_blobs_n as usize)
        };
        draw_slime_at_pos(state.slime_pos, anim_blobs, state.texture, state.curr_time);

        draw_text(state);
        draw_entities_background(state);
        draw_path(state);
        draw_entities_foreground(state);

        draw_mouse(state);

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

    // Update the music stream
    webhacks::update_music_stream(state.music);

    // Update the frame count
    state.frame_count += 1;
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
    unsafe { std::alloc::alloc(layout) }
}

#[cfg(feature = "web")]
#[no_mangle]
pub fn from_js_free(ptr: *mut u8, size: usize) {
    let layout = std::alloc::Layout::from_size_align(size, 4).unwrap();
    unsafe { std::alloc::dealloc(ptr, layout) }
}
