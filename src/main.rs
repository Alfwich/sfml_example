use sfml::window::{Window, Event, Style};
use sfml::graphics::{Image};
use core::ffi::c_void;

fn init_system() {
    gl_loader::init_gl();
    gl::load_with(|s| gl_loader::get_proc_address(s) as *const _);
    gl_loader::end_gl();
}

fn load_image() -> u32 {
    unsafe {
        let mut id : u32 = 0;
        gl::GenTextures(1, &mut id);
        if (id != 0) {
            unsafe { gl::BindTexture(gl::TEXTURE_2D, id) };
            let img_data = Image::from_file("scale.jpg").unwrap().pixel_data().as_ptr() as *const c_void;
            unsafe { gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB.try_into().unwrap(), 100, 100, 0, gl::RGB, gl::UNSIGNED_BYTE, img_data); }
            unsafe { gl::BindTexture(gl::TEXTURE_2D, 0) };
        }
        id
    }
}

fn load_vbo() -> u32 {
    unsafe {
        let mut id: u32 = 0;
        gl::GenBuffers(1, &mut id);
        id
    }
}

fn main() {
    init_system();
    
    let mut window = Window::new((800, 600), "SFML Example", Style::CLOSE, &Default::default());
    window.set_framerate_limit(60);
    
    //let texture_id = load_image();
    //let vbo = load_vbo();
    
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
            //gl::BindTexture(gl::TEXTURE_2D, texture_id);
            //gl::DrawElements(gl::QUADS, 4, gl::UNSIGNED_INT, )
            //gl::BindTexture(gl::TEXTURE_2D, 0);
        }
        
        window.display();
    }
    
}
