use raylib_wasm::{self as raylib, Color};

////////////////////////
struct ScopeCall<F: FnMut()> {
    c: F
}

impl<F: FnMut()> Drop for ScopeCall<F> {
    fn drop(&mut self) {
        (self.c)();
    }
}
    
macro_rules! defer {
    ($e:expr) => (
        let _scope_call = ScopeCall { c: || -> () { $e; } };
    )
}

////////////////////////

const MAGENTA: Color = Color { r: 255, g: 0, b: 255, a: 255 };

fn is_magenta(color: Color) -> bool {
    color.r == MAGENTA.r && color.g == MAGENTA.g && color.b == MAGENTA.b
}

pub struct Sprite {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

pub fn parse_anim(image: raylib::Image) -> Vec<Sprite> {
    
    let sprites = Vec::new();
    if image.width <= 2 || image.height <= 2 {
        // Texture too small. Definitely not a sprite sheet. Return empty list..
        return sprites;
    }

    // We need to go through all the pixels in the texture and figure out where
    // the sprites are. The background will necessarily be magenta (ff00ff).
    // To start with, lets just check that the top row of the texture is
    // all magenta.

    let colors = unsafe { raylib::LoadImageColors(image) };
    let N = image.width as usize * image.height as usize;

    defer! {
        {
            unsafe { raylib::UnloadImageColors(colors) };
            // println!("Unloaded image colors.");
        }
    }

    let mut i = 0;

    for x in 0..image.width {
        for y in 0..image.height {
            let color = colors.wrapping_add(i);
            if !is_magenta( unsafe { *color } ) {
                break;
            }
            i += 1;
        }
    }

    if i > N {
        println!("Texture is all magenta. No sprites found.");
        return sprites;
    }

    println!("Found non-magenta pixel at ({}, {}).", i % image.width as usize, i / image.width as usize);

    return sprites;

}