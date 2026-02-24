mod button;
mod graph;
mod image;
mod progress_bar;
mod text;
mod text_edit;

pub use button::Button;
pub use graph::{Graph, GraphText};
pub use image::Image;
pub use progress_bar::{ProgressBar, ProgressDirection};
pub use text::{Text, TextBit};
pub use text_edit::TextEdit;

#[enum_dispatch::enum_dispatch(TElement)]
pub enum Element {
    Button,
    Graph,
    Text,
    TextEdit,
    Image,
    ProgressBar,
}

#[enum_dispatch::enum_dispatch]
pub trait TElement: std::any::Any {
    fn draw(
        &mut self,
        _: &mut ggez::Context,
        _back: &mut ggez::graphics::MeshBuilder,
        _ui: &mut ggez::graphics::MeshBuilder,
        _front: &mut ggez::graphics::MeshBuilder,
        _: &mut crate::render::RenderRequest,
    ) -> ggez::GameResult;

    fn get_size_value(&self) -> &super::Vector;

    fn get_pos_value(&self) -> &super::Position;

    fn get_id(&self) -> super::Id;

    /*
        ↑
        Required
        Auto impls
        ↓
    */

    fn get_computed_size(&self, ctx: &mut ggez::Context) -> math::Vec2 {
        let sizev = self.get_size_value();

        math::Point::new(sizev.x().compute(ctx), sizev.y().compute(ctx))
    }

    fn get_computed_pos(
        &self,
        ctx: &mut ggez::Context,
        size_opt: Option<math::Vec2>,
    ) -> math::Point {
        let posv = self.get_pos_value();

        let size = size_opt.unwrap_or_else(|| self.get_computed_size(ctx));

        posv.compute(ctx, &size)
    }

    fn get_computed_rect(&self, ctx: &mut ggez::Context) -> math::Rect {
        let size = self.get_computed_size(ctx);

        let position = self.get_computed_pos(ctx, Some(size));

        math::Rect::new_from_center(position, size, 0.)
    }

    /*
        Events
    */

    fn on_mouse_press(
        &mut self,
        _: &mut ggez::Context,
        _: &ggez::input::mouse::MouseButton,
        _: &math::Point,
    ) {
    }
    fn on_mouse_release(
        &mut self,
        _: &mut ggez::Context,
        _: &ggez::input::mouse::MouseButton,
        _: &math::Point,
    ) {
    }
    fn on_mouse_motion(&mut self, _: &mut ggez::Context, _: &math::Point, _: &math::Point) {}
    fn on_mouse_wheel(&mut self, _: &mut ggez::Context, _: &math::Point) {}
    fn on_key_down(
        &mut self,
        _: &mut ggez::Context,
        _: &ggez::input::keyboard::KeyInput,
        _: &bool,
    ) {
    }
    fn on_key_up(&mut self, _: &mut ggez::Context, _: &ggez::input::keyboard::KeyInput) {}
    fn on_text_input(&mut self, _: &mut ggez::Context, _: &char) {}
    fn on_new_frame(&mut self) {}
}

/// Constructors
impl Element {
    #[inline]
    pub fn new_button(
        id: impl Into<super::Id>,
        position: impl Into<super::Position>, // Center
        size: impl Into<super::Vector>,
        style: super::style::Bundle,
    ) -> Self {
        Self::Button(button::Button::new(id, position, size, style))
    }

    #[inline]
    pub fn new_graph(
        id: impl Into<super::Id>,
        position: impl Into<super::Position>, // Center
        size: impl Into<super::Vector>,
        style: super::Style,
        text: Option<graph::GraphText>,
    ) -> Self {
        Self::Graph(graph::Graph::new(id, position, size, style, text))
    }

    #[inline]
    pub fn new_text(
        id: impl Into<super::Id>,
        position: impl Into<super::Position>, // Center
        size: impl Into<super::Value>,
        style: super::Style,
        parts: Vec<TextBit>,
    ) -> Self {
        let size = size.into();
        Self::Text(Text::new(id, position, size, style, parts))
    }

    #[inline]
    pub fn new_text_edit(
        id: impl Into<super::Id>,
        position: impl Into<super::Position>, // Center
        width: impl Into<super::Value>,
        rows: u64,
        font_size: f64,
        style: super::style::Bundle,
    ) -> Self {
        Self::TextEdit(TextEdit::new(id, position, width, rows, font_size, style))
    }

    #[inline]
    pub fn new_image(
        id: impl Into<super::Id>,
        position: impl Into<super::Position>, // Center
        size: impl Into<super::Vector>,
        style: super::Style,
        image: crate::assets::texture::TextureId,
    ) -> Self {
        Self::Image(image::Image::new(id, position, size, style, image))
    }
}

/// Getters
impl Element {
    //Credit: Rust Programming discord: bruh![moment] (170999103482757120)
    // https://discord.com/channels/273534239310479360/1120124565591425034/1162574037633990736
    // Could be done by a macro lmao
    pub fn try_inner<T: TElement>(&self) -> Option<&T> {
        match self {
            Self::Button(inner) => (inner as &dyn std::any::Any).downcast_ref(),
            Self::Graph(inner) => (inner as &dyn std::any::Any).downcast_ref(),
            Self::Text(inner) => (inner as &dyn std::any::Any).downcast_ref(),
            Self::TextEdit(inner) => (inner as &dyn std::any::Any).downcast_ref(),
            Self::Image(inner) => (inner as &dyn std::any::Any).downcast_ref(),
            Self::ProgressBar(inner) => (inner as &dyn std::any::Any).downcast_ref(),
        }
    }
    pub fn inner<T: TElement>(&self) -> &T {
        self.try_inner().expect("Wrong widget type")
    }

    pub fn try_inner_mut<T: TElement>(&mut self) -> Option<&mut T> {
        match self {
            Self::Button(inner) => (inner as &mut dyn std::any::Any).downcast_mut(),
            Self::Graph(inner) => (inner as &mut dyn std::any::Any).downcast_mut(),
            Self::Text(inner) => (inner as &mut dyn std::any::Any).downcast_mut(),
            Self::TextEdit(inner) => (inner as &mut dyn std::any::Any).downcast_mut(),
            Self::Image(inner) => (inner as &mut dyn std::any::Any).downcast_mut(),
            Self::ProgressBar(inner) => (inner as &mut dyn std::any::Any).downcast_mut(),
        }
    }
    pub fn inner_mut<T: TElement>(&mut self) -> &mut T {
        self.try_inner_mut().expect("Wrong widget type")
    }

    pub fn inner_as_trait(&self) -> &dyn TElement {
        match self {
            Self::Button(inner) => inner,
            Self::Graph(inner) => inner,
            Self::Text(inner) => inner,
            Self::TextEdit(inner) => inner,
            Self::Image(inner) => inner,
            Self::ProgressBar(inner) => inner,
        }
    }
    pub fn inner_as_trait_mut(&mut self) -> &mut dyn TElement {
        match self {
            Self::Button(inner) => inner,
            Self::Graph(inner) => inner,
            Self::Text(inner) => inner,
            Self::TextEdit(inner) => inner,
            Self::Image(inner) => inner,
            Self::ProgressBar(inner) => inner,
        }
    }

    // this function creates wayyy too much asm bloat
    // pub fn inner_as_trait_boxed(&mut self) -> Box<&mut dyn TElement> {
    //     match self {
    //         Self::Button(inner) => Box::new(inner),
    //         Self::Graph(inner) => Box::new(inner),
    //     }
    // }
}

macro_rules! gen_trait_fn_ref{
    ($fn_name:ident $(, $arg:ident : $arg_ty:ty)* => $ret_ty:ty) => {
        pub fn $fn_name(&self, $($arg : $arg_ty),*) -> $ret_ty {
            self.inner_as_trait().$fn_name($($arg),*)
        }
    };
    ($fn_name:ident => $ret_ty:ty) => {
        pub fn $fn_name(&self) -> $ret_ty {
            self.inner_as_trait().$fn_name()
        }
    };
}

macro_rules! gen_trait_fn_refmut {
    ($fn_name:ident $(, $arg:ident : $arg_ty:ty)* => $ret_ty:ty) => {
        pub fn $fn_name(&mut self, $($arg : $arg_ty),*) -> $ret_ty {
            self.inner_as_trait_mut().$fn_name($($arg),*)
        }
    };
    ($fn_name:ident => $ret_ty:ty) => {
        pub fn $fn_name(&mut self) -> $ret_ty {
            self.inner_as_trait().$fn_name()
        }
    };
}

// macro_rules! gen_trait_fn_value {
//     ($fn_name:ident $(, $arg:ident : $arg_ty:ty)* => $ret_ty:ty) => {
//         pub fn $fn_name(self, $($arg : $arg_ty),*) -> $ret_ty {
//             self.inner_as_trait().$fn_name($($arg),*)
//         }
//     };
//     ($fn_name:ident => $ret_ty:ty) => {
//         pub fn $fn_name(self) -> $ret_ty {
//             self.inner_as_trait().$fn_name()
//         }
//     };
// }

// macro_rules! gen_trait_fn_noself {
//     ($fn_name:ident $(, $arg:ident : $arg_ty:ty)* => $ret_ty:ty) => {
//         pub fn $fn_name($($arg : $arg_ty),*) -> $ret_ty {
//             self.inner_as_trait().$fn_name($($arg),*)
//         }
//     };
//     ($fn_name:ident => $ret_ty:ty) => {
//         pub fn $fn_name() -> $ret_ty {
//             self.inner_as_trait().$fn_name()
//         }
//     };
// }
/// This is so you don't need to import the trait everytime you want to use an Element, you can short circuit it by doing Element::trait_function()
#[allow(dead_code)]
impl Element {
    gen_trait_fn_refmut!(
        draw,
        ctx: &mut ggez::Context,
        back: &mut ggez::graphics::MeshBuilder,
        ui: &mut ggez::graphics::MeshBuilder,
        front: &mut ggez::graphics::MeshBuilder,
        render_request: &mut crate::render::RenderRequest
        => ggez::GameResult
    );
    gen_trait_fn_ref!(
        get_size_value
        => &super::Vector
    );

    gen_trait_fn_ref!(
        get_pos_value
        => &super::Position
    );

    gen_trait_fn_ref!(
        get_id
        => super::Id
    );
    /*
        ↑
        Required
        Auto impls
        ↓
    */
    gen_trait_fn_ref!(
        get_computed_size,
        ctx: &mut ggez::Context
        => math::Vec2
    );
    gen_trait_fn_ref!(
        get_computed_pos,
        ctx: &mut ggez::Context,
        size_opt: Option<math::Vec2>
        => math::Point
    );
    gen_trait_fn_ref!(
        get_computed_rect,
        ctx: &mut ggez::Context
        => math::Rect
    );

    /*
        Events
    */
    gen_trait_fn_refmut!(
        on_mouse_press,
        ctx: &mut ggez::Context,
        button: &ggez::input::mouse::MouseButton,
        position: &math::Point
        => ()
    );
    gen_trait_fn_refmut!(
        on_mouse_release,
        ctx: &mut ggez::Context,
        button: &ggez::input::mouse::MouseButton,
        position: &math::Point
        => ()
    );
    gen_trait_fn_refmut!(
        on_mouse_motion,
        ctx: &mut ggez::Context,
        position: &math::Point,
        delta: &math::Point
        => ()
    );
    gen_trait_fn_refmut!(
        on_mouse_wheel,
        ctx: &mut ggez::Context,
        delta: &math::Point
        => ()
    );
    gen_trait_fn_refmut!(
        on_key_down,
        ctx: &mut ggez::Context,
        key: &ggez::input::keyboard::KeyInput,
        repeated: &bool
        => ()
    );
    gen_trait_fn_refmut!(
        on_key_up,
        ctx: &mut ggez::Context,
        key: &ggez::input::keyboard::KeyInput
        => ()
    );
    gen_trait_fn_refmut!(
        on_text_input,
        ctx: &mut ggez::Context,
        character: &char
        => ()
    );
    gen_trait_fn_refmut!(
        on_new_frame
        =>()
    );
}
