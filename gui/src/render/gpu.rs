//! Adapter, device, queue and surface. WebGPU when the browser has a usable adapter, else WebGL2.

pub struct Gpu {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    /// Surface format: pipelines that draw on screen target it.
    pub format: wgpu::TextureFormat,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
}

impl Gpu {
    pub async fn new(
        target: impl Into<wgpu::SurfaceTarget<'static>>,
        width: u32,
        height: u32,
    ) -> Result<Gpu, String> {
        // navigator.gpu can exist without a usable adapter: then fall back to WebGL2.
        let instance = wgpu::util::new_instance_with_webgpu_detection(
            wgpu::InstanceDescriptor::new_without_display_handle(),
        )
        .await;
        let surface = instance
            .create_surface(target)
            .map_err(|e| format!("create surface: {e}"))?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
                apply_limit_buckets: false,
            })
            .await
            .map_err(|e| format!("request adapter: {e}"))?;
        log::info!("Backend: {:?}", adapter.get_info().backend);

        // WebGL2 has no compute; WebGPU does. Ask for what the adapter can give.
        let base_limits = if adapter
            .get_downlevel_capabilities()
            .flags
            .contains(wgpu::DownlevelFlags::COMPUTE_SHADERS)
        {
            wgpu::Limits::downlevel_defaults()
        } else {
            wgpu::Limits::downlevel_webgl2_defaults()
        };
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("device"),
                required_features: wgpu::Features::default(),
                // The surface may be as large as the adapter allows.
                required_limits: base_limits.using_resolution(adapter.limits()),
                experimental_features: wgpu::ExperimentalFeatures::default(),
                memory_hints: wgpu::MemoryHints::default(),
                trace: wgpu::Trace::Off,
            })
            .await
            .map_err(|e| format!("request device: {e}"))?;

        let (width, height) = surface_size(&device, width, height);
        let config = config_surface(&surface, &adapter, width, height);
        surface.configure(&device, &config);
        Ok(Self {
            device,
            queue,
            format: config.format,
            surface,
            config,
        })
    }

    /// Returns the size actually used: the GPU limits the surface size.
    pub fn resize(&mut self, width: u32, height: u32) -> (u32, u32) {
        (self.config.width, self.config.height) = surface_size(&self.device, width, height);
        self.surface.configure(&self.device, &self.config);
        self.size()
    }

    pub fn size(&self) -> (u32, u32) {
        (self.config.width, self.config.height)
    }

    /// `None`: skip this frame.
    pub fn frame(&self) -> Option<wgpu::SurfaceTexture> {
        match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame)
            | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => Some(frame),
            // The next frame gets a working surface without waiting for a resize.
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&self.device, &self.config);
                None
            }
            // A hidden tab: normal, not worth a log line every frame.
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => None,
            other => {
                log::warn!("Skip frame: {other:?}");
                None
            }
        }
    }
}

/// Zero is not a valid surface size, and the GPU limits the maximum.
fn surface_size(device: &wgpu::Device, width: u32, height: u32) -> (u32, u32) {
    let max = device.limits().max_texture_dimension_2d;
    (width.clamp(1, max), height.clamp(1, max))
}

fn config_surface(
    surface: &wgpu::Surface,
    adapter: &wgpu::Adapter,
    width: u32,
    height: u32,
) -> wgpu::SurfaceConfiguration {
    let caps = surface.get_capabilities(adapter);
    let format = caps
        .formats
        .iter()
        .copied()
        // Colors are sRGB values written as is: no conversion on store.
        .find(|f| !f.is_srgb())
        .unwrap_or(caps.formats[0]);
    wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format,
        color_space: wgpu::SurfaceColorSpace::Auto,
        width,
        height,
        present_mode: wgpu::PresentMode::AutoVsync,
        alpha_mode: caps.alpha_modes[0],
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    }
}
