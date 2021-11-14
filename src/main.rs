use std::fs::File;
use std::io::Read;
use sfml::window::{Window, Event, Style};
use sfml::graphics::{Image};
use core::ffi::c_void;

extern crate nalgebra_glm as glm;

fn load_image() -> u32 {
    unsafe {
        let mut id : u32 = 0;
        gl::GenTextures(1, &mut id);
        if id != 0 {
            gl::BindTexture(gl::TEXTURE_2D, id);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT.try_into().unwrap());
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT.try_into().unwrap());
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR.try_into().unwrap());
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR.try_into().unwrap());
            let img_data = Image::from_file("scale.jpg").unwrap();
            let img_data_ptr = img_data.pixel_data().as_ptr() as *const c_void;
            // RGBA since pixel_data pads to 4 channels
            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA.try_into().unwrap(), 500, 281, 0, gl::RGBA, gl::UNSIGNED_BYTE, img_data_ptr);
            gl::GenerateMipmap(gl::TEXTURE_2D);
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
        id
    }
}

fn gen_buffer() -> u32 {
    unsafe {
        let mut id: u32 = 0;
        gl::GenBuffers(1, &mut id);
        id
    }
}

fn gen_vertex_buffer() -> u32 {
    unsafe {
        let mut id: u32 = 0;
        gl::GenVertexArrays(1, &mut id);
        id
    }
}

fn create_shader(shader_type: u32, shader_source_location: &str) -> Result<u32, &str> {
    unsafe {
        let id = gl::CreateShader(shader_type);
        
        if id != 0 {
            let mut source = File::open(shader_source_location).unwrap();
            let mut contents = Vec::new();
            source.read_to_end(&mut contents).map_err(|err| println!("{:?}", err)).ok();
            let content_length = contents.len() as i32;
            let contents_ptr = contents.as_ptr();
            let contents_i8_ptr = contents_ptr as *const i8;
            gl::ShaderSource(id, 1, &contents_i8_ptr, &content_length);
            gl::CompileShader(id);
            
            let mut compile_status: i32 = 0;
            gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut compile_status);
            
            if compile_status == 0 {
                let mut num_written = 0;
                let mut info_log_buffer: [i8; 512] = [0; 512];
                gl::GetShaderInfoLog(id, 512, &mut num_written, info_log_buffer.as_mut_ptr());
                let mut str_data = Vec::new();
                for x in info_log_buffer {
                    if x != 0 {
                        str_data.push(x as u8);
                    }
                }
                let error_string = std::str::from_utf8(&str_data);
                println!("Failed to compile shader: {}, error_status: {}, log: {:?}", shader_source_location, compile_status, error_string);
            } else {
                return Ok(id);
            }
        } 
        Err("Failed to compile shader")
    }
}

fn create_default_program() -> u32{
    create_and_link_program("vertex.glsl", "fragment.glsl")
}

fn create_and_link_program(vertex_shader_source: &str, fragment_shader_source: &str) -> u32 {
    
    let vertex_shader = create_shader(gl::VERTEX_SHADER, vertex_shader_source).unwrap();
    let fragment_shader = create_shader(gl::FRAGMENT_SHADER, fragment_shader_source).unwrap();
    
    unsafe {
        let id = gl::CreateProgram();
        gl::AttachShader(id, vertex_shader);
        gl::AttachShader(id, fragment_shader);
        gl::LinkProgram(id);
        
        let mut link_status: i32 = 0;
        gl::GetProgramiv(id, gl::LINK_STATUS, &mut link_status);
        
        if link_status == 0 {
            let mut num_written = 0;
            let mut info_log_buffer: [i8; 512] = [0; 512];
            gl::GetProgramInfoLog(id, 512, &mut num_written, info_log_buffer.as_mut_ptr());
            let mut str_data = Vec::new();
            for x in info_log_buffer {
                if x != 0 {
                    str_data.push(x as u8);
                }
            }
            let error_string = std::str::from_utf8(&str_data);
            println!("Failed to link program with error_status: {}, log: {:?}", link_status, error_string);
        }
        
        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);
        
        id
    }
}

fn f32_size_mult(len: usize) -> isize {
    static F32_SIZE: usize = std::mem::size_of::<f32>();
    (F32_SIZE * len).try_into().unwrap()
}

struct Vertex {
    pos: [f32; 3],
    color: [f32; 3],
    uv: [f32; 2]
}

fn upload_buffer_data(vao: u32, vbo: u32, ebo: u32) {
    let vertex_data: [Vertex; 4] = [
        Vertex { pos: [0.5,   0.5, 0.], color: [1., 0., 1.], uv: [1., 0.] },
        Vertex { pos: [0.5,  -0.5, 0.], color: [1., 0., 1.], uv: [1., 1.] },
        Vertex { pos: [-0.5, -0.5, 0.], color: [0., 1., 1.], uv: [0., 1.] },
        Vertex { pos: [-0.5,  0.5, 0.], color: [0., 1., 1.], uv: [0., 0.] }
    ];
    let size_of_vertex = std::mem::size_of_val(&vertex_data[0]).try_into().unwrap();
    let size_of_vertex_pos = std::mem::size_of_val(&vertex_data[0].pos);
    let size_of_vertex_color = std::mem::size_of_val(&vertex_data[0].color);
    let _size_of_vertex_uv = std::mem::size_of_val(&vertex_data[0].uv);
    
    let index_data = [
        0, 1, 3,
        1, 2, 3
    ];
    
    // Bind VAO
    unsafe {
        gl::BindVertexArray(vao);
    }
    
    // Load VBO
    unsafe {
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(gl::ARRAY_BUFFER, f32_size_mult(size_of_vertex as usize * vertex_data.len()), vertex_data.as_ptr() as *const c_void, gl::STATIC_DRAW);
        gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, size_of_vertex, 0 as *const c_void);
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, size_of_vertex, size_of_vertex_pos as *const c_void);
        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(2, 2, gl::FLOAT, gl::FALSE, size_of_vertex, (size_of_vertex_pos + size_of_vertex_color) as *const c_void);
        gl::EnableVertexAttribArray(2);
    }
    
    // Load EBO
    unsafe {
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, (std::mem::size_of::<i32>() * index_data.len()).try_into().unwrap(), index_data.as_ptr() as *const c_void, gl::STATIC_DRAW);
    }
}

fn deg_to_rad(x:f32) -> f32 {
    x * (3.141592/180.0)
}

fn main() {
    
    // Creates GL context internally
    let mut window = Window::new((800, 600), "SFML Example", Style::CLOSE, &Default::default());
    window.set_framerate_limit(60);
    
    // Init GL after GL context has been created
    gl_loader::init_gl();
    gl::load_with(|s| gl_loader::get_proc_address(s) as *const _);
    
    let vao = gen_vertex_buffer();
    let vbo = gen_buffer();
    let ebo = gen_buffer();
    let default_program = create_default_program();
    let texture_id = load_image();
    
    upload_buffer_data(vao, vbo, ebo);
    
    while window.is_open() {
        while let Some(event) = window.poll_event() {
            if event == Event::Closed {
                window.close();
            }
        }
        
        window.set_active(true);
        
        unsafe {
            let ortho = glm::ortho(0.0f32, 800., 0., 600., -10., 100.);
            let id = glm::identity::<f32, 4>();
            let scale = glm::make_vec3(&[500., 281., 1.0]);
            let model = glm::scale(&id, &scale);
            let mve = glm::make_vec3(&[400., 300., 0.0]);
            let view = glm::translate(&id, &mve);
            let mvp = ortho * view * model;
            let mvp_name = "mvp\0".as_bytes();
            let mvp_loc = gl::GetUniformLocation(default_program, mvp_name.as_ptr() as *const i8);
            
            gl::UniformMatrix4fv(mvp_loc, 1, gl::FALSE, mvp.data.as_slice().as_ptr());
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::Viewport(0, 0, 800, 600);
            gl::BindVertexArray(vao);
            gl::BindTexture(gl::TEXTURE_2D, texture_id);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::UseProgram(default_program);
            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, 0 as *const c_void);
        }
        
        window.display();
    }
    
    unsafe {
        gl::DeleteBuffers(1, &ebo);
        gl::DeleteBuffers(1, &vbo);
        gl::DeleteVertexArrays(1, &vao);
        gl::DeleteProgram(default_program);
        gl::DeleteTextures(1, &texture_id);
    }
    
    gl_loader::end_gl();
}
