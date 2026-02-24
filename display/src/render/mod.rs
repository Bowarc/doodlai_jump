mod color;
mod draw_param;
mod layer;
mod render_log;
mod render_request;

pub use color::Color;
pub use draw_param::DrawParam;
use ggez::graphics;
pub use layer::Layer;
pub use render_log::RenderLog;
pub use render_request::{RenderRequest, RenderRequestBit};

use crate::assets::texture;

pub struct Renderer {
    render_request: RenderRequest,
}

// currently the renderer has a 1 frame delay on the framestats render
// as for some obvious reasons, i can't display the stats of the frame as it's being drawn
// is it a problem ?
// i don't think so but anyway i have an idea of how to fix it if it becomes a problem

impl Renderer {
    pub fn new() -> Self {
        Self {
            render_request: RenderRequest::new(),
        }
    }

    pub fn render_request(&mut self) -> &mut RenderRequest {
        &mut self.render_request
    }

    // Sprite rendering is not done atm
    pub fn run(
        &mut self,
        ctx: &mut ggez::Context,
        menu_backend: &mut ggegui::EguiBackend,
        texture_storage: &mut texture::Storage,
    ) -> ggez::GameResult<RenderLog> {
        let mut layer_index = 0;
        let mut global_log = RenderLog::new();

        while let Some(layer) = Layer::get(layer_index) {
            layer_index += 1;
            let Some(bits) = self.render_request.get_mut(&layer) else {
                continue;
            };

            let mut canvas = ggez::graphics::Canvas::from_frame(ctx, None);

            global_log += Self::_run(ctx, &mut canvas, bits, menu_backend, texture_storage);

            canvas.finish(ctx)?;
        }

        self.render_request.clear();

        Ok(global_log)
    }
    fn _run(
        ctx: &mut ggez::Context,
        canvas: &mut ggez::graphics::Canvas,
        bits: &mut [(render_request::RenderRequestBit, DrawParam)],
        menu_backend: &mut ggegui::EguiBackend,
        texture_storage: &mut texture::Storage,
    ) -> RenderLog {
        let mut log = RenderLog::new();

        let mut used_textures: Vec<texture::TextureId> = Vec::new();
        for (bit, dp) in bits.iter_mut() {
            match bit {
                RenderRequestBit::Texture(texture) => {
                    let ia = texture_storage
                        .get(ctx, texture)
                        .unwrap_or_else(|fallback| {
                            log.on_texture_not_found();
                            fallback
                        });

                    let img = ia.image();

                    ia.push(dp.to_ggez_scaled((img.width(), img.height())));
                    log.on_texture();
                    if !used_textures.contains(texture) {
                        used_textures.push(*texture);
                    }
                }
                RenderRequestBit::Mesh(mesh) => {
                    log.on_mesh();
                    // let dp = dp.offset((0.5, 0.5));
                    // debug!("mesh offset: {}", dp.offset); // last test says that meshes don't have any offset (even when rotated) and works fine (also it appears that thoses offsets are in px and not %)
                    canvas.draw(mesh, dp.to_ggez_unscaled());
                    log.on_draw_call();
                }
                RenderRequestBit::MeshBuilder(meshbuilder) => {
                    log.on_mesh();
                    // let dp = dp.offset((0.5, 0.5));
                    // debug!("mesh offset: {}", dp.offset); // last test says that meshes don't have any offset (even when rotated) and works fine (also it appears that thoses offsets are in px and not %)
                    canvas.draw(
                        &ggez::graphics::Mesh::from_data(ctx, meshbuilder.build()),
                        dp.to_ggez_unscaled(),
                    );
                    log.on_draw_call();
                }
                RenderRequestBit::Text(text) => {
                    use ggez::graphics::Drawable as _;
                    log.on_text();
                    canvas.draw(
                        text,
                        if let Some(dimensions) = text.dimensions(ctx).map(|d| d.size()) {
                            dp.to_ggez_scaled(dimensions)
                        } else {
                            dp.to_ggez_unscaled()
                        },
                    );
                    log.on_draw_call();
                }
                RenderRequestBit::EguiWindow => {
                    // In the ggegui implementation, the drawparam is discarded
                    canvas.draw(menu_backend, dp.to_ggez_unscaled());
                    log.on_draw_call();
                }
            }
        }

        for texture_id in used_textures {
            let ia = texture_storage
                .get(ctx, &texture_id)
                .unwrap_or_else(|fallback| fallback);
            canvas.draw(ia, graphics::DrawParam::new());
            log.on_draw_call();
            ia.clear()
        }

        log
    }
}
