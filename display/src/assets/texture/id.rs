use serde::{Deserialize, Serialize};

use crate::assets::id::AssetIdTrait;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub enum TextureId {
    DoodleLeft,
    DoodleRight,
    BluePlatform,
    GreenPlatform,
    RedPlatform,
    WhitePlatform,
    CrackedPlatform0,
    CrackedPlatform1,
    CrackedPlatform2,
    CrackedPlatform3,
    Mob0,
    Mob1,
    Mob2Left,
    Mob2Right,
    Dynamic(DynamicTextureParams), // this is used to be able to create textures, for particles etc..
    Missing,
}

impl AssetIdTrait for TextureId {}

impl Default for TextureId {
    fn default() -> Self {
        Self::Missing
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct DynamicTextureParams {
    pub size: u16,
    pub color: crate::render::Color,
}

impl Default for DynamicTextureParams {
    fn default() -> Self {
        Self {
            size: 1,
            color: [255, 255, 255, 255].into(),
        }
    }
}
