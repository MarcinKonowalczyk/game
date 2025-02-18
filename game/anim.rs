use raylib_wasm::{self as raylib, Color};
use crate::webhacks;

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

#[cfg(feature = "native")]
pub type Blobs = Vec<Blob>;
#[cfg(feature = "web")]
pub type Blobs = *const Blob;

pub fn parse_anim(image: webhacks::Image) -> (Blobs, usize) {

    let vec_blobs = find_blobs(image);
    let num = vec_blobs.len();

    #[cfg(feature = "native")]
    let blobs = vec_blobs;

    #[cfg(feature = "web")]
    let blobs = vec_blobs.as_ptr();

    return (blobs, num);
}

////////////////////////

const MAGENTA: Color = Color { r: 255, g: 0, b: 255, a: 255 };

fn is_magenta(color: Color) -> bool {
    color.r == MAGENTA.r && color.g == MAGENTA.g && color.b == MAGENTA.b
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Blob {
    pub x_min: usize,
    pub y_min: usize,
    pub x_max: usize,
    pub y_max: usize,
}

impl Blob {
    pub fn width(&self) -> usize {
        return self.x_max - self.x_min + 1;
    }

    pub fn height(&self) -> usize {
        return self.y_max - self.y_min + 1;
    }

    pub fn to_rect(&self) -> raylib::Rectangle {
        return raylib::Rectangle {
            x: self.x_min as f32,
            y: self.y_min as f32,
            width: self.width() as f32,
            height: self.height() as f32
        }
    }
}

struct FindBlobsData {
    colors: Vec<Color>,
    width: usize,
    height: usize,
    visited: Vec<bool>,
    stack: Vec<(usize, usize)>
}

fn image_to_colors(image: webhacks::Image) -> (Vec<Color>, usize, usize) {
    let width = webhacks::get_image_width(image) as usize;
    let height = webhacks::get_image_height(image) as usize;
    
    let n = width * height;
    
    let _colors = webhacks::load_image_colors(image);
    // defer! { unsafe { raylib::UnloadImageColors(_colors) } }
    defer! { webhacks::unload_image_colors(_colors) }

    // Create a new rust vec from the raw pointer
    // let colors = unsafe { Vec::from_raw_parts(_colors, n, n) };
    // let colors = mem::ManuallyDrop::new(colors);
    // return colors;

    // Copy the colors into a Rust Vec
    let mut colors = Vec::with_capacity(n);
    for i in 0..n {
        colors.push(unsafe { *_colors.add(i) });
    }

    return (colors, width, height);
}

impl FindBlobsData {
    fn from_image(image: webhacks::Image) -> FindBlobsData {
        let (colors, width, height) = image_to_colors(image);
        let visited = vec![false; colors.len()];
        let stack = Vec::new();
        return FindBlobsData { colors, width, height, visited, stack };
    }
}

impl FindBlobsData {
    #[allow(dead_code)]
    fn at(&self, x: usize, y: usize) -> Color {
        return self.colors[x + y * self.width];
    }

    #[allow(dead_code)]
    fn len(&self) -> usize {
        return self.colors.len();
    }
}

impl FindBlobsData {
    fn visit(&mut self, x: usize, y: usize) -> Option<Color> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let i = x as usize + y as usize * self.width;
        if i >= self.visited.len() {
            return None;
        }

        if self.visited[i] {
            return None;
        }
        self.visited[i] = true;
        let color = self.colors[i];

        if is_magenta(color) {
            return None;
        }

        return Some(color);
    }
}

impl FindBlobsData {
    fn append_neighbours(&mut self, x: usize, y: usize) {
        if x > 0 {
            self.stack.push((x - 1, y));
        }
        if x < self.width - 1 {
            self.stack.push((x + 1, y));
        }
        if y > 0 {
            self.stack.push((x, y - 1));
        }
        if y < self.height - 1 {
            self.stack.push((x, y + 1));
        }
    }

    fn clear_stack(&mut self) {
        self.stack.clear();
    }
}

fn find_blobs(image: webhacks::Image) -> Vec<Blob> {
    let mut blobs = Vec::new();
    let width = webhacks::get_image_width(image);
    let height = webhacks::get_image_height(image);
    if width <= 2 || height <= 2 {
        // Texture too small. Definitely not a sprite sheet. Return empty list.
        return blobs;
    }

    let mut dat = FindBlobsData::from_image(image);

    // Find all blobs.
    for x in 0..dat.width {
        for y in 0..dat.height {
            let color = dat.visit(x, y);
            if color.is_none() {
                continue;
            }

            let mut blob = Blob { x_min: x, y_min: y, x_max: x, y_max: y };

            // Flood fill the blob
            dat.clear_stack();
            dat.append_neighbours(x, y);

            while !dat.stack.is_empty() {
                let (x, y) = dat.stack.pop().unwrap();
                let color = dat.visit(x, y);
                if color.is_none() {
                    continue;
                }

                blob.x_min = blob.x_min.min(x);
                blob.y_min = blob.y_min.min(y);
                blob.x_max = blob.x_max.max(x);
                blob.y_max = blob.y_max.max(y);

                dat.append_neighbours(x, y);
            }
            
            blobs.push(blob);
        }
    }

    // println!("Found {} blobs.", i);
    // for (i, blob) in blobs.iter().enumerate() {
    //     println!("Blob {} at ({}, {}) to ({}, {})", i, blob.x_min, blob.y_min, blob.x_max, blob.y_max);
    // }

    // Sort the blobs by first y, then x
    blobs.sort_by(|a, b| {
        if a.y_min != b.y_min {
            return a.y_min.cmp(&b.y_min);
        }
        return a.x_min.cmp(&b.x_min);
    });

    return blobs;

}