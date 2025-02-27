pub struct Array2D {
    data: Vec<f32>,
    width: usize,
    height: usize,
}

use std::fmt::{Debug, Formatter, Result};

impl Debug for Array2D {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let mut s = format!("Array2D [{}x{}] {{\n", self.width, self.height);
        for y in 0..self.height {
            s.push_str("  ");
            for x in 0..self.width {
                s.push_str(&format!("{:6.2} ", self.get(x, y)));
                s.push_str(" ");
            }
            s.push_str("\n");
        }
        s.push_str("}");
        write!(f, "{}", s)
    }
}

impl Array2D {
    pub fn new(width: usize, height: usize) -> Array2D {
        Array2D {
            data: vec![f32::MAX; width * height],
            width: width,
            height: height,
        }
    }

    fn linear_index(&self, x: usize, y: usize) -> usize {
        x + y * self.width
    }

    pub fn resize(&mut self, width: usize, height: usize) -> &mut Array2D {
        self.width = width;
        self.height = height;
        self.data.resize(width * height, f32::MAX);
        self.clear();
        self
    }

    pub fn fill(&mut self, value: f32) {
        for i in 0..self.data.len() {
            self.data[i] = value;
        }
    }

    pub fn clear(&mut self) {
        self.fill(f32::MAX);
    }
}

// trait Array2DAccess {
//     fn width(&self) -> usize;
//     fn height(&self) -> usize;
//     fn get(&self, x: usize, y: usize) -> f32;
//     fn set(&mut self, x: usize, y: usize, value: f32);
//     fn get_row(&self, y: usize) -> &[f32];
//     fn get_row_mut(&mut self, y: usize) -> &mut [f32];
//     fn get_col(&self, x: usize) -> Vec<f32>;
//     fn get_col_mut(&mut self, x: usize) -> &mut [f32];
// }

impl Array2D {
    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn get(&self, x: usize, y: usize) -> &f32 {
        &self.data[self.linear_index(x, y)]
    }

    pub fn set(&mut self, x: usize, y: usize, value: f32) {
        let index = self.linear_index(x, y);
        self.data[index] = value;
    }

    pub fn get_row(&self, y: usize) -> &[f32] {
        if y >= self.height {
            panic!(
                "Index out of bounds: the array has {} rows, but the index is {}.",
                self.height, y
            );
        }
        let start = self.width * y;
        let end = start + self.width;
        &self.data[start..end]
    }

    // pub fn get_row_mut(&mut self, y: usize) -> &mut [f32] {
    //     let start = self.width * y;
    //     let end = start + self.width;
    //     &mut self.data[start..end]
    // }

    pub fn get_col(&self, x: usize) -> Vec<&f32> {
        let mut col = Vec::with_capacity(self.height);
        for y in 0..self.height {
            col.push(self.get(x, y));
        }
        col
    }

    // pub fn get_col_mut(&mut self, x: usize) -> &mut [f32] {
    //     let start = x;
    //     let end = self.data.len();
    //     &mut self.data[start..end]
    // }

    pub fn set_row(&mut self, y: usize, row: &[f32]) {
        let start = self.width * y;
        let end = start + self.width;
        self.data[start..end].copy_from_slice(row);
    }

    pub fn set_col(&mut self, x: usize, col: &[f32]) {
        for y in 0..self.height {
            self.set(x, y, col[y]);
        }
    }
}
