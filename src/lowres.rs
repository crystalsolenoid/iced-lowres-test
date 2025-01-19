use iced::advanced::renderer::{self, Quad};
use iced::{Background, Color, Pixels, Point, Rectangle, Transformation};
pub use iced_wgpu::graphics::Text;
use iced_wgpu::graphics::{self, compositor, error, Compositor};
use iced_wgpu::wgpu;

use crate::engine::Engine;
use crate::{text, triangle};

pub struct LowRes {
    default_font: iced::Font,
    default_text_size: Pixels,
    layers: crate::layer::Stack,
    triangle_storage: triangle::Storage,
    text_storage: text::Storage,
    text_viewport: text::Viewport,
    // TODO: Centralize all the image feature handling
    //#[cfg(any(feature = "svg", feature = "image"))]
    //image_cache: std::cell::RefCell<image::Cache>,
}

impl LowRes {
    pub fn new(
        device: &wgpu::Device,
        engine: &Engine,
        default_font: iced::Font,
        default_text_size: Pixels,
    ) -> Self {
        Self {
            default_font,
            default_text_size,
            layers: crate::layer::Stack::new(),
            triangle_storage: triangle::Storage::new(),
            text_storage: text::Storage::new(),
            text_viewport: engine.text_pipeline.create_viewport(device),
            //#[cfg(any(feature = "svg", feature = "image"))]
            //image_cache: std::cell::RefCell::new(engine.create_image_cache(device)),
        }
    }

    pub fn present<T: AsRef<str>>(
        &mut self,
        engine: &mut Engine,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        clear_color: Option<Color>,
        format: wgpu::TextureFormat,
        frame: &wgpu::TextureView,
        viewport: &iced_wgpu::graphics::Viewport,
        overlay: &[T],
    ) {
        self.draw_overlay(overlay, viewport);
        self.prepare(engine, device, queue, format, encoder, viewport);
        self.render(engine, encoder, frame, clear_color, viewport);
    }

    fn draw_overlay(
        &mut self,
        overlay: &[impl AsRef<str>],

        viewport: &iced_wgpu::graphics::Viewport,
    ) {
    }
    fn prepare(
        &mut self,
        engine: &mut Engine,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _format: wgpu::TextureFormat,
        encoder: &mut wgpu::CommandEncoder,
        viewport: &iced_wgpu::graphics::Viewport,
    ) {
        let scale_factor = viewport.scale_factor() as f32;

        self.text_viewport.update(queue, viewport.physical_size());

        for layer in self.layers.iter_mut() {
            if !layer.quads.is_empty() {
                engine.quad_pipeline.prepare(
                    device,
                    encoder,
                    &mut engine.staging_belt,
                    &layer.quads,
                    viewport.projection(),
                    scale_factor,
                );
            }

            if !layer.triangles.is_empty() {
                engine.triangle_pipeline.prepare(
                    device,
                    encoder,
                    &mut engine.staging_belt,
                    &mut self.triangle_storage,
                    &layer.triangles,
                    Transformation::scale(scale_factor),
                    viewport.physical_size(),
                );
            }

            if !layer.primitives.is_empty() {
                for instance in &layer.primitives {
                    instance.primitive.prepare(
                        device,
                        queue,
                        engine.format,
                        &mut engine.primitive_storage,
                        &instance.bounds,
                        viewport,
                    );
                }
            }

            #[cfg(any(feature = "svg", feature = "image"))]
            if !layer.images.is_empty() {
                engine.image_pipeline.prepare(
                    device,
                    encoder,
                    &mut engine.staging_belt,
                    &mut self.image_cache.borrow_mut(),
                    &layer.images,
                    viewport.projection(),
                    scale_factor,
                );
            }

            if !layer.text.is_empty() {
                engine.text_pipeline.prepare(
                    device,
                    queue,
                    &self.text_viewport,
                    encoder,
                    &mut self.text_storage,
                    &layer.text,
                    layer.bounds,
                    Transformation::scale(scale_factor),
                );
            }
        }
    }
    fn render(
        &mut self,
        engine: &mut Engine,
        encoder: &mut wgpu::CommandEncoder,
        frame: &wgpu::TextureView,
        clear_color: Option<Color>,
        viewport: &iced_wgpu::graphics::Viewport,
    ) {
        use std::mem::ManuallyDrop;

        let mut render_pass =
            ManuallyDrop::new(encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("iced_wgpu render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: frame,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: match clear_color {
                            Some(background_color) => wgpu::LoadOp::Clear({
                                let [r, g, b, a] =
                                    graphics::color::pack(background_color).components();

                                wgpu::Color {
                                    r: f64::from(r),
                                    g: f64::from(g),
                                    b: f64::from(b),
                                    a: f64::from(a),
                                }
                            }),
                            None => wgpu::LoadOp::Load,
                        },
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            }));

        let mut quad_layer = 0;
        let mut mesh_layer = 0;
        let mut text_layer = 0;

        #[cfg(any(feature = "svg", feature = "image"))]
        let mut image_layer = 0;
        #[cfg(any(feature = "svg", feature = "image"))]
        let image_cache = self.image_cache.borrow();

        let scale_factor = viewport.scale_factor() as f32;
        let physical_bounds =
            Rectangle::<f32>::from(Rectangle::with_size(viewport.physical_size()));

        let scale = Transformation::scale(scale_factor);

        for layer in self.layers.iter() {
            let Some(physical_bounds) = physical_bounds.intersection(&(layer.bounds * scale))
            else {
                continue;
            };

            let Some(scissor_rect) = physical_bounds.snap() else {
                continue;
            };

            if !layer.quads.is_empty() {
                engine.quad_pipeline.render(
                    quad_layer,
                    scissor_rect,
                    &layer.quads,
                    &mut render_pass,
                );

                quad_layer += 1;
            }

            if !layer.triangles.is_empty() {
                let _ = ManuallyDrop::into_inner(render_pass);

                mesh_layer += engine.triangle_pipeline.render(
                    encoder,
                    frame,
                    &self.triangle_storage,
                    mesh_layer,
                    &layer.triangles,
                    physical_bounds,
                    scale,
                );

                render_pass =
                    ManuallyDrop::new(encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("iced_wgpu render pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: frame,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    }));
            }

            if !layer.primitives.is_empty() {
                let _ = ManuallyDrop::into_inner(render_pass);

                for instance in &layer.primitives {
                    if let Some(clip_bounds) = (instance.bounds * scale)
                        .intersection(&physical_bounds)
                        .and_then(Rectangle::snap)
                    {
                        instance.primitive.render(
                            encoder,
                            &engine.primitive_storage,
                            frame,
                            &clip_bounds,
                        );
                    }
                }

                render_pass =
                    ManuallyDrop::new(encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("iced_wgpu render pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: frame,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    }));
            }

            #[cfg(any(feature = "svg", feature = "image"))]
            if !layer.images.is_empty() {
                engine.image_pipeline.render(
                    &image_cache,
                    image_layer,
                    scissor_rect,
                    &mut render_pass,
                );

                image_layer += 1;
            }

            if !layer.text.is_empty() {
                text_layer += engine.text_pipeline.render(
                    &self.text_viewport,
                    &self.text_storage,
                    text_layer,
                    &layer.text,
                    scissor_rect,
                    &mut render_pass,
                );
            }
        }

        let _ = ManuallyDrop::into_inner(render_pass);
    }
}
impl renderer::Renderer for LowRes {
    fn start_layer(&mut self, bounds: Rectangle) {
        self.layers.push_clip(bounds);
    }

    fn end_layer(&mut self) {
        self.layers.pop_clip();
    }

    fn start_transformation(&mut self, transformation: Transformation) {
        self.layers.push_transformation(transformation);
    }

    fn end_transformation(&mut self) {
        self.layers.pop_transformation();
    }

    fn fill_quad(
        &mut self,
        quad: iced_wgpu::core::renderer::Quad,
        background: impl Into<Background>,
    ) {
        let (layer, transformation) = self.layers.current_mut();
        layer.draw_quad(quad, background.into(), transformation);
    }

    fn clear(&mut self) {
        self.layers.clear();
    }
}

impl iced_wgpu::core::text::Renderer for LowRes {
    type Font = iced::Font;
    //    type Paragraph = dyn iced::advanced::text::Paragraph<Font = Self::Font>;
    type Paragraph = ();
    type Editor = ();
    //    type Editor = dyn iced::advanced::text::Editor;

    const ICON_FONT: Self::Font = iced::Font::DEFAULT;
    const CHECKMARK_ICON: char = '0';
    const ARROW_DOWN_ICON: char = '0';

    fn default_font(&self) -> Self::Font {
        self.default_font
    }
    fn default_size(&self) -> Pixels {
        self.default_text_size
    }
    fn fill_paragraph(
        &mut self,
        _text: &Self::Paragraph,
        _position: Point,
        _color: Color,
        _clip_bounds: Rectangle,
    ) {
    }
    fn fill_editor(
        &mut self,
        _editor: &Self::Editor,
        _position: Point,
        _color: Color,
        _clip_bounds: Rectangle,
    ) {
    }
    fn fill_text(
        &mut self,
        _text: iced_wgpu::core::Text<String, Self::Font>,
        _position: Point,
        _color: Color,
        _clip_bounds: Rectangle,
    ) {
    }
}

// TODO: https://github.com/iced-rs/iced/blob/e8f8216ea1f9deef7f2d02fa2600a0b4e247f8fa/src/program.rs#L630
/*
impl iced::program::Renderer for LowRes {}
*/
pub struct LowResCompositor {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    format: wgpu::TextureFormat,
    alpha_mode: wgpu::CompositeAlphaMode,
    engine: Engine,
    settings: iced_wgpu::Settings,
}

impl Compositor for LowResCompositor {
    type Renderer = LowRes;

    type Surface = iced_wgpu::wgpu::Surface<'static>;

    async fn with_backend<W: iced_wgpu::graphics::compositor::Window + Clone>(
        settings: iced_wgpu::graphics::Settings,
        compatible_window: W,
        backend: Option<&str>,
    ) -> Result<Self, iced_wgpu::graphics::Error> {
        match backend {
            None | Some("wgpu") => {
                let mut settings = iced_wgpu::Settings::from(settings);

                if let Some(backends) = iced_wgpu::wgpu::util::backend_bits_from_env() {
                    settings.backends = backends;
                }

                if let Some(present_mode) = iced_wgpu::settings::present_mode_from_env() {
                    settings.present_mode = present_mode;
                }

                Ok(new(settings, compatible_window).await?)
            }
            Some(backend) => Err(graphics::Error::GraphicsAdapterNotFound {
                backend: "wgpu",
                reason: error::Reason::DidNotMatch {
                    preferred_backend: backend.to_owned(),
                },
            }),
        }
    }

    fn create_renderer(&self) -> Self::Renderer {
        Self::Renderer::new(
            &self.device,
            &self.engine,
            self.settings.default_font,
            self.settings.default_text_size,
        )
    }

    fn create_surface<W: iced_wgpu::graphics::compositor::Window + Clone>(
        &mut self,
        window: W,
        width: u32,
        height: u32,
    ) -> Self::Surface {
        let mut surface = self
            .instance
            .create_surface(window)
            .expect("Create surface");

        if width > 0 && height > 0 {
            self.configure_surface(&mut surface, width, height);
        }

        surface
    }

    fn configure_surface(&mut self, surface: &mut Self::Surface, width: u32, height: u32) {
        surface.configure(
            &self.device,
            &wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: self.format,
                present_mode: self.settings.present_mode,
                width,
                height,
                alpha_mode: self.alpha_mode,
                view_formats: vec![],
                desired_maximum_frame_latency: 1,
            },
        );
    }

    fn fetch_information(&self) -> iced_wgpu::graphics::compositor::Information {
        todo!()
    }

    fn present<T: AsRef<str>>(
        &mut self,
        renderer: &mut Self::Renderer,
        surface: &mut Self::Surface,
        viewport: &iced_wgpu::graphics::Viewport,
        background_color: Color,
        overlay: &[T],
    ) -> Result<(), iced_wgpu::graphics::compositor::SurfaceError> {
        present(self, renderer, surface, viewport, background_color, overlay)
    }

    fn screenshot<T: AsRef<str>>(
        &mut self,
        _renderer: &mut Self::Renderer,
        _surface: &mut Self::Surface,
        _viewport: &iced_wgpu::graphics::Viewport,
        _background_color: Color,
        _overlay: &[T],
    ) -> Vec<u8> {
        todo!()
    }
}
/// Creates a [`Compositor`] with the given [`Settings`] and window.
pub async fn new<W: compositor::Window>(
    settings: iced_wgpu::Settings,
    compatible_window: W,
) -> Result<LowResCompositor, iced_wgpu::window::compositor::Error> {
    LowResCompositor::request(settings, Some(compatible_window)).await
}

impl iced::advanced::graphics::compositor::Default for LowRes {
    type Compositor = LowResCompositor;
}

impl LowResCompositor {
    /// Requests a new [`Compositor`] with the given [`Settings`].
    ///
    /// Returns `None` if no compatible graphics adapter could be found.
    pub async fn request<W: compositor::Window>(
        settings: iced_wgpu::Settings,
        compatible_window: Option<W>,
    ) -> Result<Self, iced_wgpu::window::compositor::Error> {
        let instance = iced_wgpu::wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: settings.backends,
            ..Default::default()
        });

        log::info!("{settings:#?}");

        #[cfg(not(target_arch = "wasm32"))]
        if log::max_level() >= log::LevelFilter::Info {
            let available_adapters: Vec<_> = instance
                .enumerate_adapters(settings.backends)
                .iter()
                .map(wgpu::Adapter::get_info)
                .collect();
            log::info!("Available adapters: {available_adapters:#?}");
        }

        #[allow(unsafe_code)]
        let compatible_surface =
            compatible_window.and_then(|window| instance.create_surface(window).ok());

        let adapter_options = wgpu::RequestAdapterOptions {
            power_preference: wgpu::util::power_preference_from_env().unwrap_or(
                if settings.antialiasing.is_none() {
                    wgpu::PowerPreference::LowPower
                } else {
                    wgpu::PowerPreference::HighPerformance
                },
            ),
            compatible_surface: compatible_surface.as_ref(),
            force_fallback_adapter: false,
        };

        let adapter = instance.request_adapter(&adapter_options).await.ok_or(
            iced_wgpu::window::compositor::Error::NoAdapterFound(format!("{:?}", adapter_options)),
        )?;

        log::info!("Selected: {:#?}", adapter.get_info());

        let (format, alpha_mode) = compatible_surface
            .as_ref()
            .and_then(|surface| {
                let capabilities = surface.get_capabilities(&adapter);

                let mut formats = capabilities.formats.iter().copied();

                log::info!("Available formats: {formats:#?}");

                let format = if graphics::color::GAMMA_CORRECTION {
                    formats.find(wgpu::TextureFormat::is_srgb)
                } else {
                    formats.find(|format| !wgpu::TextureFormat::is_srgb(format))
                };

                let format = format.or_else(|| {
                    log::warn!("No format found!");

                    capabilities.formats.first().copied()
                });

                let alpha_modes = capabilities.alpha_modes;

                log::info!("Available alpha modes: {alpha_modes:#?}");

                let preferred_alpha =
                    if alpha_modes.contains(&wgpu::CompositeAlphaMode::PostMultiplied) {
                        wgpu::CompositeAlphaMode::PostMultiplied
                    } else if alpha_modes.contains(&wgpu::CompositeAlphaMode::PreMultiplied) {
                        wgpu::CompositeAlphaMode::PreMultiplied
                    } else {
                        wgpu::CompositeAlphaMode::Auto
                    };

                format.zip(Some(preferred_alpha))
            })
            .ok_or(iced_wgpu::window::compositor::Error::IncompatibleSurface)?;

        log::info!("Selected format: {format:?} with alpha mode: {alpha_mode:?}");

        #[cfg(target_arch = "wasm32")]
        let limits = [wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits())];

        #[cfg(not(target_arch = "wasm32"))]
        let limits = [wgpu::Limits::default(), wgpu::Limits::downlevel_defaults()];

        let limits = limits.into_iter().map(|limits| wgpu::Limits {
            max_bind_groups: 2,
            ..limits
        });

        let mut errors = Vec::new();

        for required_limits in limits {
            let result = adapter
                .request_device(
                    &wgpu::DeviceDescriptor {
                        label: Some("iced_wgpu::window::compositor device descriptor"),
                        required_features: wgpu::Features::empty(),
                        required_limits: required_limits.clone(),
                    },
                    None,
                )
                .await;

            match result {
                Ok((device, queue)) => {
                    let engine =
                        Engine::new(&adapter, &device, &queue, format, settings.antialiasing);

                    return Ok(LowResCompositor {
                        instance,
                        adapter,
                        device,
                        queue,
                        format,
                        alpha_mode,
                        engine,
                        settings,
                    });
                }
                Err(error) => {
                    errors.push((required_limits, error));
                }
            }
        }

        Err(iced_wgpu::window::compositor::Error::RequestDeviceFailed(
            errors,
        ))
    }
}

/// Presents the given primitives with the given [`Compositor`].
pub fn present<T: AsRef<str>>(
    compositor: &mut LowResCompositor,
    renderer: &mut LowRes,
    surface: &mut wgpu::Surface<'static>,
    viewport: &iced_wgpu::graphics::Viewport,
    background_color: Color,
    overlay: &[T],
) -> Result<(), compositor::SurfaceError> {
    match surface.get_current_texture() {
        Ok(frame) => {
            let mut encoder =
                compositor
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("iced_wgpu encoder"),
                    });

            let view = &frame
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            renderer.present(
                &mut compositor.engine,
                &compositor.device,
                &compositor.queue,
                &mut encoder,
                Some(background_color),
                frame.texture.format(),
                view,
                viewport,
                overlay,
            );

            let _ = compositor.engine.submit(&compositor.queue, encoder);

            // Present the frame
            frame.present();

            Ok(())
        }
        Err(error) => match error {
            wgpu::SurfaceError::Timeout => Err(compositor::SurfaceError::Timeout),
            wgpu::SurfaceError::Outdated => Err(compositor::SurfaceError::Outdated),
            wgpu::SurfaceError::Lost => Err(compositor::SurfaceError::Lost),
            wgpu::SurfaceError::OutOfMemory => Err(compositor::SurfaceError::OutOfMemory),
        },
    }
}
