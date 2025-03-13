use std::vec;

use crate::anim;
use crate::defer;
use crate::vec2::Vector2;
use crate::webhacks;

use raylib_wasm::{self as raylib, Color};

const MAGENTA: Color = Color {
    r: 255,
    g: 0,
    b: 255,
    a: 255,
};

fn is_magenta(color: Color) -> bool {
    color.r == MAGENTA.r && color.g == MAGENTA.g && color.b == MAGENTA.b
}

#[repr(C, align(4))]
#[derive(Debug, Copy, Clone)]
pub struct Blob {
    pub x_min: u32,
    pub y_min: u32,
    pub x_max: u32,
    pub y_max: u32,
}

impl Blob {
    pub fn width(&self) -> usize {
        (self.x_max - self.x_min + 1) as usize
    }

    pub fn height(&self) -> usize {
        (self.y_max - self.y_min + 1) as usize
    }

    pub fn to_rect(&self) -> raylib::Rectangle {
        raylib::Rectangle {
            x: self.x_min as f32,
            y: self.y_min as f32,
            width: self.width() as f32,
            height: self.height() as f32,
        }
    }
}

pub struct Metablob {
    pad_blob: u32,
    #[allow(dead_code)]
    anchor: anim::Anchor,
}

struct FindBlobsData {
    colors: Vec<Color>,
    width: usize,
    height: usize,
    visited: Vec<bool>,
    stack: Vec<(usize, usize)>,
}

fn image_to_colors(image: webhacks::Image) -> (Vec<Color>, usize, usize) {
    let width = webhacks::get_image_width(image) as usize;
    let height = webhacks::get_image_height(image) as usize;

    let n = width * height;

    let _colors = webhacks::load_image_colors(image);
    defer! { webhacks::unload_image_colors(_colors, n * std::mem::size_of::<Color>()) }

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
        return FindBlobsData {
            colors,
            width,
            height,
            visited,
            stack,
        };
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

const MAGIC: u8 = 0b10101010;

enum MetablobPixel {
    Black,
    White,
    Transparent,
    Wrong,
}

fn metablob_pixel_sorter(color: Color) -> MetablobPixel {
    if color.a == 0 {
        return MetablobPixel::Transparent;
    }
    if color.r == 0 && color.g == 0 && color.b == 0 {
        return MetablobPixel::Black;
    }
    if color.r == 255 && color.g == 255 && color.b == 255 {
        return MetablobPixel::White;
    }
    return MetablobPixel::Wrong;
}

fn try_parse_as_metablob(dat: &FindBlobsData, blob: &Blob) -> Option<Metablob> {
    let mut bdat = vec![];
    for y in blob.y_min..=blob.y_max {
        for x in blob.x_min..=blob.x_max {
            let color = dat.at(x as usize, y as usize);
            bdat.push(match metablob_pixel_sorter(color) {
                MetablobPixel::Black => false,
                MetablobPixel::White => true,
                MetablobPixel::Transparent => continue, // ignore transparent pixels
                MetablobPixel::Wrong => return None,    // not a metablob
            });
        }
    }

    // convert the bools to bytes
    let bdat = bdat
        .chunks(8)
        .map(|chunk| {
            let mut byte = 0;
            for (i, &bit) in chunk.iter().enumerate() {
                if bit {
                    byte |= 1 << i;
                }
            }
            byte
        })
        .collect::<Vec<u8>>();

    if bdat.len() < 1 || bdat[0] != MAGIC {
        return None;
    }

    // read data length (two bytes, big endian)
    let data_len = u16::from_be_bytes([bdat[1], bdat[2]]) as usize;

    if data_len + 3 > bdat.len() {
        // invalid data length
        return None;
    }

    let pad_blob = bdat[3] as u32;
    let anchor = Anchor::TopLeft;

    // Anchor is not implemented yet. just ignore it
    let _anchor: Result<anim::Anchor, ()> = bdat[1].try_into();

    Some(Metablob { pad_blob, anchor })
}

pub fn find_blobs(image: webhacks::Image) -> (Vec<Blob>, Option<Metablob>) {
    let mut blobs = Vec::new();
    let mut metablob = None;
    let width = webhacks::get_image_width(image);
    let height = webhacks::get_image_height(image);

    if width <= 2 || height <= 2 {
        // Texture too small. Definitely not a sprite sheet. Return empty list.
        return (blobs, metablob);
    }

    let mut dat = FindBlobsData::from_image(image);

    // Find all blobs.
    for x in 0..dat.width {
        for y in 0..dat.height {
            let color = dat.visit(x, y);
            if color.is_none() {
                continue;
            }

            let x_u32: u32 = x.try_into().unwrap();
            let y_u32: u32 = y.try_into().unwrap();

            let mut blob = Blob {
                x_min: x_u32,
                y_min: y_u32,
                x_max: x_u32,
                y_max: y_u32,
            };

            // Flood fill the blob
            dat.clear_stack();
            dat.append_neighbours(x, y);

            while !dat.stack.is_empty() {
                let (x, y) = dat.stack.pop().unwrap();

                let color = dat.visit(x, y);
                if color.is_none() {
                    continue;
                }

                let x_u32: u32 = x.try_into().unwrap();
                let y_u32: u32 = y.try_into().unwrap();

                blob.x_min = blob.x_min.min(x_u32);
                blob.y_min = blob.y_min.min(y_u32);
                blob.x_max = blob.x_max.max(x_u32);
                blob.y_max = blob.y_max.max(y_u32);

                dat.append_neighbours(x, y);
            }

            match try_parse_as_metablob(&dat, &blob) {
                Some(new_metablob) => metablob = Some(new_metablob),
                None => blobs.push(blob),
            }
        }
    }

    // Shrink the blobs to remove any 1-pixel border
    // for blob in &mut blobs {
    //     blob.x_min += 2;
    //     blob.y_min += 2;
    //     blob.x_max -= 2;
    //     blob.y_max -= 2;
    // }

    // for (i, blob) in blobs.iter().enumerate() {
    //     println!(
    //         "Blob {} at ({}, {}) to ({}, {})",
    //         i, blob.x_min, blob.y_min, blob.x_max, blob.y_max
    //     );
    // }

    // Sort the blobs by first y, then x
    blobs.sort_by(|a, b| {
        if a.y_min != b.y_min {
            return a.y_min.cmp(&b.y_min);
        }
        return a.x_min.cmp(&b.x_min);
    });

    return (blobs, metablob);
}

#[derive(Debug, Clone)]
pub struct Anim {
    pub image: webhacks::Image,
    pub texture: webhacks::Texture,
    pub blobs: Vec<Blob>,
    pub meta: AnimMeta,
}

#[derive(Debug, Clone)]
pub struct AnimMeta {
    pub num_frames: usize,
    pub max_width: usize,
    pub max_height: usize,
    pub avg_width: f32,
    pub avg_height: f32,
    pub pad_blob: u32,
}

impl Anim {
    pub fn new(image: webhacks::Image) -> Anim {
        return Anim {
            image: image,
            texture: webhacks::null_texture(),
            blobs: vec![],
            meta: AnimMeta {
                num_frames: 0,
                max_width: 0,
                max_height: 0,
                avg_width: 0.0,
                avg_height: 0.0,
                pad_blob: 0,
            },
        };
    }
}

impl Anim {
    pub fn load_texture(&mut self) {
        if !webhacks::is_texture_loaded(self.texture) {
            self.texture = webhacks::load_texture_from_image(self.image);
        }
    }

    pub fn find_blobs(&mut self) {
        let (blobs, metablob) = find_blobs(self.image);

        self.blobs = blobs;

        self.meta.num_frames = self.blobs.len();
        for blob in &self.blobs {
            self.meta.max_width = self.meta.max_width.max(blob.width());
            self.meta.max_height = self.meta.max_height.max(blob.height());
            self.meta.avg_width += blob.width() as f32;
            self.meta.avg_height += blob.height() as f32;
        }
        self.meta.avg_width /= self.meta.num_frames as f32;
        self.meta.avg_height /= self.meta.num_frames as f32;

        if let Some(metablob) = metablob {
            self.meta.pad_blob = metablob.pad_blob;
        }
    }

    pub fn unload_image(&mut self) {
        if webhacks::is_image_loaded(self.image) {
            webhacks::unload_image(self.image);
            self.image = webhacks::null_image();
        }
    }

    pub fn draw(&self, position: Vector2, scale: f32, anchor: Anchor, time: f32) {
        draw_at_position(
            position,
            &self.blobs,
            self.texture,
            time,
            scale,
            anchor,
            self.meta.pad_blob,
        );
    }
}

fn time_to_anim_frame(time: f32, frame_duration: f32, n_frames: u32) -> usize {
    ((time / frame_duration) as u32 % n_frames) as usize
}

#[derive(PartialEq, Eq)]
pub enum Anchor {
    TopLeft,
    // TopCenter,
    // TopRight,
    // CenterLeft,
    CenterCenter,
    // CenterRight,
    // BottomLeft,
    BottomCenter,
    // BottomRight,
}

impl Anchor {
    #[allow(non_upper_case_globals)]
    pub const Center: Anchor = Anchor::CenterCenter;
}

impl TryFrom<u8> for Anchor {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Anchor::TopLeft),
            1 => Ok(Anchor::CenterCenter),
            2 => Ok(Anchor::BottomCenter),
            _ => Err(()),
        }
    }
}

fn draw_at_position(
    position: Vector2,
    anim_blobs: &[Blob],
    texture: webhacks::Texture,
    time: f32,
    scale: f32,
    anchor: Anchor,
    pad_blob: u32,
) {
    let frame = time_to_anim_frame(time, 0.1, anim_blobs.len() as u32);

    let blob = anim_blobs[frame];
    let mut source = blob.to_rect();

    // shrink the source rect by the padding
    source.x += pad_blob as f32;
    source.y += pad_blob as f32;
    source.width -= pad_blob as f32 * 2.0;
    source.height -= pad_blob as f32 * 2.0;

    let width = blob.width() as f32 * scale;
    let height = blob.height() as f32 * scale;

    let dest = match anchor {
        Anchor::CenterCenter => raylib::Rectangle {
            x: position.x - width / 2.0,
            y: position.y - height / 2.0,
            width: width,
            height: height,
        },
        Anchor::TopLeft => raylib::Rectangle {
            x: position.x,
            y: position.y,
            width: width,
            height: height,
        },
        Anchor::BottomCenter => raylib::Rectangle {
            x: position.x - width / 2.0,
            y: position.y - height,
            width: width,
            height: height,
        },
    };

    webhacks::draw_texture_pro(texture, source, dest);
    // webhacks::draw_circle(position, 5.0, RAYWHITE); // debug circle
}
