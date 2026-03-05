use {
    crate::render,
    ggez::{
        graphics::{self, Drawable},
        Context, GameResult,
    },
};

pub struct FrameStats {
    update_time: time::Stopwatch,
    draw_time: time::Stopwatch,
    frame_time: time::Stopwatch,
    render_log: render::RenderLog,
}

impl FrameStats {
    pub fn new() -> Self {
        Self {
            update_time: time::Stopwatch::new(),
            draw_time: time::Stopwatch::new(),
            frame_time: time::Stopwatch::new(),
            render_log: render::RenderLog::new(),
        }
    }
    pub fn begin_frame(&mut self) {
        self.frame_time.start()
    }
    pub fn begin_update(&mut self) {
        self.update_time.start()
    }
    pub fn begin_draw(&mut self) {
        self.draw_time.start()
    }
    pub fn end_frame(&mut self) {
        self.frame_time.stop()
    }
    pub fn end_update(&mut self) {
        self.update_time.stop()
    }
    pub fn end_draw(&mut self) {
        self.draw_time.stop()
    }
    pub fn frame_time(&self) -> std::time::Duration {
        self.frame_time.read()
    }
    pub fn update_time(&self) -> std::time::Duration {
        self.update_time.read()
    }
    pub fn draw_time(&self) -> std::time::Duration {
        self.draw_time.read()
    }
    pub fn set_render_log(&mut self, render_log: render::RenderLog) {
        self.render_log = render_log
    }
    pub fn render_log(&self) -> render::RenderLog {
        self.render_log
    }

    pub fn draw(
        &self,
        position: math::Point,
        ctx: &mut Context,
        render_request: &mut render::RenderRequest,
        thread_pool: &stp::ThreadPool,
    ) -> GameResult {
        let spacing = " ";
        let background_min_width = 272.;

        let time_frag = graphics::TextFragment::new(format!(
            "Time mesurements:\n{spacing}Fps        : {:.2}\n{spacing}Frame time : {}\n{spacing}Update time: {}\n{spacing}Draw time  : {}",
            // 1./ctx.time.delta().as_secs_f64(),
            ctx.time.fps(), // ctx.time.fps(), the first one is updating A LOT but is accurate, the latter is averaged over last 100 frames
            time::format(&self.frame_time(), 1),
            time::format(&self.update_time(), 1),
            time::format(&self.draw_time(), 1),
        ))
        .color(graphics::Color::from_rgb(0, 150, 150));

        let render_frag = graphics::TextFragment::new(format!(
            "Render:\n{spacing}Elements  : {}\n{spacing}Sprites   : {}\n{spacing}Sprite not found: {}\n{spacing}Meshes    : {}\n{spacing}Texts     : {}\n{spacing}Draw calls: {}\n{spacing}In loading assets: {}\n{spacing}Task count: {}",
            self.render_log.elements(),
            self.render_log.textures(),
            self.render_log.textures_not_found(),
            self.render_log.meshes(),
            self.render_log.texts(),
            self.render_log.draw_calls(),
            // asset_loading_debug_text,
            "TODO",
            thread_pool.flying_tasks_count()
        )).color(graphics::Color::from_rgb(150,150,0));

        let mut total_text = graphics::Text::new(time_frag);
        total_text.add(render_frag);

        total_text.set_layout(graphics::TextLayout::top_left());

        let ttd = total_text.dimensions(ctx).unwrap();
        render_request.add(
            total_text,
            render::DrawParam::new().pos(position),
            render::Layer::Ui,
        );

        render_request.add(
            graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                math::Rect::new(
                    math::Point::new(ttd.x as f64, ttd.y as f64),
                    math::Vec2::new(ttd.w.max(background_min_width) as f64, ttd.h as f64),
                    0.,
                )
                .into(),
                graphics::Color::from_rgba(0, 0, 0, 200),
            )?,
            render::DrawParam::default(),
            render::Layer::UiBackground,
        );
        Ok(())
    }
}
