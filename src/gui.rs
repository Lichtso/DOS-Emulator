use glutin::event::{Event, WindowEvent};

pub mod gl {
    include!("gl_bindings.rs");
}

const VERTEX_SHADER: &'static [u8] = b"
#version 330
precision mediump float;
layout(location = 0) in vec2 position;
out vec2 texcoord;
void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    texcoord = position.xy*vec2(0.5, -0.5)+vec2(0.5, 0.5);
}
\0";

const FRAGMENT_SHADER: &'static [u8] = b"
#version 330
precision mediump float;
uniform sampler1D palette;
uniform sampler2D vram;
in vec2 texcoord;
out vec4 color;
void main() {
    vec2 position = vec2(textureSize(vram, 0))*texcoord;
    uint offset = 7U-uint(position.x*8.0)&7U;
    uvec4 planes = uvec4(texelFetch(vram, ivec2(position), 0)*255.0);
    uvec4 nibble = (planes>>offset)&uvec4(1U);
    uint color_index = (nibble.a<<3U)|(nibble.b<<2U)|(nibble.g<<1U)|(nibble.r<<0U);
    color.rgb = texelFetch(palette, int(color_index), 0).rgb;
    color.a = 1.0;
}
\0";

static VERTICES: [f32; 8] = [
    -1.0, -1.0, 1.0, -1.0, 1.0, 1.0, -1.0, 1.0
];

unsafe fn texture_setup(gl: &gl::Gl, target: gl::types::GLenum, texture: gl::types::GLuint) {
    gl.BindTexture(target, texture);
    gl.TexParameteri(target, gl::TEXTURE_BASE_LEVEL, 0);
    gl.TexParameteri(target, gl::TEXTURE_MAX_LEVEL, 0);
    gl.TexParameteri(target, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
    gl.TexParameteri(target, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
    gl.TexParameteri(target, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
    gl.TexParameteri(target, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
}



pub enum InputEvent {
    Termination,
    Key(u8, bool),
    MouseButton(u8, bool),
    MouseMove(i32, i32),
    Focus(bool)
}

const SCALE_WIDTH: f32 = 1.0;
const SCALE_HEIGHT: f32 = 1.37;

pub fn run_loop(sender: std::sync::mpsc::Sender<InputEvent>, bus_ptr: usize) {
    let event_loop = glutin::event_loop::EventLoop::new();
    let window_builder = glutin::window::WindowBuilder::new().with_visible(false).with_resizable(false);
    let windowed_context = glutin::ContextBuilder::new().build_windowed(window_builder, &event_loop).unwrap();
    let windowed_context = unsafe { windowed_context.make_current().unwrap() };
    let gl = gl::Gl::load_with(|ptr| windowed_context.get_proc_address(ptr) as *const _);
    let texture_handles: [u32; 2] = [0, 0];
    unsafe {
        let gl_version = std::ffi::CStr::from_ptr(gl.GetString(gl::VERSION) as *const _).to_str().unwrap();
        println!("GUI: OpenGL version {}", gl_version);
        let mut vertex_buffer = std::mem::zeroed();
        gl.GenBuffers(1, &mut vertex_buffer);
        gl.BindBuffer(gl::ARRAY_BUFFER, vertex_buffer);
        gl.BufferData(gl::ARRAY_BUFFER, (VERTICES.len()*std::mem::size_of::<f32>()) as gl::types::GLsizeiptr, VERTICES.as_ptr() as *const _, gl::STATIC_DRAW);
        let mut vao = std::mem::zeroed();
        gl.GenVertexArrays(1, &mut vao);
        gl.BindVertexArray(vao);
        let vertex_shader = gl.CreateShader(gl::VERTEX_SHADER);
        gl.ShaderSource(vertex_shader, 1, [VERTEX_SHADER.as_ptr() as *const _].as_ptr(), std::ptr::null());
        gl.CompileShader(vertex_shader);
        let fragment_shader = gl.CreateShader(gl::FRAGMENT_SHADER);
        gl.ShaderSource(fragment_shader, 1, [FRAGMENT_SHADER.as_ptr() as *const _].as_ptr(), std::ptr::null());
        gl.CompileShader(fragment_shader);
        let program = gl.CreateProgram();
        gl.AttachShader(program, vertex_shader);
        gl.AttachShader(program, fragment_shader);
        gl.LinkProgram(program);
        gl.UseProgram(program);
        let palette_location = gl.GetUniformLocation(program, b"palette\0".as_ptr() as *const _);
        gl.Uniform1i(palette_location, 0);
        let vram_location = gl.GetUniformLocation(program, b"vram\0".as_ptr() as *const _);
        gl.Uniform1i(vram_location, 1);
        gl.ValidateProgram(program);
        let position_attribute = gl.GetAttribLocation(program, b"position\0".as_ptr() as *const _);
        gl.VertexAttribPointer(
            position_attribute as gl::types::GLuint,
            2, gl::FLOAT, 0,
            2*std::mem::size_of::<f32>() as gl::types::GLsizei, std::ptr::null()
        );
        gl.EnableVertexAttribArray(position_attribute as gl::types::GLuint);
        gl.GenTextures(texture_handles.len() as i32, texture_handles.as_ptr() as *mut _);
        texture_setup(&gl, gl::TEXTURE_1D, texture_handles[0]);
        gl.TexImage1D(gl::TEXTURE_1D, 0, gl::RGBA8 as i32, 16, 0, gl::RGBA as u32, gl::UNSIGNED_BYTE, std::ptr::null());
        texture_setup(&gl, gl::TEXTURE_2D, texture_handles[1]);
    }
    let bus = unsafe { &mut *(bus_ptr as *mut crate::bus::BUS) };
    let event_loop_interval = 1.0/bus.config.timing.window_update_frequency;
    let mut pressed_keys = std::collections::HashSet::new();
    event_loop.run(move |event, _event_loop_window_target, control_flow| {
        *control_flow = glutin::event_loop::ControlFlow::WaitUntil(std::time::Instant::now()+std::time::Duration::from_secs_f64(event_loop_interval));
        match event {
            Event::MainEventsCleared => {
                let window = windowed_context.window();
                if bus.vga.video_mode_dirty {
                    bus.vga.video_mode_dirty = false;
                    if bus.vga.width > 0 && bus.vga.height > 0 {
                        unsafe {
                            gl.BindTexture(gl::TEXTURE_2D, texture_handles[1]);
                            gl.TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA8 as i32, (bus.vga.width/8) as i32, bus.vga.height as i32, 0, gl::RGBA as u32, gl::UNSIGNED_BYTE, std::ptr::null());
                        }
                        window.set_inner_size(glutin::dpi::LogicalSize::new((bus.vga.width as f32*SCALE_WIDTH) as u32, (bus.vga.height as f32*SCALE_HEIGHT) as u32));
                        window.set_title(format!("VGA {}x{}", bus.vga.width, bus.vga.height).as_str());
                        window.set_visible(true);
                        println!("GUI: Changed resolution to {}x{}", bus.vga.width, bus.vga.height);
                    } else {
                        window.set_visible(false);
                    }
                }
                let palette_dirty = bus.vga.palette_dirty;
                let vram_dirty = bus.vga.vram_dirty;
                bus.vga.palette_dirty = false;
                bus.vga.vram_dirty = false;
                if palette_dirty || vram_dirty {
                    unsafe {
                        if palette_dirty {
                            gl.BindTexture(gl::TEXTURE_1D, texture_handles[0]);
                            gl.TexSubImage1D(gl::TEXTURE_1D, 0, 0, 16, gl::RGBA as u32, gl::UNSIGNED_BYTE, bus.vga.palette_rgba.as_ptr() as *const _);
                        }
                        if vram_dirty {
                            gl.BindTexture(gl::TEXTURE_2D, texture_handles[1]);
                            gl.TexSubImage2D(gl::TEXTURE_2D, 0, 0, 0, (bus.vga.width/8) as i32, bus.vga.height as i32, gl::RGBA as u32, gl::UNSIGNED_BYTE, bus.vga.vram.as_ptr() as *const _);
                        }
                    }
                    window.request_redraw();
                }
            },
            Event::RedrawRequested(_) => {
                unsafe {
                    gl.ActiveTexture(gl::TEXTURE0);
                    gl.BindTexture(gl::TEXTURE_1D, texture_handles[0]);
                    gl.ActiveTexture(gl::TEXTURE1);
                    gl.BindTexture(gl::TEXTURE_2D, texture_handles[1]);
                    gl.DrawArrays(gl::TRIANGLE_FAN, 0, 4);
                }
                windowed_context.swap_buffers().unwrap();
            },
            Event::LoopDestroyed => return,
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                    sender.send(InputEvent::Termination).unwrap();
                },
                WindowEvent::KeyboardInput { input, .. } => {
                    match input.state {
                        glutin::event::ElementState::Pressed => {
                            if pressed_keys.insert(input.scancode) {
                                sender.send(InputEvent::Key(input.scancode as u8, true)).unwrap();
                            }
                        },
                        glutin::event::ElementState::Released => {
                            if pressed_keys.remove(&input.scancode) {
                                sender.send(InputEvent::Key(input.scancode as u8, false)).unwrap();
                            }
                        }
                    }
                },
                WindowEvent::MouseInput { button, state, .. } => {
                    let button_index = match button {
                        glutin::event::MouseButton::Left => 0,
                        glutin::event::MouseButton::Right => 1,
                        glutin::event::MouseButton::Middle => 2,
                        glutin::event::MouseButton::Other(index) => 3+index
                    };
                    sender.send(InputEvent::MouseButton(button_index, state == glutin::event::ElementState::Pressed)).unwrap();
                },
                WindowEvent::CursorMoved { position, .. } => {
                    sender.send(InputEvent::MouseMove(position.x, position.y)).unwrap();
                },
                WindowEvent::Focused(focused) => {
                    sender.send(InputEvent::Focus(focused)).unwrap();
                },
                _ => ()
            },
            _ => ()
        }
    });
}
