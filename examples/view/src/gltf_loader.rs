use std::path::Path;
use glam::{Mat4, Vec3};
use aluminium::types::{PbrVertex, Vertex};
use aluminium::{Mesh, MeshDesc, Res, Transform, TransformDesc, VulkanResult, WorldRenderer};
use bytemuck::{Pod, Zeroable};

#[derive(Clone)]
pub struct GltfModel {
    pub meshes: Vec<(Res<Mesh>, Res<Transform>)>,
}

fn load_gltf_node(world: &WorldRenderer, model: &mut GltfModel, node: gltf::Node<'_>, buffers: &[gltf::buffer::Data], parent_transform: Mat4) -> VulkanResult<()> {

    let node_transform = parent_transform * Mat4::from_cols_array_2d(&node.transform().matrix());

    for child in node.children() {
        load_gltf_node(world, model, child, buffers, node_transform)?;
    }

    if let Some(mesh) = node.mesh() {
        let primitives = mesh.primitives();
        for i in primitives {
            let reader = i.reader(|i| Some(&buffers[i.index()]));

            let indices: Vec<_> = reader.read_indices().unwrap().into_u32().collect();
            let positions: Vec<_> = reader
                .read_positions()
                .unwrap()
                .map(|x| [x[0], x[1], x[2], 1.0])
                .collect();

            let tex_coords = if let Some(tex_coords) = reader.read_tex_coords(0) {
                tex_coords.into_f32().map(|x| x).collect()
            } else {
                vec![[0.0, 0.0]; positions.len()]
            };

            let tangents = if let Some(tangents) = reader.read_tangents() {
                tangents.map(|x| x).collect()
            } else {
                vec![[0.0, 0.0, 0.0, 0.0]; positions.len()]
            };

            let normals: Vec<_> = reader
                .read_normals()
                .unwrap()
                .map(|x| [x[0], x[1], x[2], 0.0])
                .collect();

            let colors: Vec<_> = if let Some(colors) = reader.read_colors(0) {
                colors.into_rgba_f32().map(|x| x).collect()
            } else {
                vec![[0.9, 0.6, 0.4, 1.0]; positions.len()]
            };

            let mut vertices = vec![];

            for (index, pos) in positions.into_iter().enumerate() {
                vertices.push(PbrVertex {
                    pos,
                    normal: normals[index],
                    uv: tex_coords[index],
                    color: colors[index],
                    tangent: tangents[index],
                });
            }

            let mesh = world.create::<Mesh>(MeshDesc::new(&vertices).with_indices(&indices))?;

            let proj = Mat4::perspective_rh(45.0_f32.to_radians(), 800.0 / 600.0, 0.1, 1000.0);
            let view = Mat4::look_at_rh(
                Vec3::new(0.0, 0.1, 0.9),
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::NEG_Y,
            );

            let mvp = proj * view * node_transform;
            let transform = world.create::<Transform>(TransformDesc::from(mvp.to_cols_array_2d()))?;

            model.meshes.push((mesh, transform));
        }
    }

    Ok(())
}

pub fn load_gltf<P: AsRef<Path>>(world: &WorldRenderer, path: P) -> VulkanResult<GltfModel> {
    let (gltf, buffers, mut images) = match gltf::import(path.as_ref()) {
        Ok(result) => result,
        Err(err) => panic!("Error load gltf model with path: {:?} with error: {:?}", path.as_ref().as_os_str(), err),
    };

    let mut gltf_model = GltfModel { meshes: vec![] };

    for scene in gltf.scenes() {
        for node in scene.nodes() {
            load_gltf_node(world, &mut gltf_model, node, &buffers, Mat4::IDENTITY)?;
        }
    }

    Ok(gltf_model)
}
