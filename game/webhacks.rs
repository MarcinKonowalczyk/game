use raylib::{cstr, Color};
use raylib_wasm::{self as raylib};

#[cfg(feature = "native")]
use crate::log::VaList;

use crate::vec2::Vector2;

#[cfg(feature = "web")]
use std::ptr::addr_of;

#[cfg(feature = "web")]
pub type Image = u32;
#[cfg(feature = "native")]
pub type Image = raylib::Image;

#[cfg(feature = "web")]
pub type Music = u32;
#[cfg(feature = "native")]
pub type Music = raylib::Music;

#[cfg(feature = "web")]
pub type Font = u32;
#[cfg(feature = "native")]
pub type Font = raylib::Font;

#[cfg(feature = "web")]
pub type Texture = u32;
#[cfg(feature = "native")]
pub type Texture = raylib::Texture;

// 4-byte bool
#[derive(Copy, Clone, Debug)]
pub struct Bool {
    pub value: u32,
}

use std::ops::Not;

impl Not for Bool {
    type Output = Bool;

    fn not(self) -> Bool {
        Bool {
            value: if self.value == 0 { 1 } else { 0 },
        }
    }
}

impl Into<bool> for Bool {
    fn into(self) -> bool {
        self.value != 0
    }
}

impl From<Bool> for u32 {
    fn from(b: Bool) -> u32 {
        b.value
    }
}

impl Bool {
    #[allow(non_snake_case)]
    pub fn True() -> Bool {
        Bool { value: 1 }
    }

    #[allow(non_snake_case)]
    pub fn False() -> Bool {
        Bool { value: 0 }
    }

    pub fn toggle(&mut self) {
        self.value = if self.value == 0 { 1 } else { 0 };
    }
}

// #[cfg(feature = "web")]
// pub type Bool = u32;
// #[cfg(feature = "native")]
// pub type Bool = bool;

// #[cfg(feature = "web")]
// pub const TRUE: Bool = 1;
// #[cfg(feature = "native")]
// pub const TRUE: Bool = true;

// #[cfg(feature = "web")]
// pub const FALSE: Bool = 0;
// #[cfg(feature = "native")]
// pub const FALSE: Bool = false;

// All the external functions which we promise to implement on the javascript side
// Some stuff directly maps to raylib functions, and some stuff does not, and needs
// helper functions below.

#[cfg(feature = "web")]
pub mod ffi {
    use super::*;
    unsafe extern "C" {
        pub fn InitAudioDevice();
        pub fn PlayMusicStream(music: Music);
        pub fn UpdateMusicStream(music: Music);
        pub fn LoadMusicStream(file_path: *const i8) -> u32;
        pub fn IsMouseButtonDown(button: i32) -> bool;
        pub fn IsMouseButtonPressed(button: i32) -> bool;
        pub fn ConsoleLog(msg: *const i8, args: *const i8);
        pub fn Log(level: i32, msg: *const i8);
        pub fn LoadFont(file_path: *const i8) -> u32;
        pub fn DrawTextEx(
            font: Font,
            text: *const i8,
            position: *const Vector2,
            fontSize: i32,
            spacing: f32,
            tint: *const Color,
        );
        pub fn MeasureTextEx(font: Font, text: *const i8, fontSize: i32, spacing: f32) -> Vector2;
        pub fn LoadTexture(file_path: *const i8) -> Texture;
        pub fn GetTextureShape(texture: Texture) -> Vector2;
        pub fn DrawTextureEx(
            texture: Texture,
            position: *const Vector2,
            rotation: f32,
            scale: f32,
            tint: *const Color,
        );
        pub fn GetTime() -> f64;
        pub fn LoadImageColors(image: Image) -> *mut Color;
        pub fn UnloadImageColors(colors: *mut Color, n: usize);
        pub fn GetImageWidth(image: Image) -> i32;
        pub fn GetImageHeight(image: Image) -> i32;
        pub fn DrawTexturePro(
            texture: Texture,
            sourceRec: *const raylib::Rectangle,
            destRec: *const raylib::Rectangle,
        );
        pub fn UnloadImage(image: Image);
        pub fn LoadTextureFromImage(image: Image) -> Texture;
        pub fn LoadImage(file_path: *const i8) -> Image;
        pub fn IsMusicLoaded(music: Music) -> bool;
        pub fn IsFontLoaded(music: Font) -> bool;
        pub fn IsImageLoaded(music: Image) -> bool;
        pub fn IsTextureLoaded(music: Texture) -> bool;
        pub fn DrawLineEx(
            startPos: *const Vector2,
            endPos: *const Vector2,
            thickness: f32,
            color: *const Color,
        );
        pub fn SetMusicVolume(music: Music, volume: f32);
        pub fn IsKeyPressed(key: i32) -> bool;
        pub fn SetTraceLogCallback(callback_name: *const i8);
        pub fn SetTraceLogLevel(level: i32);
    }
}

#[allow(dead_code)]
pub fn draw_texture_ex(
    texture: Texture,
    position: Vector2,
    rotation: f32,
    scale: f32,
    tint: Color,
) {
    #[cfg(feature = "web")]
    unsafe {
        ffi::DrawTextureEx(texture, addr_of!(position), rotation, scale, addr_of!(tint))
    }
    #[cfg(feature = "native")]
    unsafe {
        raylib::DrawTextureEx(texture, position.into(), rotation, scale, tint)
    }
}

#[cfg(feature = "web")]
const SPECIAL: &str = "<END>";

#[allow(unused)]
pub fn _console_log_args(msg: &str, args: Option<Vec<&str>>) {
    #[cfg(feature = "web")]
    {
        let args = args.unwrap_or(vec![]);
        let mut args_str = args.join("\0");
        if !args.is_empty() {
            args_str.push_str("\0");
        }
        args_str.push_str(SPECIAL);
        args_str.push_str("\0");
        let c_args = args_str.as_ptr();
        unsafe { ffi::ConsoleLog(cstr!(msg), c_args as *const i8) };
    }
    // we should not use this function in native mode, but lets not fall over
    #[cfg(feature = "native")]
    panic!("console_log should not be called in native mode! use the game::log module instead");
}

// for now we support only strings as additional arguments
#[allow(unused)]
pub fn _console_log(msg: &str) {
    #[cfg(feature = "web")]
    unsafe {
        ffi::ConsoleLog(cstr!(msg), std::ptr::null());
    };
    #[cfg(feature = "native")]
    panic!("console_log should not be called in native mode! use the game::log module instead");
}

pub fn log(level: i32, msg: &str) {
    #[cfg(feature = "web")]
    unsafe {
        ffi::Log(level, cstr!(msg))
    };
    #[cfg(feature = "native")]
    unsafe {
        raylib::TraceLog(level, cstr!(msg));
    }
}

pub fn draw_text(font: Font, text: &str, position: Vector2, size: i32, spacing: f32, color: Color) {
    #[cfg(feature = "native")]
    unsafe {
        raylib::DrawTextEx(
            font,
            cstr!(text),
            position.into(),
            size as f32,
            spacing,
            color,
        );
    }
    #[cfg(feature = "web")]
    unsafe {
        ffi::DrawTextEx(
            font,
            cstr!(text),
            addr_of!(position),
            size,
            spacing,
            addr_of!(color),
        )
    }
}

pub fn update_music_stream(music: Music) {
    #[cfg(feature = "web")]
    unsafe {
        ffi::UpdateMusicStream(music)
    };
    #[cfg(feature = "native")]
    unsafe {
        raylib::UpdateMusicStream(music)
    };
}

pub fn get_texture_shape(texture: Texture) -> Vector2 {
    #[cfg(feature = "web")]
    unsafe {
        ffi::GetTextureShape(texture)
    }

    #[cfg(feature = "native")]
    {
        Vector2 {
            x: texture.width as f32,
            y: texture.height as f32,
        }
    }
}

pub fn is_mouse_button_down(button: i32) -> bool {
    #[cfg(feature = "web")]
    unsafe {
        ffi::IsMouseButtonDown(button)
    }
    #[cfg(feature = "native")]
    unsafe {
        raylib::IsMouseButtonDown(button)
    }
}

pub fn is_mouse_button_pressed(button: i32) -> bool {
    #[cfg(feature = "web")]
    unsafe {
        ffi::IsMouseButtonPressed(button)
    }
    #[cfg(feature = "native")]
    unsafe {
        raylib::IsMouseButtonPressed(button)
    }
}

#[allow(dead_code)]
pub fn load_texture(file_path: &str) -> Texture {
    #[cfg(feature = "web")]
    unsafe {
        ffi::LoadTexture(cstr!(file_path))
    }
    #[cfg(feature = "native")]
    unsafe {
        raylib::LoadTexture(cstr!(file_path))
    }
}

pub fn load_font(file_path: &str) -> Font {
    #[cfg(feature = "web")]
    unsafe {
        ffi::LoadFont(cstr!(file_path))
    }
    #[cfg(feature = "native")]
    unsafe {
        raylib::LoadFont(cstr!(file_path))
    }
}

pub fn play_music_stream(music: Music) {
    #[cfg(feature = "web")]
    unsafe {
        ffi::PlayMusicStream(music)
    }
    #[cfg(feature = "native")]
    unsafe {
        raylib::PlayMusicStream(music)
    }
}

pub fn load_music_stream(file_path: &str) -> Music {
    #[cfg(feature = "web")]
    unsafe {
        ffi::LoadMusicStream(cstr!(file_path))
    }
    #[cfg(feature = "native")]
    unsafe {
        raylib::LoadMusicStream(cstr!(file_path))
    }
}

pub fn init_audio_device() {
    #[cfg(feature = "web")]
    unsafe {
        ffi::InitAudioDevice()
    }
    #[cfg(feature = "native")]
    unsafe {
        raylib::InitAudioDevice()
    }
}

pub fn get_time() -> f64 {
    #[cfg(feature = "web")]
    unsafe {
        ffi::GetTime()
    }
    #[cfg(feature = "native")]
    unsafe {
        raylib::GetTime()
    }
}

pub fn load_image_colors(image: Image) -> *mut Color {
    #[cfg(feature = "web")]
    return unsafe { ffi::LoadImageColors(image) };
    #[cfg(feature = "native")]
    return unsafe { raylib::LoadImageColors(image) };
}

pub fn unload_image_colors(colors: *mut Color, #[allow(unused)] n: usize) {
    #[cfg(feature = "web")]
    unsafe {
        ffi::UnloadImageColors(colors, n);
    };
    #[cfg(feature = "native")]
    unsafe {
        raylib::UnloadImageColors(colors);
    };
}

pub fn get_image_width(image: Image) -> i32 {
    #[cfg(feature = "web")]
    return unsafe { ffi::GetImageWidth(image) };
    #[cfg(feature = "native")]
    return image.width;
}

pub fn get_image_height(image: Image) -> i32 {
    #[cfg(feature = "web")]
    return unsafe { ffi::GetImageHeight(image) };
    #[cfg(feature = "native")]
    return image.height;
}

pub fn draw_texture_pro(
    texture: Texture,
    source_rec: raylib::Rectangle,
    dest_rec: raylib::Rectangle,
    // origin: raylib::Vector2,
    // rotation: f32,
    // tint: raylib::Color,
) {
    #[cfg(feature = "web")]
    unsafe {
        ffi::DrawTexturePro(texture, addr_of!(source_rec), addr_of!(dest_rec))
    };
    #[cfg(feature = "native")]
    unsafe {
        raylib::DrawTexturePro(
            texture,
            source_rec,
            dest_rec,
            raylib::Vector2 { x: 0.0, y: 0.0 },
            0.0,
            raylib::RAYWHITE,
        );
    };
}

#[allow(dead_code)]
pub fn unload_image(image: Image) {
    #[cfg(feature = "web")]
    unsafe {
        ffi::UnloadImage(image);
    };
    #[cfg(feature = "native")]
    unsafe {
        raylib::UnloadImage(image);
    };
}

pub fn load_texture_from_image(image: Image) -> Texture {
    #[cfg(feature = "web")]
    unsafe {
        return ffi::LoadTextureFromImage(image);
    };
    #[cfg(feature = "native")]
    unsafe {
        return raylib::LoadTextureFromImage(image);
    };
}

pub fn load_image(file_path: &str) -> Image {
    #[cfg(feature = "web")]
    unsafe {
        return ffi::LoadImage(cstr!(file_path));
    };
    #[cfg(feature = "native")]
    unsafe {
        return raylib::LoadImage(cstr!(file_path));
    };
}

pub fn is_music_loaded(#[allow(unused)] music: Music) -> bool {
    #[cfg(feature = "web")]
    return unsafe { ffi::IsMusicLoaded(music) };
    #[cfg(feature = "native")]
    return true;
}

pub fn is_font_loaded(#[allow(unused)] font: Font) -> bool {
    #[cfg(feature = "web")]
    return unsafe { ffi::IsFontLoaded(font) };
    #[cfg(feature = "native")]
    return true;
}

pub fn is_image_loaded(#[allow(unused)] image: Image) -> bool {
    #[cfg(feature = "web")]
    return unsafe { ffi::IsImageLoaded(image) };
    #[cfg(feature = "native")]
    return true;
}

#[allow(unused)]
pub fn is_texture_loaded(texture: Texture) -> bool {
    #[cfg(feature = "web")]
    return unsafe { ffi::IsTextureLoaded(texture) };
    #[cfg(feature = "native")]
    return texture.id != 0;
}

pub fn null_font() -> Font {
    #[cfg(feature = "web")]
    return 0;
    #[cfg(feature = "native")]
    return raylib::Font {
        baseSize: 0,
        glyphCount: 0,
        glyphPadding: 0,
        texture: null_texture(),
        recs: std::ptr::null_mut(),
        glyphs: std::ptr::null_mut(),
    };
}

pub fn null_texture() -> Texture {
    #[cfg(feature = "web")]
    return 0;
    #[cfg(feature = "native")]
    return raylib::Texture {
        id: 0,
        width: 0,
        height: 0,
        mipmaps: 0,
        format: 0,
    };
}

pub fn null_music() -> Music {
    #[cfg(feature = "web")]
    return 0;
    #[cfg(feature = "native")]
    return raylib::Music {
        stream: raylib::AudioStream {
            buffer: std::ptr::null_mut(),
            processor: std::ptr::null_mut(),
            sampleRate: 0,
            sampleSize: 0,
            channels: 0,
        },
        frameCount: 0,
        looping: false,
        ctxType: 0,
        ctxData: std::ptr::null_mut(),
    };
}

pub fn null_image() -> Image {
    #[cfg(feature = "web")]
    return 0;
    #[cfg(feature = "native")]
    return raylib::Image {
        data: std::ptr::null_mut(),
        width: 0,
        height: 0,
        mipmaps: 0,
        format: 0,
    };
}

pub fn draw_line_ex(start_pos: Vector2, end_pos: Vector2, thickness: f32, color: Color) {
    #[cfg(feature = "web")]
    unsafe {
        ffi::DrawLineEx(
            addr_of!(start_pos),
            addr_of!(end_pos),
            thickness,
            addr_of!(color),
        );
    }
    #[cfg(feature = "native")]
    unsafe {
        raylib::DrawLineEx(start_pos.into(), end_pos.into(), thickness, color);
    }
}

pub fn draw_circle(mouse_pos: Vector2, radius: f32, color: Color) {
    unsafe { raylib::DrawCircle(mouse_pos.x as i32, mouse_pos.y as i32, radius, color) }
}

pub fn get_mouse_position() -> Vector2 {
    unsafe { raylib::GetMousePosition() }.into()
}

pub fn set_music_volume(music: Music, volume: f32) {
    #[cfg(feature = "web")]
    unsafe {
        ffi::SetMusicVolume(music, volume);
    }
    #[cfg(feature = "native")]
    unsafe {
        raylib::SetMusicVolume(music, volume);
    }
}

pub fn is_key_pressed(key: raylib::KeyboardKey) -> bool {
    #[cfg(feature = "web")]
    unsafe {
        ffi::IsKeyPressed(key as i32)
    }
    #[cfg(feature = "native")]
    unsafe {
        raylib::IsKeyPressed(key)
    }
}

pub fn measure_text(font: Font, text: &str, font_size: i32, spacing: f32) -> Vector2 {
    #[cfg(feature = "web")]
    unsafe {
        ffi::MeasureTextEx(font, cstr!(text), font_size, spacing).into()
    }
    #[cfg(feature = "native")]
    unsafe { raylib::MeasureTextEx(font, cstr!(text), font_size as f32, spacing) }.into()
}

#[cfg(feature = "native")]
pub type LogCallback = unsafe extern "C" fn(i32, *const i8, *mut VaList);

#[cfg(feature = "web")]
pub type LogCallback = fn(i32, *const i8);

#[allow(unused)]
pub fn set_trace_log_callback(callback: Option<LogCallback>, callback_name: &str) {
    #[cfg(feature = "web")]
    unsafe {
        ffi::SetTraceLogCallback(cstr!(callback_name));
    }

    #[cfg(feature = "native")]
    unsafe {
        raylib::SetTraceLogCallback(callback);
    }
}

#[allow(unused)]
pub fn set_log_level(level: i32) {
    #[cfg(feature = "web")]
    unsafe {
        ffi::SetTraceLogLevel(level);
    }
    #[cfg(feature = "native")]
    unsafe {
        raylib::SetTraceLogLevel(level);
    }
}
