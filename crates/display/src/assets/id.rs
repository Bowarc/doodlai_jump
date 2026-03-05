use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum AssetId {
    Texture(super::texture::TextureId),
}

impl From<crate::assets::texture::TextureId> for AssetId {
    fn from(id: crate::assets::texture::TextureId) -> Self {
        Self::Texture(id)
    }
}

pub trait AssetIdTrait:
    Into<AssetId>
    + Default
    + std::fmt::Debug
    + Clone
    + Copy
    + PartialEq
    + Eq
    + serde::ser::Serialize
    + serde::de::DeserializeOwned
    + std::hash::Hash
{
}
