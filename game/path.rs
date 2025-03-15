use crate::entity_manager::EntityId;
use crate::vec2::Vector2;
use crate::webhacks;
use crate::State;

use raylib_wasm::RAYWHITE;

pub struct Path {
    pub id: EntityId,
    pub nodes: Vec<Vector2>,
    pub total_length: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct PathPosition {
    pub xy: Vector2,
    pub linear: f32,
    // pub path: &Path,
}

impl Path {
    pub fn new(nodes: Vec<Vector2>) -> Path {
        if nodes.len() < 2 {
            panic!("Path must have at least 2 nodes");
        }
        let total_length = nodes
            .iter()
            .fold((0.0, nodes[0]), |(acc, prev), &p| (acc + prev.dist(&p), p))
            .0;
        Path {
            id: 0,
            nodes,
            total_length,
        }
    }

    pub fn start(&self) -> PathPosition {
        PathPosition {
            xy: self.nodes[0],
            linear: 0.0,
            // path: self,
        }
    }

    pub fn end(&self) -> PathPosition {
        PathPosition {
            xy: self.nodes[self.nodes.len() - 1],
            linear: self.total_length,
            // path: self,
        }
    }
}

impl From<PathPosition> for Vector2 {
    fn from(pos: PathPosition) -> Vector2 {
        pos.xy
    }
}

impl PartialEq for PathPosition {
    fn eq(&self, other: &Self) -> bool {
        self.xy == other.xy && self.linear == other.linear
    }
}

impl Path {
    pub fn draw(&self, _state: &State) {
        // Draw the path
        for i in 1..self.nodes.len() {
            let p1 = self.nodes[i - 1];
            let p2 = self.nodes[i];
            webhacks::draw_line_ex(p1, p2, 2.0, RAYWHITE);
            // unsafe { raylib::DrawLineEx(p1, p2, 2.0, RAYWHITE) }
        }
    }
}

impl Path {
    pub fn lin_to_position(&self, linear_pos: f32) -> PathPosition {
        if linear_pos < 0.0 {
            // underflow. return the first node
            return PathPosition {
                xy: self.nodes[0],
                linear: 0.0,
                // path: self,
            };
        }

        let mut current_path_length = 0.0;
        for i in 1..self.nodes.len() {
            let p1 = self.nodes[i - 1];
            let p2 = self.nodes[i];
            let segment_length = p1.dist(&p2);
            if current_path_length + segment_length >= linear_pos {
                let segment_pos = (linear_pos - current_path_length) / segment_length;
                return PathPosition {
                    xy: p1.lerp(&p2, segment_pos),
                    linear: linear_pos,
                    // path: self,
                };
            }
            current_path_length += segment_length;
        }

        // overflow. return the last node
        // self.nodes[self.nodes.len() - 1]
        PathPosition {
            xy: self.nodes[self.nodes.len() - 1],
            linear: self.total_length,
            // path: self,
        }
    }
}

impl PathPosition {
    pub fn linear_advance(&mut self, path: &Path, distance: f32) -> &mut Self {
        let linear_pos = self.linear + distance;
        let new_pos = path.lin_to_position(linear_pos);
        self.xy = new_pos.xy;
        self.linear = new_pos.linear;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_pos_to_screen_pos() {
        let nodes = vec![
            Vector2::new(0.0, 0.0),
            Vector2::new(0.0, 100.0),
            Vector2::new(100.0, 100.0),
        ];
        let path = Path::new(nodes);
        let pos: Vector2 = path.lin_to_position(0.0).into();
        assert_eq!(pos, Vector2::new(0.0, 0.0));
        let pos: Vector2 = path.lin_to_position(50.0).into();
        assert_eq!(pos, Vector2::new(0.0, 50.0));
        let pos: Vector2 = path.lin_to_position(150.0).into();
        assert_eq!(pos, Vector2::new(50.0, 100.0));
    }
}
