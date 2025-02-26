use raylib::{KeyboardKey as KEY, MouseButton, Rectangle, RAYWHITE, RED};
use raylib_wasm::PINK;
use raylib_wasm::{self as raylib, BEIGE, BLUE};
use webhacks::Bool;

mod anim;
mod defer;
mod vec2_ext;
mod webhacks;

use crate::vec2_ext::Vector2;
use crate::vec2_ext::Vector2Ext;

const WINDOW_WIDTH: i32 = 800;
const WINDOW_HEIGHT: i32 = 600;

const SPEED_DEFAULT: f32 = 850.0;
const SPEED_BOOSTED: f32 = 1550.0;

const SPAWN_INTERVAL: f32 = 1.0;
const SPEED_ENEMY: f32 = 340.0;

const ACTIVE_RADIUS: f32 = 100.0;

#[derive(Clone, Debug)]
pub struct Enemy {
    pub position: f32, // position along the path in pixels
    pub health: f32,
    pub max_health: f32,
    pub spawn_time: f32,
    pub last_hit_time: f32,
    pub dead: Bool,
}

#[derive(Clone, Debug)]
pub struct Turret {
    pub position: Vector2, // position along the path in pixels
    pub dead: Bool,
}

fn path_pos_to_screen_pos(path_pos: f32, path: &[Vector2]) -> Vector2 {
    // path_pos in pixels

    // walk along the path until we reach the correct position
    let mut current_path_length = 0.0;
    for i in 1..path.len() {
        let p1 = path[i - 1];
        let p2 = path[i];
        let segment_length = p1.dist(p2);
        if current_path_length + segment_length >= path_pos {
            let segment_pos = (path_pos - current_path_length) / segment_length;
            return p1.lerp(p2, segment_pos);
        }
        current_path_length += segment_length;
    }

    path[path.len() - 1]
}

impl Enemy {
    fn new(time: f32) -> Enemy {
        Enemy {
            position: 0.0,
            health: 100.0,
            max_health: 100.0,
            spawn_time: time,
            last_hit_time: -1.0,
            dead: Bool::False(),
        }
    }

    fn update(&mut self, state: &State) {
        let dt = state.curr_time - state.prev_time;
        self.position += SPEED_ENEMY * dt;
        if self.position >= state.path_length {
            self.dead = Bool::True();
        };
    }

    fn draw(&self, index: usize, state: &State) {
        let path = state.get_path();
        let pos = self.screen_position(path);
        let volatile = state.get_volatile();
        let distances = volatile.get_enemy_mouse_distances();
        let distance = distances
            .unwrap_or_default()
            .get(index)
            .cloned()
            .unwrap_or(f32::MAX);
        let color = if distance < ACTIVE_RADIUS {
            PINK
        } else {
            RAYWHITE
        };
        webhacks::draw_circle(pos, 10.0, color);
    }

    fn screen_position(&self, path: &[Vector2]) -> Vector2 {
        path_pos_to_screen_pos(self.position, path)
    }
}

impl Turret {
    fn new(position: Vector2) -> Turret {
        Turret {
            position,
            dead: Bool::False(),
        }
    }

    fn update(&mut self, _state: &State) {
        //
    }

    fn draw(&self, _index: usize, _state: &State) {
        webhacks::draw_circle(self.position, 10.0, PINK);
    }
}

// Recomputed at the beginning of every frame
#[repr(C, align(4))]
#[derive(Clone, Debug)]
pub struct VolatileState {
    pub enemy_mouse_n: u32,
    pub enemy_mouse_arr: *mut f32, // distance of each enemy from the mouse
}

impl VolatileState {
    fn get_enemy_mouse_distances(&self) -> Option<Vec<f32>> {
        if self.enemy_mouse_arr.is_null() {
            None
        } else {
            let slice = unsafe {
                std::slice::from_raw_parts(self.enemy_mouse_arr, self.enemy_mouse_n as usize)
            };
            Some(slice.to_vec())
        }
    }

    fn set_enemy_mouse_distances(&mut self, distances: Vec<f32>) {
        if !self.enemy_mouse_arr.is_null() {
            free_malloced(self.enemy_mouse_n, self.enemy_mouse_arr);
        }
        let (enemy_mouse_n, enemy_mouse_arr) = clone_to_malloced(distances);
        self.enemy_mouse_n = enemy_mouse_n;
        self.enemy_mouse_arr = enemy_mouse_arr;
    }
}

#[repr(C, align(4))]
#[derive(Clone)]
pub struct State {
    pub all_loaded: Bool,
    pub curr_time: f32,
    pub prev_time: f32,
    pub frame_count: u32,
    pub rect: Rectangle,
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
    pub volatile: *mut VolatileState,
}

impl State {
    fn get_turrets_vec(&self) -> Option<Vec<Turret>> {
        if self.turrets_arr.is_null() {
            None
        } else {
            let slice =
                unsafe { std::slice::from_raw_parts(self.turrets_arr, self.turrets_n as usize) };
            Some(slice.to_vec())
        }
    }

    fn set_turrets_vec(&mut self, turrets: Vec<Turret>) {
        if !self.turrets_arr.is_null() {
            free_malloced(self.turrets_n, self.turrets_arr);
        }
        let (turrets_n, turrets_arr) = clone_to_malloced(turrets);
        self.turrets_n = turrets_n;
        self.turrets_arr = turrets_arr;
    }

    fn get_enemies_vec(&self) -> Option<Vec<Enemy>> {
        if self.enemies_arr.is_null() {
            None
        } else {
            let slice =
                unsafe { std::slice::from_raw_parts(self.enemies_arr, self.enemies_n as usize) };
            Some(slice.to_vec())
        }
    }

    fn set_enemies_vec(&mut self, enemies: Vec<Enemy>) {
        if !self.enemies_arr.is_null() {
            free_malloced(self.enemies_n, self.enemies_arr);
        }
        let (enemies_n, enemies_arr) = clone_to_malloced(enemies);
        self.enemies_n = enemies_n;
        self.enemies_arr = enemies_arr;
    }

    fn get_path(&self) -> &[Vector2] {
        unsafe { std::slice::from_raw_parts(self.path_arr, self.path_n as usize) }
    }

    fn get_volatile(&self) -> &VolatileState {
        unsafe { &*self.volatile }
    }

    fn get_volatile_mut(&mut self) -> &mut VolatileState {
        unsafe { &mut *self.volatile }
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
            (acc + prev.dist(p), p)
        })
        .0;

    (path_points, path_length)
}

fn make_initial_turrets() -> Vec<Turret> {
    vec![
        Turret::new(Vector2::new(100.0, 100.0)),
        Turret::new(Vector2::new(200.0, 200.0)),
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
    let (path_n, path_arr) = clone_to_malloced(path_points);

    let turrets = make_initial_turrets();
    let (turrets_n, turrets_arr) = clone_to_malloced(turrets);

    let volatile_local = VolatileState {
        enemy_mouse_n: 0,
        enemy_mouse_arr: std::ptr::null_mut(),
    };

    // move to malloced memory
    let mem_size = std::mem::size_of::<VolatileState>();
    let layout = std::alloc::Layout::from_size_align(mem_size, 4).unwrap();
    let volatile_ptr = unsafe { std::alloc::alloc(layout) as *mut VolatileState };
    unsafe { *volatile_ptr = volatile_local.clone() };

    State {
        all_loaded: Bool::False(),
        curr_time: webhacks::get_time() as f32,
        prev_time: 0.0,
        frame_count: 99,
        rect: Rectangle {
            x: (WINDOW_WIDTH as f32 - 100.0) / 2.0,
            y: (WINDOW_HEIGHT as f32 - 100.0) / 2.0,
            width: 100.0,
            height: 100.0,
        },
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
        volatile: volatile_ptr,
    }
}

fn clone_to_malloced<T: Clone>(vec: Vec<T>) -> (u32, *mut T) {
    let n = vec.len().try_into().unwrap();

    if n == 0 {
        return (0, std::ptr::null_mut());
    }

    let vec_mem_size = std::mem::size_of::<T>() * vec.len();
    let layout = std::alloc::Layout::from_size_align(vec_mem_size, 4).unwrap();
    let vec_ptr = unsafe { std::alloc::alloc(layout) as *mut T };

    for (i, item) in vec.iter().enumerate() {
        unsafe {
            *vec_ptr.offset(i as isize) = item.clone();
        }
    }

    (n, vec_ptr)
}

#[allow(dead_code)]
fn free_malloced<T>(len: u32, ptr: *mut T) {
    let size = std::mem::size_of::<T>() * len as usize;
    let maybe_layout = std::alloc::Layout::from_size_align(size, 4);
    if maybe_layout.is_err() {
        return;
    }
    let layout = maybe_layout.unwrap();
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

        // we have a vector of blobs. lets put it into a malloced array
        let (anim_blobs_n, anim_blobs_arr) = clone_to_malloced(blobs);

        state.anim_blobs_arr = anim_blobs_arr;
        state.anim_blobs_n = anim_blobs_n;

        webhacks::unload_image(state.image); // we don't need the image anymore    }

        if state.mute.bool() {
            webhacks::set_music_volume(state.music, 0.0);
        } else {
            webhacks::set_music_volume(state.music, 1.0);
        }
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

    state.rect.y -= dt * speed * (w as i32 as f32);
    state.rect.y += dt * speed * (s as i32 as f32);
    state.rect.x -= dt * speed * (a as i32 as f32);
    state.rect.x += dt * speed * (d as i32 as f32);

    // prevent the rect from wandering off the screen too far
    if state.rect.x < -state.rect.width {
        state.rect.x = -state.rect.width;
    } else if state.rect.x > WINDOW_WIDTH as f32 {
        state.rect.x = WINDOW_WIDTH as f32;
    }

    if state.rect.y < -state.rect.height {
        state.rect.y = -state.rect.height;
    } else if state.rect.y > WINDOW_HEIGHT as f32 {
        state.rect.y = WINDOW_HEIGHT as f32;
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

fn draw_slime_at_rect(
    rect: Rectangle,
    anim_blobs: &[anim::Blob],
    texture: webhacks::Texture,
    time: f32,
) {
    let mut position = Vector2::new(rect.x, rect.y);

    // figure out how to scale the texture to the size of the rect
    let shape = webhacks::get_texture_shape(texture);
    let scale = rect.width / shape.x as f32;

    // Move the texture so it's at the bottom of the rect
    let scaled_height = shape.y as f32 * scale;
    position.y += rect.height - scaled_height;

    let i = time_to_anim_frame(time, 0.1, anim_blobs.len() as u32);

    let blob = anim_blobs[i as usize];
    let source = blob.to_rect();
    webhacks::draw_texture_pro(
        texture,
        source,
        Rectangle {
            x: position.x,
            y: position.y,
            width: rect.width,
            height: scaled_height,
        },
    );
}

fn process_enemies(state: &mut State) {
    let mut enemies = state.get_enemies_vec().unwrap_or_default();

    let last = enemies.last();

    // spawn a new enemy every second
    if last.is_none() || state.curr_time - last.unwrap().spawn_time > SPAWN_INTERVAL {
        // spawn a new enemy
        let new_enemy = Enemy::new(state.curr_time);
        enemies.push(new_enemy);
    }

    for enemy in enemies.iter_mut() {
        enemy.update(state);
    }

    enemies = enemies
        .into_iter()
        .filter(|enemy| !enemy.dead.bool())
        .collect();

    // calculate the distance of each enemy from the mouse
    let distances = enemies
        .iter()
        .map(|enemy| {
            let pos = enemy.screen_position(state.get_path());
            let dist = pos.dist(state.mouse_pos);
            dist
        })
        .collect::<Vec<f32>>();

    state
        .get_volatile_mut()
        .set_enemy_mouse_distances(distances);

    state.set_enemies_vec(enemies);
}

fn process_turrets(state: &mut State) {
    let mut turrets = state.get_turrets_vec().unwrap_or_default();

    if state.mouse_btn_pressed.bool() {
        turrets.push(Turret::new(state.mouse_pos));
    }

    for turret in turrets.iter_mut() {
        turret.update(state);
    }

    turrets = turrets
        .into_iter()
        .filter(|turret| !turret.dead.bool())
        .collect();

    state.set_turrets_vec(turrets);
}

fn draw_enemies(state: &State) {
    let enemies = state.get_enemies_vec().unwrap_or_default();

    if enemies.is_empty() {
        return;
    }

    for (i, enemy) in enemies.iter().enumerate() {
        enemy.draw(i, state);
    }
}

fn draw_turrets(state: &State) {
    let turrets = state.get_turrets_vec().unwrap_or_default();

    if turrets.is_empty() {
        return;
    }

    for (i, turret) in turrets.iter().enumerate() {
        turret.draw(i, state);
    }
}
fn draw_mouse(state: &State) {
    let color = if state.mouse_btn.bool() {
        RED
    } else {
        RAYWHITE
    };
    webhacks::draw_circle(state.mouse_pos, ACTIVE_RADIUS, BEIGE);
    webhacks::draw_circle(state.mouse_pos, 10.0, color);

    // draw lar
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
    let rect_pos = format! {
        "rect: [{x}, {y}]",
        x = state.rect.x.round(),
        y = state.rect.y.round()
    };
    webhacks::draw_text(state.font, &rect_pos, 10, 10, 20, RAYWHITE);

    let mouse_pos = format! {
        "mouse: [{x}, {y}]",
        x = state.mouse_pos.x.round(),
        y = state.mouse_pos.y.round()
    };
    webhacks::draw_text(state.font, &mouse_pos, 10, 30, 20, RAYWHITE);

    // Draw the music indicator in the top right corner
    webhacks::draw_text(
        state.font,
        if state.mute.bool() {
            "sound: off"
        } else {
            "sound: on"
        },
        WINDOW_WIDTH - 105,
        10,
        20,
        RAYWHITE,
    );
}

pub type GameFrame = fn(state: &mut State);

#[no_mangle]
pub fn game_frame(state: &mut State) {
    state.prev_time = state.curr_time;
    state.curr_time = webhacks::get_time() as f32;

    process_enemies(state);
    process_turrets(state);

    handle_keys(state);
    handle_mouse(state);

    unsafe { raylib::BeginDrawing() };

    {
        unsafe { raylib::ClearBackground(BLUE) };

        draw_mouse(state);

        let anim_blobs = unsafe {
            std::slice::from_raw_parts(state.anim_blobs_arr, state.anim_blobs_n as usize)
        };
        draw_slime_at_rect(state.rect, anim_blobs, state.texture, state.curr_time);

        draw_text(state);
        draw_path(state);
        draw_enemies(state);
        draw_turrets(state);
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
