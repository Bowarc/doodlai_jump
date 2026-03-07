use crate::platform::Platform;
use std::collections::VecDeque;

const GRAVITY: f64 = 400.;
const JUMP_HEIGHT: f64 = 575.;
const SPEED: f64 = 250.;

const PLAYER_SIZE: f64 = 30.;

#[derive(Default, Clone, Copy, Debug)]
pub enum MoveDirection {
    #[default]
    None,
    Left,
    Right,
}

impl From<usize> for MoveDirection {
    fn from(value: usize) -> Self {
        match value {
            0 => Self::None,
            1 => Self::Left,
            _ => Self::Right,
        }
    }
}

impl From<MoveDirection> for i8 {
    fn from(value: MoveDirection) -> Self {
        match value {
            MoveDirection::None => 0,
            MoveDirection::Left => -1,
            MoveDirection::Right => 1,
        }
    }
}

pub struct Player {
    pub rect: math::Rect,
    pub velocity: math::Vec2,
    pub current_direction: MoveDirection, // True -> Right as True == 1 == positive movement == Right
    ignore_collisions_tag: bool,
}

impl Player {
    pub fn new() -> Self {
        Self {
            rect: math::Rect::new_from_center(
                (crate::GAME_WIDTH / 2., crate::GAME_HEIGHT / 2.),
                (PLAYER_SIZE, PLAYER_SIZE),
                0.,
            ),
            velocity: math::Vec2::ZERO,
            current_direction: MoveDirection::None,
            ignore_collisions_tag: false,
        }
    }

    pub fn update(&mut self, platforms: &VecDeque<Platform>, dt: f64) {
        self.rect
            .set_center(self.rect.center() + self.velocity * dt);

        self.update_collision(platforms);

        // println!("{}", self.rect.center());

        self.velocity.y += GRAVITY * dt;
        self.velocity.x = SPEED * i8::from(self.current_direction) as f64;

        self.current_direction = MoveDirection::None;

        let wrapped_x = self.rect.center().x.rem_euclid(crate::GAME_WIDTH);
        self.rect
            .set_center(math::Vec2::new(wrapped_x, self.rect.center().y));
    }

    // Returns if the player collided this frame
    fn update_collision(&mut self, platforms: &VecDeque<Platform>) -> bool {
        let mut collided_this_frame = false;
        for platform in platforms.iter() {
            if !math::collision::rect_rect_no_r(&self.rect, &platform.rect) {
                continue;
            }
            collided_this_frame = true;
            // If player entered the platform from below, ingore all collision until out
            if self.ignore_collisions_tag {
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
                    .set_center(self.rect.center() - math::Vec2::new(0., 1.));
            } else if self.velocity.y < 0. && !self.ignore_collisions_tag {
                // println!("Collision from below");
                self.ignore_collisions_tag = true;
                // self.velocity.y = 0.0;
                // self.rect.set_center(math::Vec2::new(
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
