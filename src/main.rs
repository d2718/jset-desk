use fltk::{
    prelude::*,
    button::{Button},
    enums,
    enums::Align,
    app::App,
    frame::Frame,
    group::{Pack, PackType},
    input::IntInput,
    menu,
    valuator,
    window::Window
};

const CTRL_ROW_HEIGHT: i32 = 24;

fn pick_color(rinit: u8, ginit: u8, binit: u8) {
    let mut w = Window::default().with_size(400, 400);
    let mut col = Pack::default_fill();
    col.set_spacing(10);
    
    let mut r_row = Pack::default_fill()
        .with_type(PackType::Horizontal).with_size(0, 40);
    let mut r_slider = valuator::HorNiceSlider::default().with_size(200, 0);
    r_slider.set_minimum(0.0);
    r_slider.set_maximum(255.0);
    r_slider.set_step(1.0, 1);
    r_slider.set_value(rinit.into());
    let mut r_number = IntInput::default().with_size(80, 0);
    r_number.set_value(&format!("{}",rinit));
    r_row.end();
    
    let mut end_row = Pack::default_fill()
        .with_type(PackType::Horizontal).with_size(0, 40);
    let mut cbutt  = Button::default().with_size(200, 0).with_label("cancel");
    let mut okbutt = Button::default().with_size(200, 0).with_label("set");
    end_row.end();
    
    col.end();
    w.end();
    w.show();
}

fn main() {
    let a = App::default();
    let mut ctrl_win = Window::default().with_size(400, 400);
    
    let mut ctrl_col = Pack::default_fill().with_type(PackType::Vertical);
    ctrl_col.set_spacing(6);
    
    let mut width_row = Pack::default_fill().with_type(PackType::Horizontal)
            .with_size(0, CTRL_ROW_HEIGHT);
    let width_label = Frame::default().with_label("Width").with_size(60, 0);
    let mut width_input = IntInput::default().with_size(60,0);
    width_input.set_value("1200");
    width_row.end();
    
    let mut height_row = Pack::default_fill().with_type(PackType::Horizontal)
        .with_size(0, CTRL_ROW_HEIGHT);
    let height_label = Frame::default().with_label("Height").with_size(60, 0);
    let mut height_input = IntInput::default().with_size(60, 0);
    height_input.set_value("800");
    height_row.end();
    
    let mut iter_row = Pack::default_fill().with_type(PackType::Horizontal)
        .with_size(0, CTRL_ROW_HEIGHT);
    let iter_label = Frame::default().with_label("Iterator").with_size(80, 0);
    let mut iter_input = menu::Choice::default().with_size(120, 0);
    iter_input.add("Mandlebrot", enums::Shortcut::None,
                    menu::MenuFlag::Normal,
                    |_| println!("Selected `Mandlebrot` iterator.")
    );
    iter_input.add("Polynomial", enums::Shortcut::None,
                    menu::MenuFlag::Normal,
                    |_| println!("Selected `Polynomial` iterator.")
    );
    iter_input.set_value(0);
    
    iter_row.end();
    
    let mut butt = Button::default().with_size(0, CTRL_ROW_HEIGHT)
        .with_label("color test");
    butt.set_callback(move |_| pick_color(0, 0, 0));
    
    ctrl_col.end();
    
    ctrl_win.show();
    
    iter_input.set_callback(|c| {
        if let Some(choice) = c.choice() {
            println!("Selected `{}` iterator.", &choice.as_str());
        } else {
            println!("Selected nothing?");
        }
    });
    
    a.run().unwrap();
}
