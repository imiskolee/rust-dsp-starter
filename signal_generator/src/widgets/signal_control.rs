#[macro_use]
use gtk::{Label, Scale};
use std::rc::{Rc, Weak};


#[macro_use]
use glib::prelude::*;
#[macro_use]
use gtk::subclass::prelude::*;
use glib::prelude::*;
use std::cell::{RefCell, Cell};
use glib::{glib_wrapper, WeakRef};
use gtk::{Orientation, ContainerExt, WidgetExt, RangeExt, Widget, Container, ButtonExt, BuildableExt, ApplicationWindow, DrawingArea, Inhibit};
use glib::clone::Downgrade;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::borrow::Borrow;

use super::super::generator::{sine,Signal};
use cairo::Context;
use plotters_cairo::CairoBackend;
use plotters::chart::ChartBuilder;
use plotters::drawing::IntoDrawingArea;
use plotters::style::{WHITE, RED};
use plotters::series::LineSeries;
use plotters::element::PathElement;
use plotters::prelude::IntoLinspace;


type ModelRef<T> = Rc<RefCell<T>>;

#[derive(Debug)]
pub struct SignalControlWidgetModel {
    pub sample_rate: usize,
    pub frequency: f64,
    pub amplitude: f64,
    pub length: usize,
}


impl Default for SignalControlWidgetModel {
    fn default() -> Self {
        Self {
            sample_rate: 44100,
            frequency: 440f64,
            length:1024,
            amplitude:2 as f64,
        }
    }
}

impl SignalControlWidgetModel {
    pub fn to_sine(&self)-> Signal {
        return sine(self.length,self.frequency as f32,self.sample_rate,self.amplitude as f32);
    }
}


pub struct SignalControlWidget {
    list_data: Vec<ModelRef<SignalControlWidgetModel>>,
    items: Vec<Rc<RefCell<SignalControlWidgetItem>>>,
    plot: Rc<RefCell<Option<SignalPlotWidget>>>,
}

impl SignalControlWidget {
    pub fn new() -> Rc<RefCell<Self>> {
        let a:Vec<ModelRef<SignalControlWidgetModel>> = Vec::new();
        let b:  Vec<Rc<RefCell<SignalControlWidgetItem>>> = Vec::new();
         Rc::new(RefCell::new(SignalControlWidget{
             list_data:a,
             items: b,
             plot: Default::default()
         }))
    }

    pub fn init_view(self_ : Rc<RefCell<Self>>,parent: WeakRef<ApplicationWindow>) {

        let container = gtk::Box::new(Orientation::Vertical,20);
        let item_container = gtk::Box::new(Orientation::Vertical,20);
        let self_ref = self_.downgrade();
        container.add(&item_container);
        {
            let add_button = gtk::Button::new();
            add_button.add(&gtk::Label::new(Some("+ Add")));
            add_button.connect_clicked(move |_| {
                Self::add_child(self_ref.upgrade().unwrap(),glib::ObjectExt::downgrade(&item_container));
            });
            container.add(&add_button);
        }

        let plot = Rc::new(RefCell::new(Some(SignalPlotWidget::new())));
        container.add(&plot.clone().borrow_mut().as_ref().unwrap().drawing_area);
        self_.borrow_mut().plot = plot;
        parent.upgrade().unwrap().add(&container);

    }

    fn add_child(self_ : Rc<RefCell<Self>>,parent: WeakRef<gtk::Box>) {
        let new_model:SignalControlWidgetModel = SignalControlWidgetModel::default();
        let ref_model = Rc::new(RefCell::new(new_model));
        self_.borrow_mut().list_data.push(ref_model.clone());
        let child = SignalControlWidgetItem::new(ref_model.clone());
        child.borrow_mut().init_view(glib::ObjectExt::downgrade(&parent.upgrade().unwrap().upcast::<gtk::Container>()));
        let self_clone = self_.clone();

        let listen_handler = move ||{
            let mut sum : Option<Signal> = None;
            for (i,x) in self_clone.borrow_mut().list_data.iter().map(|v|{
                return v.borrow_mut().to_sine();
            }).enumerate() {
                match sum {
                    Some(s) => sum = Some(s +x),
                    None => sum = Some(x)
                }
            }

            if sum.is_some() {
                let s = sum.unwrap();
                self_clone.borrow_mut().plot.borrow_mut().as_mut().unwrap().update_signal(s);
            }
        };

        listen_handler();
        child.borrow_mut().connect_value_change(listen_handler);

        self_.borrow_mut().items.push(child);

    }
}

pub struct SignalControlWidgetItem {
    model: ModelRef<SignalControlWidgetModel>,
    _self: Weak<RefCell<Self>>,
    listener: Box<dyn Fn()>
}

impl SignalControlWidgetItem {

    pub fn new(model: ModelRef<SignalControlWidgetModel>) -> Rc<RefCell<Self>> {
        let mut item = Rc::new(RefCell::new(Self {
            model,
            listener:Box::new( ||{}),
            _self: Default::default(),
        }));
        item.borrow_mut()._self = Rc::downgrade(&item);
        return item;
    }

    pub fn init_view(&self,parent: WeakRef<Container>) {
        let row = gtk::Box::new(Orientation::Horizontal,20);
        row.set_property_height_request(40);
        {
            let model = self.model.clone();
            let label = gtk::Label::new(Some("Freq:"));
            row.add(&label);
            let scale = gtk::Scale::with_range(Orientation::Horizontal, 220 as f64, (220 * 8) as f64, 10 as f64);
            scale.set_property_width_request(200);
            scale.set_value(model.borrow_mut().frequency);
            let  callback = self._self.clone();
            scale.connect_value_changed(move |v|  {
                model.borrow_mut().frequency = v.get_value() as f64;
                callback.upgrade().unwrap().borrow_mut().listener.as_ref()();
            });
            row.add(&scale);
        }
        {
            let model = self.model.clone();
            let label = gtk::Label::new(Some("Rate:"));
            row.add(&label);
            let scale = gtk::Scale::with_range(Orientation::Horizontal, 9600 as f64, (44100) as f64, 200 as f64);
            scale.set_property_width_request(200);
            scale.set_value(model.borrow_mut().sample_rate as f64);
            let  callback = self._self.clone();
            scale.connect_value_changed(move |v|{
                model.borrow_mut().sample_rate = v.get_value() as usize;
                callback.upgrade().unwrap().borrow_mut().listener.as_ref()();
            });
            row.add(&scale);
        }
        {
            let model = self.model.clone();
            let label = gtk::Label::new(Some("Amp:"));
            row.add(&label);
            let scale = gtk::Scale::with_range(Orientation::Horizontal, 10 as f64, (100) as f64, 5 as f64);
            scale.set_property_width_request(200);
            scale.set_value(model.borrow_mut().amplitude as f64);
            let  callback = self._self.clone();
            scale.connect_value_changed(move |v|{
                model.borrow_mut().amplitude = v.get_value() as f64;
                callback.upgrade().unwrap().borrow_mut().listener.as_ref()();
            });
            row.add(&scale);
        }
        parent.upgrade().unwrap().add(&row);
        parent.upgrade().unwrap().show_all();
    }

    pub fn connect_value_change(&mut self,f: impl Fn() + 'static) {
        self.listener =Box::new(f);
    }
}


struct SignalPlotWidget {
    signal: Rc<RefCell<Option<Signal>>>,
    drawing_area:DrawingArea,
}

impl SignalPlotWidget {
    pub fn new() -> Self {
        let drawing_area = Box::new(DrawingArea::new)();
        drawing_area.set_property_height_request(500);
        drawing_area.set_property_width_request(500);
        let widget = SignalPlotWidget{signal:Rc::new(RefCell::new(None)),drawing_area:drawing_area };
        widget.init_view();
        return widget;
    }

    pub fn init_view(&self) {
        let signal = self.signal.clone();
        self.drawing_area.connect_draw(move |area: & DrawingArea, cr: &Context| {

            SignalPlotWidget::draw(&signal.borrow_mut().as_ref(),area, cr);
            return Inhibit(false);
        });
    }

    pub fn update_signal(&mut self,signal:Signal) {
        *self.signal.borrow_mut() = (Some(signal));
        self.drawing_area.queue_draw();
    }

    pub fn draw(signal:&Option<&Signal>,area:&DrawingArea,cr:&Context) {
        if signal.is_none() {
            return;
        }

        let root = CairoBackend::new(cr, (500, 500)).unwrap().into_drawing_area();
        root.fill(&WHITE).unwrap();

        let root = root.margin(25, 25, 25, 25);
        let x_axis = (0..10000).step(1);

        let mut cc = ChartBuilder::on(&root)
            .margin(5)
            .set_all_label_area_size(50)
            .caption("Panel", ("sans-serif", 40))
            .build_cartesian_2d(0f32..1024f32, -50f32..50f32).unwrap();

        cc.configure_mesh()
            .x_labels(20)
            .y_labels(10)
            .disable_mesh()
            .x_label_formatter(&|v| format!("{:.1}", v))
            .y_label_formatter(&|v| format!("{:.1}", v))
            .draw().unwrap();

        let mut idx = 0;
        cc.draw_series(LineSeries::new(signal.unwrap().data.iter().map(|x| {
            idx +=1;
            (idx as f32,*x)}),&RED)).unwrap()
            .label("Sine")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));
    }

}

