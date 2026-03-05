pub mod config;
pub mod file;
pub mod id;
pub mod loader;
pub mod texture;

pub use id::AssetId;

// I hate naming things "manager"
pub struct Assets {
    // Im not a big fan of making important things pub, but in this case it makes a lot of things simpler
    pub texture_storage: texture::Storage,
}

impl Assets {
    pub fn new(
        ctx: &mut ggez::Context,
        cfg: &crate::config::Config,
        thread_pool: stp::ThreadPool,
    ) -> Self {

        Self {
            texture_storage: texture::Storage::new(thread_pool.clone()),
        }
    }

    pub fn update(
        &mut self,
        ctx: &mut ggez::Context,
    ) {
        self.texture_storage.update(ctx);
    }
}
