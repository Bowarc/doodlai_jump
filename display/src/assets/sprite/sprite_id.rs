#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, serde::Deserialize)]
pub enum SpriteId {
    #[default]
    MissingNo,
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
}
