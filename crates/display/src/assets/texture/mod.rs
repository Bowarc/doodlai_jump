use super::loader::Resolver;
use ggez::graphics::{Image, InstanceArray};
pub use id::TextureId;
use std::collections::HashMap;
use stp::{ArcFuture, FutureState, ThreadPool};

pub mod id;
pub mod image_type;

pub struct Storage {
    storage: HashMap<TextureId, ggez::graphics::InstanceArray>,

    resolver: Resolver,
    thread_pool: ThreadPool,
    futures: Vec<(TextureId, ArcFuture<std::borrow::Cow<'static, [u8]>>)>,
    missing_textures: Vec<TextureId>,
}

impl Storage {
    pub fn new(thread_pool: ThreadPool) -> Self {
        let resolver = super::loader::Resolver::new::<TextureId>("textures");

        let default_path = resolver.get(TextureId::default()).unwrap();
        let f = thread_pool.run(move || super::file::bytes(default_path));

        Self {
            storage: HashMap::new(),
            resolver,
            thread_pool,
            futures: vec![(TextureId::default(), f)],
            missing_textures: Vec::new(),
        }
    }

    fn request(&mut self, id: impl Into<TextureId>) -> ArcFuture<std::borrow::Cow<'static, [u8]>> {
        use super::file;
        let id = id.into();

        if let Some((_w, f)) = self.futures.iter().find(|(w, _f)| w == &id) {
            return f.clone();
        }

        let Some(path) = self.resolver.get(id) else {
            error!("Could not get the path for {id:?} in the resolver");
            panic!();
        };

        let future = self.thread_pool.run(move || file::bytes(path));

        self.futures.push((id, future.clone()));

        future
    }

    pub fn update(&mut self, ctx: &ggez::Context) {
        let mut index = 0;

        while index < self.futures.len() {
            let (_id, arcfuture) = self.futures.get(index).unwrap();

            if !arcfuture.is_done() {
                index += 1;
                continue;
            }

            let (id, arcfuture) = self.futures.remove(index);

            match arcfuture.state() {
                FutureState::Flying | FutureState::Started => unreachable!(),
                FutureState::Panicked => {
                    error!("Future for {id:?} panicked");
                    assert!(!self.missing_textures.contains(&id), "Tf are you doing ?");
                    self.missing_textures.push(id);
                    continue;
                }
                FutureState::Done => {
                    let bytes = arcfuture.output();

                    let image = match Image::from_bytes(&ctx.gfx, &bytes) {
                        Ok(image) => image,
                        Err(e) => {
                            error!(
                                "Texture storage failed to unpack data for id: {id:?} due to: {e}"
                            );
                            continue;
                        }
                    };
                    let ia = InstanceArray::new(&ctx.gfx, image);

                    if let Some(_old) = self.storage.insert(id, ia) {
                        warn!("Texture storage has replaced {id:?}");
                    }
                }
            }
        }
    }

    pub fn get(
        &mut self,
        ctx: &mut ggez::Context,
        id: &TextureId,
    ) -> Result<&mut ggez::graphics::InstanceArray, &mut ggez::graphics::InstanceArray> {
        /*
            The current borrow checker is not good enough to allow us to use

            if let Some(ia) = self.storage.get_mut(id) {
                return Ok(ia);
            }

            if let TextureId::Dynamic(params) = id {
                self.create_dynamic(ctx, params).unwrap();
                return Ok(self.storage.get_mut(id).unwrap());
            }

            Etc..

            see: https://github.com/danielhenrymantilla/polonius-the-crab.rs/

            The reason i don't use Polonius the crab is that it requires the binding to be mut,
            which is not the case of the storage (see https://github.com/danielhenrymantilla/polonius-the-crab.rs/blob/master/src/compile_fail_tests.md)

        */
        // Is it in the storage ?
        if self.storage.contains_key(id) {
            return Ok(self.storage.get_mut(id).unwrap());
        }

        // Is it a dynamic texture request ?
        // If so, we can create it !
        if let TextureId::Dynamic(params) = id {
            self.create_dynamic(ctx, params).unwrap();
            return Ok(self.storage.get_mut(id).unwrap());
        }

        // At this point, we initiate failsafe
        if !self.missing_textures.contains(id) {
            let _f = self.request(*id);
        }

        // Let's fall back to the default one
        let default_id = TextureId::default();
        if self.storage.contains_key(&default_id) {
            return Err(self.storage.get_mut(&default_id).unwrap());
        }

        // Even the default one is not loaded :c
        if self.missing_textures.contains(&TextureId::default()) {
            let _df = self.request(TextureId::default());
        }

        // Let's try to get the default dynamic (probably just a white 1x1)
        let default_dynamic_id = TextureId::Dynamic(id::DynamicTextureParams::default());
        if self.storage.contains_key(&default_dynamic_id) {
            return Err(self.storage.get_mut(&default_dynamic_id).unwrap());
        }

        // Even the default dynamic is not loaded, let's create it
        // If this fails, im fine with the panic
        let default_params = id::DynamicTextureParams::default();
        self.create_dynamic(ctx, &default_params).unwrap();

        // This unwrap cannot fail, we just created it
        Err(self
            .storage
            .get_mut(&TextureId::Dynamic(default_params))
            .unwrap())
    }

    pub fn create_dynamic(
        &mut self,
        ctx: &mut ggez::Context,
        params: &id::DynamicTextureParams,
    ) -> ggez::GameResult<TextureId> {
        let id = TextureId::Dynamic(*params);

        let gfx = &mut ctx.gfx;

        let ia = InstanceArray::new(
            gfx,
            Image::from_color(
                gfx,
                params.size.into(),
                params.size.into(),
                Some(params.color.into()),
            ),
        );

        assert!(self.storage.insert(id, ia).is_none());
        Ok(id)
    }
}
