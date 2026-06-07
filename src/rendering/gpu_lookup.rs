// This module responsible for looking up GPU suitable for
// doing rendering in Wgpu words its "adapters"

use std::{error::Error, sync::OnceLock};

use crate::{
    info,
    util::error::{CustomError, CustomErrorExt},
};

static GPU_LIST: OnceLock<Vec<wgpu::Adapter>> = OnceLock::new();

#[derive(thiserror::Error, Debug, Clone)]
pub enum GPULookupError {
    #[error("There is no GPU on this system")]
    ThereIsNoGPU,
    #[error("List of GPUs not initialized yet")]
    GPUListNotInitialized,
}

pub async fn init() -> Result<(), Box<CustomError<dyn Error + 'static>>> {
    get_gpus().await;
    Ok(())
}

pub async fn get_gpus() -> &'static Vec<wgpu::Adapter> {
    if GPU_LIST.get().is_none() {
        let adapters = super::WGPU
            .enumerate_adapters(wgpu::Backends::PRIMARY | wgpu::Backends::SECONDARY)
            .await;

        if GPU_LIST.set(adapters).is_ok() {
            // We succesfully saved our result
            // print it. Other calls don't print
            // because its suppose to only looked up once
            info!("GPUs found:");
            for (idx, adapter) in GPU_LIST.get().unwrap().iter().enumerate() {
                let info = adapter.get_info();

                info!("{idx} {}:", info.name);
                info!("  Driver: {}", info.driver);
                info!("  Driver info: {}", info.driver_info);
                info!("  PCI bus ID: {}", info.device_pci_bus_id);
                info!("  Backend: {}", info.backend);
                let dev_type = match info.device_type {
                    wgpu::DeviceType::Cpu => "CPU/Software",
                    wgpu::DeviceType::DiscreteGpu => "Dedicated GPU",
                    wgpu::DeviceType::IntegratedGpu => "Integrated GPU",
                    wgpu::DeviceType::VirtualGpu => "Virtualized GPU",
                    wgpu::DeviceType::Other => "Unknown",
                };
                info!("  Type: {}", dev_type);
            }
        }
    }

    GPU_LIST.get().unwrap()
}

pub fn find_gpu(
    compatible_with: &wgpu::Surface<'_>,
) -> Result<&'static wgpu::Adapter, CustomError<GPULookupError>> {
    Ok(GPU_LIST
        .get()
        .ok_or(GPULookupError::GPUListNotInitialized)
        .map_err(|x| x.into_custom_err())?
        .iter()
        .filter(|x| x.is_surface_supported(compatible_with))
        .next()
        .ok_or(GPULookupError::ThereIsNoGPU)
        .map_err(|x| x.into_custom_err())?)
}
