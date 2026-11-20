use super::{
    builder::Furnisher,
    lightmaps::{BakedRoom, RoomLightmaps, exposed_material, mapped_cube, with_lightmap_uv},
    materials::Palette,
};
use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};
use latch::study::{DESK, SHELF_BACK, SURFACES};

/// 使用現有牆面材質和碰撞體，替固定表面指定獨立的第二組 UV。
pub(super) fn shell(room: &mut Furnisher, palette: &Palette, lighting: &RoomLightmaps) {
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
            BakedRoom::Study,
            lighting.map(BakedRoom::Study, index),
        ));
    }
}

/// 書桌保留原本的實體碰撞大小，表面四邊切出 14 毫米倒角。
pub(super) fn desktop(room: &mut Furnisher, palette: &Palette, lighting: &RoomLightmaps) {
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
    lighting: &RoomLightmaps,
) {
    let entity = room.block(
        SHELF_BACK.geometry.center.to_array(),
        SHELF_BACK.geometry.size.to_array(),
        wood,
    );
    apply(room, lighting, entity, wood, 9, None);
}

/// 把光照網格、材質與同一圖集的區域一併套用至指定實體。
fn apply(
    room: &mut Furnisher,
    lighting: &RoomLightmaps,
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
        BakedRoom::Study,
        lighting.map(BakedRoom::Study, index),
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
