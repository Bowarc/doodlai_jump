pub const PLATFORM_BASE_WIDTH: f64 = 70.;
pub const PLATFORM_BASE_HEIGHT: f64 = 20.;

pub enum PlatformType {}

#[derive(Clone)]
pub struct Platform {
    pub rect: maths::Rect,
}

impl Platform {
    pub fn new(rect: impl Into<maths::Rect>) -> Self {
        Self { rect: rect.into() }
    }
}
