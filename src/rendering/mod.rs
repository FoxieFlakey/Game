use std::{error::Error, sync::LazyLock};

use thiserror::Error;

use crate::{util::{self, ErrorWithContext}, wgpu_async};

pub mod gpu_lookup;

pub static WGPU: LazyLock<wgpu::Instance> = LazyLock::new(|| {
    wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::PRIMARY,
        flags: wgpu::InstanceFlags::debugging(),
        memory_budget_thresholds: wgpu::MemoryBudgetThresholds {
          for_resource_creation: None,
          for_device_loss: None
        },
        backend_options: wgpu::BackendOptions::default(),
        display: None,
    })
});

// Init global stuffs about rendering
// that is not per renderer
pub async fn init() -> Result<(), ErrorWithContext<dyn Error + 'static>> {
  LazyLock::force(&WGPU);
  gpu_lookup::init().await?;
  Ok(())
}

pub struct Renderer {
	queue: wgpu_async::AsyncQueue
}

#[derive(Error, Debug)]
pub enum RendererCreateFailed {
	#[error("Cannot request device: {0}")]
	CannotRequestDevice(#[from] wgpu::RequestDeviceError)
}

impl Renderer {
	pub async fn new(gpu: &wgpu::Adapter) -> Result<Self, ErrorWithContext<RendererCreateFailed>> {
		let desc = wgpu::DeviceDescriptor {
			..Default::default()
		};
		
		let (device, queue) = gpu.request_device(&desc)
			.await
			.map_err(|e| util::add_err_context(RendererCreateFailed::from(e)))?;
		
		Ok(Self {
			queue: wgpu_async::AsyncQueue::new(device, queue)
		})
	}
}


