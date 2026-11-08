use super::builder::Furnisher;
use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

#[derive(Clone, Copy)]
pub(super) enum Surface {
    Wood,
    Cloth,
    Paper,
    Plaster,
}

/// 建立 128 平方像素的原創程序表面，顏色細節與粗糙度共用單張小貼圖。
/// 紋理座標為單一物件 UV，不涉及世界公尺；沒有每幀生成或載入。
pub(super) fn apply(
    room: &mut Furnisher,
    images: &mut Assets<Image>,
    surfaces: &[(&Handle<StandardMaterial>, Surface)],
) {
    for &(material, surface) in surfaces {
        let mut pixels = Vec::with_capacity(128 * 128 * 4);
        for y in 0..128_u32 {
            for x in 0..128_u32 {
                let noise = ((x * 73 + y * 151 + x * y * 19) % 97) as f32 / 97.0;
                let value = match surface {
                    Surface::Wood => {
                        let wave = (x as f32 * 0.36 + (y as f32 * 0.04).sin() * 1.4).sin();
                        0.76 + wave * 0.12 + noise * 0.1
                    }
                    Surface::Cloth => 0.72 + if (x + y) % 4 == 0 { 0.2 } else { noise * 0.11 },
                    Surface::Paper => 0.86 + noise * 0.13,
                    Surface::Plaster => 0.9 + noise * 0.08,
                };
                let shade = (value * 255.0) as u8;
                pixels.extend_from_slice(&[shade, shade, shade, 255]);
            }
        }
        let image = images.add(Image::new(
            Extent3d {
                width: 128,
                height: 128,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            pixels,
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default(),
        ));
        if let Some(mut material) = room.materials.get_mut(material) {
            material.base_color_texture = Some(image);
            material.perceptual_roughness = match surface {
                Surface::Wood => 0.48,
                Surface::Cloth => 0.94,
                _ => 0.8,
            };
        }
    }
}
