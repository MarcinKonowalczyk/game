use raylib_wasm;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}
// Vector extension trait

#[allow(dead_code)]
pub trait Vector2Ext {
    fn new(x: f32, y: f32) -> Vector2;

    fn dist2(&self, other: &Vector2) -> f32;
    fn dist(&self, other: &Vector2) -> f32;

    fn mag2(&self) -> f32;
    fn mag(&self) -> f32;

    fn normalize(&mut self) -> &mut Self;

    fn adds(&mut self, other: f32) -> &mut Self;
    fn add(&mut self, other: &Vector2) -> &mut Self;

    fn subs(&mut self, other: f32) -> &mut Self;
    fn sub(&mut self, other: &Vector2) -> &mut Self;

    fn muls(&mut self, other: f32) -> &mut Self;
    fn mul(&mut self, other: &Vector2) -> &mut Self;

    fn divs(&mut self, other: f32) -> &mut Self;
    fn div(&mut self, other: &Vector2) -> &mut Self;

    fn dot(&self, other: &Vector2) -> f32;

    fn lerp(&mut self, other: &Vector2, t: f32) -> &mut Self;
}

impl Vector2Ext for Vector2 {
    fn new(x: f32, y: f32) -> Vector2 {
        Vector2 { x, y }
    }

    fn dist2(&self, other: &Vector2) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
    }

    fn dist(&self, other: &Vector2) -> f32 {
        self.dist2(other).sqrt()
    }

    fn mag2(&self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    fn mag(&self) -> f32 {
        self.mag2().sqrt()
    }

    fn normalize(&mut self) -> &mut Self {
        let mag = self.mag();
        if mag == 0.0 {
            self.x = 0.0;
            self.y = 0.0;
        } else {
            self.x /= mag;
            self.y /= mag;
        }
        self
    }

    fn adds(&mut self, other: f32) -> &mut Self {
        self.x += other;
        self.y += other;
        self
    }

    fn add(&mut self, other: &Vector2) -> &mut Self {
        self.x += other.x;
        self.y += other.y;
        self
    }

    fn subs(&mut self, other: f32) -> &mut Self {
        self.adds(-other)
    }

    fn sub(&mut self, other: &Vector2) -> &mut Self {
        self.x -= other.x;
        self.y -= other.y;
        self
    }

    fn muls(&mut self, other: f32) -> &mut Self {
        self.x *= other;
        self.y *= other;
        self
    }

    fn mul(&mut self, other: &Vector2) -> &mut Self {
        self.x *= other.x;
        self.y *= other.y;
        self
    }

    fn divs(&mut self, other: f32) -> &mut Self {
        self.x /= other;
        self.y /= other;
        self
    }

    fn div(&mut self, other: &Vector2) -> &mut Self {
        self.x /= other.x;
        self.y /= other.y;
        self
    }

    fn dot(&self, other: &Vector2) -> f32 {
        self.x * other.x + self.y * other.y
    }

    fn lerp(&mut self, other: &Vector2, t: f32) -> &mut Self {
        let mut temp = other.clone();
        self.muls(1.0 - t).add(temp.muls(t));
        self
    }
}

use std::cmp::PartialEq;

impl PartialEq<Vector2> for Vector2 {
    fn eq(&self, other: &Vector2) -> bool {
        self.x == other.x && self.y == other.y
    }

    fn ne(&self, other: &Vector2) -> bool {
        self.x != other.x || self.y != other.y
    }
}

use std::convert::{From, Into};

impl From<raylib_wasm::Vector2> for Vector2 {
    fn from(v: raylib_wasm::Vector2) -> Self {
        Vector2::new(v.x, v.y)
    }
}

impl Into<raylib_wasm::Vector2> for Vector2 {
    fn into(self) -> raylib_wasm::Vector2 {
        raylib_wasm::Vector2 {
            x: self.x,
            y: self.y,
        }
    }
}
