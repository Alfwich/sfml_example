use sfml::window::{Window, Context, Event, Style, Key};
use std::thread;
use std::sync::{Arc, Mutex};
use std::time::{Instant};
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::collections::{VecDeque};

extern crate nalgebra_glm as glm;

mod app_gl;
mod util;

#[derive(Debug)]
pub struct Viewport {
    pos: [f32; 2],
    desired_pos: [f32; 2]
}

impl Default for Viewport {
    fn default() -> Self {
        Viewport {
            pos: [0., 0.],
            desired_pos: [0., 0.],
        }
    }
}

#[derive(Debug)]
struct DImageLoaded {
    texture_id: u32,
    container_idx: usize,
}

#[derive(Debug)]
pub struct DImage {
    pub scale: f32,
    pub border: f32,
    pub texture_id: u32,
}

#[derive(Debug)]
pub struct DImageRow {
    pub title: app_gl::RenderedImage,
    pub images: Vec<DImage>,
    pub selected_tile_idx: f32,
    pub desired_selected_tile_idx: f32,
}

impl Drop for DImageRow {
    fn drop(&mut self) {
        app_gl::release_texture(self.title.texture_id);
        for image in &self.images {
            app_gl::release_texture(image.texture_id);
        }
    }
}

fn get_item_image_url_from_json_value(item: &serde_json::Value) -> String {
    let url;
    if item["image"]["tile"]["1.78"]["series"]["default"]["url"].is_string() {
        url = item["image"]["tile"]["1.78"]["series"]["default"]["url"].to_string();
    } else if item["image"]["tile"]["1.78"]["program"]["default"]["url"].is_string() {
        url = item["image"]["tile"]["1.78"]["program"]["default"]["url"].to_string();
    } else if item["image"]["tile"]["1.78"]["default"]["default"]["url"].is_string() {
        url = item["image"]["tile"]["1.78"]["default"]["default"]["url"].to_string();
    } else {
        println!("Failed to fish out image url: {:?}", item["image"]["tile"]["1.78"]);
        return "".to_string();
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

#[derive(Debug)]
struct ImageLoadingBundle {
    pub refset_id: String,
    pub refset_type: String,
    pub container_idx: usize,
    pub images_to_load: Vec<String>
}

// Loads initial page data and kicks off worker threads to finish image loading and refset loading
fn load_page_data(app: &mut App) -> Receiver<DImageLoaded> {
    static APP_DATA_SOURCE: &str = "https://cd-static.bamgrid.com/dp-117731241344/home.json";
    
    let (tx, rx): (Sender<DImageLoaded>, Receiver<DImageLoaded>) = mpsc::channel();
    let rows_to_load: Arc<Mutex<VecDeque<ImageLoadingBundle>>> = Arc::new(Mutex::new(VecDeque::new()));
    let resp = reqwest::blocking::get(APP_DATA_SOURCE).unwrap().text().unwrap();
    let data: serde_json::Value = serde_json::from_str(&resp).unwrap();
    let json_containers: Vec<serde_json::Value> = data["data"]["StandardCollection"]["containers"].as_array().unwrap().to_vec();
    let mut container_idx = 0;
    
    for container in json_containers {
        let title = app_gl::render_text_to_texture(&get_container_title_from_json_value(&container));
        let refset_id = get_container_refset_id_from_json_value(&container);
        let refset_type = get_container_refset_type_from_json_value(&container);
        
        let mut bundle = ImageLoadingBundle {
            refset_id,
            refset_type,
            container_idx,
            images_to_load: Vec::new()
        };
        
        container_idx += 1;
        app.containers.push(DImageRow { images: Vec::new(), title: title, selected_tile_idx: 0., desired_selected_tile_idx: 0. });
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
    for _thread_idx in 0..(num_cpus::get()-1) {
        spawn_worker_thread_for_thread_rows(&tx, &rows_to_load)
    }
    
    rx
}

fn spawn_worker_thread_for_thread_rows(tx: &Sender<DImageLoaded>, rows_to_load: &Arc<Mutex<VecDeque<ImageLoadingBundle>>>) {
    static APP_DATA_REFSET_SOURCE: &str = "https://cd-static.bamgrid.com/dp-117731241344/sets/{{id}}.json";
    
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
                            match app_gl::load_image_from_url(&client, &image_url) {
                                Ok(texture_id) => {
                                    match thread_tx.send(DImageLoaded { texture_id, container_idx: row_to_load.container_idx }) {
                                        Err(_e) => { return; }
                                        _ => {}
                                    };
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

pub struct App {
    gl: app_gl::AppGL,
    background_image_texture_id: u32,
    has_tiles_loaded: bool,
    title_height: f32,
    row_height: f32,
    tile_width: f32,
    pub selected_container_idx: usize,
    pub containers: Vec<DImageRow>,
    pub viewport: Viewport,
}

impl Default for App {
    fn default() -> Self {
        
        App {
            gl: app_gl::AppGL::default(),
            background_image_texture_id: app_gl::load_image_from_disk("res/img/background.png", 1440, 1070).unwrap(),
            has_tiles_loaded: false,
            title_height: 200.,
            row_height: 280.,
            tile_width: 625.,
            selected_container_idx: 0,
            containers: Vec::new(),
            viewport: Viewport::default()
        }
    }
}

impl Drop for App {
    fn drop(&mut self) {
        self.containers.clear();
        app_gl::release_texture(self.background_image_texture_id);
    }
}

fn handle_window_events(app: &mut App, window: &mut Window) {
    while let Some(event) = window.poll_event() {
        match event {
            Event::Closed => {
                window.close();
            },
            Event::KeyPressed { code, .. } => {
                let containers = &mut app.containers;
                match code {
                    Key::Q => {
                        window.close();
                    },
                    Key::D => {
                        if containers[app.selected_container_idx].images.len() > 0 && containers[app.selected_container_idx].desired_selected_tile_idx < (containers[app.selected_container_idx].images.len() - 1) as f32 {
                            containers[app.selected_container_idx].desired_selected_tile_idx += 1.;
                        }
                    },
                    Key::A => {
                        if containers[app.selected_container_idx].desired_selected_tile_idx > 0. {
                            containers[app.selected_container_idx].desired_selected_tile_idx -= 1.;
                        }
                    },
                    Key::W => {
                        if app.selected_container_idx >= 1 {
                            app.selected_container_idx -= 1;
                        }
                    },
                    Key::S => {
                        if app.selected_container_idx < app.containers.len() - 1 {
                            app.selected_container_idx += 1;
                        }
                    },
                    _ => {}
                }
                
            }
            _ => {}
        }
    }
    

}

fn process_tile_loads(app: &mut App, rx: &Receiver<DImageLoaded>) {
    loop {
        match rx.try_recv() {
            Ok(image_loaded) => {
                app.containers[image_loaded.container_idx].images.push( DImage { texture_id: image_loaded.texture_id, scale: 1., border: 0. });
            }
            Err(_type) => {
                break;
            }
        }
    }
}

fn update(app: &mut App, dt: f32) {
    static TILE_ZOOM_FACTOR: f32 = 1.20;
    app.viewport.desired_pos[1] = (app.title_height + app.row_height) * app.selected_container_idx as f32;
    app.viewport.pos[1] += ((app.viewport.desired_pos[1] - app.viewport.pos[1]) / 0.1) * dt;
    
    if app.containers[1].images.len() > 3 {
        app.has_tiles_loaded = true;
    }
    
    for (c_idx, container) in app.containers.iter_mut().enumerate() {
        container.selected_tile_idx += ((container.desired_selected_tile_idx - container.selected_tile_idx) / 0.1) * dt;
        
        let selected_tile_idx_i32 = container.desired_selected_tile_idx.round() as usize;
        for (idx, tile) in container.images.iter_mut().enumerate() {
            if c_idx == app.selected_container_idx && selected_tile_idx_i32 == idx {
                tile.border = 0.01;
                if tile.scale < TILE_ZOOM_FACTOR {
                    tile.scale = util::clamp(tile.scale + dt, 1., TILE_ZOOM_FACTOR);
                }
            } else {
                tile.border = 0.;
                if tile.scale > 1. {
                    tile.scale = util::clamp(tile.scale - dt, 1., TILE_ZOOM_FACTOR);
                }
            }
        }
    }
}

fn main() {
    static WINDOW_SIZE: (u32, u32) = (1920, 1080);
    static WINDOW_FPS: u32 = 200;
    
    // Creates GL context internally
    let mut window = Window::new(WINDOW_SIZE, "SFML Example", Style::CLOSE, &Default::default());
    window.set_framerate_limit(WINDOW_FPS);
    
    let mut app = App::default();
    let loader_rx = load_page_data(&mut app);
    let mut last = Instant::now();
    
    while window.is_open() {
        let current = Instant::now();
        let dt = (current - last).as_secs_f32();
        last = current;
        
        handle_window_events(&mut app, &mut window);
        process_tile_loads(&mut app, &loader_rx);
        update(&mut app, dt);
        
        window.set_active(true);
        
        app_gl::render(&app, &WINDOW_SIZE);
        
        window.display();
    }
}
