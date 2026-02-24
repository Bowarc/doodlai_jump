#[macro_use]
extern crate log;

mod action;
mod assets;
mod config;
mod gui;
mod input;
mod render;
mod ui;
mod utils;

struct Display {
    cfg: config::Config,
    renderer: render::Renderer,
    asset_mgr: assets::Assets,
    frame_stats: utils::framestats::FrameStats,
    gui_menu: gui::Gui,
    global_ui: ui::UserInterface,
    game: game::Game,
    nn: neat::NeuralNetwork<{ ring::AGENT_IN }, { ring::AGENT_OUT }>,
    threadpool: stp::ThreadPool,
}

impl Display {
    fn new(ctx: &mut ggez::Context, mut cfg: config::Config) -> ggez::GameResult<Self> {
        let threadpool = stp::ThreadPool::new(2);

        let renderer = render::Renderer::new();

        let gui_menu = gui::Gui::new(ctx, &mut cfg)?;

        let asset_mgr = assets::Assets::new(ctx, &cfg, threadpool.clone());

        let mut global_ui = ui::UserInterface::default();

        let _ = global_ui.add_element(
            ui::element::Element::new_graph(
                "fps graph",
                (ui::Anchor::TopRight, (-2., 2.)),
                (200., 50.),
                ui::Style::new(
                    render::Color::random_rgb(),
                    Some(ui::style::Background::new(
                        render::Color::from_rgba(23, 23, 23, 150),
                        None,
                    )),
                    Some(ui::style::Border::new(render::Color::WHITE, 1.)),
                ),
                Some(
                    ui::element::GraphText::default()
                        .anchor(ui::Anchor::Topleft)
                        .offset(math::Vec2::ONE)
                        .text(|val| -> String { format!("{}fps", val as i32) })
                        .size(5.)
                        .color(render::Color::random_rgb()),
                ),
            ),
            "",
        );

        let _ = global_ui.add_element(
            ui::element::Element::new_text(
                "Score",
                (ui::Anchor::TopRight, (-50., 70.)),
                3.,
                ui::Style::new(
                    render::Color::random_rgb(),
                    Some(ui::style::Background::new(
                        render::Color::from_rgba(23, 23, 23, 150),
                        None,
                    )),
                    None,
                ),
                vec![],
            ),
            "",
        );

        let _ = global_ui.add_element(
            ui::element::Element::new_text(
                "mouse pos text",
                (ui::Anchor::BotRight, (-1., -1.)),
                20.,
                ui::Style::new(
                    render::Color::random_rgb(),
                    Some(ui::style::Background::new(
                        render::Color::from_rgba(20, 20, 20, 100),
                        None,
                    )),
                    Some(ui::style::Border::new(render::Color::random_rgb(), 1.)),
                ),
                vec![],
            ),
            "",
        );

        Ok(
            Self {
                cfg,
                renderer,
                asset_mgr,
                frame_stats: utils::framestats::FrameStats::new(),
                gui_menu,
                global_ui,
                game: game::Game::new(),
                nn: serde_json::from_str::<
                    neat::NeuralNetwork<{ ring::AGENT_IN }, { ring::AGENT_OUT }>,
                >(include_str!("./nnt.json"))
                .unwrap(),
                threadpool,
            },
        )
    }
}

impl ggez::event::EventHandler for Display {
    fn update(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {
        self.frame_stats.end_frame();
        self.frame_stats.begin_frame();
        self.frame_stats.begin_update();

        let dt: f64 = ctx.time.delta().as_secs_f64();

        self.game.update(dt);

        // {
        //     if input::pressed(ctx, input::Input::KeyboardQ) {
        //         self.game.player_move_left()
        //     } else if input::pressed(ctx, input::Input::KeyboardD) {
        //         self.game.player_move_right()
        //     }
        // }
        {
            let output = self.nn.predict(ring::generate_inputs(&self.game));

            println!("output: {output:?}");
            match neat::MaxIndex::max_index(output.iter()).unwrap() {
                0 => (), // No action
                1 => self.game.player_move_left(),
                2 => self.game.player_move_right(),
                _ => (),
            }
        }

        self.gui_menu.update(ctx, &mut self.cfg)?;

        self.global_ui
            .get_element("Score")
            .inner_mut::<ui::element::Text>()
            .replace_bits(vec![format!("{}", self.game.score()).into()]);

        self.global_ui.update(ctx);

        self.global_ui
            .get_element("fps graph")
            .inner_mut::<ui::element::Graph>()
            .push(ctx.time.fps(), dt);

        self.global_ui
            .get_element("mouse pos text")
            .inner_mut::<ui::element::Text>()
            .replace_bits(vec![format!("{:?}", ctx.mouse.position()).into()]);

        self.asset_mgr.update(ctx);

        self.frame_stats.end_update();
        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {
        self.frame_stats.begin_draw();

        ggez::graphics::Canvas::from_frame(ctx, Some(render::Color::BLACK.into())).finish(ctx)?;

        let render_request = self.renderer.render_request();

        self.frame_stats.draw(
            math::Point::ZERO,
            ctx,
            render_request,
            &self.threadpool
        )?;

        self.gui_menu.draw(ctx, render_request)?;

        self.global_ui.draw(ctx, render_request)?;

        for platform in self.game.platforms.iter() {
            render_request.add(
                assets::texture::TextureId::GreenPlatform,
                render::DrawParam::new()
                    .pos(platform.rect.center() - math::Vec2::new(0., self.game.scroll as f64))
                    .size(platform.rect.size()),
                render::Layer::Game,
            );
        }

        render_request.add(
            match self.game.player.direction() {
                0 | 1 => assets::texture::TextureId::DoodleRight,
                -1 => assets::texture::TextureId::DoodleLeft,
                _ => unreachable!(),
            },
            render::DrawParam::new()
                .pos(self.game.player.rect.center() - math::Vec2::new(0., self.game.scroll as f64))
                .size(self.game.player.rect.size()),
            render::Layer::Game,
        );

        let render_log = self.renderer.run(
            ctx,
            self.gui_menu.backend_mut(),
            &mut self.asset_mgr.texture_storage
        )?;

        self.frame_stats.set_render_log(render_log);
        self.frame_stats.end_draw();

        Ok(())
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut ggez::Context,
        button: ggez::input::mouse::MouseButton,
        x: f32,
        y: f32,
    ) -> ggez::GameResult {
        self.global_ui.register_mouse_press(button, x, y);

        Ok(())
    }

    fn mouse_button_up_event(
        &mut self,
        _ctx: &mut ggez::Context,
        button: ggez::input::mouse::MouseButton,
        x: f32,
        y: f32,
    ) -> ggez::GameResult {
        self.global_ui.register_mouse_release(button, x, y);
        Ok(())
    }

    fn mouse_motion_event(
        &mut self,
        _ctx: &mut ggez::Context,
        x: f32,
        y: f32,
        dx: f32,
        dy: f32,
    ) -> ggez::GameResult {
        self.global_ui.register_mouse_motion(x, y, dx, dy);
        Ok(())
    }

    fn mouse_enter_or_leave(
        &mut self,
        _ctx: &mut ggez::Context,
        _entered: bool,
    ) -> ggez::GameResult {
        Ok(())
    }

    fn mouse_wheel_event(&mut self, _ctx: &mut ggez::Context, x: f32, y: f32) -> ggez::GameResult {
        self.gui_menu
            .backend_mut()
            .input
            .mouse_wheel_event(x * 10., y * 10.);
        self.global_ui.register_mouse_wheel(x, y);

        Ok(())
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut ggez::Context,
        input: ggez::input::keyboard::KeyInput,
        repeated: bool,
    ) -> ggez::GameResult {
        self.global_ui.register_key_down(input, repeated);
        Ok(())
    }

    fn key_up_event(
        &mut self,
        _ctx: &mut ggez::Context,
        input: ggez::input::keyboard::KeyInput,
    ) -> ggez::GameResult {
        self.global_ui.register_key_up(input);
        Ok(())
    }

    fn text_input_event(&mut self, _ctx: &mut ggez::Context, character: char) -> ggez::GameResult {
        self.gui_menu
            .backend_mut()
            .input
            .text_input_event(character);
        self.global_ui.register_text_input(character);
        Ok(())
    }

    fn touch_event(
        &mut self,
        _ctx: &mut ggez::Context,
        _phase: ggez::event::winit_event::TouchPhase,
        _x: f64,
        _y: f64,
    ) -> ggez::GameResult {
        Ok(())
    }

    fn gamepad_button_down_event(
        &mut self,
        _ctx: &mut ggez::Context,
        _btn: ggez::event::Button,
        _id: ggez::input::gamepad::GamepadId,
    ) -> ggez::GameResult {
        Ok(())
    }

    fn gamepad_button_up_event(
        &mut self,
        _ctx: &mut ggez::Context,
        _btn: ggez::event::Button,
        _id: ggez::input::gamepad::GamepadId,
    ) -> ggez::GameResult {
        Ok(())
    }

    fn gamepad_axis_event(
        &mut self,
        _ctx: &mut ggez::Context,
        _axis: ggez::event::Axis,
        _value: f32,
        _id: ggez::input::gamepad::GamepadId,
    ) -> ggez::GameResult {
        Ok(())
    }

    fn focus_event(&mut self, _ctx: &mut ggez::Context, _gained: bool) -> ggez::GameResult {
        Ok(())
    }

    fn quit_event(&mut self, _ctx: &mut ggez::Context) -> ggez::GameResult<bool> {
        debug!("See you next time. . .");

        Ok(false)
    }

    fn resize_event(
        &mut self,
        _ctx: &mut ggez::Context,
        _width: f32,
        _height: f32,
    ) -> ggez::GameResult {
        Ok(())
    }

    fn on_error(
        &mut self,
        _ctx: &mut ggez::Context,
        _origin: ggez::event::ErrorOrigin,
        e: ggez::GameError,
    ) -> bool {
        error!("{e}");

        true
    }
}

fn main() -> ggez::GameResult {
    logger::init(
        logger::Config::default()
            .output(logger::Output::Stdout)
            .level(log::LevelFilter::Trace)
            .filters(&[
                ("wgpu_core", log::LevelFilter::Warn),
                ("wgpu_hal", log::LevelFilter::Error),
                ("gilrs", log::LevelFilter::Off),
                ("naga", log::LevelFilter::Warn),
                ("networking", log::LevelFilter::Debug),
                ("ggez", log::LevelFilter::Warn),
            ]),
    );

    let config: config::Config = config::load();

    let cb = ggez::ContextBuilder::new("Doodlai display window", "Bowarc")
        .resources_dir_name("resources\\external\\")
        .window_setup(
            ggez::conf::WindowSetup::default()
                .title("Display game")
                .samples(config.window.samples)
                .vsync(config.window.v_sync)
                .srgb(config.window.srgb),
        )
        .window_mode(config.window.into())
        .backend(ggez::conf::Backend::Vulkan);

    let (mut ctx, event_loop) = cb.build()?;

    let game = Display::new(&mut ctx, config)?;

    ggez::event::run(ctx, event_loop, game);
}
