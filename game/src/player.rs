use crate::platform::Platform;

const GRAVITY: f64 = 400.;
const JUMP_HEIGHT: f64 = 575.;
const SPEED: f64 = 400.;

const PLAYER_SIZE: f64 = 30.;

pub struct Player {
    pub rect: maths::Rect,
    pub velocity: maths::Vec2,
    pub current_direction: Option<bool>, // True -> Right as True == 1 == positive movement == Right
    ignore_collisions_tag: bool,
}

impl Player {
    pub fn new() -> Self {
        Self {
            rect: maths::Rect::new_from_center((540. / 2., 960. / 2.), (PLAYER_SIZE, PLAYER_SIZE), 0.),
            velocity: maths::Vec2::ZERO,
            current_direction: None,
            ignore_collisions_tag: false,
        }
    }

    pub fn direction(&self) -> i8 {
        match self.current_direction {
            Some(true) => 1,
            Some(false) => -1,
            None => 0,
        }
    }

    pub fn update(&mut self, platforms: &[Platform], dt: f64) {
        self.rect
            .set_center(self.rect.center() + self.velocity * dt);

        self.update_collision(platforms);


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

    // Returns if the player collided this frame
    fn update_collision(&mut self, platforms: &[Platform]) -> bool {
        let mut collided_this_frame = false;
        for platform in platforms.iter() {
            if !maths::collision::rect_rect_no_r(self.rect, platform.rect) {
                continue;
            }
			collided_this_frame = true;
			// If player entered the platform from below, ingore all collision until out
			if self.ignore_collisions_tag{
				break;
			}

            if self.rect.center().x < platform.rect.aa_topleft().x && self.velocity.x > 0. {
                // println!("Collsion from the left");
                self.ignore_collisions_tag = true;
                // self.velocity.x = 0.;
                // self.rect.set_center((
                //     platform.rect.aa_topleft().x - self.rect.width() / 2. - 1.,
                //     self.rect.center().y,
                // ));

            } else if self.rect.center().x > platform.rect.aa_topright().x && self.velocity.x < 0. {
                // println!("Collsion from the right");
                self.ignore_collisions_tag = true;
                // self.velocity.x = 0.;
                // self.rect.set_center((
                //     platform.rect.aa_topright().x + self.rect.width() / 2.,
                //     self.rect.center().y,
                // ));

            } else if self.velocity.y > 0. && !self.ignore_collisions_tag {
                // println!("Collision from above");

                self.velocity.y = -JUMP_HEIGHT;
                self.rect
                    .set_center(self.rect.center() - maths::Vec2::new(0., 1.));
            } else if self.velocity.y < 0. && !self.ignore_collisions_tag {
                // println!("Collision from below");
                self.ignore_collisions_tag = true;
                // self.velocity.y = 0.0;
                // self.rect.set_center(maths::Vec2::new(
                //     self.rect.center().x,
                //     platform.rect.aa_botleft().y
                //         + platform.rect.height()
                //         + self.rect.height() / 2.
                //         + 1.,
                // ));
            }

            break;
        }

        if !collided_this_frame {
            self.ignore_collisions_tag = false;
        }

        collided_this_frame
    }
}
