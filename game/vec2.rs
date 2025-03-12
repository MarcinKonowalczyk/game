use raylib_wasm;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}
// Vector extension trait

// #[allow(dead_code)]
// pub trait Vector2Ext {
//     fn new(x: f32, y: f32) -> Vector2;

//     fn dist2(&self, other: &Vector2) -> f32;
//     fn dist(&self, other: &Vector2) -> f32;

//     fn mag2(&self) -> f32;
//     fn mag(&self) -> f32;

//     fn normalize(&mut self) -> &mut Self;

//     fn adds(&mut self, other: f32) -> &mut Self;
//     fn add(&mut self, other: &Vector2) -> &mut Self;

//     fn subs(&mut self, other: f32) -> &mut Self;
//     fn sub(&mut self, other: &Vector2) -> &mut Self;

//     fn muls(&mut self, other: f32) -> &mut Self;
//     fn mul(&mut self, other: &Vector2) -> &mut Self;

//     fn divs(&mut self, other: f32) -> &mut Self;
//     fn div(&mut self, other: &Vector2) -> &mut Self;

//     fn dot(&self, other: &Vector2) -> f32;

//     fn lerp(&mut self, other: &Vector2, t: f32) -> &mut Self;
// }

impl Vector2 {
    pub fn new(x: f32, y: f32) -> Vector2 {
        Vector2 { x, y }
    }

    pub fn zero() -> Vector2 {
        Vector2::new(0.0, 0.0)
    }

    pub fn dist2(&self, other: &Vector2) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
    }

    pub fn dist(&self, other: &Vector2) -> f32 {
        self.dist2(other).sqrt()
    }

    pub fn mag2(&self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    pub fn mag(&self) -> f32 {
        self.mag2().sqrt()
    }

    pub fn normalize(&self) -> Vector2 {
        let mag = self.mag();
        if mag == 0.0 {
            Vector2::zero()
        } else {
            Vector2::new(self.x / mag, self.y / mag)
        }
    }

    // pub fn adds(&self, other: f32) -> Vector2 {
    //     Vector2::new(self.x + other, self.y + other)
    // }

    // pub fn add(&self, other: &Vector2) -> Vector2 {
    //     Vector2::new(self.x + other.x, self.y + other.y)
    // }

    // pub fn subs(&self, other: f32) -> Vector2 {
    //     Vector2::new(self.x - other, self.y - other)
    // }

    // pub fn sub(&self, other: &Vector2) -> Vector2 {
    //     Vector2::new(self.x - other.x, self.y - other.y)
    // }

    // pub fn muls(&self, other: f32) -> Vector2 {
    //     Vector2::new(self.x * other, self.y * other)
    // }

    // pub fn mul(&self, other: &Vector2) -> Vector2 {
    //     Vector2::new(self.x * other.x, self.y * other.y)
    // }

    // pub fn divs(&self, other: f32) -> Vector2 {
    //     Vector2::new(self.x / other, self.y / other)
    // }

    // pub fn div(&self, other: &Vector2) -> Vector2 {
    //     Vector2::new(self.x / other.x, self.y / other.y)
    // }

    pub fn dot(&self, other: &Vector2) -> f32 {
        self.x * other.x + self.y * other.y
    }

    pub fn lerp(&self, other: &Vector2, t: f32) -> Vector2 {
        Vector2::new(
            self.x * (1.0 - t) + other.x * t,
            self.y * (1.0 - t) + other.y * t,
        )
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

use std::default::Default;

impl Default for Vector2 {
    fn default() -> Self {
        Vector2::zero()
    }
}

// `AddAssign<&mut vec2::Vector2>

// use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

impl std::ops::Add<Vector2> for Vector2 {
    type Output = Vector2;

    fn add(self, other: Vector2) -> Vector2 {
        Vector2::new(self.x + other.x, self.y + other.y)
    }
}

impl std::ops::Add<f32> for Vector2 {
    type Output = Vector2;

    fn add(self, other: f32) -> Vector2 {
        Vector2::new(self.x + other, self.y + other)
    }
}

impl std::ops::AddAssign<Vector2> for Vector2 {
    fn add_assign(&mut self, other: Vector2) {
        self.x += other.x;
        self.y += other.y;
    }
}

impl std::ops::AddAssign<f32> for Vector2 {
    fn add_assign(&mut self, other: f32) {
        self.x += other;
        self.y += other;
    }
}

impl std::ops::Sub<Vector2> for Vector2 {
    type Output = Vector2;

    fn sub(self, other: Vector2) -> Vector2 {
        Vector2::new(self.x - other.x, self.y - other.y)
    }
}

impl std::ops::Sub<f32> for Vector2 {
    type Output = Vector2;

    fn sub(self, other: f32) -> Vector2 {
        Vector2::new(self.x - other, self.y - other)
    }
}

impl std::ops::SubAssign<Vector2> for Vector2 {
    fn sub_assign(&mut self, other: Vector2) {
        self.x -= other.x;
        self.y -= other.y;
    }
}

impl std::ops::SubAssign<f32> for Vector2 {
    fn sub_assign(&mut self, other: f32) {
        self.x -= other;
        self.y -= other;
    }
}

impl std::ops::Mul<Vector2> for Vector2 {
    type Output = Vector2;

    fn mul(self, other: Vector2) -> Vector2 {
        Vector2::new(self.x * other.x, self.y * other.y)
    }
}

impl std::ops::Mul<f32> for Vector2 {
    type Output = Vector2;

    fn mul(self, other: f32) -> Vector2 {
        Vector2::new(self.x * other, self.y * other)
    }
}

impl std::ops::MulAssign<Vector2> for Vector2 {
    fn mul_assign(&mut self, other: Vector2) {
        self.x *= other.x;
        self.y *= other.y;
    }
}

impl std::ops::MulAssign<f32> for Vector2 {
    fn mul_assign(&mut self, other: f32) {
        self.x *= other;
        self.y *= other;
    }
}

impl std::ops::Div<Vector2> for Vector2 {
    type Output = Vector2;

    fn div(self, other: Vector2) -> Vector2 {
        Vector2::new(self.x / other.x, self.y / other.y)
    }
}

impl std::ops::Div<f32> for Vector2 {
    type Output = Vector2;

    fn div(self, other: f32) -> Vector2 {
        Vector2::new(self.x / other, self.y / other)
    }
}

impl std::ops::DivAssign<Vector2> for Vector2 {
    fn div_assign(&mut self, other: Vector2) {
        self.x /= other.x;
        self.y /= other.y;
    }
}

impl std::ops::DivAssign<f32> for Vector2 {
    fn div_assign(&mut self, other: f32) {
        self.x /= other;
        self.y /= other;
    }
}

impl std::ops::Neg for Vector2 {
    type Output = Vector2;

    fn neg(self) -> Vector2 {
        Vector2::new(-self.x, -self.y)
    }
}
