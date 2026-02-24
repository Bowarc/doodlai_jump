/*
    Used to target an abstract position
*/
#[derive(Clone, Copy, Debug)]
pub enum Anchor {
    CenterCenter,
    Topleft,
    TopCenter,
    TopRight,
    RightCenter,
    BotRight,
    BotCenter,
    BotLeft,
    LeftCenter,
}

impl Anchor {
    /// Computes and returns the center point of the element
    pub fn compute(&self, drawable_size: &math::Point, element_size: &math::Point) -> math::Point {
        match self {
            Anchor::CenterCenter => *drawable_size * 0.5,
            Anchor::Topleft => *element_size * 0.5,
            Anchor::TopCenter => math::Point::new(drawable_size.x * 0.5, element_size.y * 0.5),
            Anchor::TopRight => {
                math::Point::new(drawable_size.x - element_size.x * 0.5, element_size.y * 0.5)
            }
            Anchor::RightCenter => math::Point::new(
                drawable_size.x - element_size.x * 0.5,
                drawable_size.y * 0.5,
            ),
            Anchor::BotRight => *drawable_size - *element_size * 0.5,
            Anchor::BotCenter => math::Point::new(
                drawable_size.x * 0.5,
                drawable_size.y - element_size.y * 0.5,
            ),
            Anchor::BotLeft => {
                math::Point::new(element_size.x * 0.5, drawable_size.y - element_size.y * 0.5)
            }
            Anchor::LeftCenter => math::Point::new(element_size.x * 0.5, drawable_size.y * 0.5),
        }

        // match self {
        // Anchor::CenterCenter => {
        // math::Point::new(drawable_size.x * 0.5, drawable_size.y * 0.5)
        // - element_size * 0.5
        // }
        // Anchor::Topleft => math::Point::ZERO,
        // Anchor::TopCenter => {
        // math::Point::new(drawable_size.x * 0.5 - element_size.x * 0.5, 0.)
        // }
        // Anchor::TopRight => math::Point::new(drawable_size.x - element_size.x, 0.),
        // Anchor::RightCenter => math::Point::new(
        // drawable_size.x - element_size.x,
        // drawable_size.y * 0.5 - element_size.y * 0.5,
        // ),
        // Anchor::BotRight => math::Point::new(
        // drawable_size.x - element_size.x,
        // drawable_size.y - element_size.y,
        // ),
        // Anchor::BotCenter => math::Point::new(
        // drawable_size.x * 0.5 - element_size.x * 0.5,
        // drawable_size.y - element_size.y,
        // ),
        // Anchor::BotLeft => math::Point::new(0., drawable_size.y - element_size.y),
        // Anchor::LeftCenter => {
        // math::Point::new(0., drawable_size.y * 0.5 - element_size.y * 0.5)
        // }
        // }
    }
    /// Returns the center point
    pub fn as_value(&self, size: impl Into<super::Vector>) -> super::Vector {
        use super::value::MagicValue;
        let size = size.into();
        match self {
            Anchor::CenterCenter => {
                super::Vector::new(MagicValue::ScreenSizeW * 0.5, MagicValue::ScreenSizeH * 0.5)
            }
            Anchor::Topleft => size * 0.5,
            Anchor::TopCenter => {
                super::Vector::new(MagicValue::ScreenSizeW * 0.5, size.y().clone() * 0.5)
            }
            Anchor::TopRight => super::Vector::new(
                MagicValue::ScreenSizeW - size.x().clone() * 0.5,
                size.y().clone() * 0.5,
            ),
            Anchor::RightCenter => super::Vector::new(
                MagicValue::ScreenSizeW - size.x().clone() * 0.5,
                MagicValue::ScreenSizeH * 0.5,
            ),
            Anchor::BotRight => {
                super::Vector::new(MagicValue::ScreenSizeW, MagicValue::ScreenSizeH) - size * 0.5
            }
            Anchor::BotCenter => super::Vector::new(
                MagicValue::ScreenSizeW * 0.5,
                MagicValue::ScreenSizeH - size.y().clone() * 0.5,
            ),
            Anchor::BotLeft => super::Vector::new(
                size.x().clone() * 0.5,
                MagicValue::ScreenSizeH - size.y().clone() * 0.5,
            ),
            Anchor::LeftCenter => {
                super::Vector::new(size.x().clone() * 0.5, MagicValue::ScreenSizeW * 0.5)
            }
        }

        // match self{
        //     Anchor::CenterCenter => (MagicValue::ScreenSizeW*0.5, MagicValue::ScreenSizeH * 0.5),
        //     Anchor::Topleft => (0f64.into(), 0f64.into()),
        //     Anchor::TopCenter => (MagicValue::ScreenSizeW*0.5, 0f64.into()),
        //     Anchor::TopRight => (MagicValue::ScreenSizeW.into(), 0f64.into()),
        //     Anchor::RightCenter => (MagicValue::ScreenSizeW.into(), MagicValue::ScreenSizeH * 0.5),
        //     Anchor::BotRight => (MagicValue::ScreenSizeW.into(), MagicValue::ScreenSizeH.into()),
        //     Anchor::BotCenter =>(MagicValue::ScreenSizeW * 0.5, MagicValue::ScreenSizeH.into()),
        //     Anchor::BotLeft =>(0f64.into(), MagicValue::ScreenSizeH.into()),
        //     Anchor::LeftCenter => (0f64.into(), MagicValue::ScreenSizeH * 0.5),
        // }
    }
}
