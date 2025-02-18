use raylib::{KeyboardKey as KEY, MouseButton, Rectangle, Vector2, DARKGREEN, RAYWHITE, RED};
use raylib_wasm::{self as raylib, cstr, GetTime};

mod webhacks;

mod anim;

const WINDOW_WIDTH: i32 = 800;
const WINDOW_HEIGHT: i32 = 600;

const SPEED_DEFAULT: f32 = 850.0;
const SPEED_BOOSTED: f32 = 1550.0;


// All of the state that we need to keep track of in the game. The bits which are different for native and web
// are in the webhacks::State.
pub struct State {
    pub frame_count: u32,
    pub rect: Rectangle,
    pub speed: f32,
    pub mouse_pos: Vector2,
    pub mouse_btn: bool,
    pub webhacks: webhacks::State,
    pub anim_blobs: Option<Vec<anim::Blob>>,
}


#[no_mangle]
pub unsafe fn game_init() -> State {
    webhacks::log("game_init".to_string());

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
    webhacks::log("loading texture from rust".to_string());
    let image = raylib::LoadImage(cstr!("assets/Blue_Slime-Idle-mag.png"));

    let texture = raylib::LoadTextureFromImage(image);
    // let texture = webhacks::load_texture("assets/Blue_Slime-Idle-mag.png");
    webhacks::log("loaded texture from rust".to_string());

    let anim_blobs = anim::parse_anim(image);

    raylib::UnloadImage(image);

    State {
        frame_count: 0,
        rect: Rectangle {
            x: (WINDOW_WIDTH as f32 - 100.0) / 2.0,
            y: (WINDOW_HEIGHT as f32 - 100.0) / 2.0,
            width: 100.0,
            height: 100.0,
        },
        speed: 850.0,
        mouse_pos: Vector2 { x: 0.0, y: 0.0 },
        mouse_btn: false,
        webhacks: webhacks::State {

            music: Some(music),
            font: Some(font),
            texture: Some(texture),
        },
        anim_blobs: Some(anim_blobs),
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
    state.mouse_btn = webhacks::is_mouse_button_down(MouseButton::Left as i32);
}

pub type GameFrame = unsafe fn(state: &mut State);

#[no_mangle]
pub unsafe fn game_frame(state: &mut State) {

    let t = GetTime();

    handle_keys(state);
    handle_mouse(state);

    raylib::BeginDrawing();
    {
        raylib::ClearBackground(DARKGREEN);
        webhacks::draw_text(state.webhacks.font, "hello world", 250, 500, 50, RAYWHITE);

        if state.webhacks.texture.is_some() {
            let texture = state.webhacks.texture.unwrap();
            let mut position = Vector2 {
                x: state.rect.x,
                y: state.rect.y,
            };
            let rotation = 0.0;

            // figure out how to scale the texture to the size of the rect
            #[cfg(feature = "web")]
            let width = webhacks::get_texture_width(texture);
            #[cfg(feature = "native")]
            let width = texture.width;

            #[cfg(feature = "web")]
            let height = webhacks::get_texture_height(texture);
            #[cfg(feature = "native")]
            let height = texture.height;

            let scale = state.rect.width / width as f32;

            // Move the texture so it's at the bottom of the rect
            let scaled_height = height as f32 * scale;
            position.y += state.rect.height - scaled_height;

            let tint = RAYWHITE;
            // webhacks::draw_texture_ex(texture, position, rotation, scale, tint);

            let anim_blobs = state.anim_blobs.as_ref().unwrap();
            let i= time_to_anim_frame(t, 0.1, anim_blobs.len() as u32);
            let source = anim_blobs[i as usize].to_rect();

            raylib::DrawTexturePro(
                texture,
                source,
                Rectangle {
                    x: position.x,
                    y: position.y,
                    width: state.rect.width,
                    height: scaled_height,
                },
                Vector2 { x: 0.0, y: 0.0 },
                rotation,
                tint,
            );
        } else {
            raylib::DrawRectangle(
                state.rect.x as i32,
                state.rect.y as i32,
                state.rect.width as i32,
                state.rect.height as i32,
                RAYWHITE,
            );
        }

        let rect_pos = format! {
            "rect: [{x}, {y}]",
            x = state.rect.x.round(),
            y = state.rect.y.round()
        };
        webhacks::draw_text(state.webhacks.font, &rect_pos, 10, 10, 20, RAYWHITE);

        let mouse_pos = format! {
            "mouse: [{x}, {y}]",
            x = state.mouse_pos.x.round(),
            y = state.mouse_pos.y.round()
        };
        webhacks::draw_text(state.webhacks.font, &mouse_pos, 10, 30, 20, RAYWHITE);

        let color = if state.mouse_btn { RED } else { RAYWHITE };

        raylib::DrawCircle(
            state.mouse_pos.x as i32,
            state.mouse_pos.y as i32,
            10.0,
            color,
        );
    }
    raylib::EndDrawing();

    // Update the music stream
    if state.webhacks.music.is_some() {
        webhacks::update_music_stream(state.webhacks.music.unwrap());
    }

    // Update the frame count
    state.frame_count += 1;

}

#[no_mangle]
pub unsafe fn game_over() {
    raylib::CloseWindow();
}
