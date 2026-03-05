pub struct ProgressBar {
    id: crate::ui::Id,
    position: crate::ui::Position,
    size: crate::ui::Vector,
    state: crate::ui::State,
    style: crate::ui::style::Bundle,
    direction: ProgressDirection,
    max_value: f64,
    current_value: f64,
    ratio: f64,
}

pub enum ProgressDirection {
    Horizontal,
    Vertical,
}

impl ProgressBar {
    pub fn new(
        id: impl Into<crate::ui::Id>,
        position: impl Into<crate::ui::Position>, // Center
        size: impl Into<crate::ui::Vector>,
        style: crate::ui::style::Bundle,
        direction: ProgressDirection,
        max_value: f64,
        current_value: f64,
    ) -> Self {
        Self {
            id: id.into(),
            position: position.into(),
            size: size.into(),
            state: crate::ui::State::default(),
            style,
            direction,
            max_value,
            current_value,
            ratio: current_value / max_value,
        }
    }

    pub fn set_current_value(&mut self, new_value: f64) {
        self.current_value = new_value;
        self.ratio = self.current_value / self.max_value;
    }

    fn get_current_style(&self) -> &crate::ui::style::Bundle {
        &self.style
    }

    pub fn get_state(&self) -> crate::ui::State {
        self.state
    }
}

impl super::TElement for ProgressBar {
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

        let value_display_rect = match self.direction {
            ProgressDirection::Horizontal => math::Rect::new(
                rect.aa_topleft(),
                (rect.width() * self.ratio, rect.height()),
                0.,
            ),
            ProgressDirection::Vertical => {
                let topleft = rect.aa_topleft();

                math::Rect::new(
                    (topleft.x, topleft.y + rect.height() * (1.0 - self.ratio)),
                    (rect.width(), rect.height() * self.ratio),
                    0.,
                )
            }
        };

        ui_mesh.rectangle(
            ggez::graphics::DrawMode::fill(),
            value_display_rect.into(),
            (*style.get_color()).into(),
        )?;

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
