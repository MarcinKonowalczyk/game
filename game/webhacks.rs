use raylib::{cstr, Color, Rectangle, Vector2};
use raylib_wasm as raylib;

#[cfg(feature = "web")]
use std::ptr::addr_of;

// All of the state that we need to keep track of in the game
// This is subtly different between the native and web versions,
// since we use u32 ids for web
#[cfg(feature = "native")]
pub struct State {
    pub rect: Rectangle,
    pub speed: f32,
    pub mouse_pos: Vector2,
    pub mouse_btn: bool,
    pub music: Option<raylib::Music>,
    pub font: Option<raylib::Font>,
    pub texture: Option<raylib::Texture>,
}

#[cfg(feature = "web")]
pub struct State {
    pub rect: Rectangle,
    pub speed: f32,
    pub mouse_pos: Vector2,
    pub mouse_btn: bool,
    pub music: Option<u32>,
    pub font: Option<u32>,
    pub texture: Option<u32>,
}

// All the external functions which we promise to implement on the javascript side
// Some stuff directly maps to raylib functions, and some stuff does not, and needs
// helper functions below.

#[cfg(feature = "web")]
pub mod ffi {
    use super::*;
    unsafe extern "C" {
        pub fn InitAudioDevice();
        pub fn PlayMusicStream(music: u32);
        pub fn UpdateMusicStream(music: u32);
        pub fn LoadMusicStream(file_path: *const i8) -> u32;
        // pub fn IsMusicReady(music: u32) -> bool;
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
        // pub fn LoadTexture_(file_path: *const i8) -> u32;
        // #[no_mangle]
        pub fn LoadTexture(file_path: *const i8) -> u32;
        pub fn GetTextureWidth(texture: u32) -> i32;
        pub fn GetTextureHeight(texture: u32) -> i32;
        pub fn DrawTextureEx_(
            texture: u32,
            positionX: i32,
            positionY: i32,
            rotation: f32,
            scale: f32,
            tint: *const Color,
        );
    }
}

#[cfg(feature = "web")]
pub fn draw_texture_ex(texture: u32, position: Vector2, rotation: f32, scale: f32, tint: Color) {
    unsafe {
        ffi::DrawTextureEx_(
            texture,
            position.x as i32,
            position.y as i32,
            rotation,
            scale,
            addr_of!(tint),
        )
    }
}

#[cfg(feature = "native")]
pub fn draw_texture_ex(
    texture: raylib::Texture,
    position: Vector2,
    rotation: f32,
    scale: f32,
    tint: Color,
) {
    unsafe { raylib::DrawTextureEx(texture, position, rotation, scale, tint) }
}

#[allow(non_snake_case)]
pub fn log(msg: String) {
    #[cfg(feature = "web")]
    unsafe {
        ffi::ConsoleLog_(cstr!(msg))
    };
    #[cfg(feature = "native")]
    println!("{}", msg);
}

#[cfg(feature = "native")]
pub fn draw_text(font: Option<raylib::Font>, text: &str, x: i32, y: i32, size: i32, color: Color) {
    if font.is_none() {
        unsafe {
            raylib::DrawText(cstr!(text), x, y, size, color);
        }
    } else {
        unsafe {
            raylib::DrawTextEx(
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
pub fn draw_text(font: Option<u32>, text: &str, x: i32, y: i32, size: i32, color: Color) {
    unsafe {
        if font.is_none() {
            raylib::DrawText(cstr!(text), x, y, size, color);
        } else {
            ffi::DrawTextEx_(font.unwrap(), cstr!(text), x, y, size, 2.0, addr_of!(color))
        }
    }
}

#[cfg(feature = "web")]
pub fn update_music_stream(music: u32) {
    unsafe { ffi::UpdateMusicStream(music) };
}

#[cfg(feature = "native")]
pub fn update_music_stream(music: raylib::Music) {
    unsafe { raylib::UpdateMusicStream(music) };
}

#[cfg(feature = "web")]
pub fn get_texture_height(texture: u32) -> i32 {
    unsafe { ffi::GetTextureHeight(texture) }
}

#[cfg(feature = "web")]
pub fn get_texture_width(texture: u32) -> i32 {
    unsafe { ffi::GetTextureWidth(texture) }
}

#[cfg(feature = "web")]
pub fn is_mouse_button_down(button: i32) -> bool {
    unsafe { ffi::IsMouseButtonDown(button) }
}

#[cfg(feature = "native")]
pub fn is_mouse_button_down(button: i32) -> bool {
    unsafe { raylib::IsMouseButtonDown(button) }
}

#[cfg(feature = "web")]
pub fn load_texture(file_path: &str) -> u32 {
    unsafe { ffi::LoadTexture(cstr!(file_path)) }
}

#[cfg(feature = "native")]
pub fn load_texture(file_path: &str) -> raylib::Texture {
    unsafe { raylib::LoadTexture(cstr!(file_path)) }
}

#[cfg(feature = "web")]
pub fn load_font(file_path: &str) -> u32 {
    unsafe { ffi::LoadFont(cstr!(file_path)) }
}

#[cfg(feature = "native")]
pub fn load_font(file_path: &str) -> raylib::Font {
    unsafe { raylib::LoadFont(cstr!(file_path)) }
}

#[cfg(feature = "web")]
pub fn play_music_stream(music: u32) {
    unsafe { ffi::PlayMusicStream(music) }
}

#[cfg(feature = "native")]
pub fn play_music_stream(music: raylib::Music) {
    unsafe { raylib::PlayMusicStream(music) }
}

#[cfg(feature = "web")]
pub fn load_music_stream(file_path: &str) -> u32 {
    unsafe { ffi::LoadMusicStream(cstr!(file_path)) }
}

#[cfg(feature = "native")]
pub fn load_music_stream(file_path: &str) -> raylib::Music {
    unsafe { raylib::LoadMusicStream(cstr!(file_path)) }
}

#[cfg(feature = "web")]
pub fn init_audio_device() {
    unsafe { ffi::InitAudioDevice() }
}

#[cfg(feature = "native")]
pub fn init_audio_device() {
    unsafe { raylib::InitAudioDevice() }
}
