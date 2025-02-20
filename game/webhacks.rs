use raylib::{cstr, Color, Vector2};
use raylib_wasm as raylib;

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
        // pub fn IsMusicReady(music: u32) -> bool;
        pub fn IsMouseButtonDown(button: i32) -> bool;
        pub fn ConsoleLog_(msg: *const i8);
        pub fn LoadFont(file_path: *const i8) -> u32;
        pub fn DrawTextEx_(
            font: Font,
            text: *const i8,
            positionX: i32,
            positionY: i32,
            fontSize: i32,
            spacing: f32,
            tint: *const Color,
        );
        // pub fn LoadTexture_(file_path: *const i8) -> u32;
        // #[no_mangle]
        pub fn LoadTexture(file_path: *const i8) -> Texture;
        pub fn GetTextureWidth(texture: Texture) -> i32;
        pub fn GetTextureHeight(texture: Texture) -> i32;
        pub fn DrawTextureEx_(
            texture: Texture,
            positionX: i32,
            positionY: i32,
            rotation: f32,
            scale: f32,
            tint: *const Color,
        );
        pub fn GetTime() -> f64;
        pub fn LoadImageColors(image: Image) -> *mut Color;
        pub fn UnloadImageColors(colors: *mut Color, n: usize);
        pub fn GetImageWidth(image: Image) -> i32;
        pub fn GetImageHeight(image: Image) -> i32;
        pub fn DrawTexturePro_(
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
    }
}

#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[allow(non_snake_case, dead_code)]
pub fn log(msg: String) {
    #[cfg(feature = "web")]
    unsafe {
        ffi::ConsoleLog_(cstr!(msg))
    };
    #[cfg(feature = "native")]
    println!("{}", msg);
}

pub fn draw_text(font: Font, text: &str, x: i32, y: i32, size: i32, color: Color) {
    #[cfg(feature = "native")]
    unsafe {
        raylib::DrawTextEx(
            font,
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
    #[cfg(feature = "web")]
    unsafe {
        ffi::DrawTextEx_(font, cstr!(text), x, y, size, 2.0, addr_of!(color))
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

pub fn get_texture_height(texture: Texture) -> i32 {
    #[cfg(feature = "web")]
    unsafe {
        ffi::GetTextureHeight(texture)
    }
    #[cfg(feature = "native")]
    texture.height
}

pub fn get_texture_width(texture: Texture) -> i32 {
    #[cfg(feature = "web")]
    unsafe {
        ffi::GetTextureWidth(texture)
    }
    #[cfg(feature = "native")]
    texture.width
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
        ffi::DrawTexturePro_(texture, addr_of!(source_rec), addr_of!(dest_rec))
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
