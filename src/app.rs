use glam::{vec3, Vec2, Vec3};
use std::{sync::Arc, time::Instant};
use vulkano::{
    buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer},
    command_buffer::{
        allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
        RenderPassBeginInfo, SubpassBeginInfo, SubpassContents,
    },
    device::{
        physical::PhysicalDeviceType, Device, DeviceCreateInfo, DeviceExtensions, Queue,
        QueueCreateInfo, QueueFlags,
    },
    image::{view::ImageView, Image, ImageUsage},
    instance::{Instance, InstanceCreateFlags, InstanceCreateInfo},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
    pipeline::{
        graphics::{
            color_blend::{ColorBlendAttachmentState, ColorBlendState},
            input_assembly::InputAssemblyState,
            multisample::MultisampleState,
            rasterization::RasterizationState,
            vertex_input::{Vertex, VertexDefinition},
            viewport::{Viewport, ViewportState},
            GraphicsPipelineCreateInfo,
        },
        layout::PipelineDescriptorSetLayoutCreateInfo,
        DynamicState, GraphicsPipeline, Pipeline, PipelineLayout, PipelineShaderStageCreateInfo,
    },
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass},
    swapchain::{
        acquire_next_image, Surface, Swapchain, SwapchainCreateInfo, SwapchainPresentInfo,
    },
    sync::{self, GpuFuture},
    Validated, VulkanError, VulkanLibrary,
};
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, ElementState, KeyEvent, StartCause, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

use crate::camera::{Camera, CameraEvent};
use crate::shaders::fragment;
use crate::shaders::vertex;

pub struct App {
    instance: Arc<Instance>,
    device: Arc<Device>,
    queue: Arc<Queue>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    vertex_buffer: Subbuffer<[MyVertex]>,
    render_ctx: Option<RenderContext>,
    camera: Camera,
    mouse_pressed: bool,
    last_mouse_pos: Vec2,
    cam_rotation: Option<Vec2>,
    cam_up: bool,
    cam_down: bool,
    cam_left: bool,
    cam_right: bool,
    frame_time: Instant,
}

struct RenderContext {
    window: Arc<Window>,
    swapchain: Arc<Swapchain>,
    render_pass: Arc<RenderPass>,
    framebuffers: Vec<Arc<Framebuffer>>,
    pipeline: Arc<GraphicsPipeline>,
    viewport: Viewport,
    recreate_swapchain: bool,
    previous_frame_end: Option<Box<dyn GpuFuture>>,
}

impl App {
    pub fn new(event_loop: &EventLoop<()>) -> Self {
        let library = VulkanLibrary::new().unwrap();

        let required_extensions = Surface::required_extensions(event_loop).unwrap();

        let instance = Instance::new(
            library,
            InstanceCreateInfo {
                // Enable enumerating devices that use non-conformant Vulkan implementations.
                // (e.g. MoltenVK)
                flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
                enabled_extensions: required_extensions,
                ..Default::default()
            },
        )
        .unwrap();

        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::empty()
        };

        let (physical_device, queue_family_index) = instance
            .enumerate_physical_devices()
            .unwrap()
            .filter(|p| p.supported_extensions().contains(&device_extensions))
            .filter_map(|p| {
                p.queue_family_properties()
                    .iter()
                    .enumerate()
                    .position(|(i, q)| {
                        q.queue_flags.intersects(QueueFlags::GRAPHICS)
                            && p.presentation_support(i as u32, event_loop).unwrap()
                    })
                    .map(|i| (p, i as u32))
            })
            .min_by_key(|(p, _)| {
                // We assign a lower score to device types that are likely to be faster/better.
                match p.properties().device_type {
                    PhysicalDeviceType::DiscreteGpu => 0,
                    PhysicalDeviceType::IntegratedGpu => 1,
                    PhysicalDeviceType::VirtualGpu => 2,
                    PhysicalDeviceType::Cpu => 3,
                    PhysicalDeviceType::Other => 4,
                    _ => 5,
                }
            })
            .expect("no suitable physical device found");

        // Some little debug infos.
        println!(
            "Using device: {} (type: {:?})",
            physical_device.properties().device_name,
            physical_device.properties().device_type,
        );

        let (device, mut queues) = Device::new(
            physical_device,
            DeviceCreateInfo {
                enabled_extensions: device_extensions,
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }],

                ..Default::default()
            },
        )
        .unwrap();

        let queue = queues.next().unwrap();

        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));

        let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            Default::default(),
        ));

        let vertices = [
            MyVertex {
                position: [-1.0, -1.0],
            },
            MyVertex {
                position: [-1.0, 1.0],
            },
            MyVertex {
                position: [1.0, -1.0],
            },
            MyVertex {
                position: [1.0, 1.0],
            },
            MyVertex {
                position: [1.0, -1.0],
            },
            MyVertex {
                position: [-1.0, 1.0],
            },
        ];
        let vertex_buffer = Buffer::from_iter(
            memory_allocator,
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            vertices,
        )
        .unwrap();

        let rcx = None;

        let camera = Camera::new_with_pos(Vec3::new(-0.5, 3., 8.0), Vec3::new(0., -1., -5.));

        App {
            instance,
            device,
            queue,
            command_buffer_allocator,
            vertex_buffer,
            render_ctx: rcx,
            camera,
            mouse_pressed: false,
            last_mouse_pos: Vec2::ZERO,
            cam_rotation: None,
            cam_down: false,
            cam_left: false,
            cam_right: false,
            cam_up: false,
            frame_time: Instant::now(),
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );
        let surface = Surface::from_window(self.instance.clone(), window.clone()).unwrap();
        let window_size = window.inner_size();

        let (swapchain, images) = {
            let surface_capabilities = self
                .device
                .physical_device()
                .surface_capabilities(&surface, Default::default())
                .unwrap();

            let (image_format, _) = self
                .device
                .physical_device()
                .surface_formats(&surface, Default::default())
                .unwrap()[0];

            Swapchain::new(
                self.device.clone(),
                surface,
                SwapchainCreateInfo {
                    min_image_count: surface_capabilities.min_image_count.max(2),
                    image_format,
                    image_extent: window_size.into(),
                    image_usage: ImageUsage::COLOR_ATTACHMENT,
                    composite_alpha: surface_capabilities
                        .supported_composite_alpha
                        .into_iter()
                        .next()
                        .unwrap(),
                    present_mode: vulkano::swapchain::PresentMode::Fifo,
                    ..Default::default()
                },
            )
            .unwrap()
        };

        let render_pass = vulkano::single_pass_renderpass!(
            self.device.clone(),
            attachments: {
                color: {
                    format: swapchain.image_format(),
                    samples: 1,
                    load_op: Clear,
                    store_op: Store,
                },
            },
            pass: {
                color: [color],
                depth_stencil: {},
            },
        )
        .unwrap();

        let framebuffers = window_size_dependent_setup(&images, &render_pass);
        let pipeline = {
            let vs = vertex::load(self.device.clone())
                .unwrap()
                .entry_point("main")
                .unwrap();
            let fs = fragment::load(self.device.clone())
                .unwrap()
                .entry_point("main")
                .unwrap();

            let vertex_input_state = MyVertex::per_vertex().definition(&vs).unwrap();

            let stages = [
                PipelineShaderStageCreateInfo::new(vs),
                PipelineShaderStageCreateInfo::new(fs),
            ];

            let layout = PipelineLayout::new(
                self.device.clone(),
                PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
                    .into_pipeline_layout_create_info(self.device.clone())
                    .unwrap(),
            )
            .unwrap();

            let subpass = Subpass::from(render_pass.clone(), 0).unwrap();

            GraphicsPipeline::new(
                self.device.clone(),
                None,
                GraphicsPipelineCreateInfo {
                    stages: stages.into_iter().collect(),
                    vertex_input_state: Some(vertex_input_state),
                    input_assembly_state: Some(InputAssemblyState::default()),
                    viewport_state: Some(ViewportState::default()),
                    rasterization_state: Some(RasterizationState::default()),
                    multisample_state: Some(MultisampleState::default()),
                    color_blend_state: Some(ColorBlendState::with_attachment_states(
                        subpass.num_color_attachments(),
                        ColorBlendAttachmentState::default(),
                    )),
                    dynamic_state: [DynamicState::Viewport].into_iter().collect(),
                    subpass: Some(subpass.into()),
                    ..GraphicsPipelineCreateInfo::layout(layout)
                },
            )
            .unwrap()
        };

        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: window_size.into(),
            depth_range: 0.0..=1.0,
        };

        let recreate_swapchain = false;

        let previous_frame_end = Some(sync::now(self.device.clone()).boxed());

        self.render_ctx = Some(RenderContext {
            window,
            swapchain,
            render_pass,
            framebuffers,
            pipeline,
            viewport,
            recreate_swapchain,
            previous_frame_end,
        });
    }

    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause) {
        let elapsed = self.frame_time.elapsed();
        let ts = elapsed.as_secs_f32();

        let mut events: Vec<CameraEvent> = vec![];
        if self.cam_up {
            events.push(CameraEvent::Up)
        }
        if self.cam_down {
            events.push(CameraEvent::Down)
        }
        if self.cam_left {
            events.push(CameraEvent::Left)
        }
        if self.cam_right {
            events.push(CameraEvent::Right)
        }
        if let Some(delta) = self.cam_rotation {
            events.push(CameraEvent::RotateXY { delta })
        }
        if !events.is_empty() {
            self.camera.update(&events, ts);
        }
        self.cam_rotation = None;
        self.frame_time = Instant::now();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let rcx = self.render_ctx.as_mut().unwrap();

        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                        state: ElementState::Pressed,
                        repeat: false,
                        ..
                    },
                ..
            } => {
                event_loop.exit();
            }

            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key,
                        state,
                        repeat: false,
                        ..
                    },
                ..
            } => match physical_key {
                PhysicalKey::Code(KeyCode::KeyW) => {
                    self.cam_up = state.is_pressed();
                }
                PhysicalKey::Code(KeyCode::KeyS) => {
                    self.cam_down = state.is_pressed();
                }
                PhysicalKey::Code(KeyCode::KeyA) => {
                    self.cam_left = state.is_pressed();
                }
                PhysicalKey::Code(KeyCode::KeyD) => {
                    self.cam_right = state.is_pressed();
                }
                _ => {}
            },

            WindowEvent::CursorMoved {
                device_id,
                position,
            } => {
                let pos = Vec2::new(position.x as f32, position.y as f32);
                if self.mouse_pressed {
                    if self.last_mouse_pos == Vec2::ZERO {
                        self.last_mouse_pos = pos;
                    }
                    let delta = (pos - self.last_mouse_pos) * 0.05;
                    self.last_mouse_pos = pos;
                    if delta.x != 0.0 || delta.y != 0.0 {
                        self.cam_rotation = Some(delta);
                    }
                }
            }
            WindowEvent::MouseInput {
                device_id,
                state,
                button,
            } => {
                self.mouse_pressed = state.is_pressed();
                self.render_ctx
                    .as_mut()
                    .unwrap()
                    .window
                    .set_cursor_visible(false);
                if !self.mouse_pressed {
                    self.render_ctx
                        .as_mut()
                        .unwrap()
                        .window
                        .set_cursor_visible(true);
                    self.cam_rotation = None;
                    self.last_mouse_pos = Vec2::ZERO;
                }
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(_) => {
                rcx.recreate_swapchain = true;
            }
            WindowEvent::RedrawRequested => {
                let window_size = rcx.window.inner_size();

                if window_size.width == 0 || window_size.height == 0 {
                    return;
                }
                rcx.previous_frame_end.as_mut().unwrap().cleanup_finished();
                if rcx.recreate_swapchain {
                    let (new_swapchain, new_images) = rcx
                        .swapchain
                        .recreate(SwapchainCreateInfo {
                            image_extent: window_size.into(),
                            ..rcx.swapchain.create_info()
                        })
                        .expect("failed to recreate swapchain");

                    rcx.swapchain = new_swapchain;
                    rcx.framebuffers = window_size_dependent_setup(&new_images, &rcx.render_pass);
                    rcx.viewport.extent = window_size.into();
                    rcx.recreate_swapchain = false;
                }

                let (image_index, suboptimal, acquire_future) = match acquire_next_image(
                    rcx.swapchain.clone(),
                    None,
                )
                .map_err(Validated::unwrap)
                {
                    Ok(r) => r,
                    Err(VulkanError::OutOfDate) => {
                        rcx.recreate_swapchain = true;
                        return;
                    }
                    Err(e) => panic!("failed to acquire next image: {e}"),
                };

                if suboptimal {
                    rcx.recreate_swapchain = true;
                }

                let pc_screen = fragment::AppData {
                    screen: [(window_size.width as f32), (window_size.height as f32)].into(),
                    cam_position: self.camera.position.to_array().into(),
                    cam_uu: self.camera.uu.to_array().into(),
                    cam_vv: self.camera.vv.to_array().into(),
                    cam_ww: self.camera.ww.to_array().into(),
                    materials: [
                        fragment::Material {
                            specular: 2.9.into(),
                            shininess: 620.0.into(),
                            roughness: 0.8.into(),
                            diffuse: 0.9.into(),
                            color: vec3(0.7, 0.0, 0.0).to_array().into()
                        }.into(),
                        fragment::Material {
                            specular: 0.5.into(),
                            shininess: 80.0.into(),
                            roughness: 0.8.into(),
                            diffuse: 1.1.into(),
                            color: vec3(0.9, 0.9, 0.8).to_array().into()
                        }.into(),
                    ],
                };

                let layout = rcx.pipeline.layout().clone();

                let mut builder = AutoCommandBufferBuilder::primary(
                    self.command_buffer_allocator.clone(),
                    self.queue.queue_family_index(),
                    CommandBufferUsage::OneTimeSubmit,
                )
                .unwrap();

                builder
                    .begin_render_pass(
                        RenderPassBeginInfo {
                            clear_values: vec![Some([0.0, 0.0, 0.0, 1.0].into())],
                            ..RenderPassBeginInfo::framebuffer(
                                rcx.framebuffers[image_index as usize].clone(),
                            )
                        },
                        SubpassBeginInfo {
                            contents: SubpassContents::Inline,
                            ..Default::default()
                        },
                    )
                    .unwrap()
                    .set_viewport(0, [rcx.viewport.clone()].into_iter().collect())
                    .unwrap()
                    .push_constants(layout, 0, pc_screen)
                    .unwrap()
                    .bind_pipeline_graphics(rcx.pipeline.clone())
                    .unwrap()
                    .bind_vertex_buffers(0, self.vertex_buffer.clone())
                    .unwrap();

                // We add a draw command.
                unsafe { builder.draw(self.vertex_buffer.len() as u32, 1, 0, 0) }.unwrap();

                builder.end_render_pass(Default::default()).unwrap();

                let command_buffer = builder.build().unwrap();

                let mut sc_info =
                    SwapchainPresentInfo::swapchain_image_index(rcx.swapchain.clone(), image_index);
                //sc_info.present_mode = Some(vulkano::swapchain::PresentMode::Fifo);

                let future = rcx
                    .previous_frame_end
                    .take()
                    .unwrap()
                    .join(acquire_future)
                    .then_execute(self.queue.clone(), command_buffer)
                    .unwrap()
                    .then_swapchain_present(self.queue.clone(), sc_info)
                    .then_signal_fence_and_flush();

                match future.map_err(Validated::unwrap) {
                    Ok(future) => {
                        rcx.previous_frame_end = Some(future.boxed());
                    }
                    Err(VulkanError::OutOfDate) => {
                        rcx.recreate_swapchain = true;
                        rcx.previous_frame_end = Some(sync::now(self.device.clone()).boxed());
                    }
                    Err(e) => {
                        panic!("failed to flush future: {e}");
                        // previous_frame_end = Some(sync::now(device.clone()).boxed());
                    }
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        let rcx = self.render_ctx.as_mut().unwrap();
        rcx.window.request_redraw();
    }
}

#[derive(BufferContents, Vertex)]
#[repr(C)]
struct MyVertex {
    #[format(R32G32_SFLOAT)]
    position: [f32; 2],
}

fn window_size_dependent_setup(
    images: &[Arc<Image>],
    render_pass: &Arc<RenderPass>,
) -> Vec<Arc<Framebuffer>> {
    images
        .iter()
        .map(|image| {
            let view = ImageView::new_default(image.clone()).unwrap();

            Framebuffer::new(
                render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![view],
                    ..Default::default()
                },
            )
            .unwrap()
        })
        .collect::<Vec<_>>()
}
