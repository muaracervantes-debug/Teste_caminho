use std::{borrow::Cow, error::Error, sync::Arc};

use glam::{Mat4, Vec3};
use wgpu::util::DeviceExt;
use winit::{dpi::PhysicalSize, window::Window};

use crate::diagnostics;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

impl Vertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];

    fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_projection: [[f32; 4]; 4],
}

pub struct Renderer {
    _instance: wgpu::Instance,
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    depth_view: wgpu::TextureView,
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> Result<Self, Box<dyn Error>> {
        diagnostics::record("Renderer: iniciando criacao da instancia wgpu.");
        let size = window.inner_size();
        let width = size.width.max(1);
        let height = size.height.max(1);

        // No Windows, priorizamos DX12. O log do usuario mostrou que o
        // backend padrao escolheu Vulkan na Intel UHD e a execucao parou
        // imediatamente durante request_device().
        let mut instance_descriptor =
            wgpu::InstanceDescriptor::new_without_display_handle();

        #[cfg(target_os = "windows")]
        {
            instance_descriptor.backends = wgpu::Backends::DX12;
            diagnostics::record("Renderer: backend grafico restrito a DirectX 12 no Windows.");
        }

        #[cfg(not(target_os = "windows"))]
        {
            diagnostics::record("Renderer: usando os backends graficos padrao da plataforma.");
        }

        let instance = wgpu::Instance::new(instance_descriptor);
        let surface = instance.create_surface(window.clone())?;
        diagnostics::record("Renderer: surface da janela criada.");

        // Alguns computadores possuem mais de um adaptador ou nao conseguem
        // disponibilizar o adaptador de alto desempenho para esta janela.
        // Tentamos primeiro a GPU principal e depois um adaptador compativel
        // de baixo consumo/fallback, evitando encerrar o programa na inicializacao.
        let adapter_options = |power_preference, force_fallback_adapter| {
            wgpu::RequestAdapterOptions {
                power_preference,
                compatible_surface: Some(&surface),
                force_fallback_adapter,
                apply_limit_buckets: false,
            }
        };

        let adapter = match instance
            .request_adapter(&adapter_options(wgpu::PowerPreference::HighPerformance, false))
            .await
        {
            Ok(adapter) => adapter,
            Err(primary_error) => {
                eprintln!(
                    "Adaptador de alto desempenho indisponivel ({primary_error}). Tentando alternativa..."
                );

                match instance
                    .request_adapter(&adapter_options(wgpu::PowerPreference::LowPower, false))
                    .await
                {
                    Ok(adapter) => adapter,
                    Err(secondary_error) => {
                        eprintln!(
                            "Adaptador de baixo consumo indisponivel ({secondary_error}). Tentando fallback..."
                        );

                        instance
                            .request_adapter(&adapter_options(
                                wgpu::PowerPreference::LowPower,
                                true,
                            ))
                            .await
                            .map_err(|fallback_error| {
                                format!(
                                    "Nenhum adaptador grafico compativel foi encontrado.                                      Alto desempenho: {primary_error}.                                      Baixo consumo: {secondary_error}.                                      Fallback: {fallback_error}."
                                )
                            })?
                    }
                }
            }
        };

        let adapter_info = adapter.get_info();
        diagnostics::record(&format!(
            "Renderer: adaptador selecionado: {}; tipo: {:?}; backend: {:?}; driver: {} ({})",
            adapter_info.name,
            adapter_info.device_type,
            adapter_info.backend,
            adapter_info.driver,
            adapter_info.driver_info
        ));

        diagnostics::record("Renderer: solicitando criacao do dispositivo GPU.");
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("main-device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
                ..Default::default()
            })
            .await?;
        diagnostics::record("Renderer: dispositivo e fila GPU criados.");

        let mut config = surface
            .get_default_config(&adapter, width, height)
            .ok_or("A GPU adapter nao oferece uma configuracao compativel com a janela")?;

        let capabilities = surface.get_capabilities(&adapter);
        if let Some(format) = capabilities.formats.iter().copied().find(|format| format.is_srgb()) {
            config.format = format;
        }

        diagnostics::record(&format!(
            "Renderer: configurando surface {}x{} no formato {:?}.",
            config.width, config.height, config.format
        ));
        surface.configure(&device, &config);
        diagnostics::record("Renderer: surface configurada.");

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("basic-3d-shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shaders/basic.wgsl"))),
        });

        let camera_uniform = CameraUniform {
            view_projection: camera_matrix(width, height).to_cols_array_2d(),
        };

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("camera-uniform-buffer"),
            contents: bytemuck::bytes_of(&camera_uniform),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("camera-bind-group-layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera-bind-group"),
            layout: &camera_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("main-pipeline-layout"),
            bind_group_layouts: &[Some(&camera_layout)],
            immediate_size: 0,
        });

        diagnostics::record("Renderer: criando pipeline de renderizacao.");
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("main-render-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[Some(Vertex::layout())],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let vertices = cube_vertices();
        let indices = cube_indices();

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("test-cube-vertices"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("test-cube-indices"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let depth_view = create_depth_view(&device, width, height);
        diagnostics::record("Renderer: buffers, pipeline e depth texture criados.");

        diagnostics::record("Renderer: inicializacao concluida.");

        Ok(Self {
            _instance: instance,
            window,
            surface,
            device,
            queue,
            config,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
            camera_buffer,
            camera_bind_group,
            depth_view,
        })
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.width == 0 || size.height == 0 {
            return;
        }

        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
        self.depth_view = create_depth_view(&self.device, size.width, size.height);

        let uniform = CameraUniform {
            view_projection: camera_matrix(size.width, size.height).to_cols_array_2d(),
        };

        self.queue
            .write_buffer(&self.camera_buffer, 0, bytemuck::bytes_of(&uniform));
    }

    pub fn render(&mut self) {
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame)
            | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
            _ => {
                self.surface.configure(&self.device, &self.config);
                return;
            }
        };

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("main-command-encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("main-render-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.055,
                            g: 0.075,
                            b: 0.10,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.index_count, 0, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        self.window.pre_present_notify();
        self.queue.present(frame);
    }
}

fn camera_matrix(width: u32, height: u32) -> Mat4 {
    let aspect = width as f32 / height.max(1) as f32;
    let projection = Mat4::perspective_rh(45.0_f32.to_radians(), aspect, 0.1, 100.0);
    let view = Mat4::look_at_rh(
        Vec3::new(4.0, 3.0, 5.0),
        Vec3::ZERO,
        Vec3::Y,
    );

    projection * view
}

fn create_depth_view(
    device: &wgpu::Device,
    width: u32,
    height: u32,
) -> wgpu::TextureView {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("depth-texture"),
        size: wgpu::Extent3d {
            width: width.max(1),
            height: height.max(1),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });

    texture.create_view(&wgpu::TextureViewDescriptor::default())
}

fn cube_vertices() -> [Vertex; 24] {
    let red = [0.90, 0.28, 0.22];
    let green = [0.28, 0.78, 0.40];
    let blue = [0.24, 0.48, 0.92];
    let yellow = [0.92, 0.76, 0.24];
    let cyan = [0.22, 0.76, 0.82];
    let purple = [0.68, 0.36, 0.86];

    [
        Vertex { position: [-1.0, -1.0,  1.0], color: red },
        Vertex { position: [ 1.0, -1.0,  1.0], color: red },
        Vertex { position: [ 1.0,  1.0,  1.0], color: red },
        Vertex { position: [-1.0,  1.0,  1.0], color: red },

        Vertex { position: [ 1.0, -1.0, -1.0], color: green },
        Vertex { position: [-1.0, -1.0, -1.0], color: green },
        Vertex { position: [-1.0,  1.0, -1.0], color: green },
        Vertex { position: [ 1.0,  1.0, -1.0], color: green },

        Vertex { position: [-1.0, -1.0, -1.0], color: blue },
        Vertex { position: [-1.0, -1.0,  1.0], color: blue },
        Vertex { position: [-1.0,  1.0,  1.0], color: blue },
        Vertex { position: [-1.0,  1.0, -1.0], color: blue },

        Vertex { position: [ 1.0, -1.0,  1.0], color: yellow },
        Vertex { position: [ 1.0, -1.0, -1.0], color: yellow },
        Vertex { position: [ 1.0,  1.0, -1.0], color: yellow },
        Vertex { position: [ 1.0,  1.0,  1.0], color: yellow },

        Vertex { position: [-1.0,  1.0,  1.0], color: cyan },
        Vertex { position: [ 1.0,  1.0,  1.0], color: cyan },
        Vertex { position: [ 1.0,  1.0, -1.0], color: cyan },
        Vertex { position: [-1.0,  1.0, -1.0], color: cyan },

        Vertex { position: [-1.0, -1.0, -1.0], color: purple },
        Vertex { position: [ 1.0, -1.0, -1.0], color: purple },
        Vertex { position: [ 1.0, -1.0,  1.0], color: purple },
        Vertex { position: [-1.0, -1.0,  1.0], color: purple },
    ]
}

fn cube_indices() -> [u16; 36] {
    [
        0, 1, 2, 0, 2, 3,
        4, 5, 6, 4, 6, 7,
        8, 9, 10, 8, 10, 11,
        12, 13, 14, 12, 14, 15,
        16, 17, 18, 16, 18, 19,
        20, 21, 22, 20, 22, 23,
    ]
}
