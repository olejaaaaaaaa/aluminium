use ash::vk;
use puffin::profile_scope;
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};

use super::app::App;
use super::instance::Instance;
use super::{VulkanError, VulkanResult};

pub struct Surface {
    pub(crate) raw: vk::SurfaceKHR,
    loader: ash::khr::surface::Instance,
}

pub struct SurfaceBuilder<'a> {
    window: &'a winit::window::Window,
    app: &'a App,
    instance: &'a Instance,
}

impl<'a> SurfaceBuilder<'a> {
    pub fn new(app: &'a App, instance: &'a Instance, window: &'a winit::window::Window) -> Self {
        Self {
            window,
            app,
            instance,
        }
    }

    pub fn build(self) -> VulkanResult<Surface> {
        profile_scope!("Surface");

        let surface = unsafe {
            ash_window::create_surface(
                &self.app.entry,
                &self.instance.raw,
                self.window.display_handle().unwrap().into(),
                self.window.window_handle().unwrap().into(),
                None,
            )
            .map_err(VulkanError::Unknown)?
        };

        let loader = ash::khr::surface::Instance::new(&self.app.entry, &self.instance.raw);

        Ok(Surface {
            raw: surface,
            loader,
        })
    }
}

impl Surface {
    pub fn destroy(&self) {
        unsafe { self.loader.destroy_surface(self.raw, None) };
    }

    pub fn get_physical_device_surface_capabilities(
        &self,
        phys_dev: vk::PhysicalDevice,
    ) -> vk::SurfaceCapabilitiesKHR {
        unsafe {
            self.loader
                .get_physical_device_surface_capabilities(phys_dev, self.raw)
                .unwrap()
        }
    }

    #[allow(dead_code)]
    pub fn get_physical_device_surface_formats(
        &self,
        phys_dev: vk::PhysicalDevice,
    ) -> Vec<vk::SurfaceFormatKHR> {
        unsafe {
            self.loader
                .get_physical_device_surface_formats(phys_dev, self.raw)
                .unwrap()
        }
    }
}
