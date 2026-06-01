use std::{error::Error, sync::LazyLock};

static WGPU: LazyLock<wgpu::Instance> = LazyLock::new(|| {
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
pub fn init() -> Result<(), Box<dyn Error>> {
  LazyLock::force(&WGPU);
  Ok(())
}


