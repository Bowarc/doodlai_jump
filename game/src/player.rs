use crate::platform::Platform;

const GRAVITY: f64 = 300.;
const JUMP_HEIGHT: f64 = 500.;
const SPEED: f64 = 400.;

pub struct Player {
    pub rect: maths::Rect,
    pub velocity: maths::Vec2,
    pub current_direction: Option<bool>, // True -> Right as True == 1 == positive movement == Right
}

impl Player {
    pub fn new() -> Self {
        Self {
            rect: maths::Rect::new_from_center((540. / 2., 960. / 2.), (100., 100.), 0.),
            velocity: maths::Vec2::ZERO,
            current_direction: None,
        }
    }

    pub fn direction(&self) -> i8 {
        match self.current_direction {
            Some(true) => 1,
            Some(false) => -1,
            None => 0,
        }
    }

    pub fn update(&mut self, dt: f64, platforms: &[Platform]) {
        self.rect
            .set_center(self.rect.center() + self.velocity * dt);

        for platform in platforms.iter() {
            if maths::collision::rect_rect_no_r(self.rect, platform.rect) {
                // collision from above
                if self.rect.aa_botleft().y > platform.rect.aa_topleft().y && self.velocity.y > 0. {
                    self.velocity.y = -JUMP_HEIGHT;
                    self.rect
                        .set_center(self.rect.center() - maths::Vec2::new(0., 1.));
                    println!("Collision from above")
                } else if self.rect.aa_topleft().y < platform.rect.aa_botleft().y
                    && self.velocity.y < 0.
                {
                    self.velocity.y = 0.0;
                    self.rect.set_center(maths::Vec2::new(
                        0.,
                        platform.rect.aa_botleft().y + platform.rect.height(),
                    ));
                    println!("Collision from below")
                } else {
                    println!("Collision from side")
                }
                break;
            }
        }
        println!("{}", self.rect.center());

        self.velocity.y += GRAVITY * dt;
        self.velocity.x = SPEED * self.direction() as f64;

        self.current_direction = None;

        if self.rect.center().x > 540. {
            self.rect
                .set_center(maths::Vec2::new(0., self.rect.center().y))
        } else if self.rect.center().x < 0. {
            self.rect
                .set_center(maths::Vec2::new(540., self.rect.center().y))
        }
    }
}
