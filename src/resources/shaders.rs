use anyhow::anyhow;

use crate::{
    error, registries::util, registry::Registry, runtimes, screen, states, ui,
    util::identifier::Identifier,
};

#[derive(Clone)]
pub struct Shader {
    identifier: Identifier,
    source: &'static str,
}

impl Shader {
    pub fn new(identifier: Identifier, source: &'static str) -> Self {
        Self { identifier, source }
    }
}

async fn load_list(shaders: &[Shader]) -> anyhow::Result<Registry<wgpu::ShaderModule>> {
    util::build_registry(shaders.into_iter(), |info| async move {
        let info = info.clone();

        let label = format!("Shader at '{}'", info.identifier);
        let (module, future) = runtimes::compute::exec(move || {
            let error_future =
                states::main_dev::get().push_error_scope(wgpu::ErrorFilter::Validation);
            let module =
                states::main_dev::get().create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some(&label),
                    source: wgpu::ShaderSource::Wgsl(info.source.into()),
                });
            (module, error_future.pop())
        })
        .await;

        match future.await {
            None => Ok((info.identifier, module)),
            Some(e) => Err((info.identifier, e)),
        }
    })
    .await
    .inspect_err(|failures| {
        error!("Errors while loading these shaders");

        for (i, (identifier, error)) in failures.iter().enumerate() {
            error!("{i}: {identifier}: {error:#}");
        }
    })
    .map_err(|_| anyhow!("error loading shaders, check log"))
}

#[rustfmt::skip]
pub async fn load() -> anyhow::Result<Registry<wgpu::ShaderModule>> {
    load_list(&[Shader::new(
        ui::primitives::ColoredRectangle::SHADER_ID,
        include_str!("ui/colored_rectangle_shader.wgsl"),
    )])
    .await
}

#[rustfmt::skip]
pub async fn early_load() -> anyhow::Result<Registry<wgpu::ShaderModule>> {
    load_list(&[Shader::new(
        screen::LoadingScreen::ICON_SHADER_ID,
        include_str!("early/loading_screen.wgsl"),
    )])
    .await
}
