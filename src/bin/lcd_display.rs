//! GameBoy LCD display using wgpu (Metal on macOS) + Dear ImGui

use std::env;
use std::fs;
use std::sync::Arc;
use std::time::{Duration, Instant};

use imgui::*;
use imgui_wgpu::{Renderer, RendererConfig, Texture, TextureConfig};
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use pollster::block_on;
use wgpu::Extent3d;
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    window::Window,
};

use gb_emu::{EmulatorFlags, System};
use gb_emu::display::{SharedFrameBuffer, SCREEN_HEIGHT, SCREEN_WIDTH};

const SCALE: u32 = 3;
const INFO_PANEL_WIDTH: f32 = 220.0;
const GB_W: f32 = SCREEN_WIDTH as f32 * SCALE as f32;
const GB_H: f32 = SCREEN_HEIGHT as f32 * SCALE as f32;
const TITLE_BAR_H: f32 = 19.0; // imgui default title bar height

struct EmulatorState {
    system: System,
    frame_buffer: SharedFrameBuffer,
    fps_counter: u32,
    fps_timer: Instant,
    current_fps: f32,
    total_frames: u64,
}

struct ImguiState {
    context: imgui::Context,
    platform: WinitPlatform,
    renderer: Renderer,
    gb_texture_id: TextureId,
    last_frame: Instant,
    last_cursor: Option<MouseCursor>,
}

struct AppWindow {
    device: wgpu::Device,
    queue: wgpu::Queue,
    window: Arc<Window>,
    surface_desc: wgpu::SurfaceConfiguration,
    surface: wgpu::Surface<'static>,
    hidpi_factor: f64,
    imgui: Option<ImguiState>,
    emu: EmulatorState,
}

struct App {
    rom_data: Vec<u8>,
    window_app: Option<AppWindow>,
}

impl App {
    fn new(rom_data: Vec<u8>) -> Self {
        Self { rom_data, window_app: None }
    }
}

impl AppWindow {
    fn init(event_loop: &ActiveEventLoop, rom_data: Vec<u8>) -> Self {
        // ── wgpu setup ──────────────────────────────────────────────────────
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY, // Metal on macOS
            ..Default::default()
        });

        let win_w = GB_W as u32 + INFO_PANEL_WIDTH as u32 + 16;
        let win_h = GB_H as u32 + TITLE_BAR_H as u32 + 8;

        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("GameBoy Emulator")
                        .with_inner_size(LogicalSize::new(win_w, win_h))
                        .with_resizable(false),
                )
                .unwrap(),
        );

        let size = window.inner_size();
        let hidpi_factor = window.scale_factor();
        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .expect("No suitable GPU adapter found");

        let (device, queue) =
            block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
                .expect("Failed to create GPU device");

        let surface_desc = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo, // vsync ~60fps
            desired_maximum_frame_latency: 2,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![wgpu::TextureFormat::Bgra8Unorm],
        };
        surface.configure(&device, &surface_desc);

        // ── emulator setup ──────────────────────────────────────────────────
        let mut system = System::new(rom_data, EmulatorFlags::default());
        let frame_buffer = system.get_frame_buffer();
        system.start();

        let emu = EmulatorState {
            system,
            frame_buffer,
            fps_counter: 0,
            fps_timer: Instant::now(),
            current_fps: 0.0,
            total_frames: 0,
        };

        let mut app = Self {
            device,
            queue,
            window,
            surface_desc,
            surface,
            hidpi_factor,
            imgui: None,
            emu,
        };
        app.setup_imgui();
        app
    }

    fn setup_imgui(&mut self) {
        let mut context = imgui::Context::create();
        context.set_ini_filename(None);

        let mut platform = WinitPlatform::new(&mut context);
        platform.attach_window(context.io_mut(), &self.window, HiDpiMode::Default);

        // Scale fonts for HiDPI
        let font_size = (13.0 * self.hidpi_factor) as f32;
        context.io_mut().font_global_scale = (1.0 / self.hidpi_factor) as f32;
        context.fonts().add_font(&[FontSource::DefaultFontData {
            config: Some(FontConfig {
                oversample_h: 1,
                pixel_snap_h: true,
                size_pixels: font_size,
                ..Default::default()
            }),
        }]);

        let renderer_config = RendererConfig {
            texture_format: self.surface_desc.format,
            ..Default::default()
        };
        let mut renderer =
            Renderer::new(&mut context, &self.device, &self.queue, renderer_config);

        // ── create GameBoy screen texture (160×144 RGBA8) ───────────────────
        let texture_config = TextureConfig {
            size: Extent3d {
                width: SCREEN_WIDTH as u32,
                height: SCREEN_HEIGHT as u32,
                depth_or_array_layers: 1,
            },
            label: Some("gb_screen"),
            format: Some(wgpu::TextureFormat::Rgba8Unorm),
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            ..Default::default()
        };
        let gb_texture = Texture::new(&self.device, &renderer, texture_config);
        // Fill with white (GB startup)
        let white = vec![0xFFu8; SCREEN_WIDTH * SCREEN_HEIGHT * 4];
        gb_texture.write(&self.queue, &white, SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32);
        let gb_texture_id = renderer.textures.insert(gb_texture);

        self.imgui = Some(ImguiState {
            context,
            platform,
            renderer,
            gb_texture_id,
            last_frame: Instant::now(),
            last_cursor: None,
        });
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.window_app = Some(AppWindow::init(event_loop, self.rom_data.clone()));
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let app = self.window_app.as_mut().unwrap();
        let imgui = app.imgui.as_mut().unwrap();

        match &event {
            WindowEvent::Resized(size) => {
                app.surface_desc.width = size.width;
                app.surface_desc.height = size.height;
                app.surface.configure(&app.device, &app.surface_desc);
            }
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput { event: key_event, .. } => {
                if let Key::Named(NamedKey::Escape) = key_event.logical_key {
                    if key_event.state.is_pressed() {
                        event_loop.exit();
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                // ── step emulator one full frame ─────────────────────────────
                if app.emu.system.is_running() {
                    app.emu.system.run_frame();
                } else {
                    event_loop.exit();
                    return;
                }

                // ── update GB screen texture when a frame is ready ───────────
                if app.emu.system.frame_complete {
                    let fb = app.emu.frame_buffer.lock().unwrap();
                    if fb.frame_ready {
                        // Convert 0xAARRGGBB u32 pixels → RGBA u8 bytes
                        let pixels: Vec<u8> = fb
                            .pixels
                            .iter()
                            .flat_map(|&p| {
                                let r = ((p >> 16) & 0xFF) as u8;
                                let g = ((p >> 8) & 0xFF) as u8;
                                let b = (p & 0xFF) as u8;
                                let a = ((p >> 24) & 0xFF) as u8;
                                [r, g, b, a]
                            })
                            .collect();

                        let gb_tex = imgui
                            .renderer
                            .textures
                            .get(imgui.gb_texture_id)
                            .unwrap();
                        gb_tex.write(
                            &app.queue,
                            &pixels,
                            SCREEN_WIDTH as u32,
                            SCREEN_HEIGHT as u32,
                        );

                        app.emu.fps_counter += 1;
                        app.emu.total_frames += 1;
                    }
                    drop(fb);

                    app.emu.frame_buffer.lock().unwrap().clear();
                    app.emu.system.frame_complete = false;
                }

                // FPS counter (updated every second)
                if app.emu.fps_timer.elapsed() >= Duration::from_secs(1) {
                    app.emu.current_fps = app.emu.fps_counter as f32
                        / app.emu.fps_timer.elapsed().as_secs_f32();
                    app.emu.fps_counter = 0;
                    app.emu.fps_timer = Instant::now();
                }

                // ── ImGui frame ──────────────────────────────────────────────
                let now = Instant::now();
                imgui.context.io_mut().update_delta_time(now - imgui.last_frame);
                imgui.last_frame = now;

                let frame = match app.surface.get_current_texture() {
                    Ok(f) => f,
                    Err(e) => {
                        eprintln!("surface error: {e:?}");
                        return;
                    }
                };

                imgui
                    .platform
                    .prepare_frame(imgui.context.io_mut(), &app.window)
                    .expect("Failed to prepare ImGui frame");
                let ui = imgui.context.frame();

                // ── GameBoy screen window ─────────────────────────────────────
                ui.window("GameBoy")
                    .size([GB_W + 16.0, GB_H + TITLE_BAR_H + 4.0], Condition::Always)
                    .position([0.0, 0.0], Condition::Always)
                    .no_decoration()
                    .no_inputs()
                    .build(|| {
                        Image::new(imgui.gb_texture_id, [GB_W, GB_H]).build(ui);
                    });

                // ── Info / debug panel ────────────────────────────────────────
                let running = app.emu.system.is_running();
                let fps = app.emu.current_fps;
                let frames = app.emu.total_frames;

                ui.window("Info")
                    .size([INFO_PANEL_WIDTH, GB_H + TITLE_BAR_H + 4.0], Condition::Always)
                    .position([GB_W + 16.0, 0.0], Condition::Always)
                    .build(|| {
                        ui.text("GameBoy DMG");
                        ui.separator();

                        ui.text(format!("FPS:    {fps:.1}"));
                        ui.text(format!("Frames: {frames}"));
                        ui.separator();

                        ui.text(format!("Screen: {SCREEN_WIDTH}x{SCREEN_HEIGHT}"));
                        ui.text(format!("Scale:  {SCALE}x → {GB_W:.0}x{GB_H:.0}"));
                        ui.separator();

                        if running {
                            ui.text_colored([0.2, 1.0, 0.2, 1.0], "Running");
                        } else {
                            ui.text_colored([1.0, 0.3, 0.3, 1.0], "Stopped");
                        }
                    });

                // ── render ────────────────────────────────────────────────────
                if imgui.last_cursor != ui.mouse_cursor() {
                    imgui.last_cursor = ui.mouse_cursor();
                    imgui.platform.prepare_render(ui, &app.window);
                }

                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                let mut encoder = app
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

                {
                    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 0.05,
                                    g: 0.05,
                                    b: 0.05,
                                    a: 1.0,
                                }),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });

                    imgui
                        .renderer
                        .render(
                            imgui.context.render(),
                            &app.queue,
                            &app.device,
                            &mut rpass,
                        )
                        .expect("ImGui render failed");
                }

                app.queue.submit(Some(encoder.finish()));
                frame.present();
            }
            _ => (),
        }

        imgui.platform.handle_event::<()>(
            imgui.context.io_mut(),
            &app.window,
            &Event::WindowEvent { window_id, event },
        );
    }

    fn user_event(&mut self, _: &ActiveEventLoop, event: ()) {
        let app = self.window_app.as_mut().unwrap();
        let imgui = app.imgui.as_mut().unwrap();
        imgui.platform.handle_event::<()>(
            imgui.context.io_mut(),
            &app.window,
            &Event::UserEvent(event),
        );
    }

    fn device_event(
        &mut self,
        _: &ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        let app = self.window_app.as_mut().unwrap();
        let imgui = app.imgui.as_mut().unwrap();
        imgui.platform.handle_event::<()>(
            imgui.context.io_mut(),
            &app.window,
            &Event::DeviceEvent { device_id, event },
        );
    }

    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        let app = self.window_app.as_mut().unwrap();
        let imgui = app.imgui.as_mut().unwrap();
        app.window.request_redraw();
        imgui.platform.handle_event::<()>(
            imgui.context.io_mut(),
            &app.window,
            &Event::AboutToWait,
        );
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <rom_file>", args[0]);
        std::process::exit(1);
    }

    let rom_data = fs::read(&args[1]).unwrap_or_else(|e| {
        eprintln!("Failed to load ROM '{}': {e}", args[1]);
        std::process::exit(1);
    });

    println!("Loaded ROM: {} bytes — starting GameBoy emulator", rom_data.len());

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(&mut App::new(rom_data)).unwrap();
}
