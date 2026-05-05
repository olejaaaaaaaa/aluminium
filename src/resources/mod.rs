mod resources;
use std::sync::Arc;

pub use resources::Resources;

mod sampler;
pub use sampler::*;

mod types;
pub use types::*;

mod textures;
pub use textures::*;

mod buffers;
pub use buffers::*;

mod descriptor_cache;
pub use descriptor_cache::*;

mod pipeline_cache;
pub use pipeline_cache::*;

use crate::render_context::RenderContext;
use crate::VulkanResult;

#[allow(missing_docs)]
pub trait Create: Sized {
    type Desc<'a>;
    fn create(
        ctx: &Arc<RenderContext>,
        resources: &Arc<Resources>,
        desc: Self::Desc<'_>,
    ) -> VulkanResult<Res<Self>>;
}

#[allow(missing_docs)]
pub trait GetMut: Sized {
    type Output;
    fn try_get_mut(&self) -> Option<RefMut<'_, Self::Output>>;
    fn get_mut(&self) -> RefMut<'_, Self::Output> {
        self.try_get_mut().unwrap()
    }
}

#[allow(missing_docs)]
pub trait Get: Sized {
    type Output;
    fn try_get(&self) -> Option<Ref<'_, Self::Output>>;
    fn get(&self) -> Ref<'_, Self::Output> {
        self.try_get().unwrap()
    }
}
