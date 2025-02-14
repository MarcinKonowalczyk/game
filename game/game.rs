use raylib_wasm::{KeyboardKey as KEY, *};

const WINDOW_WIDTH: i32 = 800;
const WINDOW_HEIGHT: i32 = 600;

const SPEED_DEFAULT: f32 = 850.0;
const SPEED_BOOSTED: f32 = 1550.0;

#[cfg(feature = "native")]
pub struct State {
    rect: Rectangle,
    speed: f32,
    mouse_pos: Vector2,
    music: Option<Music>
}

#[cfg(feature = "web")]
pub struct State {
    rect: Rectangle,
    speed: f32,
    mouse_pos: Vector2,
    music: Option<u32>
}


#[cfg(feature = "web")]
unsafe extern "C" {
    pub fn InitAudioDevice();
    pub fn PlayMusicStream(music: u32);
    pub fn UpdateMusicStream(music: u32);
    pub fn LoadMusicStream(file_path: *const i8) -> u32;
    pub fn IsMusicReady(music: u32) -> bool;
}

// InitAudioDevice();

// // start playing the music on loop
// let music = LoadMusicStream(c"assets/hello_01.wav".as_ptr());
// PlayMusicStream(music);

#[no_mangle]
pub unsafe fn game_init() -> State {
    SetTargetFPS(144);
    init_window(WINDOW_WIDTH, WINDOW_HEIGHT, "game");

    InitAudioDevice();

    let filename = c"assets/hello_02.wav";
    let music = LoadMusicStream(filename.as_ptr());

    PlayMusicStream(music);

    State {
        rect: Rectangle {
            x: (WINDOW_WIDTH as f32 - 100.0)/2.0,
            y: (WINDOW_HEIGHT as f32 - 100.0)/2.0,
            width: 100.0,
            height: 100.0
        },
        speed: 850.0,
        mouse_pos: Vector2 { x: 0.0, y: 0.0 },
        music: Some(music)
    }
}

unsafe fn handle_keys(state: &mut State) {
    if IsKeyDown(KEY::Space)  { state.speed = SPEED_BOOSTED }
    if !IsKeyDown(KEY::Space) { state.speed = SPEED_DEFAULT }

    let dt = GetFrameTime();
    if IsKeyDown(KEY::W)      { state.rect.y -= dt*state.speed }
    if IsKeyDown(KEY::A)      { state.rect.x -= dt*state.speed }
    if IsKeyDown(KEY::S)      { state.rect.y += dt*state.speed }
    if IsKeyDown(KEY::D)      { state.rect.x += dt*state.speed }
}

unsafe fn handle_mouse(state: &mut State) {
    state.mouse_pos = GetMousePosition();    
}

pub type GameFrame = unsafe fn(state: &mut State);

#[no_mangle]
pub unsafe fn game_frame(state: &mut State) {
    handle_keys(state);
    handle_mouse(state);

    BeginDrawing(); {
        ClearBackground(DARKGREEN);
        draw_text("hello world", 250, 500, 50, RAYWHITE);

        DrawRectangleRec(state.rect, RAYWHITE);

        DrawFPS(WINDOW_WIDTH - 100, 10);

        let rect_pos = format!{
            "rect: [{x}, {y}]",
            x = state.rect.x.round(),
            y = state.rect.y.round()
        };
        draw_text(&rect_pos, 10, 10, 20, RAYWHITE);

        let mouse_pos = format!{
            "mouse: [{x}, {y}]",
            x = state.mouse_pos.x.round(),
            y = state.mouse_pos.y.round()
        };
        draw_text(&mouse_pos, 10, 30, 20, RAYWHITE);

        DrawCircle(state.mouse_pos.x as i32, state.mouse_pos.y as i32, 10.0, RAYWHITE);
    } EndDrawing();

    if state.music.is_some() {
        UpdateMusicStream(state.music.unwrap());
    }

}

#[no_mangle]
pub unsafe fn game_over() {
    CloseWindow();
}
