
use std::path::Path;
use aluminium::{VulkanError, VulkanResult, WorldRenderer};
use gltf::Gltf;

pub struct GltfModel {

}

pub fn load_gltf<P: AsRef<Path>>(world: &mut WorldRenderer, path: P) -> VulkanResult<GltfModel> {
    let gltf = Gltf::open(path).expect("Not found path to gltf model");
    todo!()
}