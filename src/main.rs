use fltk::{
    prelude::*,
    app::App,
    enums::Shortcut,
    frame::Frame,
    group::{Pack, PackType},
    input::IntInput,
    menu,
    menu::MenuFlag,
    window::Window
};

use jset_desk::rgb;

const CTRL_ROW_HEIGHT: i32 = 24;

fn main() {
    let a = App::default();
    let mut ctrl_win = Window::default().with_size(400, 400);
    
    let mut ctrl_col = Pack::default_fill().with_type(PackType::Vertical);
    ctrl_col.set_spacing(6);
    
    let width_row = Pack::default_fill().with_type(PackType::Horizontal)
            .with_size(0, CTRL_ROW_HEIGHT);
    let _width_label = Frame::default().with_label("Width").with_size(60, 0);
    let mut width_input = IntInput::default().with_size(60,0);
    width_input.set_value("1200");
    width_row.end();
    
    let height_row = Pack::default_fill().with_type(PackType::Horizontal)
        .with_size(0, CTRL_ROW_HEIGHT);
    let _height_label = Frame::default().with_label("Height").with_size(60, 0);
    let mut height_input = IntInput::default().with_size(60, 0);
    height_input.set_value("800");
    height_row.end();
    
    let iter_row = Pack::default_fill().with_type(PackType::Horizontal)
        .with_size(0, CTRL_ROW_HEIGHT);
    let _iter_label = Frame::default().with_label("Iterator").with_size(80, 0);
    let mut iter_input = menu::Choice::default().with_size(120, 0);
    iter_input.add("Mandlebrot", Shortcut::None, MenuFlag::Normal,
                    |_| println!("Selected `Mandlebrot` iterator.")
    );
    iter_input.add("Polynomial", Shortcut::None, MenuFlag::Normal,
                    |_| println!("Selected `Polynomial` iterator.")
    );
    iter_input.set_value(0);
    
    iter_row.end();
    
    ctrl_col.end();
    
    ctrl_win.show();
    
    iter_input.set_callback(|c| {
        if let Some(choice) = c.choice() {
            println!("Selected `{}` iterator.", &choice.as_str());
        } else {
            println!("Selected nothing?");
        }
    });
    
    let mut _cm_pane = rgb::Pane::default();
    
    a.run().unwrap();
}
