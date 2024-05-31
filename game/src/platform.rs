pub const PLATFORM_BASE_WIDTH: f64 = 100.;
pub const PLATFORM_BASE_HEIGHT: f64 = 20.;



pub enum PlatformType{

}




pub struct Platform{
    pub rect: maths::Rect
}


impl Platform{
    pub fn new(rect: impl Into<maths::Rect>) -> Self{
        Self{
            rect: rect.into()
        }
    }
}