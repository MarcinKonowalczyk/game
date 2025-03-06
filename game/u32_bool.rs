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

// impl Into<bool> for Bool {
//     fn into(self) -> bool {
//         self.value != 0
//     }
// }

// impl Into<Bool> for bool {
//     fn into(self) -> Bool {
//         Bool {
//             value: if self { 1 } else { 0 },
//         }
//     }
// }

impl From<bool> for Bool {
    fn from(b: bool) -> Bool {
        Bool {
            value: if b { 1 } else { 0 },
        }
    }
}

impl From<Bool> for bool {
    fn from(b: Bool) -> bool {
        b.value != 0
    }
}

impl From<Bool> for u32 {
    fn from(b: Bool) -> u32 {
        b.value
    }
}
