use serde::{Deserialize, Serialize};
use time::DTDelay;
use super::TextureId;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ImageType {
    Static(TextureId),
    Animated(Animation),
}

type Frame = (TextureId, f64);

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(from = "Vec<Frame>")]
#[serde(into = "Vec<Frame>")]
pub struct Animation {
    frames: Vec<Frame>, // (Texture, frame time in ms)
    current_frame: usize,
    total_cycle_time: f64,
    delay: DTDelay,
}

impl ImageType {
    #[inline]
    pub fn new_static(id: impl Into<TextureId>) -> Self {
        Self::Static(id.into())
    }

    #[inline]
    pub fn new_animated(anim: impl Into<Animation>) -> Self {
        Self::Animated(anim.into())
    }

    pub fn update(&mut self, dt: f64) {
        if let ImageType::Animated(animation) = self {
            animation.update(dt)
        }
    }
    pub fn get(&self) -> TextureId {
        match self {
            ImageType::Static(texture_id) => *texture_id,
            ImageType::Animated(animation) => animation.get_texture(),
        }
    }
    pub fn upget(&mut self, dt: f64) -> TextureId {
        match self {
            ImageType::Static(texture_id) => *texture_id,
            ImageType::Animated(animation) => {
                animation.update(dt);
                animation.get_texture()
            }
        }
    }
}

impl Animation {
    pub fn new() -> Self {
        Self {
            frames: Vec::new(),
            current_frame: 0,
            total_cycle_time: 0.0,
            delay: DTDelay::new(0_f64),
        }
    }
    pub fn from_frames(frames: Vec<Frame>) -> Self {
        if frames.is_empty() {
            panic!("Can't initialize an Animation with empty frames");
            // panic!("..");
        }
        let first_frame_time = frames.first().unwrap().1;
        let total_cycle_time: f64 = frames.iter().map(|(_, time)| time).sum::<f64>();
        Self {
            frames,
            current_frame: 0,
            total_cycle_time,
            delay: DTDelay::new(first_frame_time),
        }
    }
    fn get_texture(&self) -> TextureId {
        self.frames.get(self.current_frame).unwrap().0
    }

    pub fn update(&mut self, dt: f64) {
        // "since_elapsed" is always positive(unless delay not done)
        // example:
        // Timeline: --------------------------------
        //       delay ended here|    |but is checked here
        // "overtime" is the time between the two bars
        // and we use it to start a delay 'too late' without consequences
        self.delay.update(dt);

        if self.delay.ended() {
            let mut overtime = self.delay.time_since_ended();
            let freeze_prevention = true;
            if freeze_prevention {
                // Here we take out big chunks of time
                // This is supposed to negate the possiblility of the
                // system going through the same frame multiple times
                overtime %= self.total_cycle_time;
            }

            loop {
                // This take out little chunks of time
                // This is supposed to go fast-forward frames
                // till the overtime is less than the frame's time
                let this_frame_time = self.frames.get(self.current_frame).unwrap().1;
                if overtime > this_frame_time {
                    overtime -= this_frame_time;
                    self.current_frame = (self.current_frame + 1) % self.frames.len();
                } else {
                    break;
                }
            }

            self.current_frame = (self.current_frame + 1) % self.frames.len();

            let mut new_delay = DTDelay::new(self.frames.get(self.current_frame).unwrap().1);

            new_delay.update(overtime); // More info about this line in the comment at the start of this function

            self.delay = new_delay;
        }
    }
}

impl From<Vec<Frame>> for Animation {
    fn from(v: Vec<Frame>) -> Self {
        Animation::from_frames(v)
    }
}

impl From<Animation> for Vec<Frame> {
    fn from(anim: Animation) -> Self {
        anim.frames
    }
}
