use super::{LightKind, SwitchableLight, builder::Furnisher, materials::Palette};
use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology, VertexAttributeValues},
    pbr::Lightmap,
    prelude::*,
};
use latch::study::{
    ATLAS_HEIGHT, ATLAS_WIDTH, DESK, LAMP_ATLAS, LIGHTMAP_EXPOSURE, MOON_ATLAS, PADDING,
    SHELF_BACK, SURFACES, Surface, TILE_SIZE, tile_origin,
};

#[derive(Component)]
pub struct StudySurface;

#[derive(Resource)]
pub struct StudyLightmaps {
    moon: Handle<Image>,
    lamp: Handle<Image>,
}

impl StudyLightmaps {
    /// 載入兩種桌燈狀態共用的書房光照圖集。
    pub(super) fn load(assets: &AssetServer) -> Self {
        Self {
            moon: assets.load(MOON_ATLAS),
            lamp: assets.load(LAMP_ATLAS),
        }
    }

    /// 建立指定表面的圖集 UV 範圍；往內縮至邊緣像素中心避免跨圖塊取樣。
    fn map(&self, index: u32) -> Lightmap {
        let (x, y) = tile_origin(index);
        let size = Vec2::new(ATLAS_WIDTH as f32, ATLAS_HEIGHT as f32);
        let min = (Vec2::new(x as f32, y as f32) + (PADDING as f32 + 0.5)) / size;
        let max = (Vec2::new((x + TILE_SIZE) as f32, (y + TILE_SIZE) as f32)
            - (PADDING as f32 + 0.5))
            / size;
        Lightmap {
            image: self.lamp.clone(),
            uv_rect: Rect::from_corners(min, max),
            bicubic_sampling: false,
        }
    }
}

/// 使用現有牆面材質和碰撞體，替固定表面指定獨立的第二組 UV。
pub(super) fn shell(room: &mut Furnisher, palette: &Palette, lighting: &StudyLightmaps) {
    let floor = room.block(
        SURFACES[0].geometry.center.to_array(),
        SURFACES[0].geometry.size.to_array(),
        &palette.floor,
    );
    apply(room, lighting, floor, &palette.floor, 0, None);
    let wall = exposed_material(room, &palette.wall);
    for (index, surface) in SURFACES.iter().enumerate().take(8).skip(1) {
        let material = if index == 7 {
            &palette.trim
        } else {
            &palette.wall
        };
        let entity = room.solid_block(
            surface.geometry.center.to_array(),
            surface.geometry.size.to_array(),
            material,
        );
        let mesh = mapped_cube(room, *surface);
        let material = if index == 7 {
            exposed_material(room, &palette.trim)
        } else {
            wall.clone()
        };
        room.commands.entity(entity).insert((
            Mesh3d(mesh),
            MeshMaterial3d(material),
            StudySurface,
            lighting.map(index as u32),
        ));
    }
}

/// 書桌保留原本的實體碰撞大小，表面四邊切出 14 毫米倒角。
pub(super) fn desktop(room: &mut Furnisher, palette: &Palette, lighting: &StudyLightmaps) {
    let entity = room.solid_block(
        DESK.geometry.center.to_array(),
        DESK.geometry.size.to_array(),
        &palette.wood,
    );
    apply(
        room,
        lighting,
        entity,
        &palette.wood,
        8,
        Some(beveled_desk()),
    );
}

/// 替換書架背板的渲染網格，讓書本的間接陰影落到它的表面。
pub(super) fn shelf_back(
    room: &mut Furnisher,
    wood: &Handle<StandardMaterial>,
    lighting: &StudyLightmaps,
) {
    let entity = room.block(
        SHELF_BACK.geometry.center.to_array(),
        SHELF_BACK.geometry.size.to_array(),
        wood,
    );
    apply(room, lighting, entity, wood, 9, None);
}

/// 當玩家切換桌燈時更新光照圖集，關燈不會殘留暖色反彈光。
pub fn update_study_lightmaps(
    lighting: Res<StudyLightmaps>,
    switches: Query<&SwitchableLight>,
    mut surfaces: Query<&mut Lightmap, With<StudySurface>>,
) {
    let lamp_on = switches
        .iter()
        .any(|switch| switch.kind == LightKind::DeskLamp && switch.on);
    let image = if lamp_on {
        &lighting.lamp
    } else {
        &lighting.moon
    };
    for mut map in &mut surfaces {
        if map.image != *image {
            map.image = image.clone();
        }
    }
}

/// 以共用長方體 UV0 與切線為基礎，為指定平面新增獨立的 UV1。
fn mapped_cube(room: &mut Furnisher, surface: Surface) -> Handle<Mesh> {
    let mesh = room
        .meshes
        .get(&room.cube)
        .cloned()
        .unwrap_or_else(|| Cuboid::new(1.0, 1.0, 1.0).into());
    room.meshes.add(with_lightmap_uv(mesh, surface))
}

/// 將 UV1 增加至網格；UV0 繼續供 PBR 材質使用。
fn with_lightmap_uv(mut mesh: Mesh, surface: Surface) -> Mesh {
    if let Some(VertexAttributeValues::Float32x3(positions)) =
        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    {
        let uv: Vec<_> = positions
            .iter()
            .map(|&position| surface.uv(Vec3::from_array(position)))
            .collect();
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_1, uv);
    }
    mesh
}

/// 光照圖集儲存 sRGB 的正規化照度，這裡還原其場景亮度。
fn exposed_material(
    room: &mut Furnisher,
    source: &Handle<StandardMaterial>,
) -> Handle<StandardMaterial> {
    let mut material = room.materials.get(source).cloned().unwrap_or_default();
    material.lightmap_exposure = LIGHTMAP_EXPOSURE;
    room.materials.add(material)
}

/// 把光照網格、材質與同一圖集的區域一併套用至指定實體。
fn apply(
    room: &mut Furnisher,
    lighting: &StudyLightmaps,
    entity: Entity,
    material: &Handle<StandardMaterial>,
    index: usize,
    mesh: Option<Mesh>,
) {
    let mesh = match mesh {
        Some(mesh) => room.meshes.add(with_lightmap_uv(mesh, SURFACES[index])),
        None => mapped_cube(room, SURFACES[index]),
    };
    let material = exposed_material(room, material);
    room.commands.entity(entity).insert((
        Mesh3d(mesh),
        MeshMaterial3d(material),
        StudySurface,
        lighting.map(index as u32),
    ));
}

/// 沿桌板四邊生成斜面。頂點仍在單位方盒內，實際厚度由 Transform 給定。
fn beveled_desk() -> Mesh {
    let cut = 0.014 / DESK.geometry.size;
    let ring = |width: f32, y: f32, depth: f32| {
        [
            Vec3::new(width, y, -depth),
            Vec3::new(-width, y, -depth),
            Vec3::new(-width, y, depth),
            Vec3::new(width, y, depth),
        ]
    };
    let layers = [
        ring(0.5 - cut.x, 0.5, 0.5 - cut.z),
        ring(0.5, 0.5 - cut.y, 0.5),
        ring(0.5, -0.5 + cut.y, 0.5),
        ring(0.5 - cut.x, -0.5, 0.5 - cut.z),
    ];
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uv = Vec::new();
    let mut triangles = Vec::new();
    let mut quad = |points: [Vec3; 4]| {
        let normal = (points[1] - points[0])
            .cross(points[2] - points[0])
            .normalize();
        let base = positions.len() as u32;
        let planar = normal.y.abs() > 0.75;
        for point in points {
            positions.push(point.to_array());
            normals.push(normal.to_array());
            let coords = if planar {
                DESK.uv(point)
            } else {
                let horizontal = if normal.x.abs() > normal.z.abs() {
                    point.z
                } else {
                    point.x
                };
                [horizontal + 0.5, 0.5 - point.y]
            };
            uv.push(coords);
        }
        triangles.extend([base, base + 1, base + 2, base, base + 2, base + 3]);
    };
    quad(layers[0]);
    for pair in layers.windows(2) {
        for i in 0..4 {
            let next = (i + 1) % 4;
            quad([pair[0][next], pair[0][i], pair[1][i], pair[1][next]]);
        }
    }
    let mut bottom = layers[3];
    bottom.reverse();
    quad(bottom);
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uv)
    .with_inserted_indices(Indices::U32(triangles));
    let _ = mesh.generate_tangents();
    mesh
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn study_lightmaps_follow_the_desk_switch_only() {
        let mut app = App::new();
        let mut images = Assets::<Image>::default();
        let lamp = images.add(Image::default());
        let moon = images.add(Image::default());
        let other = images.add(Image::default());
        let maps = StudyLightmaps {
            moon: moon.clone(),
            lamp: lamp.clone(),
        };
        let surface = app.world_mut().spawn((StudySurface, maps.map(8))).id();
        let unrelated = app
            .world_mut()
            .spawn(Lightmap {
                image: other.clone(),
                ..default()
            })
            .id();
        let switch = app
            .world_mut()
            .spawn(SwitchableLight {
                kind: LightKind::DeskLamp,
                on: true,
                intensity: 1550.0,
            })
            .id();
        app.insert_resource(maps)
            .add_systems(Update, update_study_lightmaps);
        app.update();
        assert_eq!(app.world().get::<Lightmap>(surface).unwrap().image, lamp);
        app.world_mut()
            .get_mut::<SwitchableLight>(switch)
            .unwrap()
            .on = false;
        app.update();
        assert_eq!(app.world().get::<Lightmap>(surface).unwrap().image, moon);
        assert_eq!(app.world().get::<Lightmap>(unrelated).unwrap().image, other);
        app.world_mut()
            .get_mut::<SwitchableLight>(switch)
            .unwrap()
            .on = true;
        app.update();
        assert_eq!(app.world().get::<Lightmap>(surface).unwrap().image, lamp);
    }
}
