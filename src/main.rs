use std::fs::File;
use std::io::Read;
use sfml::window::{Window, Event, Style, Key};
use sfml::graphics::{Image};
use core::ffi::c_void;
//use serde::{Serialize, Deserialize};
//use std::collections::HashMap;

extern crate nalgebra_glm as glm;

#[derive(Debug)]
struct Vertex {
    pos: [f32; 3],
    uv: [f32; 2]
}

#[derive(Debug)]
struct Viewport {
    pos: [f32; 2]
}

#[derive(Debug)]
struct DImage {
    pub image_id: u32,
    
    image_url: String,
    pos: [i32; 2]
}

/*
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
*/

fn load_image_from_url(url: &str) -> u32 {
    println!("Getting image data for: {:?}", url);
    let resp = reqwest::blocking::get(url).unwrap().bytes().unwrap();
    
    unsafe {
        let mut id : u32 = 0;
        gl::GenTextures(1, &mut id);
        if id != 0 {
            gl::BindTexture(gl::TEXTURE_2D, id);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT.try_into().unwrap());
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT.try_into().unwrap());
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR.try_into().unwrap());
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR.try_into().unwrap());
            let img_data = Image::from_memory(&resp);
            match img_data {
                Some(img_data) => {
                    let img_data_ptr = img_data.pixel_data().as_ptr() as *const c_void;
                    // RGBA since pixel_data pads to 4 channels
                    gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA.try_into().unwrap(), 500, 281, 0, gl::RGBA, gl::UNSIGNED_BYTE, img_data_ptr);
                    gl::GenerateMipmap(gl::TEXTURE_2D);
                    gl::BindTexture(gl::TEXTURE_2D, 0);                
                }
                None => {
                    println!("Bad Image for url: {:?}", url);
                }
            }

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

fn upload_buffer_data(vao: u32, vbo: u32, ebo: u32) {
    let vertex_data: [Vertex; 4] = [
        Vertex { pos: [0.5,   0.5, 0.], uv: [1., 0.] },
        Vertex { pos: [0.5,  -0.5, 0.], uv: [1., 1.] },
        Vertex { pos: [-0.5, -0.5, 0.], uv: [0., 1.] },
        Vertex { pos: [-0.5,  0.5, 0.], uv: [0., 0.] }
    ];
    let size_of_vertex = std::mem::size_of_val(&vertex_data[0]).try_into().unwrap();
    let size_of_vertex_pos = std::mem::size_of_val(&vertex_data[0].pos);
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
        gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, size_of_vertex, size_of_vertex_pos as *const c_void);
        gl::EnableVertexAttribArray(1);
    }
    
    // Load EBO
    unsafe {
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, (std::mem::size_of::<i32>() * index_data.len()).try_into().unwrap(), index_data.as_ptr() as *const c_void, gl::STATIC_DRAW);
    }
}

static WINDOW_SIZE: (u32, u32) = (800, 600);
static APP_FPS: u32 = 60;
static APP_DATA_SOURCE: &str = "https://cd-static.bamgrid.com/dp-117731241344/home.json";

fn load_all_images(images: &mut Vec<DImage>) {
    let resp = reqwest::blocking::get(APP_DATA_SOURCE).unwrap().text().unwrap();
    let data: serde_json::Value = serde_json::from_str(&resp).unwrap();
    let containers: Vec<serde_json::Value> = data["data"]["StandardCollection"]["containers"].as_array().unwrap().to_vec();
    let mut pos: (i32, i32) = (0, 0);
    for container in containers {
        let items = container["set"]["items"].as_array();
        match items {
            Some(arr) => {
                for item in arr.to_vec() {
                    let mut url:String = "".to_string();
                    if item["image"]["tile"]["1.78"]["series"]["default"]["url"].is_string() {
                        url = item["image"]["tile"]["1.78"]["series"]["default"]["url"].to_string();
                    } else if item["image"]["tile"]["1.78"]["program"]["default"]["url"].is_string() {
                        url = item["image"]["tile"]["1.78"]["program"]["default"]["url"].to_string();
                    } else if item["image"]["tile"]["1.78"]["default"]["default"]["url"].is_string() {
                        url = item["image"]["tile"]["1.78"]["default"]["default"]["url"].to_string();
                    } else {
                        println!("Failed to fish out image url: {:#?}", item["image"]["tile"]["1.78"]);
                    }
                    
                    images.push( DImage {image_url: url[1..url.len()-1].to_string(), image_id: 0, pos: [pos.0, pos.1]} );
                    pos.1 += 1;
                }
            },
            _ => {}
        }
        pos.0 += 1;
        pos.1 = 0;
    }
    
    for mut image in images {
        image.image_id = load_image_from_url(&image.image_url);
    }
    
    //println!("{:#?}", images.len());
}

fn main() {
    
    // Creates GL context internally
    let mut window = Window::new(WINDOW_SIZE, "SFML Example", Style::CLOSE, &Default::default());
    window.set_framerate_limit(APP_FPS);
    
    // Init GL after GL context has been created
    gl_loader::init_gl();
    gl::load_with(|s| gl_loader::get_proc_address(s) as *const _);
    
    let mut images : Vec<DImage> = Vec::new();
    load_all_images(&mut images);
    
    let vao = gen_vertex_buffer();
    let vbo = gen_buffer();
    let ebo = gen_buffer();
    let default_program = create_default_program();
    //let texture_id = load_image();
    
    upload_buffer_data(vao, vbo, ebo);
    
    let ortho = glm::ortho(0.0f32, WINDOW_SIZE.0 as f32, 0., WINDOW_SIZE.1 as f32, -10., 100.);
    let id = glm::identity::<f32, 4>();
    let base_move = glm::make_vec3(&[WINDOW_SIZE.0 as f32 / 2., WINDOW_SIZE.1 as f32 / 2., 0.0]);
    
    let mvp_name = "mvp\0".as_bytes();
    let mvp_loc; 
    unsafe { mvp_loc = gl::GetUniformLocation(default_program, mvp_name.as_ptr() as *const i8); };
    
    let mut viewport = Viewport { pos: [0., 0.] };
    
    while window.is_open() {
        while let Some(event) = window.poll_event() {
            match event {
                Event::Closed => {
                    window.close();
                },
                Event::KeyPressed { code, .. } => {
                    match code {
                        Key::A => {
                            viewport.pos[0] -= 1.;
                        },
                        Key::D => {
                            viewport.pos[0] += 1.;
                        },
                        Key::W => {
                            viewport.pos[1] += 1.;
                        },
                        Key::S => {
                            viewport.pos[1] -= 1.;
                        },
                        _ => {}
                    }
                    
                    println!("KEY PRESSED: {:?}", code)
                }
                _ => {}
            }
        }
        
        window.set_active(true);
        
        unsafe {
            let scale = glm::make_vec3(&[500., 281., 1.0]);
            let model = glm::scale(&id, &scale);
            let mve = base_move + glm::make_vec3(&[viewport.pos[0], viewport.pos[1], 0.]);
            let view = glm::translate(&id, &mve);
            let mvp = ortho * view * model;
            
            gl::UniformMatrix4fv(mvp_loc, 1, gl::FALSE, mvp.data.as_slice().as_ptr());
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::Viewport(0, 0, 800, 600);
            gl::BindVertexArray(vao);
            gl::BindTexture(gl::TEXTURE_2D, images[0].image_id);
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
        for image in images {
            gl::DeleteTextures(1, &image.image_id);
        }
    }
    
    gl_loader::end_gl();
}
