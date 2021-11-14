use sfml::window::{Window, Event, Style};
use sfml::graphics::{Image};
use core::ffi::c_void;

fn init_gl() {
    gl_loader::init_gl();
    gl::load_with(|s| gl_loader::get_proc_address(s) as *const _);
}

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

fn cleanup() {
    gl_loader::end_gl();
}

fn main() {
    
    // Creates GL context internally
    let mut window = Window::new((800, 600), "SFML Example", Style::CLOSE, &Default::default());
    window.set_framerate_limit(60);
    
    // Init GL after GL context has been created
    init_gl();
    
    let vao = gen_vertex_buffer();
    let vbo = gen_buffer();
    let ibo = gen_buffer();
    let texture_id = load_image();
    
    let vertex_data: [f32; 6] = [
        -0.5, 0.,
        0.5, 0.,
        0., 0.5
    ];
    
    let index_data: [i32; 3] = [
        0, 1, 2
    ];
    
    // Bind VAO
    unsafe {
        gl::BindVertexArray(vao);
    }
    
    // Load VBO
    unsafe {
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(gl::ARRAY_BUFFER, 36, vertex_data.as_ptr() as *const c_void, gl::STATIC_DRAW);
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, 0, 0 as *const c_void);
    }
    
    // Load IBO
    unsafe {
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);
        gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, 12, index_data.as_ptr() as *const c_void, gl::STATIC_DRAW);
    }
    
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
            gl::DrawElements(gl::TRIANGLES, 3, gl::UNSIGNED_INT, 0 as *const c_void);
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
        
        window.display();
    }
    
    cleanup();
}
