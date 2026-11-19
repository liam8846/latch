use bevy::{
    image::{ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor},
    math::Affine2,
    prelude::*,
};

pub(super) struct Palette {
    pub(super) floor: Handle<StandardMaterial>,
    pub(super) wall: Handle<StandardMaterial>,
    pub(super) trim: Handle<StandardMaterial>,
    pub(super) wood: Handle<StandardMaterial>,
    pub(super) walnut: Handle<StandardMaterial>,
    pub(super) brass: Handle<StandardMaterial>,
    pub(super) cream: Handle<StandardMaterial>,
    pub(super) ink: Handle<StandardMaterial>,
    pub(super) rust: Handle<StandardMaterial>,
    pub(super) steel: Handle<StandardMaterial>,
    pub(super) moon: Handle<StandardMaterial>,
}

/// 載入重複平鋪貼圖；sRGB 用於色彩圖，法線與 ARM 貼圖關閉 sRGB 以維持線性空間。
fn tiled(assets: &AssetServer, path: &'static str, srgb: bool) -> Handle<Image> {
    assets
        .load_builder()
        .with_settings(move |settings: &mut ImageLoaderSettings| {
            settings.is_srgb = srgb;
            settings.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                address_mode_u: ImageAddressMode::Repeat,
                address_mode_v: ImageAddressMode::Repeat,
                ..default()
            });
        })
        .load(path)
}

/// 以 Poly Haven CC0 1K PBR 貼圖建立整間書房的統一材質表，不再使用純色或低解析雜訊。
pub(super) fn build(assets: &AssetServer, materials: &mut Assets<StandardMaterial>) -> Palette {
    let floor_arm = tiled(assets, "textures/old_wooden_floor_01/arm.jpg", false);
    let floor = materials.add(StandardMaterial {
        base_color: Color::srgb(0.88, 0.80, 0.72),
        base_color_texture: Some(tiled(
            assets,
            "textures/old_wooden_floor_01/diffuse.jpg",
            true,
        )),
        normal_map_texture: Some(tiled(
            assets,
            "textures/old_wooden_floor_01/nor_gl.jpg",
            false,
        )),
        metallic_roughness_texture: Some(floor_arm.clone()),
        occlusion_texture: Some(floor_arm),
        perceptual_roughness: 0.72,
        uv_transform: Affine2::from_scale(Vec2::splat(4.0)),
        ..default()
    });

    let wall_arm = tiled(assets, "textures/painted_plaster_wall/arm.jpg", false);
    let wall_diff = tiled(assets, "textures/painted_plaster_wall/diffuse.jpg", true);
    let wall_nor = tiled(assets, "textures/painted_plaster_wall/nor_gl.jpg", false);
    let wall = materials.add(StandardMaterial {
        base_color: Color::srgb(0.16, 0.34, 0.34),
        base_color_texture: Some(wall_diff.clone()),
        normal_map_texture: Some(wall_nor.clone()),
        metallic_roughness_texture: Some(wall_arm.clone()),
        occlusion_texture: Some(wall_arm.clone()),
        perceptual_roughness: 0.92,
        uv_transform: Affine2::from_scale(Vec2::new(3.0, 2.0)),
        ..default()
    });

    let cream = materials.add(StandardMaterial {
        base_color: Color::srgb(0.90, 0.84, 0.68),
        base_color_texture: Some(wall_diff),
        normal_map_texture: Some(wall_nor),
        metallic_roughness_texture: Some(wall_arm.clone()),
        occlusion_texture: Some(wall_arm),
        perceptual_roughness: 0.88,
        uv_transform: Affine2::from_scale(Vec2::splat(1.2)),
        ..default()
    });

    let fabric_arm = tiled(assets, "textures/fabric_pattern_05/arm.jpg", false);
    let fabric_nor = tiled(assets, "textures/fabric_pattern_05/nor_gl.jpg", false);
    let trim = materials.add(StandardMaterial {
        base_color: Color::srgb(0.06, 0.14, 0.14),
        normal_map_texture: Some(fabric_nor.clone()),
        metallic_roughness_texture: Some(fabric_arm.clone()),
        occlusion_texture: Some(fabric_arm.clone()),
        perceptual_roughness: 0.94,
        uv_transform: Affine2::from_scale(Vec2::splat(2.5)),
        ..default()
    });
    let rust = materials.add(StandardMaterial {
        base_color: Color::srgb(0.42, 0.16, 0.10),
        normal_map_texture: Some(fabric_nor),
        metallic_roughness_texture: Some(fabric_arm.clone()),
        occlusion_texture: Some(fabric_arm),
        perceptual_roughness: 0.92,
        uv_transform: Affine2::from_scale(Vec2::splat(3.5)),
        ..default()
    });

    let wood_arm = tiled(assets, "textures/american_walnut_veneer/arm.jpg", false);
    let wood = materials.add(StandardMaterial {
        base_color: Color::srgb(0.92, 0.80, 0.70),
        base_color_texture: Some(tiled(
            assets,
            "textures/american_walnut_veneer/diffuse.jpg",
            true,
        )),
        normal_map_texture: Some(tiled(
            assets,
            "textures/american_walnut_veneer/nor_gl.jpg",
            false,
        )),
        metallic_roughness_texture: Some(wood_arm.clone()),
        occlusion_texture: Some(wood_arm),
        perceptual_roughness: 0.58,
        uv_transform: Affine2::from_scale(Vec2::splat(1.6)),
        ..default()
    });

    let walnut_arm = tiled(assets, "textures/dark_wood/arm.jpg", false);
    let walnut_diff = tiled(assets, "textures/dark_wood/diffuse.jpg", true);
    let walnut_nor = tiled(assets, "textures/dark_wood/nor_gl.jpg", false);
    let walnut = materials.add(StandardMaterial {
        base_color: Color::srgb(0.84, 0.76, 0.70),
        base_color_texture: Some(walnut_diff.clone()),
        normal_map_texture: Some(walnut_nor.clone()),
        metallic_roughness_texture: Some(walnut_arm.clone()),
        occlusion_texture: Some(walnut_arm.clone()),
        perceptual_roughness: 0.64,
        uv_transform: Affine2::from_scale(Vec2::splat(1.5)),
        ..default()
    });
    let ink = materials.add(StandardMaterial {
        base_color: Color::srgb(0.05, 0.08, 0.09),
        base_color_texture: Some(walnut_diff),
        normal_map_texture: Some(walnut_nor),
        metallic_roughness_texture: Some(walnut_arm.clone()),
        occlusion_texture: Some(walnut_arm),
        perceptual_roughness: 0.75,
        ..default()
    });

    let metal_arm = tiled(assets, "textures/metal_plate_02/arm.jpg", false);
    let metal_diff = tiled(assets, "textures/metal_plate_02/diffuse.jpg", true);
    let metal_nor = tiled(assets, "textures/metal_plate_02/nor_gl.jpg", false);
    let brass = materials.add(StandardMaterial {
        base_color: Color::srgb(0.92, 0.68, 0.30),
        base_color_texture: Some(metal_diff.clone()),
        normal_map_texture: Some(metal_nor.clone()),
        metallic_roughness_texture: Some(metal_arm.clone()),
        occlusion_texture: Some(metal_arm.clone()),
        metallic: 0.88,
        perceptual_roughness: 0.38,
        uv_transform: Affine2::from_scale(Vec2::splat(1.4)),
        ..default()
    });
    let steel = materials.add(StandardMaterial {
        base_color: Color::srgb(0.24, 0.32, 0.34),
        base_color_texture: Some(metal_diff),
        normal_map_texture: Some(metal_nor),
        metallic_roughness_texture: Some(metal_arm.clone()),
        occlusion_texture: Some(metal_arm),
        metallic: 0.78,
        perceptual_roughness: 0.48,
        uv_transform: Affine2::from_scale(Vec2::splat(1.4)),
        ..default()
    });

    let moon = materials.add(StandardMaterial {
        base_color: Color::srgb(0.025, 0.055, 0.10),
        emissive: LinearRgba::rgb(0.018, 0.045, 0.10),
        ..default()
    });

    Palette {
        floor,
        wall,
        trim,
        wood,
        walnut,
        brass,
        cream,
        ink,
        rust,
        steel,
        moon,
    }
}
