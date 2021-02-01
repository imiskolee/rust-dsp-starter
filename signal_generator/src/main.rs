use glib::{glib_wrapper, WeakRef};

mod widgets;
mod generator;

pub use generator::*;

use std::env::args;
use crate::widgets::signal_control::SignalControlWidget;
use std::rc::Rc;
use glib::clone::Downgrade;
use gio::ApplicationExt;
use gtk::{GtkWindowExt, WidgetExt};
use gio::prelude::ApplicationExtManual;

fn main() {
    let application = gtk::Application::new(
        Some("co.noiselabs.dsp-starter"),
        Default::default(),
    ).expect("Initialization failed...");


    let control = SignalControlWidget::new();

    let control_clone = control.clone();

    application.connect_activate( move |app| {
        let width = 500;
        let height = 500;
        let window = gtk::ApplicationWindow::new(app);
        window.set_title("Signal Generator V0.0.1 - Misko Lee");
        window.set_default_size(width, height + 300);
        SignalControlWidget::init_view(control_clone.clone(),glib::ObjectExt::downgrade(&window));
        window.show_all();
    });
    application.run(&args().collect::<Vec<_>>());
}
