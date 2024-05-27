use crate::platform::Platform;

const GRAVITY: f64 = 300.;
const JUMP_HEIGHT: f64 = 500.;
const SPEED: f64 = 400.;

pub struct Player {
    pub rect: maths::Rect,
    pub velocity: maths::Vec2,
    pub current_direction: Option<bool>, // True -> Right as True == 1 == positive movement == Right
    side_collision_tag: bool,
}

impl Player {
    pub fn new() -> Self {
        Self {
            rect: maths::Rect::new_from_center((540. / 2., 960. / 2.), (60., 60.), 0.),
            velocity: maths::Vec2::ZERO,
            current_direction: None,
            side_collision_tag: false,
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

        let mut collided_this_frame = false;

        for platform in platforms.iter() {
            if !maths::collision::rect_rect_no_r(self.rect, platform.rect) {
                continue;
            }
            if self.rect.center().x < platform.rect.aa_topleft().x && self.velocity.x > 0. {
                // collision from the left
                self.velocity.x = 0.;
                self.rect.set_center((
                    platform.rect.aa_topleft().x - self.rect.width() / 2. - 1.,
                    self.rect.center().y,
                ));

                println!("Collsion from the left")
            } else if self.rect.center().x > platform.rect.aa_topright().x && self.velocity.x < 0. {
                // collision from the right

                self.rect.set_center((
                    platform.rect.aa_topright().x + self.rect.width() / 2.,
                    self.rect.center().y,
                ));

                self.velocity.x = 0.;
                println!("Collsion from the right")
            } else if self.velocity.y > 0. && !self.side_collision_tag {
                // collision from above
                self.velocity.y = -JUMP_HEIGHT;
                self.rect
                    .set_center(self.rect.center() - maths::Vec2::new(0., 1.));
                println!("Collision from above")
            } else if self.velocity.y < 0. && !self.side_collision_tag {
                // collision from below
                self.velocity.y = 0.0;
                self.rect.set_center(maths::Vec2::new(
                    self.rect.center().x,
                    platform.rect.aa_botleft().y
                        + platform.rect.height()
                        + self.rect.height() / 2.
                        + 1.,
                ));
                println!("Collision from below")
            }
            collided_this_frame = true;

            break;
        }
        if !collided_this_frame {
            self.side_collision_tag = false;
        }
        // println!("{}", self.rect.center());

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
