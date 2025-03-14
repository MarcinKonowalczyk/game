use std::vec;

use crate::anim;
use crate::defer;
use crate::vec2::Vector2;
use crate::webhacks;

use raylib_wasm::{self as raylib, Color};

trait SpecificColors {
    fn is_magenta(&self) -> bool;
    fn is_alpha(&self) -> bool;
    fn is_black(&self) -> bool;
    fn is_white(&self) -> bool;
}

impl SpecificColors for Color {
    fn is_magenta(&self) -> bool {
        self.r == 255 && self.g == 0 && self.b == 255 && self.a == 255
    }

    fn is_alpha(&self) -> bool {
        self.a == 0
    }

    fn is_black(&self) -> bool {
        self.r == 0 && self.g == 0 && self.b == 0
    }

    fn is_white(&self) -> bool {
        self.r == 255 && self.g == 255 && self.b == 255
    }
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
    pub fn new(x_min: u32, y_min: u32, x_max: u32, y_max: u32) -> Blob {
        return Blob {
            x_min: x_min,
            y_min: y_min,
            x_max: x_max,
            y_max: y_max,
        };
    }
}

impl Dimensions for Blob {
    fn width(&self) -> usize {
        (self.x_max - self.x_min + 1) as usize
    }

    fn height(&self) -> usize {
        (self.y_max - self.y_min + 1) as usize
    }
}

impl From<Blob> for raylib::Rectangle {
    fn from(blob: Blob) -> raylib::Rectangle {
        return raylib::Rectangle {
            x: blob.x_min as f32,
            y: blob.y_min as f32,
            width: blob.width() as f32,
            height: blob.height() as f32,
        };
    }
}

pub struct Metablob {
    pad_blob: u32,
    #[allow(dead_code)]
    anchor: anim::Anchor,
}

struct Colors {
    colors: Vec<Color>,
    width: usize,
    height: usize,
}

trait PixelAccess<T> {
    // 2D index
    fn at(&self, x: usize, y: usize) -> T;
    // Linear index
    fn index(&self, i: usize) -> T;
}

pub trait Dimensions {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn size(&self) -> usize {
        return self.width() * self.height();
    }
}

trait FromImage {
    fn from_image(image: webhacks::Image) -> Self;
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

impl FromImage for Colors {
    fn from_image(image: webhacks::Image) -> Colors {
        let (colors, width, height) = image_to_colors(image);
        return Colors {
            colors,
            width,
            height,
        };
    }
}

impl PixelAccess<Color> for Colors {
    fn at(&self, x: usize, y: usize) -> Color {
        return self.colors[x + y * self.width];
    }
    fn index(&self, i: usize) -> Color {
        return self.colors[i];
    }
}

impl Dimensions for Colors {
    fn width(&self) -> usize {
        return self.width;
    }

    fn height(&self) -> usize {
        return self.height;
    }
}

struct FindBlobsData {
    colors: Colors,
    visited: Vec<bool>,
    stack: Vec<(usize, usize)>,
}

impl PixelAccess<Color> for FindBlobsData {
    fn at(&self, x: usize, y: usize) -> Color {
        return self.colors.at(x, y);
    }
    fn index(&self, i: usize) -> Color {
        return self.colors.index(i);
    }
}

impl Dimensions for FindBlobsData {
    fn width(&self) -> usize {
        return self.colors.width();
    }

    fn height(&self) -> usize {
        return self.colors.height();
    }
}

impl FromImage for FindBlobsData {
    fn from_image(image: webhacks::Image) -> FindBlobsData {
        let colors = Colors::from_image(image);
        let visited = vec![false; colors.size()];
        return FindBlobsData {
            colors: colors,
            visited: visited,
            stack: Vec::new(),
        };
    }
}

impl FindBlobsData {
    fn visit(&mut self, x: usize, y: usize) -> Option<Color> {
        if x >= self.width() || y >= self.height() {
            return None;
        }
        let i = x as usize + y as usize * self.width();
        if i >= self.visited.len() {
            return None;
        }

        if self.visited[i] {
            return None;
        }
        self.visited[i] = true;
        let color = self.colors.index(i);

        if color.is_magenta() {
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
        if x < self.width() - 1 {
            self.stack.push((x + 1, y));
        }
        if y > 0 {
            self.stack.push((x, y - 1));
        }
        if y < self.height() - 1 {
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
    if color.is_alpha() {
        return MetablobPixel::Transparent;
    }
    if color.is_black() {
        return MetablobPixel::Black;
    }
    if color.is_white() {
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
    for x in 0..dat.width() {
        for y in 0..dat.height() {
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

    // DEBUG!!
    // Shrink the blobs to remove any 1-pixel border
    // for blob in &mut blobs {
    //     blob.x_min += 2;
    //     blob.y_min += 2;
    //     blob.x_max -= 2;
    //     blob.y_max -= 2;
    // }

    // DEBUG!!
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

// Iterate through the pixels of a blob, starting from the top-left corner
// in rings, moving clockwise.
struct BlobRings {
    blob: Blob,
    pub x: u32,
    pub y: u32,
    direction: Direction,
    steps: u32,
    ring: u32,
}

#[derive(PartialEq)]
enum Direction {
    Contract,
    Right,
    Down,
    Left,
    Up,
}

impl BlobRings {
    fn new(blob: Blob) -> BlobRings {
        return BlobRings {
            blob: blob,
            x: blob.x_min,
            y: blob.y_min,
            direction: Direction::Right,
            steps: 0,
            ring: 0,
        };
    }
}
impl Iterator for BlobRings {
    type Item = (u32, u32, u32);

    fn next(&mut self) -> Option<Self::Item> {
        if self.steps >= self.blob.size() as u32 {
            // we have visited all pixels in the blob
            None
        } else {
            let out = Some((self.x, self.y, self.ring));
            match self.direction {
                Direction::Right | Direction::Contract => {
                    self.x += 1;
                    if self.x == (self.blob.x_max - self.ring) {
                        // this is the last step of walking right
                        self.direction = Direction::Down;
                    }
                    if self.direction == Direction::Contract {
                        // This is a first ste of walking right into the new ring.
                        self.ring += 1;
                        self.direction = Direction::Right;
                    }
                }
                Direction::Down => {
                    self.y += 1;
                    if self.y == (self.blob.y_max - self.ring) {
                        // this is the last step of walking down
                        self.direction = Direction::Left;
                    }
                }
                Direction::Left => {
                    self.x -= 1;
                    if self.x == (self.blob.x_min + self.ring) {
                        // this is the last step of walking left
                        self.direction = Direction::Up;
                    }
                }
                Direction::Up => {
                    self.y -= 1;
                    if self.y == (self.blob.y_min + self.ring + 1) {
                        // this is the last step of walking up
                        self.direction = Direction::Contract;
                    }
                }
            };
            self.steps += 1;
            out
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // expected!["000100200300310"] == vec![(0, 0, 0), (1, 0, 0), (2, 0, 0), (3, 0, 0), (3, 1, 0)]
    macro_rules! expected {
        ($e:expr) => {
            $e.chars()
                .map(|c| c.to_digit(10).unwrap() as u32)
                .collect::<Vec<u32>>()
                .chunks(3)
                .map(|chunk| (chunk[0], chunk[1], chunk[2]))
                .collect::<Vec<(u32, u32, u32)>>()
        };
    }

    #[test]
    fn test_blob_rings_4x4() {
        let blob = Blob::new(0, 0, 3, 3);
        let rings = BlobRings::new(blob);

        let coords: Vec<(u32, u32, u32)> = rings.collect();
        let expected = expected!["000100200300310320330230130030020010111211221121"];

        assert_eq!(coords, expected);
    }

    #[test]
    fn test_blob_rings_3x3() {
        let blob = Blob::new(0, 0, 2, 2);
        let rings = BlobRings::new(blob);
        let coords: Vec<(u32, u32, u32)> = rings.collect();
        let expected = expected!["000100200210220120020010111"];

        assert_eq!(coords, expected);
    }

    #[test]
    fn test_blob_rings_2x2() {
        let blob = Blob::new(0, 0, 1, 1);
        let rings = BlobRings::new(blob);
        let coords: Vec<(u32, u32, u32)> = rings.collect();
        let expected = expected!["000100110010"];

        assert_eq!(coords, expected);
    }

    #[test]
    fn test_blob_rings_1x1() {
        let blob = Blob::new(0, 0, 0, 0);
        let rings = BlobRings::new(blob);
        let coords: Vec<(u32, u32, u32)> = rings.collect();
        let expected = vec![(0, 0, 0)];

        assert_eq!(coords, expected);
    }

    #[test]
    fn test_blob_rings_0x0() {
        let blob = Blob::new(0, 0, 0, 0);
        let rings = BlobRings::new(blob);
        let coords: Vec<(u32, u32, u32)> = rings.collect();
        let expected = vec![(0, 0, 0)];

        assert_eq!(coords, expected);
    }

    #[test]
    fn test_blob_rings_wide() {
        let blob = Blob::new(0, 0, 3, 1);
        let rings = BlobRings::new(blob);
        let coords: Vec<(u32, u32, u32)> = rings.collect();
        let expected = expected!["000100200300310210110010"];

        assert_eq!(coords, expected);
    }
}

// Infer the padding of a blob by walking the rings of the blob until a non-transparent pixel is found.
// The padding for all the blobs is the min over all the blobs.
fn infer_blob_padding(image: webhacks::Image, blobs: &[Blob]) -> u32 {
    let colors = Colors::from_image(image);
    blobs
        .iter()
        .map(|blob| {
            let ring = BlobRings::new(*blob);
            let mut max_ring = 0;
            for (x, y, ring) in ring {
                let color = colors.at(x as usize, y as usize);
                if color.a == 0 {
                    max_ring = max_ring.max(ring)
                } else {
                    break;
                }
            }
            max_ring
        })
        .min()
        .unwrap_or(0)
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
        } else {
            self.meta.pad_blob = infer_blob_padding(self.image, &self.blobs);
            // println!("Inferred padding: {}", self.meta.pad_blob);
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
    let mut source = raylib::Rectangle::from(blob);

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
