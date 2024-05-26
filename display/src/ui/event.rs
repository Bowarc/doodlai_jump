#[derive(Copy, Clone, Debug)]
pub enum Event {
    MousePress {
        button: ggez::input::mouse::MouseButton,
        position: maths::Point,
    },
    MouseRelease {
        button: ggez::input::mouse::MouseButton,
        position: maths::Point,
    },
    MouseMotion {
        position: maths::Point,
        delta: maths::Vec2,
    },
    MouseWheel {
        delta: maths::Point,
    },
    KeyDown {
        key: ggez::input::keyboard::KeyInput,
        repeated: bool,
    },
    KeyUp {
        key: ggez::input::keyboard::KeyInput,
    },
    TextInput {
        character: char,
    },
}
