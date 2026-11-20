use super::{CEILING, Lamp, OAK, PLASTER, Shape, WALNUT};
use bevy::math::{Vec3, vec3};
use latch::{
    room_lighting::{
        FOYER_LANTERN, FOYER_PASSAGE_LIGHT, FOYER_SCONCE, Finish, GATE_X, PANEL_Z, PORCH_LIGHT,
        RoomSurface, SHORE_LIGHT, SIGNAL_ENTRY_LIGHT, SIGNAL_PANEL_LIGHT, foyer_surfaces,
        signal_surfaces,
    },
    study::{BoxGeometry, Surface},
};

pub(super) const FOYER_FILL: Vec3 = vec3(28.0, 35.0, 45.0);
pub(super) const SIGNAL_FILL: Vec3 = vec3(30.0, 39.0, 46.0);
const IRON: Vec3 = vec3(0.12, 0.16, 0.18);

pub(super) const FOYER_FIXED: [Lamp; 2] = [
    Lamp {
        position: FOYER_PASSAGE_LIGHT,
        color: vec3(0.56, 0.65, 1.0),
        lumens: 620.0,
    },
    Lamp {
        position: PORCH_LIGHT,
        color: vec3(0.35, 0.53, 1.0),
        lumens: 720.0,
    },
];
pub(super) const FOYER_SWITCHED: [Lamp; 2] = [
    Lamp {
        position: FOYER_SCONCE,
        color: vec3(1.0, 0.58, 0.24),
        lumens: 950.0,
    },
    Lamp {
        position: FOYER_LANTERN,
        color: vec3(0.75, 0.65, 0.50),
        lumens: 1250.0,
    },
];
pub(super) const SIGNAL_LIGHTS: [Lamp; 3] = [
    Lamp {
        position: SIGNAL_ENTRY_LIGHT,
        color: vec3(0.32, 0.55, 0.83),
        lumens: 1050.0,
    },
    Lamp {
        position: SIGNAL_PANEL_LIGHT,
        color: vec3(0.81, 0.48, 0.17),
        lumens: 1000.0,
    },
    Lamp {
        position: SHORE_LIGHT,
        color: vec3(0.25, 0.51, 1.0),
        lumens: 640.0,
    },
];

/// 建立與玄關表面同位置的烘焙幾何。可移動木箱、門板和拉桿由即時光照處理。
pub(super) fn foyer() -> (Vec<Surface>, Vec<Shape>) {
    let (surfaces, mut shapes) = shell(foyer_surfaces());
    for x in [-2.47, 3.47] {
        shapes.push(block([x, 0.52, 10.3], [0.05, 0.88, 6.4], WALNUT));
    }
    shapes.extend([
        block(
            [0.5, 0.044, 10.4],
            [2.4, 0.04, 4.4],
            vec3(0.24, 0.076, 0.032),
        ),
        block([3.44, 1.62, 8.20], [0.10, 0.96, 0.64], WALNUT),
        block([3.46, 1.48, 10.2], [0.08, 0.70, 1.84], WALNUT),
        block([-2.46, 0.72, 10.1], [0.04, 0.56, 0.72], WALNUT),
        block([-1.18, 1.28, 13.44], [0.34, 0.42, 0.14], IRON),
        block([-0.46, 1.28, 13.48], [0.14, 2.56, 0.22], WALNUT),
        block([1.46, 1.28, 13.48], [0.14, 2.56, 0.22], WALNUT),
        block([0.5, 2.58, 13.48], [2.06, 0.16, 0.22], WALNUT),
    ]);
    (surfaces, shapes)
}

/// 建立信號盤、固定支架與門框的近似遮擋。活動閘門、光束和接收器不烘焙。
pub(super) fn signal() -> (Vec<Surface>, Vec<Shape>) {
    let (surfaces, mut shapes) = shell(signal_surfaces());
    for x in [-3.3_f32, 4.3] {
        for z in [17.35, 19.6, 21.85] {
            shapes.push(block(
                [x.signum().mul_add(-0.13, x), 1.83, z],
                [0.17, 3.65, 0.15],
                IRON,
            ));
        }
        shapes.push(block(
            [x - 0.155 * x.signum(), 0.30, 20.1],
            [0.095, 0.48, 6.83],
            IRON,
        ));
    }
    shapes.extend([
        block([1.55, 1.87, PANEL_Z + 0.03], [3.28, 2.88, 0.15], IRON),
        block([3.58, 1.78, 23.42], [0.082, 3.20, 0.095], IRON),
        block([GATE_X - 0.91, 1.28, 23.51], [0.15, 2.56, 0.26], IRON),
        block([GATE_X + 0.91, 1.28, 23.51], [0.15, 2.56, 0.26], IRON),
        block([GATE_X, 2.52, 23.51], [1.92, 0.16, 0.24], IRON),
    ]);
    (surfaces, shapes)
}

/// 將房體表面轉成烘焙採樣與射線遮擋的共用幾何。
fn shell(definitions: impl Iterator<Item = RoomSurface>) -> (Vec<Surface>, Vec<Shape>) {
    definitions
        .map(|definition| {
            let surface = definition.surface;
            let albedo = match definition.finish {
                Finish::Floor | Finish::Wood => OAK,
                Finish::Wall => PLASTER,
                Finish::Ceiling => CEILING,
                Finish::Night => vec3(0.033, 0.053, 0.11),
            };
            (surface, Shape::new(surface.geometry, albedo))
        })
        .unzip()
}

/// 建立帶有漫反射係數的近似遮擋物。
fn block(at: [f32; 3], size: [f32; 3], albedo: Vec3) -> Shape {
    Shape::new(BoxGeometry::new(at, size), albedo)
}
