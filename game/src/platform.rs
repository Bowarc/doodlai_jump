pub struct Platform{
    pub rect: maths::Rect
}


impl Platform{
    pub fn new(pos: impl Into<maths::Vec2>) -> Self{
        Self{
            rect: maths::Rect::new_from_center(pos, (300., 20.), 0.)
        }
    }
}