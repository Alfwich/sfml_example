use sfml::window::{Window, Event, Style};

fn main() {
    let mut window = Window::new((800, 600), "SFML Example", Style::CLOSE, &Default::default());
    window.set_framerate_limit(60);
    
    while window.is_open() {
        while let Some(event) = window.poll_event() {
            if event == Event::Closed {
                window.close();
            }
        }
        
        window.set_active(true);
        
        window.display();
    }
    
}
