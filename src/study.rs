use bevy::math::{Vec2, Vec3, vec3};

pub const TILE_SIZE: u32 = 192;
pub const ATLAS_COLUMNS: u32 = 4;
pub const ATLAS_WIDTH: u32 = TILE_SIZE * ATLAS_COLUMNS;
pub const ATLAS_HEIGHT: u32 = TILE_SIZE * 3;
pub const PADDING: u32 = 2;
pub const LIGHTMAP_EXPOSURE: f32 = 120.0;
pub const MOON_ATLAS: &str = "lighting/study-moon.png";
pub const LAMP_ATLAS: &str = "lighting/study-lamp.png";

#[derive(Clone, Copy)]
pub struct BoxGeometry {
    pub center: Vec3,
    pub size: Vec3,
}

impl BoxGeometry {
    /// 建立以公尺為單位的世界座標長方體。
    pub const fn new(center: [f32; 3], size: [f32; 3]) -> Self {
        Self {
            center: vec3(center[0], center[1], center[2]),
            size: vec3(size[0], size[1], size[2]),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Surface {
    pub geometry: BoxGeometry,
    pub normal: Vec3,
}

impl Surface {
    /// 指定需烘焙的長方體內側面；normal 是世界座標的主軸單位向量。
    const fn new(center: [f32; 3], size: [f32; 3], normal: Vec3) -> Self {
        Self {
            geometry: BoxGeometry::new(center, size),
            normal,
        }
    }

    /// 從光照貼圖座標還原到實際表面；(0, 0) 在圖塊左上。
    pub fn point(self, uv: Vec2) -> Vec3 {
        let plane = self.normal * 0.5;
        let horizontal = if self.normal.x != 0.0 {
            Vec3::Z
        } else {
            Vec3::X
        };
        let vertical = if self.normal.y != 0.0 {
            Vec3::Z
        } else {
            Vec3::NEG_Y
        };
        self.geometry.center
            + self.geometry.size * (plane + horizontal * (uv.x - 0.5) + vertical * (uv.y - 0.5))
    }

    /// 將單位立方體頂點投影成該面在圖塊內的 UV 座標。
    pub fn uv(self, local: Vec3) -> [f32; 2] {
        let u = if self.normal.x != 0.0 {
            local.z
        } else {
            local.x
        };
        let v = if self.normal.y != 0.0 {
            local.z
        } else {
            -local.y
        };
        [u + 0.5, v + 0.5]
    }
}

pub const FLOOR: Surface = Surface::new([0.0, 0.0, 0.0], [9.2, 0.025, 9.2], Vec3::Y);
pub const WINDOWS: Surface = Surface::new([0.0, 1.75, -4.6], [9.2, 3.5, 0.2], Vec3::Z);
pub const WEST: Surface = Surface::new([-4.6, 1.75, 0.0], [0.2, 3.5, 9.2], Vec3::X);
pub const EAST: Surface = Surface::new([4.6, 1.75, 0.0], [0.2, 3.5, 9.2], Vec3::NEG_X);
pub const SOUTH_WEST: Surface = Surface::new([-1.8, 1.75, 4.6], [5.6, 3.5, 0.2], Vec3::NEG_Z);
pub const SOUTH_EAST: Surface = Surface::new([3.7, 1.75, 4.6], [1.8, 3.5, 0.2], Vec3::NEG_Z);
pub const LINTEL: Surface = Surface::new([1.9, 3.0, 4.6], [1.8, 1.0, 0.2], Vec3::NEG_Z);
pub const CEILING: Surface = Surface::new([0.0, 3.55, 0.0], [9.3, 0.1, 9.3], Vec3::NEG_Y);
pub const DESK: Surface = Surface::new([-0.8, 0.94, -2.85], [2.65, 0.13, 1.12], Vec3::Y);
pub const SHELF_BACK: Surface = Surface::new([-3.92, 1.35, 1.42], [0.08, 2.33, 2.70], Vec3::X);

pub const SURFACES: [Surface; 10] = [
    FLOOR, WINDOWS, WEST, EAST, SOUTH_WEST, SOUTH_EAST, LINTEL, CEILING, DESK, SHELF_BACK,
];
pub const DESK_LEGS: [BoxGeometry; 4] = [
    BoxGeometry::new([-1.93, 0.44, -3.27], [0.12, 0.87, 0.12]),
    BoxGeometry::new([-1.93, 0.44, -2.43], [0.12, 0.87, 0.12]),
    BoxGeometry::new([0.33, 0.44, -3.27], [0.12, 0.87, 0.12]),
    BoxGeometry::new([0.33, 0.44, -2.43], [0.12, 0.87, 0.12]),
];
pub const SHELF_LEVELS: [f32; 5] = [0.22, 0.83, 1.44, 2.05, 2.52];
pub const MOON_LIGHT: Vec3 = vec3(-0.8, 2.65, -3.9);
pub const DESK_LIGHT: Vec3 = vec3(0.18, 1.42, -2.92);
pub const CEILING_LIGHT: Vec3 = vec3(0.0, 2.55, 0.1);

/// 回傳指定表面在貼圖中的圖塊左上角像素座標。
pub const fn tile_origin(index: u32) -> (u32, u32) {
    (
        index % ATLAS_COLUMNS * TILE_SIZE,
        index / ATLAS_COLUMNS * TILE_SIZE,
    )
}

/// 回傳書架各層跨越前後深度的水平木板。
pub const fn shelf_level(y: f32) -> BoxGeometry {
    BoxGeometry::new([-3.68, y, 1.42], [0.85, 0.07, 2.87])
}

/// 回傳相對於攝影機正面可見的兩側書架側板。
pub const fn shelf_side(z: f32) -> BoxGeometry {
    BoxGeometry::new([-3.68, 1.35, z], [0.85, 2.40, 0.075])
}

/// 回傳書架各層書本位置，固定偏移讓書背高低與深度略有變化。
pub fn shelf_book(tier: usize, index: usize) -> BoxGeometry {
    let height = 0.31 + ((index * 3 + tier) % 5) as f32 * 0.04;
    let inset = ((tier + index * 3) % 5) as f32 * 0.014;
    BoxGeometry::new(
        [
            -3.61 - inset,
            0.27 + tier as f32 * 0.61 + height * 0.5,
            0.22 + index as f32 * 0.2,
        ],
        [0.38, height, 0.14],
    )
}
