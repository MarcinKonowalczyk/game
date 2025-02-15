use raylib_wasm::{KeyboardKey as KEY, *};
// use crate::small_c_string::run_with_cstr;
// use std::sys::pal::common::small_c_string::run_with_cstr;

#[cfg(feature = "web")]
use std::ptr::addr_of;

const WINDOW_WIDTH: i32 = 800;
const WINDOW_HEIGHT: i32 = 600;

const SPEED_DEFAULT: f32 = 850.0;
const SPEED_BOOSTED: f32 = 1550.0;

#[cfg(feature = "native")]
pub struct State {
    rect: Rectangle,
    speed: f32,
    mouse_pos: Vector2,
    mouse_btn: bool,
    music: Option<Music>,
    font: Option<Font>,
    texture: Option<Texture>,
}

#[cfg(feature = "web")]
pub struct State {
    rect: Rectangle,
    speed: f32,
    mouse_pos: Vector2,
    mouse_btn: bool,
    music: Option<u32>,
    font: Option<u32>,
    texture: Option<Texture>,
}

#[cfg(feature = "web")]
unsafe extern "C" {
    pub fn InitAudioDevice();
    pub fn PlayMusicStream(music: u32);
    pub fn UpdateMusicStream(music: u32);
    pub fn LoadMusicStream(file_path: *const i8) -> u32;
    pub fn IsMusicReady(music: u32) -> bool;
    pub fn IsMouseButtonDown(button: i32) -> bool;
    pub fn ConsoleLog_(msg: *const i8);
    pub fn LoadFont(file_path: *const i8) -> u32;
    pub fn DrawTextEx_(
        font: u32,
        text: *const i8,
        positionX: i32,
        positionY: i32,
        fontSize: i32,
        spacing: f32,
        tint: *const Color,
    );
}

#[cfg(feature = "web")]
#[allow(non_snake_case)]
pub fn DrawTextEx(
    font: u32,
    text: *const i8,
    position: Vector2,
    fontSize: f32,
    spacing: f32,
    tint: Color,
) {
    unsafe {
        DrawTextEx_(
            font,
            text,
            position.x as i32,
            position.y as i32,
            fontSize as i32,
            spacing,
            addr_of!(tint),
        )
    }
}

#[allow(non_snake_case)]
pub fn ConsoleLog(msg: String) {
    #[cfg(feature = "web")]
    unsafe {
        ConsoleLog_(cstr!(msg))
    };
    #[cfg(feature = "native")]
    println!("{}", msg);
}

#[cfg(feature = "native")]
fn draw_text(font: Option<Font>, text: &str, x: i32, y: i32, size: i32, color: Color) {
    if font.is_none() {
        unsafe {
            DrawText(cstr!(text), x, y, size, color);
        }
    } else {
        unsafe {
            DrawTextEx(
                font.unwrap(),
                cstr!(text),
                Vector2 {
                    x: x as f32,
                    y: y as f32,
                },
                size as f32,
                2.0,
                color,
            );
        }
    }
}

#[cfg(feature = "web")]
fn draw_text(font: Option<u32>, text: &str, x: i32, y: i32, size: i32, color: Color) {
    if font.is_none() {
        unsafe {
            DrawText(cstr!(text), x, y, size, color);
        }
    } else {
        DrawTextEx(
            font.unwrap(),
            cstr!(text),
            Vector2 {
                x: x as f32,
                y: y as f32,
            },
            size as f32,
            2.0,
            color,
        );
    }
}

#[no_mangle]
pub unsafe fn game_init() -> State {

    // We do not cap the framerate, since it leads to sluggish mouse input, since raylib cannot detect mouse input
    // between the frames and we don't really want to dig down to the GLFW layer and poll for events ourselves.
    // See: https://github.com/raysan5/raylib/issues/3354
    // Apparently this is known and solutions are unplanned. I guess it's not that much of a problem from C.

    // SetTargetFPS(300);
    
    init_window(WINDOW_WIDTH, WINDOW_HEIGHT, "game");

    InitAudioDevice();

    let filename = c"assets/hello_03.wav";
    let music = LoadMusicStream(filename.as_ptr());

    // SetMusicVolume(music, 1.0);

    PlayMusicStream(music);

    let font = LoadFont(cstr!("assets/Kavoon-Regular.ttf"));

    // Load slime texture
    // Blue_Slime-Idle-mag.png
    let texture = LoadTexture(cstr!("assets/Blue_Slime-Idle-mag.png"));

    State {
        rect: Rectangle {
            x: (WINDOW_WIDTH as f32 - 100.0) / 2.0,
            y: (WINDOW_HEIGHT as f32 - 100.0) / 2.0,
            width: 100.0,
            height: 100.0,
        },
        speed: 850.0,
        mouse_pos: Vector2 { x: 0.0, y: 0.0 },
        mouse_btn: false,
        music: Some(music),
        font: Some(font),
        texture: Some(texture),
    }
}

unsafe fn handle_keys(state: &mut State) {
    if IsKeyDown(KEY::Space) {
        state.speed = SPEED_BOOSTED
    }
    if !IsKeyDown(KEY::Space) {
        state.speed = SPEED_DEFAULT
    }

    let dt = GetFrameTime();
    if IsKeyDown(KEY::W) {
        state.rect.y -= dt * state.speed
    }
    if IsKeyDown(KEY::A) {
        state.rect.x -= dt * state.speed
    }
    if IsKeyDown(KEY::S) {
        state.rect.y += dt * state.speed
    }
    if IsKeyDown(KEY::D) {
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
    state.mouse_pos = GetMousePosition();
    state.mouse_btn = IsMouseButtonDown(MouseButton::Left as i32);
}

pub type GameFrame = unsafe fn(state: &mut State);

#[no_mangle]
pub unsafe fn game_frame(state: &mut State) {
    handle_keys(state);
    handle_mouse(state);

    BeginDrawing();
    {
        ClearBackground(DARKGREEN);
        draw_text(state.font, "hello world", 250, 500, 50, RAYWHITE);

        if state.texture.is_some() {
            let texture = state.texture.unwrap();
            let mut position = Vector2 {
                x: state.rect.x,
                y: state.rect.y,
            };
            let rotation = 0.0;
            // figure out how to scale the texture to the size of the rect
            let scale = state.rect.width / texture.width as f32;

            // Move the texture so it's at the bottom of the rect
            let scaled_height = texture.height as f32 * scale;
            position.y += state.rect.height - scaled_height;

            let tint = RAYWHITE;
            DrawTextureEx(texture, position, rotation, scale, tint);
        } else {
            DrawRectangle(
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
        draw_text(state.font, &rect_pos, 10, 10, 20, RAYWHITE);

        let mouse_pos = format! {
            "mouse: [{x}, {y}]",
            x = state.mouse_pos.x.round(),
            y = state.mouse_pos.y.round()
        };
        draw_text(state.font, &mouse_pos, 10, 30, 20, RAYWHITE);

        let color = if state.mouse_btn { RED } else { RAYWHITE };

        DrawCircle(
            state.mouse_pos.x as i32,
            state.mouse_pos.y as i32,
            10.0,
            color,
        );
    }
    EndDrawing();

    if state.music.is_some() {
        UpdateMusicStream(state.music.unwrap());
    }
}

#[no_mangle]
pub unsafe fn game_over() {
    CloseWindow();
}
