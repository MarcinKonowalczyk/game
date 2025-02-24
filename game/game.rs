use raylib::{KeyboardKey as KEY, MouseButton, Rectangle, Vector2, DARKGREEN, RAYWHITE, RED};
use raylib_wasm::{self as raylib, BEIGE};

mod anim;
mod defer;
mod webhacks;

const WINDOW_WIDTH: i32 = 800;
const WINDOW_HEIGHT: i32 = 600;

const SPEED_DEFAULT: f32 = 850.0;
const SPEED_BOOSTED: f32 = 1550.0;

const SPAWN_INTERVAL: f32 = 1.0;
const SPEED_ENEMY: f32 = 100.0;

#[derive(Clone, Debug)]
pub struct Enemy {
    pub position: f32, // position along the path in pixels
    pub health: f32,
    pub max_health: f32,
    pub spawn_time: f32,
    pub last_hit_time: f32,
    pub dead: bool,
}

// All of the state that we need to keep track of in the game. The bits which are different for native and web
// are in the webhacks::State.
#[repr(C, align(4))]
#[derive(Clone)]
pub struct State {
    pub all_loaded: bool,
    pub curr_time: f32,
    pub prev_time: f32,
    pub frame_count: u32,
    pub rect: Rectangle,
    pub speed: f32,
    pub mouse_pos: Vector2,
    pub mouse_btn: u32,
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
}

#[no_mangle]
pub fn get_state_size() -> usize {
    std::mem::size_of::<State>()
}

#[no_mangle]
pub fn game_init() -> State {
    // We do not cap the framerate, since it leads to sluggish mouse input, since raylib cannot detect mouse input
    // between the frames and we don't really want to dig down to the GLFW layer and poll for events ourselves.
    // See: https://github.com/raysan5/raylib/issues/3354
    // Apparently this is known and solutions are unplanned. I guess it's not that much of a problem from C.

    // SetTargetFPS(300);

    raylib::init_window(WINDOW_WIDTH, WINDOW_HEIGHT, "game");

    webhacks::init_audio_device();

    let music = webhacks::load_music_stream("assets/hello_03.wav");

    // SetMusicVolume(music, 1.0);

    webhacks::play_music_stream(music);

    let font = webhacks::load_font("assets/Kavoon-Regular.ttf");

    // Load slime texture
    // Blue_Slime-Idle-mag.png
    // webhacks::log("loading texture from rust".to_string());
    // let image = raylib::LoadImage(cstr!("assets/Blue_Slime-Idle-mag.png"));
    let image = webhacks::load_image("assets/Blue_Slime-Idle-mag.png");

    let path_points: Vec<Vector2> = vec![
        Vector2 { x: 100.0, y: 100.0 },
        Vector2 { x: 100.0, y: 200.0 },
        Vector2 { x: 300.0, y: 300.0 },
    ];

    let path_length = path_points
        .iter()
        .fold((0.0, path_points[0]), |(acc, prev), &p| {
            (acc + pp_distance2(prev, p).sqrt(), p)
        })
        .0;

    let (path_n, path_arr) = clone_to_malloced(path_points);

    State {
        all_loaded: false,
        curr_time: webhacks::get_time() as f32,
        prev_time: 0.0,
        frame_count: 99,
        rect: Rectangle {
            x: (WINDOW_WIDTH as f32 - 100.0) / 2.0,
            y: (WINDOW_HEIGHT as f32 - 100.0) / 2.0,
            width: 100.0,
            height: 100.0,
        },
        speed: 850.0,
        mouse_pos: Vector2 { x: 0.0, y: 0.0 },
        mouse_btn: 0,
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
    }
}

fn clone_to_malloced<T: Clone>(vec: Vec<T>) -> (u32, *mut T) {
    // webhacks::log("clone_to_malloced".to_string());
    let n = vec.len().try_into().unwrap();
    // webhacks::log(format!(
    //     "clone_to_malloced: {:?}, {:?}",
    //     std::mem::size_of::<T>(),
    //     n
    // ));
    let vec_mem_size = std::mem::size_of::<T>() * vec.len();
    let layout = std::alloc::Layout::from_size_align(vec_mem_size, 4).unwrap();
    // webhacks::log(format!("malloc: {}, {:?}", vec_mem_size, layout));
    let vec_ptr = unsafe { std::alloc::alloc(layout) as *mut T };

    for (i, item) in vec.iter().enumerate() {
        unsafe {
            *vec_ptr.offset(i as isize) = item.clone();
        }
    }

    (n, vec_ptr)
}

#[allow(dead_code)]
fn free_malloced_array<T>(len: u32, ptr: *mut T) {
    // webhacks::log(format!("free_malloced_array: {}, {:?}", len, ptr));
    let size = std::mem::size_of::<T>() * len as usize;
    let maybe_layout = std::alloc::Layout::from_size_align(size, 4);
    if maybe_layout.is_err() {
        // webhacks::log("free_malloced_array: layout error".to_string());
        return;
    }
    let layout = maybe_layout.unwrap();
    // webhacks::log("free_malloced_array _ptr".to_string());
    let _ptr = ptr as *mut u8;
    // webhacks::log("free_malloced_array dealloc".to_string());
    // webhacks::log(format!(
    //     "free_malloced_array dealloc: {:?}, {:?}",
    //     layout, _ptr
    // ));
    unsafe { std::alloc::dealloc(_ptr, layout) }
    // webhacks::log("free_malloced_array done".to_string());
}

pub type GameLoad = fn(state: &mut State);

#[no_mangle]
pub fn game_load(state: &mut State) {
    state.prev_time = state.curr_time;
    state.curr_time = webhacks::get_time() as f32;

    if state.all_loaded {
        return;
    }

    let mut any_not_loaded = false;

    // check if the music is loaded
    if !webhacks::is_music_loaded(state.music) {
        // webhacks::log("music not loaded".to_string());
        any_not_loaded = true;
    }

    // check if the font is loaded
    if !webhacks::is_font_loaded(state.font) {
        // webhacks::log("font not loaded".to_string());
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
            // webhacks::log("texture not loaded".to_string());
            any_not_loaded = true;
        }
    }

    if !any_not_loaded {
        state.all_loaded = true;

        // Once we've determined that init/load is done, we can unload some resources

        let blobs = anim::find_blobs(state.image);

        // we have a vector of blobs. lets put it into a malloced array
        let (anim_blobs_n, anim_blobs_arr) = clone_to_malloced(blobs);

        state.anim_blobs_arr = anim_blobs_arr;
        state.anim_blobs_n = anim_blobs_n;

        webhacks::unload_image(state.image); // we don't need the image anymore    }

        webhacks::log(format!("current time: {}", state.curr_time));
        webhacks::log(format!("pre time: {}", state.prev_time));
    }
}

fn time_to_anim_frame(time: f32, frame_duration: f32, n_frames: u32) -> u32 {
    let frame = (time / frame_duration) as u32 % n_frames;
    frame
}

fn handle_keys(state: &mut State) {
    unsafe {
        if raylib::IsKeyDown(KEY::Space) {
            state.speed = SPEED_BOOSTED
        } else {
            state.speed = SPEED_DEFAULT
        }
    }

    let dt = unsafe { raylib::GetFrameTime() };

    let (w, s, a, d);
    unsafe {
        w = raylib::IsKeyDown(KEY::W);
        s = raylib::IsKeyDown(KEY::S);
        a = raylib::IsKeyDown(KEY::A);
        d = raylib::IsKeyDown(KEY::D);
    }

    state.rect.y -= dt * state.speed * (w as i32 as f32);
    state.rect.y += dt * state.speed * (s as i32 as f32);
    state.rect.x -= dt * state.speed * (a as i32 as f32);
    state.rect.x += dt * state.speed * (d as i32 as f32);

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
}

fn handle_mouse(state: &mut State) {
    let mut mouse_pos = unsafe { raylib::GetMousePosition() };
    let is_outside = mouse_pos.x < 0.0
        || mouse_pos.y < 0.0
        || mouse_pos.x > WINDOW_WIDTH as f32
        || mouse_pos.y > WINDOW_HEIGHT as f32;
    if is_outside {
        mouse_pos = Vector2 { x: -1.0, y: -1.0 };
    }
    state.mouse_pos = mouse_pos;
    state.mouse_btn = webhacks::is_mouse_button_down(MouseButton::Left as i32) as u32;
}

fn draw_slime_at_rect(
    rect: Rectangle,
    anim_blobs: &[anim::Blob],
    texture: webhacks::Texture,
    time: f32,
) {
    let mut position = Vector2 {
        x: rect.x,
        y: rect.y,
    };

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
    // webhacks::log("processing enemies".to_string());
    let mut new_enemies: Vec<Enemy> = if state.enemies_arr.is_null() {
        // we don't have any enemies yet
        vec![]
    } else {
        let enemies =
            unsafe { std::slice::from_raw_parts(state.enemies_arr, state.enemies_n as usize) };

        enemies.to_vec()
    };

    // remove all the dead enemies from the list
    // these will be the ones which were marked as dead in the previous frame
    new_enemies = new_enemies
        .into_iter()
        .filter(|enemy| !enemy.dead)
        .collect();

    let last_enemy = new_enemies.last();

    // spawn a new enemy every second
    if last_enemy.is_none() || state.curr_time - last_enemy.unwrap().spawn_time > SPAWN_INTERVAL {
        // spawn a new enemy
        let new_enemy = Enemy {
            position: 0.0,
            health: 100.0,
            max_health: 100.0,
            spawn_time: state.curr_time,
            last_hit_time: state.curr_time,
            dead: false,
        };
        new_enemies.push(new_enemy);
    }

    // move the enemies along the path
    let dt = state.curr_time - state.prev_time;
    for enemy in new_enemies.iter_mut() {
        enemy.position += SPEED_ENEMY * dt as f32;
    }

    // mark enemies that have reached the end of the path as dead
    for enemy in new_enemies.iter_mut() {
        if enemy.position >= state.path_length {
            enemy.dead = true;
        }
    }

    // update the state
    if !state.enemies_arr.is_null() {
        free_malloced_array(state.enemies_n, state.enemies_arr);
    }
    let (enemies_n, enemies_arr) = clone_to_malloced(new_enemies);
    state.enemies_n = enemies_n;
    state.enemies_arr = enemies_arr;
}

fn pp_distance2(p1: Vector2, p2: Vector2) -> f32 {
    let dx = p1.x - p2.x;
    let dy = p1.y - p2.y;
    dx * dx + dy * dy
}

fn path_pos_to_screen_pos(path_pos: f32, path: &[Vector2]) -> Vector2 {
    // path_pos in pixels

    // walk along the path until we reach the correct position
    let mut current_path_length = 0.0;
    for i in 1..path.len() {
        let p1 = path[i - 1];
        let p2 = path[i];
        let segment_length = pp_distance2(p1, p2).sqrt();
        if current_path_length + segment_length >= path_pos {
            // we've found the segment that contains the position
            let segment_pos = (path_pos - current_path_length) / segment_length;
            let dx = p2.x - p1.x;
            let dy = p2.y - p1.y;
            return Vector2 {
                x: p1.x + dx * segment_pos,
                y: p1.y + dy * segment_pos,
            };
        }
        current_path_length += segment_length;
    }

    Vector2 {
        x: path[path.len() - 1].x,
        y: path[path.len() - 1].y,
    }
}

fn draw_enemies(state: &State) {
    let enemies =
        unsafe { std::slice::from_raw_parts(state.enemies_arr, state.enemies_n as usize) };

    let path = unsafe { std::slice::from_raw_parts(state.path_arr, state.path_n as usize) };

    for enemy in enemies.iter() {
        let pos = path_pos_to_screen_pos(enemy.position, path);

        webhacks::draw_circle(pos, 10.0, BEIGE);
    }
}

pub type GameFrame = fn(state: &mut State);

#[no_mangle]
pub fn game_frame(state: &mut State) {
    state.prev_time = state.curr_time;
    state.curr_time = webhacks::get_time() as f32;

    handle_keys(state);
    handle_mouse(state);

    process_enemies(state);

    unsafe { raylib::BeginDrawing() };

    {
        unsafe { raylib::ClearBackground(DARKGREEN) };

        let anim_blobs = unsafe {
            std::slice::from_raw_parts(state.anim_blobs_arr, state.anim_blobs_n as usize)
        };
        draw_slime_at_rect(state.rect, anim_blobs, state.texture, state.curr_time);

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

        let color = if state.mouse_btn == 1 { RED } else { RAYWHITE };

        webhacks::draw_circle(state.mouse_pos, 10.0, color);

        // Draw the path
        let path = unsafe { std::slice::from_raw_parts(state.path_arr, state.path_n as usize) };
        for i in 1..path.len() {
            let p1 = path[i - 1];
            let p2 = path[i];
            webhacks::draw_line_ex(p1, p2, 2.0, RAYWHITE);
            // unsafe { raylib::DrawLineEx(p1, p2, 2.0, RAYWHITE) }
        }

        // Draw the enemies
        draw_enemies(state);
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
    // webhacks::log(format!("malloc: {}", size));
    let layout = std::alloc::Layout::from_size_align(size, 4).unwrap();
    unsafe { std::alloc::alloc(layout) }
}

#[cfg(feature = "web")]
#[no_mangle]
pub fn from_js_free(ptr: *mut u8, size: usize) {
    // webhacks::log(format!("free: {}", size));
    let layout = std::alloc::Layout::from_size_align(size, 4).unwrap();
    unsafe { std::alloc::dealloc(ptr, layout) }
}
