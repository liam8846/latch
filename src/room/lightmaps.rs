use super::{LightKind, SwitchableLight, builder::Furnisher, materials::Palette};
use bevy::{mesh::VertexAttributeValues, pbr::Lightmap, prelude::*};
use latch::{
    room_lighting::{
        FOYER_LIT, FOYER_NIGHT, FOYER_SURFACE_COUNT, Finish, RoomSurface, SIGNAL_ATLAS,
        SIGNAL_SURFACE_COUNT, STUDY_SURFACE_COUNT, atlas_height,
    },
    study::{
        ATLAS_WIDTH, LAMP_ATLAS, LIGHTMAP_EXPOSURE, MOON_ATLAS, PADDING, Surface, TILE_SIZE,
        tile_origin,
    },
};

#[derive(Clone, Copy, Component)]
pub enum BakedRoom {
    Study,
    Foyer,
    Signal,
}

struct Atlas {
    base: Handle<Image>,
    lit: Handle<Image>,
    height: u32,
}

#[derive(Resource)]
pub struct RoomLightmaps {
    study: Atlas,
    foyer: Atlas,
    signal: Atlas,
}

impl RoomLightmaps {
    /// 載入各區圖集。書房與玄關各有獨立的開燈與關燈版本。
    pub(super) fn load(assets: &AssetServer) -> Self {
        let signal = assets.load(SIGNAL_ATLAS);
        Self {
            study: Atlas {
                base: assets.load(MOON_ATLAS),
                lit: assets.load(LAMP_ATLAS),
                height: atlas_height(STUDY_SURFACE_COUNT),
            },
            foyer: Atlas {
                base: assets.load(FOYER_NIGHT),
                lit: assets.load(FOYER_LIT),
                height: atlas_height(FOYER_SURFACE_COUNT),
            },
            signal: Atlas {
                base: signal.clone(),
                lit: signal,
                height: atlas_height(SIGNAL_SURFACE_COUNT),
            },
        }
    }

    /// 取得對應房間的圖集。
    fn atlas(&self, room: BakedRoom) -> &Atlas {
        match room {
            BakedRoom::Study => &self.study,
            BakedRoom::Foyer => &self.foyer,
            BakedRoom::Signal => &self.signal,
        }
    }

    /// 建立指定表面的圖集 UV 範圍；內縮至像素中心以免取樣到相鄰圖塊。
    pub(super) fn map(&self, room: BakedRoom, index: usize) -> Lightmap {
        let atlas = self.atlas(room);
        let (x, y) = tile_origin(index as u32);
        let size = Vec2::new(ATLAS_WIDTH as f32, atlas.height as f32);
        let min = (Vec2::new(x as f32, y as f32) + (PADDING as f32 + 0.5)) / size;
        let max = (Vec2::new((x + TILE_SIZE) as f32, (y + TILE_SIZE) as f32)
            - (PADDING as f32 + 0.5))
            / size;
        Lightmap {
            image: atlas.lit.clone(),
            uv_rect: Rect::from_corners(min, max),
            bicubic_sampling: false,
        }
    }
}

/// 燈具開關只切換所屬房間的反彈光；信號室的圖集跟著固定燈源。
pub fn update_room_lightmaps(
    lighting: Res<RoomLightmaps>,
    switches: Query<&SwitchableLight>,
    mut surfaces: Query<(&BakedRoom, &mut Lightmap)>,
) {
    let mut desk = false;
    let mut foyer = false;
    for switch in &switches {
        desk |= switch.kind == LightKind::DeskLamp && switch.on;
        foyer |= switch.kind == LightKind::FoyerLamp && switch.on;
    }
    for (&room, mut map) in &mut surfaces {
        let atlas = lighting.atlas(room);
        let on = match room {
            BakedRoom::Study => desk,
            BakedRoom::Foyer => foyer,
            BakedRoom::Signal => true,
        };
        let image = if on { &atlas.lit } else { &atlas.base };
        if map.image != *image {
            map.image = image.clone();
        }
    }
}

/// 建立預先烘焙的固定房體；按材質重用光照貼圖專用的材質設定。
pub(super) fn spawn_surfaces(
    room: &mut Furnisher,
    palette: &Palette,
    lighting: &RoomLightmaps,
    kind: BakedRoom,
    surfaces: impl Iterator<Item = RoomSurface>,
) {
    let materials = [
        &palette.floor,
        &palette.wood,
        &palette.wall,
        &palette.trim,
        &palette.moon,
    ]
    .map(|handle| exposed_material(room, handle));
    for (index, definition) in surfaces.enumerate() {
        let material = &materials[match definition.finish {
            Finish::Floor => 0,
            Finish::Wood => 1,
            Finish::Wall => 2,
            Finish::Ceiling => 3,
            Finish::Night => 4,
        }];
        let surface = definition.surface;
        let at = surface.geometry.center.to_array();
        let size = surface.geometry.size.to_array();
        let entity = if definition.solid {
            room.solid_block(at, size, material)
        } else {
            room.block(at, size, material)
        };
        let mesh = mapped_cube(room, surface);
        room.commands
            .entity(entity)
            .insert((Mesh3d(mesh), kind, lighting.map(kind, index)));
    }
}

/// 以共用長方體 UV0 與切線為基礎，另外為指定平面建立 UV1。
pub(super) fn mapped_cube(room: &mut Furnisher, surface: Surface) -> Handle<Mesh> {
    let mesh = room
        .meshes
        .get(&room.cube)
        .cloned()
        .unwrap_or_else(|| Cuboid::new(1.0, 1.0, 1.0).into());
    room.meshes.add(with_lightmap_uv(mesh, surface))
}

/// 加入光照貼圖座標；保留原有 PBR 材質座標。
pub(super) fn with_lightmap_uv(mut mesh: Mesh, surface: Surface) -> Mesh {
    if let Some(VertexAttributeValues::Float32x3(positions)) =
        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    {
        let uv: Vec<_> = positions
            .iter()
            .map(|&p| surface.uv(Vec3::from_array(p)))
            .collect();
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_1, uv);
    }
    mesh
}

/// 把 sRGB 圖集中的正規化照度還原到場景光照範圍。
pub(super) fn exposed_material(
    room: &mut Furnisher,
    source: &Handle<StandardMaterial>,
) -> Handle<StandardMaterial> {
    let mut material = room.materials.get(source).cloned().unwrap_or_default();
    material.lightmap_exposure = LIGHTMAP_EXPOSURE;
    room.materials.add(material)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn room_lightmaps_follow_only_their_own_switch() {
        let mut app = App::new();
        let mut images = Assets::<Image>::default();
        let mut atlas = |count| Atlas {
            base: images.add(Image::default()),
            lit: images.add(Image::default()),
            height: atlas_height(count),
        };
        let maps = RoomLightmaps {
            study: atlas(STUDY_SURFACE_COUNT),
            foyer: atlas(FOYER_SURFACE_COUNT),
            signal: atlas(SIGNAL_SURFACE_COUNT),
        };
        let kinds = [BakedRoom::Study, BakedRoom::Foyer, BakedRoom::Signal];
        let expected =
            kinds.map(|kind| (maps.atlas(kind).base.clone(), maps.atlas(kind).lit.clone()));
        let surfaces = kinds.map(|kind| app.world_mut().spawn((kind, maps.map(kind, 0))).id());
        let switches = [LightKind::DeskLamp, LightKind::FoyerLamp].map(|kind| {
            app.world_mut()
                .spawn(SwitchableLight {
                    kind,
                    on: true,
                    intensity: 1000.0,
                })
                .id()
        });
        app.insert_resource(maps)
            .add_systems(Update, update_room_lightmaps);
        for modes in [
            [true, true],
            [true, false],
            [false, true],
            [false, false],
            [true, true],
        ] {
            for (switch, on) in switches.into_iter().zip(modes) {
                app.world_mut()
                    .get_mut::<SwitchableLight>(switch)
                    .unwrap()
                    .on = on;
            }
            app.update();
            for (i, on) in [modes[0], modes[1], true].into_iter().enumerate() {
                let actual = &app.world().get::<Lightmap>(surfaces[i]).unwrap().image;
                assert_eq!(actual, if on { &expected[i].1 } else { &expected[i].0 });
            }
        }
    }
}
