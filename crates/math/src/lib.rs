use glam::DVec2;

pub type Vec2 = DVec2;
pub type Point = DVec2;

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Rect {
    pos: Point,
    size: Vec2,
    rotation: f64,
}

impl Rect {
    pub fn new(pos: impl Into<Point>, size: impl Into<Vec2>, rotation: f64) -> Self {
        Self {
            pos: pos.into(),
            size: size.into(),
            rotation,
        }
    }

    pub fn new_from_center(center: impl Into<Point>, size: impl Into<Vec2>, rotation: f64) -> Self {
        let size = size.into();
        let center = center.into();
        Self {
            pos: center - size * 0.5,
            size,
            rotation,
        }
    }

    pub fn pos(&self) -> Point {
        self.pos
    }

    pub fn center(&self) -> Point {
        self.pos + self.size * 0.5
    }

    pub fn set_center(&mut self, center: impl Into<Point>) {
        self.pos = center.into() - self.size * 0.5;
    }

    pub fn size(&self) -> Vec2 {
        self.size
    }

    pub fn width(&self) -> f64 {
        self.size.x
    }

    pub fn height(&self) -> f64 {
        self.size.y
    }

    pub fn rotation(&self) -> f64 {
        self.rotation
    }

    pub fn aa_topleft(&self) -> Point {
        self.pos
    }

    pub fn aa_topright(&self) -> Point {
        self.pos + Vec2::new(self.size.x, 0.0)
    }

    pub fn aa_botleft(&self) -> Point {
        self.pos + Vec2::new(0.0, self.size.y)
    }

    pub fn aa_botright(&self) -> Point {
        self.pos + self.size
    }

    pub fn r_topleft(&self) -> Point {
        self.rotate_point_around_center(self.aa_topleft())
    }

    pub fn r_topright(&self) -> Point {
        self.rotate_point_around_center(self.aa_topright())
    }

    pub fn r_botleft(&self) -> Point {
        self.rotate_point_around_center(self.aa_botleft())
    }

    pub fn r_botright(&self) -> Point {
        self.rotate_point_around_center(self.aa_botright())
    }

    fn rotate_point_around_center(&self, point: Point) -> Point {
        if self.rotation == 0.0 {
            return point;
        }

        let center = self.center();
        let rel = point - center;
        let (sin_r, cos_r) = self.rotation.sin_cos();
        center + Vec2::new(rel.x * cos_r - rel.y * sin_r, rel.x * sin_r + rel.y * cos_r)
    }
}

pub fn get_distance(a: &Point, b: &Point) -> f64 {
    a.distance(*b)
}

pub mod collision {
    use super::{Point, Rect};

    pub fn point_rect(point: &Point, rect: &Rect) -> bool {
        let top_left = rect.aa_topleft();
        let bottom_right = rect.aa_botright();

        point.x >= top_left.x
            && point.x <= bottom_right.x
            && point.y >= top_left.y
            && point.y <= bottom_right.y
    }

    pub fn rect_rect_no_r(a: &Rect, b: &Rect) -> bool {
        let a_tl = a.aa_topleft();
        let a_br = a.aa_botright();
        let b_tl = b.aa_topleft();
        let b_br = b.aa_botright();

        a_tl.x < b_br.x && a_br.x > b_tl.x && a_tl.y < b_br.y && a_br.y > b_tl.y
    }
}

#[cfg(feature = "ggez")]
impl From<Rect> for ggez::graphics::Rect {
    fn from(value: Rect) -> Self {
        let top_left = value.aa_topleft();
        let size = value.size();
        Self::new(top_left.x as f32, top_left.y as f32, size.x as f32, size.y as f32)
    }
}
