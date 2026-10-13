//! Modelos 3D importados: las motos y sus pilotos hechos con Meshy, ya
//! preparados para el juego por `tools/import_models.py` e incrustados en el
//! ejecutable.
//!
//! El script deja cada moto en cuatro piezas (chasis, dirección y ruedas) en
//! el espacio de la moto, más las bocas de sus escapes, y su piloto con sus
//! pesos y su esqueleto en reposo. El formato está descrito en el script.

use bevy::{
    asset::RenderAssetUsages,
    image::{
        CompressedImageFormats, ImageAddressMode, ImageFilterMode, ImageSampler,
        ImageSamplerDescriptor, ImageType,
    },
    mesh::{Indices, PrimitiveTopology, VertexAttributeValues},
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

use super::{Geometry, Mount, Parts, tyre};
use crate::world::generate_mipmaps;

/// Una moto de Meshy ya preparada: sus cuatro piezas, sus texturas, su
/// piloto y sus medidas.
pub(super) struct Model {
    pub geometry: Geometry,
    /// Si la horquilla y el manillar giran con la dirección. En las motos con
    /// la horquilla soldada al carenado no se pueden separar, y no giran.
    pub steers: bool,
    bike: &'static [u8],
    base_color: &'static [u8],
    metallic_roughness: &'static [u8],
    normal: &'static [u8],
    pub pilot: &'static [u8],
    pub pilot_color: &'static [u8],
}

/// Los ficheros de `assets/models/CARPETA/`, incrustados en el ejecutable.
macro_rules! model {
    ($folder:literal, $geometry:expr) => {
        model!($folder, $geometry, true)
    };
    ($folder:literal, $geometry:expr, $steers:expr) => {
        Model {
            geometry: $geometry,
            steers: $steers,
            bike: include_bytes!(concat!("../../../assets/models/", $folder, "/bike.bin")),
            base_color: include_bytes!(concat!(
                "../../../assets/models/",
                $folder,
                "/base_color.jpg"
            )),
            metallic_roughness: include_bytes!(concat!(
                "../../../assets/models/",
                $folder,
                "/metallic_roughness.jpg"
            )),
            normal: include_bytes!(concat!("../../../assets/models/", $folder, "/normal.jpg")),
            pilot: include_bytes!(concat!("../../../assets/models/", $folder, "/pilot.bin")),
            pilot_color: include_bytes!(concat!(
                "../../../assets/models/",
                $folder,
                "/pilot_color.jpg"
            )),
        }
    };
}

// Medidas sacadas por `tools/import_models.py`. Las de la Shogun TZ se midieron a
// mano sobre el modelo y el script las usa de referencia para las demás.

/// Shogun TZ, azul y blanca.
pub(super) static SHOGUN: Model = model!(
    "shogun",
    Geometry {
        front_axle: Vec3::new(0.0, 0.2826, -0.70),
        rear_axle: Vec3::new(0.0, 0.313, 0.68),
        front_radius: 0.2882,
        rear_radius: 0.313,
        steering_head: Vec3::new(0.0, 0.8883, -0.423),
        grip: Vec3::new(0.33, 0.79, -0.36),
        peg: Vec3::new(0.16, 0.36, 0.27),
        seat: Vec3::new(0.0, 0.91, 0.2),
    }
);

/// Kraken RS, verde.
pub(super) static KRAKEN: Model = model!(
    "kraken",
    Geometry {
        front_axle: Vec3::new(0.0, 0.2871, -0.7),
        rear_axle: Vec3::new(0.0, 0.3129, 0.68),
        front_radius: 0.2927,
        rear_radius: 0.3099,
        steering_head: Vec3::new(0.0, 0.92, -0.4326),
        grip: Vec3::new(0.2913, 0.8336, -0.3921),
        peg: Vec3::new(0.16, 0.3798, 0.245),
        seat: Vec3::new(0.0, 0.9298, 0.175),
    }
);

/// Falcon KRZ, roja, azul y blanca.
pub(super) static FALCON: Model = model!(
    "falcon",
    Geometry {
        front_axle: Vec3::new(0.0, 0.2923, -0.7),
        rear_axle: Vec3::new(0.0, 0.3324, 0.68),
        front_radius: 0.2953,
        rear_radius: 0.3294,
        steering_head: Vec3::new(0.0, 0.9637, -0.4209),
        grip: Vec3::new(0.2656, 0.83, -0.391),
        peg: Vec3::new(0.16, 0.4183, 0.27),
        seat: Vec3::new(0.0, 0.9683, 0.2),
    }
);

/// Stiletto V4, roja.
pub(super) static STILETTO: Model = model!(
    "stiletto",
    Geometry {
        front_axle: Vec3::new(0.0, 0.2837, -0.7),
        rear_axle: Vec3::new(0.0, 0.3305, 0.68),
        front_radius: 0.2929,
        rear_radius: 0.3173,
        steering_head: Vec3::new(0.0, 0.9508, -0.4631),
        grip: Vec3::new(0.2548, 0.8416, -0.447),
        peg: Vec3::new(0.16, 0.4206, 0.27),
        seat: Vec3::new(0.0, 0.9706, 0.2),
    }
);

/// Legend 88, blanca y roja. Su horquilla viene soldada al carenado: la
/// dirección no gira.
pub(super) static LEGEND: Model = model!(
    "legend",
    Geometry {
        front_axle: Vec3::new(0.0, 0.2866, -0.7),
        rear_axle: Vec3::new(0.0, 0.3113, 0.68),
        front_radius: 0.2866,
        rear_radius: 0.3083,
        steering_head: Vec3::new(0.0, 0.8895, -0.4316),
        grip: Vec3::new(0.2759, 0.8466, -0.4571),
        peg: Vec3::new(0.16, 0.3689, 0.245),
        seat: Vec3::new(0.0, 0.9189, 0.175),
    },
    false
);

/// Cobra XT, naranja y negra.
pub(super) static COBRA: Model = model!(
    "cobra",
    Geometry {
        front_axle: Vec3::new(0.0, 0.281, -0.7),
        rear_axle: Vec3::new(0.0, 0.3063, 0.68),
        front_radius: 0.2901,
        rear_radius: 0.3063,
        steering_head: Vec3::new(0.0, 0.8891, -0.3867),
        grip: Vec3::new(0.2891, 0.8138, -0.3693),
        peg: Vec3::new(0.16, 0.3236, 0.295),
        seat: Vec3::new(0.0, 0.8736, 0.225),
    }
);

/// Ronin YS, azul y negra.
pub(super) static RONIN: Model = model!(
    "ronin",
    Geometry {
        front_axle: Vec3::new(0.0, 0.2809, -0.7),
        rear_axle: Vec3::new(0.0, 0.3107, 0.68),
        front_radius: 0.2855,
        rear_radius: 0.3097,
        steering_head: Vec3::new(0.0, 0.8746, -0.4629),
        grip: Vec3::new(0.2643, 0.7959, -0.4458),
        peg: Vec3::new(0.16, 0.3693, 0.27),
        seat: Vec3::new(0.0, 0.9193, 0.2),
    }
);

/// Azura RZ, amarilla.
pub(super) static AZURA: Model = model!(
    "azura",
    Geometry {
        front_axle: Vec3::new(0.0, 0.2966, -0.7),
        rear_axle: Vec3::new(0.0, 0.3082, 0.68),
        front_radius: 0.2936,
        rear_radius: 0.3082,
        steering_head: Vec3::new(0.0, 0.9079, -0.41),
        grip: Vec3::new(0.29, 0.8285, -0.3881),
        peg: Vec3::new(0.16, 0.3682, 0.22),
        seat: Vec3::new(0.0, 0.9182, 0.15),
    }
);

/// Una malla de un paquete, con su nombre.
pub(super) struct PackedMesh {
    pub name: String,
    pub mesh: Mesh,
}

/// Un hueso del esqueleto en reposo, en el espacio de su malla.
#[derive(Clone, Debug)]
pub(super) struct Joint {
    pub name: String,
    pub parent: Option<usize>,
    pub position: Vec3,
    pub rotation: Quat,
}

/// Contenido de un `.bin` de `tools/import_models.py`.
pub(super) struct Pack {
    pub meshes: Vec<PackedMesh>,
    pub skeleton: Vec<Joint>,
}

/// Lector de los datos en orden, sin copias.
struct Reader<'a> {
    bytes: &'a [u8],
}

impl<'a> Reader<'a> {
    fn take(&mut self, count: usize) -> &'a [u8] {
        let (head, rest) = self.bytes.split_at(count);
        self.bytes = rest;
        head
    }

    fn u32(&mut self) -> u32 {
        u32::from_le_bytes(self.take(4).try_into().unwrap())
    }

    fn i32(&mut self) -> i32 {
        i32::from_le_bytes(self.take(4).try_into().unwrap())
    }

    fn f32s(&mut self, count: usize) -> Vec<f32> {
        self.take(count * 4)
            .as_chunks::<4>()
            .0
            .iter()
            .map(|&b| f32::from_le_bytes(b))
            .collect()
    }

    fn name(&mut self) -> String {
        let length = self.u32() as usize;
        String::from_utf8_lossy(self.take(length)).into_owned()
    }

    fn vec3s(&mut self, count: usize) -> Vec<[f32; 3]> {
        self.f32s(count * 3).as_chunks::<3>().0.to_vec()
    }
}

pub(super) fn read(bytes: &[u8]) -> Pack {
    let mut reader = Reader { bytes };
    assert_eq!(reader.take(8), b"MRMESH01", "no es un paquete de mallas");
    let count = reader.u32();
    let meshes = (0..count)
        .map(|_| {
            let name = reader.name();
            let (vertices, indices, flags) =
                (reader.u32() as usize, reader.u32() as usize, reader.u32());
            let positions = reader.vec3s(vertices);
            let normals = reader.vec3s(vertices);
            let uvs: Vec<[f32; 2]> = reader.f32s(vertices * 2).as_chunks::<2>().0.to_vec();
            let mut mesh = Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::default(),
            )
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
            if flags & 1 != 0 {
                let joints: Vec<[u16; 4]> = reader
                    .take(vertices * 8)
                    .as_chunks::<8>()
                    .0
                    .iter()
                    .map(|b| [0, 2, 4, 6].map(|i| u16::from_le_bytes([b[i], b[i + 1]])))
                    .collect();
                let weights: Vec<[f32; 4]> = reader.f32s(vertices * 4).as_chunks::<4>().0.to_vec();
                mesh.insert_attribute(
                    Mesh::ATTRIBUTE_JOINT_INDEX,
                    VertexAttributeValues::Uint16x4(joints),
                );
                mesh.insert_attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT, weights);
            }
            let indices = reader
                .take(indices * 4)
                .as_chunks::<4>()
                .0
                .iter()
                .map(|&b| u32::from_le_bytes(b))
                .collect();
            mesh.insert_indices(Indices::U32(indices));
            PackedMesh { name, mesh }
        })
        .collect();

    let mut skeleton = Vec::new();
    if !reader.bytes.is_empty() {
        assert_eq!(reader.take(8), b"MRSKEL01", "esqueleto mal formado");
        for _ in 0..reader.u32() {
            let name = reader.name();
            let parent = usize::try_from(reader.i32()).ok();
            let position = Vec3::from_slice(&reader.f32s(3));
            let rotation = Quat::from_slice(&reader.f32s(4)).normalize();
            skeleton.push(Joint {
                name,
                parent,
                position,
                rotation,
            });
        }
    }
    Pack { meshes, skeleton }
}

/// Textura de un JPEG incrustado, con mipmaps. Las de color van en sRGB; las
/// de datos (normales, metal y rugosidad), en lineal.
pub(super) fn texture(bytes: &[u8], srgb: bool) -> Image {
    let sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::ClampToEdge,
        address_mode_v: ImageAddressMode::ClampToEdge,
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        mipmap_filter: ImageFilterMode::Linear,
        anisotropy_clamp: 8,
        ..default()
    });
    let mut image = Image::from_buffer(
        bytes,
        ImageType::Extension("jpg"),
        CompressedImageFormats::NONE,
        srgb,
        sampler,
        RenderAssetUsages::RENDER_WORLD,
    )
    .expect("las texturas incrustadas son JPEG válidos");
    generate_mipmaps(&mut image);
    image
}

/// Material de una moto: color, metal y rugosidad, y relieve.
pub(super) fn material(model: &Model, images: &mut Assets<Image>) -> StandardMaterial {
    StandardMaterial {
        base_color_texture: Some(images.add(texture(model.base_color, true))),
        metallic_roughness_texture: Some(images.add(texture(model.metallic_roughness, false))),
        normal_map_texture: Some(images.add(texture(model.normal, false))),
        metallic: 1.0,
        perceptual_roughness: 1.0,
        ..default()
    }
}

/// Tono de las bocas de los escapes según lo de dentro que esté cada punto
/// (de 0, el canto del labio, a 1, el centro del fondo), en sRGB: el labio del
/// tubo es de metal y por dentro se va oscureciendo, con hollín, hasta el
/// negro.
fn exhaust_shade(depth: f32) -> f32 {
    const METAL: f32 = 0.62;
    const SOOT: f32 = 0.2;
    const DARK: f32 = 0.06;
    const BLACK: f32 = 0.02;
    if depth < 0.02 {
        METAL
    } else if depth < 0.1 {
        METAL.lerp(SOOT, (depth - 0.02) / 0.08)
    } else if depth < 0.6 {
        SOOT.lerp(DARK, (depth - 0.1) / 0.5)
    } else {
        DARK.lerp(BLACK, ((depth - 0.6) / 0.4).min(1.0))
    }
}

/// Material de las bocas de los escapes: la coordenada u de su pieza dice lo
/// de dentro que está cada punto, y una textura de una fila le da el tono.
pub(super) fn exhaust_material(images: &mut Assets<Image>) -> StandardMaterial {
    const WIDTH: u32 = 64;
    let data = (0..WIDTH)
        .flat_map(|i| {
            let shade = (exhaust_shade((i as f32 + 0.5) / WIDTH as f32) * 255.0).round() as u8;
            [shade, shade, shade, 255]
        })
        .collect();
    let mut image = Image::new(
        Extent3d {
            width: WIDTH,
            height: 1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::ClampToEdge,
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        ..default()
    });
    // Sin reflejos: el brillo de la luz ambiente aclararía el hueco y dejaría
    // de parecer hondo.
    StandardMaterial {
        base_color_texture: Some(images.add(image)),
        perceptual_roughness: 0.6,
        reflectance: 0.0,
        ..default()
    }
}

/// Los materiales de una moto importada: el suyo y el de las bocas de los
/// escapes, que es el mismo para todas, and that of its tyres' pattern.
pub(super) struct Materials {
    pub body: Handle<StandardMaterial>,
    pub exhaust: Handle<StandardMaterial>,
    pub tyres: Handle<StandardMaterial>,
}

/// Añade las piezas de una moto, cada una en su punto de montaje. The
/// wheels lose their Meshy tyres for round ones (see [`tyre`]).
pub(super) fn build(parts: &mut Parts, model: &Model, materials: &Materials) {
    let geometry = model.geometry;
    for PackedMesh { name, mesh } in read(model.bike).meshes {
        let mount = match name.as_str() {
            "chassis" | "exhaust" => Mount::Chassis,
            "steering" => Mount::Steering,
            "front_wheel" => Mount::FrontWheel,
            "rear_wheel" => Mount::RearWheel,
            other => panic!("pieza desconocida en una moto importada: {other}"),
        };
        let wheel = match mount {
            Mount::FrontWheel => Some((geometry.front_axle, geometry.front_radius)),
            Mount::RearWheel => Some((geometry.rear_axle, geometry.rear_radius)),
            _ => None,
        };
        let mesh = match wheel {
            Some((axle, radius)) => {
                let (rim, half_width) = tyre::strip(mesh, axle, radius);
                let round = tyre::mesh(radius, half_width).translated_by(axle);
                let round = parts.meshes.add(round);
                parts.add_textured(mount, round, materials.tyres.clone());
                rim
            }
            None => mesh,
        };
        // Las bocas no llevan relieve: no les hacen falta tangentes.
        let (mesh, material) = if name == "exhaust" {
            (mesh, materials.exhaust.clone())
        } else {
            let mesh = mesh
                .with_generated_tangents()
                .expect("las piezas tienen normales y coordenadas de textura");
            (mesh, materials.body.clone())
        };
        let mesh = parts.meshes.add(mesh);
        parts.add_textured(mount, mesh, material);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MODELS: [&Model; 8] = [
        &SHOGUN, &KRAKEN, &FALCON, &STILETTO, &AZURA, &LEGEND, &COBRA, &RONIN,
    ];

    #[test]
    fn bikes_have_their_parts_and_wheels_on_the_ground() {
        for model in MODELS {
            let pack = read(model.bike);
            let names: Vec<&str> = pack.meshes.iter().map(|m| m.name.as_str()).collect();
            assert_eq!(
                names,
                [
                    "chassis",
                    "steering",
                    "front_wheel",
                    "rear_wheel",
                    "exhaust"
                ]
            );
            assert!(pack.skeleton.is_empty());
            // Cada rueda está alrededor de su eje y toca el suelo.
            let geometry = &model.geometry;
            for (part, axle, radius) in [
                (2, geometry.front_axle, geometry.front_radius),
                (3, geometry.rear_axle, geometry.rear_radius),
            ] {
                let Some(VertexAttributeValues::Float32x3(positions)) =
                    pack.meshes[part].mesh.attribute(Mesh::ATTRIBUTE_POSITION)
                else {
                    panic!("sin posiciones");
                };
                let lowest = positions.iter().map(|p| p[1]).fold(f32::MAX, f32::min);
                assert!(lowest.abs() < 0.02, "apoya en el suelo: {lowest}");
                let farthest = positions
                    .iter()
                    .map(|&p| Vec3::from(p).distance(axle))
                    .fold(0.0, f32::max);
                assert!(
                    (farthest - radius).abs() < 0.03,
                    "radio {farthest} frente a {radius}"
                );
            }
        }
    }

    #[test]
    fn exhausts_open_at_the_back_of_the_silencers() {
        for model in MODELS {
            let pack = read(model.bike);
            let exhaust = &pack.meshes[4].mesh;
            let Some(VertexAttributeValues::Float32x3(positions)) =
                exhaust.attribute(Mesh::ATTRIBUTE_POSITION)
            else {
                panic!("sin posiciones");
            };
            assert!(!positions.is_empty(), "cada moto tiene al menos una boca");
            for &[x, y, z] in positions {
                // Detrás del asiento, por encima del suelo y a un lado o bajo
                // el colín.
                assert!(
                    z > 0.25 && y > 0.2 && (x.abs() > 0.05 || y > 0.7),
                    "boca en {x} {y} {z}"
                );
            }
        }
    }

    #[test]
    fn exhaust_lips_are_metal_and_their_insides_dark() {
        assert!(exhaust_shade(0.0) > 0.5);
        assert!(exhaust_shade(1.0) < 0.05);
        let shades: Vec<f32> = (0..=20).map(|i| exhaust_shade(i as f32 / 20.0)).collect();
        assert!(shades.windows(2).all(|pair| pair[1] <= pair[0]));
    }

    #[test]
    fn pilots_have_a_rigged_body() {
        for model in MODELS {
            let pack = read(model.pilot);
            assert_eq!(pack.meshes.len(), 1);
            assert!(
                pack.meshes[0]
                    .mesh
                    .attribute(Mesh::ATTRIBUTE_JOINT_INDEX)
                    .is_some()
            );
            assert_eq!(pack.skeleton.len(), 23);
            assert_eq!(pack.skeleton[0].name, "Hips");
            assert!(pack.skeleton[0].parent.is_none());
            for (index, joint) in pack.skeleton.iter().enumerate().skip(1) {
                let parent = joint.parent.expect("todos menos la cadera tienen padre");
                assert!(parent < index, "los padres van antes que los hijos");
            }
        }
    }
}
