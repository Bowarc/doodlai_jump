use std::collections::HashMap;
use crate::assets::file::Path;

type ResolverMap = HashMap<crate::assets::AssetId, Path>;

pub struct Resolver {
    base_path: Path,
    map: Option<ResolverMap>,
}

impl Resolver {
    pub fn new<AssetCategory: crate::assets::id::AssetIdTrait>(base_path: &str) -> Self {
        let map = fetch::<AssetCategory>(base_path);
        if map.is_none() {
            warn!("Could not load resolver at /{base_path}");
        }

        Self {
            base_path: Path::new_owned(base_path.to_string()),
            map,
        }
    }

    pub fn get<'a>(
        &'a self,
        id: impl Into<crate::assets::AssetId>,
    ) -> Option<crate::assets::file::Path> {
        let id = id.into();

        let get = |map_opt: Option<&'a ResolverMap>| -> Option<&'a Path> {
            map_opt.and_then(|m: &'a ResolverMap| m.get(&id))
        };

        if let Some(p) = get(self.map.as_ref()) {
            use crate::assets::file::Path;
            return Some(Path::new_owned(format!("{}/{p}", self.base_path)));
        }

        None
    }
}

fn fetch<AssetCategory: crate::assets::id::AssetIdTrait>(base_path: &str) -> Option<ResolverMap> {
    use crate::assets::AssetId;

    let bytes = crate::assets::file::try_bytes(crate::assets::file::Path::new_owned(format!(
        "{base_path}/resolver.ron"
    )))
    .ok()?;

    Some(
        ron::de::from_bytes::<HashMap<AssetCategory, String>>(&bytes)
            .ok()?
            .into_iter()
            .map(|(k, v)| (k.into(), Path::new_owned(v)))
            .collect::<HashMap<AssetId, Path>>(),
    )
}
