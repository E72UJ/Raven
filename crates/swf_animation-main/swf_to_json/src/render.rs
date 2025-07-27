use anyhow::anyhow;

pub(crate) mod mesh;

fn create_wgpu_instance() -> anyhow::Result<(wgpu::Instance, wgpu::Backends)> {
    for backend in wgpu::Backends::all() {
        if let Some(instance) = try_wgpu_backend(backend) {
            return Ok((instance, backend));
        }
    }
    Err(anyhow!("没有找到可用渲染后端"))
}

fn try_wgpu_backend(backends: wgpu::Backends) -> Option<wgpu::Instance> {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends,
        flags: wgpu::InstanceFlags::default().with_env(),
        ..Default::default()
    });
    if instance.enumerate_adapters(backends).is_empty() {
        None
    } else {
        Some(instance)
    }
}

pub fn get_device_and_queue() -> anyhow::Result<(wgpu::Device, wgpu::Queue)> {
    let (instance, _backend) = create_wgpu_instance()?;

    let (_adapter, device, queue) = futures::executor::block_on(request_adapter_and_device(
        &instance,
        None,
        wgpu::PowerPreference::HighPerformance,
    ))
    .map_err(|e| anyhow!(e.to_string()))?;

    Ok((device, queue))
}

type Error = Box<dyn std::error::Error>;

pub async fn request_adapter_and_device(
    instance: &wgpu::Instance,
    surface: Option<&wgpu::Surface<'static>>,
    power_preference: wgpu::PowerPreference,
) -> Result<(wgpu::Adapter, wgpu::Device, wgpu::Queue), Error> {
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference,
            compatible_surface: surface,
            force_fallback_adapter: false,
        })
        .await
        .inspect_err(|e| {
            eprintln!("请求适配器失败: {:?}", e);
        })?;

    let mut features = Default::default();

    let try_features = [
        wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
        wgpu::Features::TEXTURE_COMPRESSION_BC,
        wgpu::Features::FLOAT32_FILTERABLE,
    ];

    for feature in try_features {
        if adapter.features().contains(feature) {
            features |= feature;
        }
    }

    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: Some("设备"),
            required_features: features,
            required_limits: wgpu::Limits::default(),
            memory_hints: Default::default(),
            trace: wgpu::Trace::Off,
        })
        .await?;
    Ok((adapter, device, queue))
}
