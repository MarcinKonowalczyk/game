pub use raylib_wasm::Vector2;

// Vector extension trait

#[allow(dead_code)]
pub trait Vector2Ext {
    fn new(x: f32, y: f32) -> Vector2;

    fn dist2(&self, other: Vector2) -> f32;
    fn dist(&self, other: Vector2) -> f32;

    fn mag2(&self) -> f32;
    fn mag(&self) -> f32;

    fn normalize(&self) -> Vector2;

    fn adds(&self, other: f32) -> Vector2;
    fn add(&self, other: Vector2) -> Vector2;

    fn subs(&self, other: f32) -> Vector2;
    fn sub(&self, other: Vector2) -> Vector2;

    fn muls(&self, other: f32) -> Vector2;
    fn mul(&self, other: Vector2) -> Vector2;

    fn divs(&self, other: f32) -> Vector2;
    fn div(&self, other: Vector2) -> Vector2;

    fn dot(&self, other: Vector2) -> f32;

    fn lerp(&self, other: Vector2, t: f32) -> Vector2 {
        self.muls(1.0 - t).add(other.muls(t))
    }
}

impl Vector2Ext for Vector2 {
    fn new(x: f32, y: f32) -> Vector2 {
        Vector2 { x, y }
    }

    fn dist2(&self, other: Vector2) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
    }

    fn dist(&self, other: Vector2) -> f32 {
        self.dist2(other).sqrt()
    }

    fn mag2(&self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    fn mag(&self) -> f32 {
        self.mag2().sqrt()
    }

    fn normalize(&self) -> Vector2 {
        let mag = self.mag();
        if mag == 0.0 {
            Vector2::new(0.0, 0.0)
        } else {
            Vector2::new(self.x / mag, self.y / mag)
        }
    }

    fn adds(&self, other: f32) -> Vector2 {
        Vector2 {
            x: self.x + other,
            y: self.y + other,
        }
    }

    fn add(&self, other: Vector2) -> Vector2 {
        Vector2 {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }

    fn subs(&self, other: f32) -> Vector2 {
        Vector2 {
            x: self.x - other,
            y: self.y - other,
        }
    }

    fn sub(&self, other: Vector2) -> Vector2 {
        Vector2 {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }

    fn muls(&self, other: f32) -> Vector2 {
        Vector2 {
            x: self.x * other,
            y: self.y * other,
        }
    }

    fn mul(&self, other: Vector2) -> Vector2 {
        Vector2 {
            x: self.x * other.x,
            y: self.y * other.y,
        }
    }

    fn divs(&self, other: f32) -> Vector2 {
        Vector2 {
            x: self.x / other,
            y: self.y / other,
        }
    }

    fn div(&self, other: Vector2) -> Vector2 {
        Vector2 {
            x: self.x / other.x,
            y: self.y / other.y,
        }
    }

    fn dot(&self, other: Vector2) -> f32 {
        self.x * other.x + self.y * other.y
    }
}
