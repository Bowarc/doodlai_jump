pub struct Button {
    id: crate::ui::Id,
    position: crate::ui::Position,
    size: crate::ui::Vector,
    state: crate::ui::State,
    style: crate::ui::style::Bundle,
}

impl Button {
    pub fn new(
        id: impl Into<crate::ui::Id>,
        position: impl Into<crate::ui::Position>, // Center
        size: impl Into<crate::ui::Vector>,
        style: crate::ui::style::Bundle,
    ) -> Self {
        Self {
            id: id.into(),
            position: position.into(),
            size: size.into(),
            state: crate::ui::State::default(),
            style,
        }
    }

    pub fn clicked_this_frame(&self) -> bool {
        self.state.clicked_this_frame()
    }

    fn get_current_style(&self) -> &crate::ui::style::Bundle {
        &self.style
    }

    pub fn get_state(&self) -> crate::ui::State {
        self.state
    }
}

impl super::TElement for Button {
    fn draw(
        &mut self,
        ctx: &mut ggez::Context,
        back_mesh: &mut ggez::graphics::MeshBuilder,
        ui_mesh: &mut ggez::graphics::MeshBuilder,
        front_mesh: &mut ggez::graphics::MeshBuilder,
        render_request: &mut crate::render::RenderRequest,
    ) -> ggez::GameResult {
        let rect = self.get_computed_rect(ctx);
        let style = self.style.get(&self.state);

        // draw background
        if let Some(bg) = style.get_bg() {
            bg.draw(back_mesh, render_request, rect)?
        }

        // draw border
        if let Some(border) = style.get_border() {
            border.draw(front_mesh, rect)?;
        };

        // I don't really see the point of this rect, if you want a background, use the background style option.
        // ui_mesh.rectangle(
        //     ggez::graphics::DrawMode::fill(),
        //     rect.into(),
        //     (*style.get_color()).into(),
        // )?;

        Ok(())
    }
    fn get_size_value(&self) -> &crate::ui::Vector {
        &self.size
    }
    fn get_pos_value(&self) -> &crate::ui::Position {
        &self.position
    }
    fn get_id(&self) -> crate::ui::Id {
        self.id.clone()
    }

    fn on_new_frame(&mut self) {
        self.state.new_frame();
    }
    fn on_mouse_motion(
        &mut self,
        ctx: &mut ggez::Context,
        position: &math::Point,
        _delta: &math::Point,
    ) {
        if math::collision::point_rect(position, &self.get_computed_rect(ctx)) {
            self.state.mouse_hover_self()
        } else {
            self.state.mouse_hover_not_self()
        }
    }

    fn on_mouse_press(
        &mut self,
        ctx: &mut ggez::Context,
        _button: &ggez::input::mouse::MouseButton,
        position: &math::Point,
    ) {
        if math::collision::point_rect(position, &self.get_computed_rect(ctx)) {
            self.state.mouse_press_self()
        } else {
            self.state.mouse_press_not_self()
        }
    }
    fn on_mouse_release(
        &mut self,
        ctx: &mut ggez::Context,
        _button: &ggez::input::mouse::MouseButton,
        position: &math::Point,
    ) {
        if math::collision::point_rect(position, &self.get_computed_rect(ctx)) {
            self.state.mouse_release_self()
        } else {
            self.state.mouse_release_not_self()
        }
    }
}
