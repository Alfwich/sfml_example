use std::fs::File;
use std::io::Read;
use sfml::window::{Window, Context, Event, Style, Key};
use sfml::graphics::{Image};
use core::ffi::c_void;
use core::result::Result;
use std::thread;
use std::sync::{Arc, Mutex};
use std::time::{Instant};
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::collections::{HashMap, VecDeque};
use itertools::Itertools;

extern crate nalgebra_glm as glm;

#[derive(Debug)]
struct Vertex {
    pos: [f32; 3],
    uv: [f32; 2]
}

#[derive(Debug)]
struct Viewport {
    pos: [f32; 2],
    desired_pos: [f32; 2]
}

#[derive(Debug)]
struct DImageLoaded {
    texture_id: u32,
    container_idx: usize,
}

#[derive(Debug)]
struct DImage {
    pub scale: f32,
    pub border: f32,
    pub texture_id: u32,
}

#[derive(Debug)]
struct DImageRow {
    pub title: RenderedImage,
    
    pub images: Vec<DImage>,
    pub selected_tile_idx: f32,
    pub desired_selected_tile_idx: f32,
}

#[derive(Debug)]
struct RenderedImage {
    texture_id: u32,
    width: u32,
    height: u32
}

fn load_image_from_disk(path: &str, width: i32, height: i32) -> Result<u32, String> {
    let mut f = File::open(path).unwrap();
    let mut img_bytes = Vec::new();
    f.read_to_end(&mut img_bytes).unwrap();
    
    unsafe {
        let mut id : u32 = 0;
        gl::GenTextures(1, &mut id);
        if id != 0 {
            gl::BindTexture(gl::TEXTURE_2D, id);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE.try_into().unwrap());
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE.try_into().unwrap());
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR.try_into().unwrap());
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR.try_into().unwrap());
            let img_data = Image::from_memory(&img_bytes);
            match img_data {
                Some(img_data) => {
                    let img_data_ptr = img_data.pixel_data().as_ptr() as *const c_void;
                    // RGBA since pixel_data pads to 4 channels
                    gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA.try_into().unwrap(), width, height, 0, gl::RGBA, gl::UNSIGNED_BYTE, img_data_ptr);
                    gl::GenerateMipmap(gl::TEXTURE_2D);
                    gl::BindTexture(gl::TEXTURE_2D, 0);                
                }
                None => {
                    gl::DeleteTextures(1, &id);
                    println!("Bad Image for path: {:?}", path);
                    return Err("Bad Image".to_string());
                }
            }

        }
        
        return Ok(id);
    }
}


fn load_image_from_url(client: &reqwest::blocking::Client, url: &str) -> Result<u32, String> {
    match client.get(url).send() {
        Ok(response) => {
            let resp_bytes = response.bytes().unwrap();
            unsafe {
                let mut id : u32 = 0;
                gl::GenTextures(1, &mut id);
                if id != 0 {
                    gl::BindTexture(gl::TEXTURE_2D, id);
                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE.try_into().unwrap());
                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE.try_into().unwrap());
                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR.try_into().unwrap());
                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR.try_into().unwrap());
                    let img_data = Image::from_memory(&resp_bytes);
                    match img_data {
                        Some(img_data) => {
                            let img_data_ptr = img_data.pixel_data().as_ptr() as *const c_void;
                            // RGBA since pixel_data pads to 4 channels
                            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA.try_into().unwrap(), 500, 281, 0, gl::RGBA, gl::UNSIGNED_BYTE, img_data_ptr);
                            gl::GenerateMipmap(gl::TEXTURE_2D);
                            gl::BindTexture(gl::TEXTURE_2D, 0);                
                        }
                        None => {
                            gl::DeleteTextures(1, &id);
                            println!("Bad Image for url: {:?}", url);
                            return Err("Bad Image".to_string());
                        }
                    }

                }
                
                return Ok(id);
            }
    
        }
        Err(_) => {
            return Err("Bad Image Url".to_string());
        }
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

fn get_item_image_url_from_json_value(item: &serde_json::Value) -> String {
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
    
    url[1..url.len()-1].to_string()
}

fn get_container_title_from_json_value(container: &serde_json::Value) -> String {
    let result = container["set"]["text"]["title"]["full"]["set"]["default"]["content"].to_string();
    if container["set"]["text"]["title"]["full"]["set"]["default"]["content"].is_string() {
        result[1..result.len()-1].to_string()
    } else {
        result
    }
}

fn get_container_refset_id_from_json_value(container: &serde_json::Value) -> String {
    let result = container["set"]["refId"].to_string();
    if container["set"]["refId"].is_string() {
        result[1..result.len()-1].to_string()
    } else {
        result
    }
}

fn get_container_refset_type_from_json_value(container: &serde_json::Value) -> String {
    let result = container["set"]["refType"].to_string();
    if container["set"]["refType"].is_string() {
        result[1..result.len()-1].to_string()
    } else {
        result
    }
}

struct TextTextureData {
    pub rows: HashMap<i32, Vec<u8>>,
    pub width: usize,
    pub height: usize,
    pub data: Vec<u8>,
}

impl Default for TextTextureData {
    fn default() -> Self {
        TextTextureData {
            rows: HashMap::new(),
            width: 0,
            height: 0,
            data: Vec::new()
        }
    }
}


fn sw_blit_to_buffer(offset: (u32, u32), size: (u32, u32), top: i32, dst: &mut TextTextureData, src: &[u8]) {
    let y_offset = -top as i32;
    for x in 0..size.0 {
        let x_pos = (x + offset.0) as usize;
        for y in 0..size.1 {
            let y_dst_pos = (y as i32 + y_offset) as i32;
            if !dst.rows.contains_key(&y_dst_pos) {
                dst.rows.insert(y_dst_pos, Vec::new());
            }
            
            while dst.rows[&y_dst_pos].len() <= x_pos {
                dst.rows.get_mut(&y_dst_pos).unwrap().push(0);
            }
                
            dst.rows.get_mut(&y_dst_pos).unwrap()[x_pos] = src[x as usize + ((y * size.0) as usize)];
        }
    }
}

fn sw_render_text_to_buffer(str: &str, data: &mut TextTextureData) {
    static FONT_FILE: &str = "GlacialIndifference-Bold.otf";
    let lib = freetype::Library::init().unwrap();
    let face = lib.new_face(FONT_FILE, 0).unwrap();
    face.set_char_size(80 * 32, 0, 100, 0).map_err(|err| println!("{:?}", err)).ok();
    let mut offset = (0u32, 0u32);
    for c in str.chars() {
        face.load_char(c as usize, freetype::face::LoadFlag::RENDER).map_err(|err| println!("{:?}", err)).ok();
        let glyph = face.glyph();
        let glyph_bitmap = glyph.bitmap();
        let bitmap_data = glyph_bitmap.buffer();
        sw_blit_to_buffer(offset, (glyph_bitmap.width() as u32, glyph_bitmap.rows() as u32), glyph.bitmap_top(), data, bitmap_data);
        offset.0 += (glyph.advance().x / 64) as u32;
    }
    
    data.height = data.rows.len();
    
    let mut max_width = 0;
    for (_k, v) in &data.rows {
        if v.len() > max_width {
            max_width = v.len();
        }
    }
    data.width = max_width;
    
    for k in data.rows.keys().sorted() {
        let row = data.rows.get(k).unwrap();
        for j in 0..data.width {
            if j >= row.len() {
                data.data.push(0);
            } else {
                data.data.push(row[j]);
            }
        }
    }
    
    assert!(data.data.len() == data.width * data.height, "data should be width * height");
}

fn render_text_to_texture(str: &str) -> RenderedImage {
        
    unsafe {
        let mut id : u32 = 0;
        gl::GenTextures(1, &mut id);
        gl::BindTexture(gl::TEXTURE_2D, id);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE.try_into().unwrap());
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE.try_into().unwrap());
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR.try_into().unwrap());
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR.try_into().unwrap());
        gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
        
        let mut texture_data = TextTextureData::default();
        sw_render_text_to_buffer(str, &mut texture_data);
        let texture_data_ptr = texture_data.data.as_ptr() as *const c_void;
        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RED.try_into().unwrap(), texture_data.width as i32, texture_data.height as i32, 0, gl::RED, gl::UNSIGNED_BYTE, texture_data_ptr);
        gl::GenerateMipmap(gl::TEXTURE_2D);
        gl::BindTexture(gl::TEXTURE_2D, 0);
    
        RenderedImage {
            texture_id: id,
            width: texture_data.width as u32,
            height: texture_data.height as u32
        }
    }
}

// Loads initial page data and kicks off worker threads to finish image loading and refset loading
fn load_page_data(containers: &mut Vec<DImageRow>) -> Receiver<DImageLoaded> {
    static APP_DATA_SOURCE: &str = "https://cd-static.bamgrid.com/dp-117731241344/home.json";
    static APP_DATA_REFSET_SOURCE: &str = "https://cd-static.bamgrid.com/dp-117731241344/sets/{{id}}.json";
    let (tx, rx): (Sender<DImageLoaded>, Receiver<DImageLoaded>) = mpsc::channel();
    
    #[derive(Debug)]
    struct ImageLoadingBundle {
        pub refset_id: String,
        pub refset_type: String,
        pub container_idx: usize,
        pub images_to_load: Vec<String>
    }
    
    let rows_to_load: Arc<Mutex<VecDeque<ImageLoadingBundle>>> = Arc::new(Mutex::new(VecDeque::new()));
    
    let resp = reqwest::blocking::get(APP_DATA_SOURCE).unwrap().text().unwrap();
    let data: serde_json::Value = serde_json::from_str(&resp).unwrap();
    let json_containers: Vec<serde_json::Value> = data["data"]["StandardCollection"]["containers"].as_array().unwrap().to_vec();
    let mut container_idx = 0;
    for container in json_containers {
        let title = render_text_to_texture(&get_container_title_from_json_value(&container));
        let refset_id = get_container_refset_id_from_json_value(&container);
        let refset_type = get_container_refset_type_from_json_value(&container);
        
        let mut bundle = ImageLoadingBundle {
            refset_id,
            refset_type,
            container_idx,
            images_to_load: Vec::new()
        };
        
        container_idx += 1;
        containers.push(DImageRow {images: Vec::new(), title: title, selected_tile_idx: 0., desired_selected_tile_idx: 0. });
        let items = container["set"]["items"].as_array();
        match items {
            Some(arr) => {
                for item in arr.to_vec() {
                    let url = get_item_image_url_from_json_value(&item);
                    bundle.images_to_load.push(url);
                }
            },
            _ => {}
        }
        
        rows_to_load.lock().unwrap().push_back(bundle);
    }
    
    // Spawn threads to acquire images and populate refsets
    for _thread_idx in 0..8 {
        let thread_tx = tx.clone();
        let thread_rows_to_load = Arc::clone(&rows_to_load);
        thread::spawn(move || {
            loop {
                // Create GL context to allow async threads to load image data
                let _context = Context::new();
                let client = reqwest::blocking::Client::new();
                let row_to_load = thread_rows_to_load.lock().unwrap().pop_front();
                match row_to_load {
                    Some(mut row_to_load) => {
                        // Populate refset if needed
                        if row_to_load.refset_id != "null" {
                            let set_id = row_to_load.refset_id.to_string();
                            let set_type = row_to_load.refset_type.to_string();
                            let set_url = APP_DATA_REFSET_SOURCE.replace("{{id}}", &set_id);
                            let ref_resp = reqwest::blocking::get(set_url).unwrap().text().unwrap();
                            let ref_data: serde_json::Value = serde_json::from_str(&ref_resp).unwrap();
                            
                            let mut refset_data_key = set_type.to_string();
                            for key in ref_data["data"].as_object().unwrap().keys() {
                                refset_data_key = key.to_string();
                                break;
                            }
                            
                            let items = ref_data["data"][refset_data_key]["items"].as_array();
                            match items {
                                Some(arr) => {
                                    for item in arr.to_vec() {
                                        let url = get_item_image_url_from_json_value(&item);
                                        row_to_load.images_to_load.push(url);
                                    }
                                },
                                _ => {
                                    println!("Failed to load refset id: {:?}, type: {:?}", set_id, set_type);
                                }
                            }
                        }
                        
                        for image_url in &row_to_load.images_to_load {
                            match load_image_from_url(&client, &image_url) {
                                Ok(texture_id) => {
                                    thread_tx.send(DImageLoaded { texture_id, container_idx: row_to_load.container_idx }).map_err(|err| println!("{:?}", err)).ok();
                                },
                                _ => {
                                }
                            }        
                        }
                    },
                    _ => {
                        break;
                    }
                };
            }
        });
    }
    
    rx
}


fn main() {
    static WINDOW_SIZE: (u32, u32) = (1920, 1080);
    let mut containers: Vec<DImageRow> = Vec::new();
    
    // Creates GL context internally
    let mut window = Window::new(WINDOW_SIZE, "SFML Example", Style::CLOSE, &Default::default());
    window.set_framerate_limit(200);
    
    // Init GL after GL context has been created
    gl_loader::init_gl();
    gl::load_with(|s| gl_loader::get_proc_address(s) as *const _);
    
    let loader_rx = load_page_data(&mut containers);
    
    let vao = gen_vertex_buffer();
    let vbo = gen_buffer();
    let ebo = gen_buffer();
    let tile_program = create_and_link_program("vertex.glsl", "tile.glsl");
    let text_program = create_and_link_program("vertex.glsl", "text.glsl");
    let background_image_texture_id = load_image_from_disk("background.png", 1440, 1070).unwrap();
    
    upload_buffer_data(vao, vbo, ebo);
    
    let ortho = glm::ortho(0.0f32, WINDOW_SIZE.0 as f32, 0., WINDOW_SIZE.1 as f32, -10., 100.);
    let id = glm::identity::<f32, 4>();
    let base_move = glm::make_vec3(&[WINDOW_SIZE.0 as f32 / 2. - 550., WINDOW_SIZE.1 as f32 / 2. + 350., 0.0]);
    
    let tile_mvp_loc; 
    let tile_border_loc; 
    let text_mvp_loc; 
    
    unsafe { 
        let mvp_name = "mvp\0".as_bytes();
        let border_name = "border\0".as_bytes();
        
        tile_mvp_loc = gl::GetUniformLocation(tile_program, mvp_name.as_ptr() as *const i8);
        tile_border_loc = gl::GetUniformLocation(tile_program, border_name.as_ptr() as *const i8);
        text_mvp_loc = gl::GetUniformLocation(text_program, mvp_name.as_ptr() as *const i8); 
    }
    
    let mut viewport = Viewport { pos: [0., 0.], desired_pos: [0., 0.] };
    let mut selected_container_idx = 0;
    let mut last = Instant::now();
    
    while window.is_open() {
        let current = Instant::now();
        let dt = (current - last).as_secs_f32();
        last = current;
        
        while let Some(event) = window.poll_event() {
            match event {
                Event::Closed => {
                    window.close();
                },
                Event::KeyPressed { code, .. } => {
                    match code {
                        Key::D => {
                            if containers[selected_container_idx].images.len() > 0 && containers[selected_container_idx].desired_selected_tile_idx < (containers[selected_container_idx].images.len() - 1) as f32 {
                                containers[selected_container_idx].desired_selected_tile_idx += 1.;
                            }
                        },
                        Key::A => {
                            if containers[selected_container_idx].desired_selected_tile_idx > 0. {
                                containers[selected_container_idx].desired_selected_tile_idx -= 1.;
                            }
                        },
                        Key::W => {
                            if selected_container_idx >= 1 {
                                selected_container_idx -= 1;
                            }
                        },
                        Key::S => {
                            if selected_container_idx < containers.len() - 1 {
                                selected_container_idx += 1;
                            }
                        },
                        _ => {}
                    }
                    
                }
                _ => {}
            }
        }
        
        loop {
            match loader_rx.try_recv() {
                Ok(image_loaded) => {
                    containers[image_loaded.container_idx].images.push( DImage { texture_id: image_loaded.texture_id, scale: 1., border: 0. });
                }
                Err(_type) => {
                    break;
                }
            }
        }
        
        viewport.desired_pos[1] = 480. * selected_container_idx as f32;
        viewport.pos[1] += ((viewport.desired_pos[1] - viewport.pos[1]) / 0.1) * dt;
        
        window.set_active(true);
        
        if containers[1].images.len() < 3 {
            continue;
        }
        
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::Enable(gl::BLEND); 
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Viewport(0, 0, WINDOW_SIZE.0.try_into().unwrap(), WINDOW_SIZE.1.try_into().unwrap());
            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            
            // Draw Background
            { 
                let scale = glm::make_vec3(&[WINDOW_SIZE.0 as f32, WINDOW_SIZE.1 as f32, 1.]);
                let model = glm::scale(&id, &scale);
                let mve = glm::make_vec3(&[WINDOW_SIZE.0 as f32 / 2., WINDOW_SIZE.1 as f32 / 2., 0.]);
                let view = glm::translate(&id, &mve);
                let mvp = ortho * view * model;
                
                
                gl::UseProgram(tile_program);
                gl::UniformMatrix4fv(tile_mvp_loc, 1, gl::FALSE, mvp.data.as_slice().as_ptr());
                gl::Uniform1f(tile_border_loc, 0.);
                gl::BindTexture(gl::TEXTURE_2D, background_image_texture_id);
                gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, 0 as *const c_void);
            }
            
            let mut idx = (0, 0);
            for container_idx in 0..containers.len() {
                { 
                    let scale = glm::make_vec3(&[containers[container_idx].title.width as f32, containers[container_idx].title.height as f32, 1.]);
                    let model = glm::scale(&id, &scale);
                    let mve = base_move + glm::make_vec3(&[viewport.pos[0] + (containers[container_idx].title.width as f32 / 2.) - 250., viewport.pos[1] - idx.1 as f32, 0.]);
                    let view = glm::translate(&id, &mve);
                    let mvp = ortho * view * model;
                    
                    containers[container_idx].selected_tile_idx += ((containers[container_idx].desired_selected_tile_idx - containers[container_idx].selected_tile_idx) / 0.1) * dt;
                    
                    gl::UseProgram(text_program);
                    gl::UniformMatrix4fv(text_mvp_loc, 1, gl::FALSE, mvp.data.as_slice().as_ptr());
                    gl::BindTexture(gl::TEXTURE_2D, containers[container_idx].title.texture_id);
                    gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, 0 as *const c_void);
                    
                    idx.1 += 200;
                }
                
                let selected_tile_idx_i32 = containers[container_idx].desired_selected_tile_idx.round() as usize;
                
                for image_idx in 0..containers[container_idx].images.len() {
                    if selected_tile_idx_i32 == image_idx && selected_container_idx == container_idx {
                        containers[container_idx].images[image_idx].border = 0.01;
                        if containers[container_idx].images[image_idx].scale < 1.15 {
                            containers[container_idx].images[image_idx].scale += 1. * dt;
                            if containers[container_idx].images[image_idx].scale > 1.15 {
                                containers[container_idx].images[image_idx].scale = 1.15;
                            }
                        }
                    } else if containers[container_idx].images[image_idx].scale > 1. {
                        containers[container_idx].images[image_idx].border = 0.;
                        containers[container_idx].images[image_idx].scale -= 1. * dt;
                        if containers[container_idx].images[image_idx].scale < 1. {
                            containers[container_idx].images[image_idx].scale = 1.;
                        }
                    }
                    
                    {
                        let scale = glm::make_vec3(&[500. * containers[container_idx].images[image_idx].scale, 281. * containers[container_idx].images[image_idx].scale, 1.0]);
                        let model = glm::scale(&id, &scale);
                        let mve = base_move + glm::make_vec3(&[viewport.pos[0] + idx.0 as f32 - (containers[container_idx].selected_tile_idx * 625.) as f32, viewport.pos[1] - idx.1 as f32, 0.]);
                        let view = glm::translate(&id, &mve);
                        let mvp = ortho * view * model;
                        
                        gl::UseProgram(tile_program);
                        gl::UniformMatrix4fv(tile_mvp_loc, 1, gl::FALSE, mvp.data.as_slice().as_ptr());
                        // TODO: x, y border factors
                        gl::Uniform1f(tile_border_loc, containers[container_idx].images[image_idx].border);
                        gl::BindTexture(gl::TEXTURE_2D, containers[container_idx].images[image_idx].texture_id);
                        gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, 0 as *const c_void);
                        idx.0 += 625;
                    }
                }
                idx.1 += 280;
                idx.0 = 0;
            }
        }
        
        window.display();
    }
    
    unsafe {
        gl::DeleteBuffers(1, &ebo);
        gl::DeleteBuffers(1, &vbo);
        gl::DeleteVertexArrays(1, &vao);
        gl::DeleteProgram(tile_program);
        gl::DeleteProgram(text_program);
        gl::DeleteTextures(1, &background_image_texture_id);
        
        for container_idx in 0..containers.len() {
            gl::DeleteTextures(1, &containers[container_idx].title.texture_id);
            for image_idx in 0..containers[container_idx].images.len() {
                gl::DeleteTextures(1, &containers[container_idx].images[image_idx].texture_id);
            }
        }
    }
    
    gl_loader::end_gl();
}
