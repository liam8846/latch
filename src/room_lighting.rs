use crate::study::{ATLAS_COLUMNS, Surface, TILE_SIZE};
use bevy::math::{Vec3, vec3};

pub const FOYER_NIGHT: &str = "lighting/foyer-night.png";
pub const FOYER_LIT: &str = "lighting/foyer-lit.png";
pub const SIGNAL_ATLAS: &str = "lighting/signal-room.png";

pub const FOYER_PASSAGE_LIGHT: Vec3 = vec3(1.9, 2.35, 5.8);
pub const FOYER_SCONCE: Vec3 = vec3(3.05, 1.98, 12.2);
pub const FOYER_LANTERN: Vec3 = vec3(0.5, 2.55, 10.3);
pub const PORCH_LIGHT: Vec3 = vec3(0.5, 2.2, 15.3);
pub const SIGNAL_ENTRY_LIGHT: Vec3 = vec3(0.5, 2.95, 18.0);
pub const SIGNAL_PANEL_LIGHT: Vec3 = vec3(1.55, 2.95, 21.55);
pub const SHORE_LIGHT: Vec3 = vec3(GATE_X, 2.35, 25.4);
pub const GATE_X: f32 = -1.85;
pub const PANEL_Z: f32 = 23.40;

#[derive(Clone, Copy)]
pub enum Finish {
    Floor,
    Wood,
    Wall,
    Ceiling,
    Night,
}

#[derive(Clone, Copy)]
pub struct RoomSurface {
    pub surface: Surface,
    pub finish: Finish,
    pub solid: bool,
}

impl RoomSurface {
    /// 定義固定表面的幾何、朝室內的法線與材質，供遊戲和烘焙工具共用。
    const fn new(
        center: [f32; 3],
        size: [f32; 3],
        normal: Vec3,
        finish: Finish,
        solid: bool,
    ) -> Self {
        Self {
            surface: Surface::new(center, size, normal),
            finish,
            solid,
        }
    }

    /// 定義本身含有碰撞體的普通室內牆面。
    const fn wall(center: [f32; 3], size: [f32; 3], normal: Vec3) -> Self {
        Self::new(center, size, normal, Finish::Wall, true)
    }
}

pub const STUDY_SURFACE_COUNT: usize = crate::study::SURFACES.len();

pub const FOYER_PASSAGE: [RoomSurface; 4] = [
    RoomSurface::new(
        [1.9, 0.0, 5.8],
        [2.2, 0.025, 2.5],
        Vec3::Y,
        Finish::Floor,
        false,
    ),
    RoomSurface::new(
        [1.9, 3.55, 5.8],
        [2.4, 0.1, 2.5],
        Vec3::NEG_Y,
        Finish::Ceiling,
        true,
    ),
    RoomSurface::wall([0.85, 1.75, 5.8], [0.2, 3.5, 2.5], Vec3::X),
    RoomSurface::wall([2.95, 1.75, 5.8], [0.2, 3.5, 2.5], Vec3::NEG_X),
];

pub const FOYER_ROOM: [RoomSurface; 10] = [
    RoomSurface::new(
        [0.5, 0.0, 10.3],
        [6.4, 0.025, 6.8],
        Vec3::Y,
        Finish::Floor,
        false,
    ),
    RoomSurface::new(
        [0.5, 3.55, 10.3],
        [6.6, 0.1, 6.8],
        Vec3::NEG_Y,
        Finish::Ceiling,
        true,
    ),
    RoomSurface::wall([-0.85, 1.75, 7.0], [3.5, 3.5, 0.2], Vec3::Z),
    RoomSurface::wall([3.25, 1.75, 7.0], [0.7, 3.5, 0.2], Vec3::Z),
    RoomSurface::wall([1.9, 3.0, 7.0], [2.0, 1.0, 0.2], Vec3::Z),
    RoomSurface::wall([-2.6, 1.75, 10.3], [0.2, 3.5, 6.8], Vec3::X),
    RoomSurface::wall([3.6, 1.75, 10.3], [0.2, 3.5, 6.8], Vec3::NEG_X),
    RoomSurface::wall([-1.5, 1.75, 13.6], [2.2, 3.5, 0.2], Vec3::NEG_Z),
    RoomSurface::wall([2.5, 1.75, 13.6], [2.2, 3.5, 0.2], Vec3::NEG_Z),
    RoomSurface::wall([0.5, 3.0, 13.6], [1.8, 1.0, 0.2], Vec3::NEG_Z),
];

pub const FOYER_PORCH: [RoomSurface; 4] = [
    RoomSurface::new(
        [0.5, 0.0, 15.1],
        [2.4, 0.06, 3.2],
        Vec3::Y,
        Finish::Wood,
        false,
    ),
    RoomSurface::new(
        [0.5, 3.4, 15.2],
        [2.4, 0.12, 3.2],
        Vec3::NEG_Y,
        Finish::Ceiling,
        true,
    ),
    RoomSurface::wall([-0.6, 1.65, 15.2], [0.2, 3.4, 3.2], Vec3::X),
    RoomSurface::wall([1.6, 1.65, 15.2], [0.2, 3.4, 3.2], Vec3::NEG_X),
];

pub const FOYER_SURFACE_COUNT: usize = FOYER_PASSAGE.len() + FOYER_ROOM.len() + FOYER_PORCH.len();

pub const SIGNAL_ROOM: [RoomSurface; 10] = [
    RoomSurface::new(
        [0.5, 0.011, 20.1],
        [7.6, 0.024, 7.1],
        Vec3::Y,
        Finish::Wood,
        false,
    ),
    RoomSurface::new(
        [0.5, 3.76, 20.1],
        [7.8, 0.16, 7.1],
        Vec3::NEG_Y,
        Finish::Ceiling,
        true,
    ),
    RoomSurface::wall([-3.3, 1.87, 20.1], [0.2, 3.74, 7.0], Vec3::X),
    RoomSurface::wall([4.3, 1.87, 20.1], [0.2, 3.74, 7.0], Vec3::NEG_X),
    RoomSurface::wall([-1.95, 1.87, 16.7], [2.7, 3.74, 0.2], Vec3::Z),
    RoomSurface::wall([2.95, 1.87, 16.7], [2.7, 3.74, 0.2], Vec3::Z),
    RoomSurface::wall([0.5, 3.15, 16.7], [2.2, 1.18, 0.2], Vec3::Z),
    RoomSurface::wall([-3.02, 1.87, 23.6], [0.56, 3.74, 0.2], Vec3::NEG_Z),
    RoomSurface::wall([1.68, 1.87, 23.6], [5.30, 3.74, 0.2], Vec3::NEG_Z),
    RoomSurface::wall([GATE_X, 3.10, 23.6], [1.78, 1.28, 0.2], Vec3::NEG_Z),
];

pub const SHORE: [RoomSurface; 4] = [
    RoomSurface::new(
        [GATE_X, -0.10, 24.86],
        [2.10, 0.2, 2.96],
        Vec3::Y,
        Finish::Wood,
        true,
    ),
    RoomSurface::wall([GATE_X - 1.0, 1.28, 24.9], [0.17, 2.56, 2.72], Vec3::X),
    RoomSurface::wall([GATE_X + 1.0, 1.28, 24.9], [0.17, 2.56, 2.72], Vec3::NEG_X),
    RoomSurface::new(
        [GATE_X, 1.75, 26.27],
        [2.2, 3.5, 0.16],
        Vec3::NEG_Z,
        Finish::Night,
        true,
    ),
];

pub const SIGNAL_SURFACE_COUNT: usize = SIGNAL_ROOM.len() + SHORE.len();

/// 回傳容納各室表面所需的圖集像素高度。
pub const fn atlas_height(surface_count: usize) -> u32 {
    (surface_count as u32).div_ceil(ATLAS_COLUMNS) * TILE_SIZE
}

/// 依圖塊順序提供玄關和前後門廊的固定表面。
pub fn foyer_surfaces() -> impl Iterator<Item = RoomSurface> {
    FOYER_PASSAGE
        .into_iter()
        .chain(FOYER_ROOM)
        .chain(FOYER_PORCH)
}

/// 依圖塊順序提供信號室和岸邊出口的固定表面。
pub fn signal_surfaces() -> impl Iterator<Item = RoomSurface> {
    SIGNAL_ROOM.into_iter().chain(SHORE)
}
