use std::fs::File;
use std::io::Read;
use sfml::window::{Window, Event, Style};
use sfml::graphics::{Image};
use core::ffi::c_void;

fn load_image() -> u32 {
    unsafe {
        let mut id : u32 = 0;
        gl::GenTextures(1, &mut id);
        if id != 0 {
            gl::BindTexture(gl::TEXTURE_2D, id);
            let img_data = Image::from_file("scale.jpg").unwrap();
            let img_data_ptr = img_data.pixel_data().as_ptr() as *const c_void;
            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB.try_into().unwrap(), 100, 100, 0, gl::RGB, gl::UNSIGNED_BYTE, img_data_ptr);
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

fn upload_buffer_data(vao: u32, vbo: u32, ibo: u32) {
    let vertex_data = [
        -1.0f32, 1.,
        0., 1.,
        0., 0.,
        -1., 0.
    ];
    
    let index_data = [
        0, 1, 2,
        0, 2, 3
    ];
    
    // Bind VAO
    unsafe {
        gl::BindVertexArray(vao);
    }
    
    // Load VBO
    unsafe {
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(gl::ARRAY_BUFFER, (std::mem::size_of::<f32>() * vertex_data.len()).try_into().unwrap(), vertex_data.as_ptr() as *const c_void, gl::STATIC_DRAW);
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, 0, 0 as *const c_void);
    }
    
    // Load IBO
    unsafe {
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);
        gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, (std::mem::size_of::<i32>() * index_data.len()).try_into().unwrap(), index_data.as_ptr() as *const c_void, gl::STATIC_DRAW);
    }
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
    let ibo = gen_buffer();
    let default_program = create_default_program();
    let texture_id = load_image();
    
    upload_buffer_data(vao, vbo, ibo);
    
    while window.is_open() {
        while let Some(event) = window.poll_event() {
            if event == Event::Closed {
                window.close();
            }
        }
        
        window.set_active(true);
        
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::Viewport(0, 0, 800, 600);
            gl::BindTexture(gl::TEXTURE_2D, texture_id);
            gl::BindVertexArray(vbo);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);
            gl::UseProgram(default_program);
            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, 0 as *const c_void);
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
        
        window.display();
    }
    
    unsafe {
        gl::DeleteBuffers(1, &ibo);
        gl::DeleteBuffers(1, &vbo);
        gl::DeleteVertexArrays(1, &vao);
        gl::DeleteProgram(default_program);
        gl::DeleteTextures(1, &texture_id);
    }
    
    gl_loader::end_gl();
}
