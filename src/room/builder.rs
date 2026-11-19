use crate::{
    interaction::Target,
    physics::{interactable_layers, world_layers},
};
use avian3d::prelude::*;
use bevy::prelude::*;

pub(super) struct Furnisher<'a, 'w, 's> {
    pub(super) commands: &'a mut Commands<'w, 's>,
    pub(super) meshes: &'a mut Assets<Mesh>,
    pub(super) materials: &'a mut Assets<StandardMaterial>,
    pub(super) cube: Handle<Mesh>,
}

impl Furnisher<'_, '_, '_> {
    /// 建立可重用 PBR 材質；sRGB 色值為 0 至 1，粗糙度與金屬度由用途決定。
    pub(super) fn material(&mut self, rgb: [f32; 3], metallic: f32) -> Handle<StandardMaterial> {
        self.materials.add(StandardMaterial {
            base_color: Color::srgb(rgb[0], rgb[1], rgb[2]),
            metallic,
            perceptual_roughness: if metallic > 0.5 { 0.28 } else { 0.8 },
            ..default()
        })
    }

    /// 以公尺制世界座標建立長方體，縮放共用單位立方體以減少網格資源。
    pub(super) fn block(
        &mut self,
        at: [f32; 3],
        size: [f32; 3],
        material: &Handle<StandardMaterial>,
    ) -> Entity {
        self.commands
            .spawn((
                Mesh3d(self.cube.clone()),
                MeshMaterial3d(material.clone()),
                Transform::from_translation(Vec3::from(at)).with_scale(Vec3::from(size)),
            ))
            .id()
    }

    /// 建立附帶靜態 Avian 碰撞體的結構長方體，座標與尺寸皆為公尺。
    pub(super) fn solid_block(
        &mut self,
        at: [f32; 3],
        size: [f32; 3],
        material: &Handle<StandardMaterial>,
    ) -> Entity {
        self.commands
            .spawn((
                Mesh3d(self.cube.clone()),
                MeshMaterial3d(material.clone()),
                Transform::from_translation(Vec3::from(at)).with_scale(Vec3::from(size)),
                RigidBody::Static,
                Collider::cuboid(1.0, 1.0, 1.0),
                world_layers(),
            ))
            .id()
    }

    /// 建立垂直圓柱，半徑與高度均以公尺表示。
    pub(super) fn cylinder(
        &mut self,
        at: [f32; 3],
        radius: f32,
        height: f32,
        material: &Handle<StandardMaterial>,
    ) -> Entity {
        let mesh = self.meshes.add(Cylinder::new(radius, height));
        self.commands
            .spawn((
                Mesh3d(mesh),
                MeshMaterial3d(material.clone()),
                Transform::from_translation(Vec3::from(at)),
            ))
            .id()
    }

    /// 建立不透明橢球裝飾，以公尺制三軸縮放球體。
    pub(super) fn ellipsoid(
        &mut self,
        at: [f32; 3],
        size: [f32; 3],
        material: &Handle<StandardMaterial>,
    ) -> Entity {
        let mesh = self.meshes.add(Sphere::new(1.0).mesh().uv(12, 8));
        self.commands
            .spawn((
                Mesh3d(mesh),
                MeshMaterial3d(material.clone()),
                Transform::from_translation(Vec3::from(at)).with_scale(Vec3::from(size)),
            ))
            .id()
    }

    /// 加入無可視網格的靜態長方體碰撞體，尺寸為全寬高深（公尺）。
    pub(super) fn static_box(&mut self, at: [f32; 3], size: [f32; 3]) -> Entity {
        self.commands
            .spawn((
                Transform::from_translation(Vec3::from(at)),
                RigidBody::Static,
                Collider::cuboid(size[0], size[1], size[2]),
                world_layers(),
            ))
            .id()
    }

    /// 建立純射線互動感應盒，讓準星可直接命中特定物件而不產生實體阻擋。
    pub(super) fn sensor(&mut self, at: [f32; 3], size: [f32; 3], target: Target) -> Entity {
        self.commands
            .spawn((
                Transform::from_translation(Vec3::from(at)),
                Collider::cuboid(size[0], size[1], size[2]),
                interactable_layers(),
                target,
            ))
            .id()
    }

    /// 載入 glTF 場景節點；位置與縮放採公尺制，繞 Y 軸旋轉角度以弧度表示。
    pub(super) fn model(
        &mut self,
        assets: &AssetServer,
        path: &'static str,
        at: [f32; 3],
        rotation_y: f32,
        scale: f32,
    ) -> Entity {
        self.commands
            .spawn((
                WorldAssetRoot(assets.load(GltfAssetLabel::Scene(0).from_asset(path))),
                Transform::from_translation(Vec3::from(at))
                    .with_rotation(Quat::from_rotation_y(rotation_y))
                    .with_scale(Vec3::splat(scale)),
            ))
            .id()
    }

    /// 建立暖色或月光點光源；光通量為流明，只有主要燈源產生陰影。
    pub(super) fn light(&mut self, at: [f32; 3], rgb: [f32; 3], lumens: f32, shadows: bool) {
        self.commands.spawn((
            PointLight {
                color: Color::srgb(rgb[0], rgb[1], rgb[2]),
                intensity: lumens,
                range: 16.0,
                radius: 0.25,
                shadow_maps_enabled: shadows,
                ..default()
            },
            Transform::from_translation(Vec3::from(at)),
        ));
    }
}
