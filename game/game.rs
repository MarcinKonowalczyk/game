use raylib::{KeyboardKey as KEY, MouseButton, Rectangle, Vector2, DARKGREEN, RAYWHITE, RED};
use raylib_wasm::{self as raylib};

mod webhacks;

mod anim;

const WINDOW_WIDTH: i32 = 800;
const WINDOW_HEIGHT: i32 = 600;

const SPEED_DEFAULT: f32 = 850.0;
const SPEED_BOOSTED: f32 = 1550.0;


// All of the state that we need to keep track of in the game. The bits which are different for native and web
// are in the webhacks::State.
#[repr(C, align(4))]
#[derive(Clone)]
pub struct State {
    pub all_loaded: bool,
    pub frame_count: u32,
    pub rect: Rectangle,
    pub speed: f32,
    pub mouse_pos: Vector2,
    pub mouse_btn: u32,
    pub music: webhacks::Music,
    pub font: webhacks::Font,
    pub image: webhacks::Image,
    pub texture: webhacks::Texture,
    pub anim_blobs_N: u32, // not usize to keep a predictable alignment
    pub anim_blobs_arr: anim::Blobs, // as many as anim_frames
    pub test_N: u32,
    pub test_arr: *const u32,
}

#[no_mangle]
pub unsafe fn get_state_size() -> usize {
    std::mem::size_of::<State>()
}

#[no_mangle]
pub unsafe fn game_init() -> State {
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

    State {
        all_loaded: false,
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
        anim_blobs_N: 0,
        anim_blobs_arr: anim::null_blobs(),
        test_N: 3,
        test_arr: [1, 2, 3].as_ptr(),
    }
}

#[no_mangle]
pub unsafe fn game_load(state: &mut State) {
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
        // webhacks::log("image not loaded".to_string());
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
        } else {
            // texture is loaded! let's parse the animation
            let (anim_blobs_arr, anim_blobs_N) = anim::parse_anim(state.image);
            state.anim_blobs_arr = anim_blobs_arr;
            state.anim_blobs_N = anim_blobs_N as u32;
        }
    }

    if any_not_loaded {
        // webhacks::log("not all assets loaded".to_string());
    } else {
        state.all_loaded = true;
        // webhacks::log("all assets loaded".to_string());
    }

}


fn time_to_anim_frame(time: f64, frame_duration: f64, n_frames: u32) -> u32 {
    let frame = (time / frame_duration) as u32 % n_frames;
    frame
}


unsafe fn handle_keys(state: &mut State) {
    if raylib::IsKeyDown(KEY::Space) {
        state.speed = SPEED_BOOSTED
    }
    if !raylib::IsKeyDown(KEY::Space) {
        state.speed = SPEED_DEFAULT
    }

    let dt = raylib::GetFrameTime();
    if raylib::IsKeyDown(KEY::W) {
        state.rect.y -= dt * state.speed
    }
    if raylib::IsKeyDown(KEY::A) {
        state.rect.x -= dt * state.speed
    }
    if raylib::IsKeyDown(KEY::S) {
        state.rect.y += dt * state.speed
    }
    if raylib::IsKeyDown(KEY::D) {
        state.rect.x += dt * state.speed
    }

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

unsafe fn handle_mouse(state: &mut State) {
    let mut mouse_pos = raylib::GetMousePosition();
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

pub type GameFrame = unsafe fn(state: &mut State);
pub type GameLoad = unsafe fn(state: &mut State);

#[no_mangle]
pub unsafe fn game_frame(state: &mut State) {

    let t = webhacks::get_time();

    handle_keys(state);
    handle_mouse(state);

    raylib::BeginDrawing();
    {
        raylib::ClearBackground(DARKGREEN);
        webhacks::draw_text(state.font, "hello world", 250, 500, 50, RAYWHITE);

        let mut position = Vector2 {
            x: state.rect.x,
            y: state.rect.y,
        };
        // let rotation = 0.0;

        // figure out how to scale the texture to the size of the rect
        let width = webhacks::get_texture_width(state.texture);
        let height = webhacks::get_texture_height(state.texture);

        let scale = state.rect.width / width as f32;

        // Move the texture so it's at the bottom of the rect
        let scaled_height = height as f32 * scale;
        position.y += state.rect.height - scaled_height;

        // let tint = RAYWHITE;
        // webhacks::draw_texture_ex(texture, position, rotation, scale, tint);

        let anim_blobs = &state.anim_blobs_arr;
        let i= time_to_anim_frame(t, 0.1, state.anim_blobs_N as u32);
        
        #[cfg(feature = "native")]
        let blob = anim_blobs[i as usize];
        #[cfg(feature = "web")]
        let blob = unsafe { * anim_blobs.wrapping_add(i as usize) };
        
        let source = blob.to_rect();

        // raylib::DrawTexturePro(
        webhacks::draw_texture_pro(
            state.texture,
            source,
            Rectangle {
                x: position.x,
                y: position.y,
                width: state.rect.width,
                height: scaled_height,
            },
        );

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

        raylib::DrawCircle(
            state.mouse_pos.x as i32,
            state.mouse_pos.y as i32,
            10.0,
            color,
        );
    }
    raylib::EndDrawing();

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