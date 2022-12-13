
use std::f64::consts::PI;

use cgmath::{Vector3};
use egui::{Context, Vec2, FontId, FontFamily::{Proportional, self}, TextStyle, Rect, ScrollArea, RichText, Id, collapsing_header::CollapsingState, plot::{PlotPoints, Line}};
use winit::{window::Window};

use crate::{orbitals::{ALLOWED_ORBITALS, Orbital, self, orbital_to_name}};

#[derive()]
pub struct Guindow {
    pub enabled: bool,

    pub window_size: (f32, f32),

    pub new_resolution: f32,
    pub resolution: f32,
    pub new_size: f32,
    pub size: f32,

    pub orbitals: Vec<Orbital>,

    pub status: bool,
    pub submit_success: bool,
}

pub trait Gui {
    fn name(&self) -> &'static str;

    fn show(&mut self, ctx: &Context);

    fn recter(&mut self, x_pos: f32, y_pos: f32, x_size: f32, y_size: f32) -> Rect;
    fn vecter(&mut self, x_size: f32, y_size: f32) -> Vec2;
}
impl Guindow {
    pub fn new(window: &Window) -> Self {
        let window_size = (window.inner_size().width as f32, window.inner_size().height as f32);
        let orbitals = vec![orbitals::Orbital::new(Vector3::new(0.0, 0.0, 0.0), (0.0, 0.0, 0.0), (0, 0), 0,true); 2];

        Self {window_size, orbitals, enabled: false, size: 6.0, new_size: 6.0, new_resolution: 5.0, resolution: 5.0, status: false, submit_success: false}
    }
}
impl Gui for Guindow {
    fn name(&self) -> &'static str {
        "gui"
    }
    fn show(&mut self, ctx: &Context) {

    //Creates all text styles – their size is proportional to the monitor resolution, so they should loox fine in every device.
        let mut style = (*ctx.style()).clone();
        style.text_styles = [(TextStyle::Heading, FontId::new(self.window_size.1 / 15.0, Proportional)),
        (TextStyle::Name("Heading2".into()), FontId::new(self.window_size.1 / 20.0, Proportional)),
        (TextStyle::Name("Context".into()), FontId::new(23.0, Proportional)),
        (TextStyle::Name("Play Button".into()), FontId::new(self.window_size.1 / 10.0, egui::FontFamily::Proportional)),
        (TextStyle::Name("Reload Button".into()), FontId::new(self.window_size.1 / 12.0, egui::FontFamily::Proportional)),
        (TextStyle::Name("Input".into()), FontId::new(self.window_size.1 / 28.0, egui::FontFamily::Monospace)),
        (TextStyle::Body, FontId::new(self.window_size.1 / 28.0, Proportional)),
        (TextStyle::Monospace, FontId::new(self.window_size.1 / 40.0, egui::FontFamily::Monospace)),
        (TextStyle::Button, FontId::new(self.window_size.1 / 21.0, egui::FontFamily::Monospace)),
        (TextStyle::Small, FontId::new(self.window_size.1 / 40.0, Proportional)),].into();

    //Updates the play button
        //let status_symbol: char;
        //if self.status == false {status_symbol = '\u{23f5}';} else {status_symbol = '\u{23f8}';}

        let allowed_orbitals: (Vec<&str>, Vec<(u8, u8)>) = ALLOWED_ORBITALS.to_vec().into_iter().unzip();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_enabled_ui(self.enabled, |ui|{
                ui.set_style(style.clone());
            
            //Title!!!
                ui.put(
                self.recter(0.5, 0.065, 0.9, 0.08),
                 egui::Label::new(RichText::new("Orbital Simulation").text_style(TextStyle::Heading)));
                ui.separator();
            
            //General settings
                ui.allocate_ui_at_rect(self.recter(0.5, 0.215, 0.9, 0.13), |ui| {
                    egui::Grid::new("top_grid").show(ui, |ui| {
                        
                    //Resolution
                        //Creates a Drag value to input a new resolution
                        ui.label(RichText::new("Resolution: ").text_style(TextStyle::Body));
                        ui.add(egui::DragValue::new(&mut self.new_resolution).speed(0.0).fixed_decimals(0));

                        //This creates a upper and lower limit on resolution
                        if self.new_resolution > 11.0 {self.new_resolution = 11.0} else if self.new_resolution < 2.0 {self.new_resolution = 2.0}

                        //Tiny button to submit resolution
                        if ui.small_button('\u{2ba8}'.to_string()).clicked() {
                            self.resolution = self.new_resolution}

                        //Displays current res
                        ui.label(String::from("(Current: ") + &self.resolution.to_string() + ")");
                        ui.end_row();
                        ui.end_row();

                    //Size
                        //Creates a Drag value to input a new size
                        ui.label(RichText::new("Size: ").text_style(TextStyle::Body));
                        ui.add(egui::DragValue::new(&mut self.new_size).speed(0.0).fixed_decimals(0));

                        //This creates a upper and lower limit on size
                        if self.new_size > 20.0 {self.new_size = 20.0} else if self.new_size < 1.0 {self.new_size = 1.0}

                        //Tiny button to submit size
                        if ui.small_button('\u{2ba8}'.to_string()).clicked() {
                            self.size = self.new_size}

                        //Displays current size
                        ui.label(String::from("(Current: ") + &self.size.to_string() + ")");
                        
                    });
                });

                ui.separator();

                ui.put(self.recter(0.2, 0.345, 0.3, 0.08), egui::Label::new(
                    RichText::new("Orbitals").text_style(TextStyle::Name("Heading2".into()))));
                
            //Orbitals menu
                ui.allocate_ui_at_rect(self.recter(0.5, 0.68, 0.8, 0.59), |ui| {
                    ScrollArea::vertical().always_show_scroll(false)
                    .max_height(self.window_size.1 / (1.0 / 0.55)).show(ui, |ui| {
                    
                    //This creates a collapsing header with info for each orbital
                        self.orbitals.clone().iter().enumerate().for_each(|orbital| {
                        
                        //Creates an ID so all elements are unique
                            let id = &orbital.0.to_string();

                        //Creates the collapsing header with a selectable title
                            egui::collapsing_header::CollapsingState::show_header(CollapsingState::load_with_default_open(ctx, Id::new(
                            String::from("collapsing ") + id), true), ui, |ui|{
                            //Selectable title – Changes the orbital's quantum acording to what you chose
                                egui::ComboBox::from_id_source(String::from("combo ") + id).selected_text(RichText::new(orbital_to_name(orbital.1.quantum)).text_style(TextStyle::Body)).show_ui(ui, |ui| {
                                    allowed_orbitals.1.to_vec().into_iter().for_each(|valid_quantum| {
                                        ui.selectable_value(&mut self.orbitals[orbital.0].quantum, valid_quantum, orbital_to_name(valid_quantum));
                                    })
                                });
                            }).body(|ui| {
                            //Inside the collapsing header: creates a menu from which you can change the orbital's parameters.
                                egui::Grid::new(String::from("grid ") + id).striped(true).show(ui, |ui| {
                                    
                                //This creates a graph that matches the selected orbital
                                    ui.collapsing(RichText::new("Graph: ").text_style(TextStyle::Small), |ui|{
                                        let n = 128;
                                        let line_points: PlotPoints = (0..=n)
                                            .map(|i| {
                                                let x = egui::remap(i as f64, 0.0..=n as f64, 0.0..=10.0);
                                                [x, match orbital.1.quantum { //All of the formulae for each orbital
                                                    (1, 0) => {((1.0 / (PI).sqrt()) * (-x).exp()).powi(2) * 4.0 * PI * x.powi(2)}
                                                    (2, 0) => {((1.0 / (PI).sqrt()) * (-x).exp() * (1.0 - x)).powi(2) * 4.0 * PI * x.powi(2)}
                                                    (2, 1) => {((1.0 / (4.5 * PI).sqrt()) * (-x).exp() * x).powi(2) * 4.0 * PI * x.powi(2)}
                                                    (3, 0) => {((1.0 / (32.0 * PI).sqrt()) * (-x).exp() * (6.0 - 12.0 * x + (2.0 * x).powi(2))).powi(2) * 4.0 * PI * x.powi(2)}
                                                    (3, 1) => {((1.0 / (4.5 * PI).sqrt()) * (-x).exp() * x * (2.0 - x)).powi(2) * 4.0 * PI * x.powi(2)}
                                                    (3, 2) => {((1.0 / (31.0 * PI).sqrt()) * (-x).exp() * x.powi(2)).powi(2) * 4.0 * PI * x.powi(2)}
                                                    _ => {0.0}
                                                }]
                                            }
                                        ).collect();
                                        let line = Line::new(line_points);
                                        egui::plot::Plot::new("orbital_graph")
                                            .height(self.vecter(0.6, 0.2).y)
                                            .width(self.vecter(0.6, 0.2).x)
                                            .data_aspect(4.0).allow_scroll(false)
                                            .show(ui, |ui| {ui.line(line)})
                                            .response;
                                        ui.small("Left click to drag, ctrl + scroll to zoom");
                                    });
                                    ui.end_row();
                                    
                                //Position row - here you can change the orbital's position
                                    ui.horizontal( |ui| {
                                        ui.small(RichText::new("Position: ").family(FontFamily::Monospace));

                                        ui.add_sized(self.vecter(0.05, 0.04), egui::DragValue::new(&mut self.orbitals[orbital.0].position.x).speed(0).max_decimals(1).suffix(" Å"));
                                        ui.add_sized(self.vecter(0.05, 0.04), egui::DragValue::new(&mut self.orbitals[orbital.0].position.y).speed(0).max_decimals(1).suffix(" Å"));
                                        ui.add_sized(self.vecter(0.05, 0.04), egui::DragValue::new(&mut self.orbitals[orbital.0].position.z).speed(0).max_decimals(1).suffix(" Å"));
                                        ui.label("        ");
                                    });
                                    ui.end_row();

                                //Rotation row - here you can change the orbital's rotation, in euler angles
                                    ui.horizontal( |ui|{
                                        ui.small(RichText::new("Rotation: ").family(FontFamily::Monospace));

                                        ui.add_sized(self.vecter(0.05, 0.04), egui::DragValue::new(&mut self.orbitals[orbital.0].euler.0).speed(0).max_decimals(1).suffix("°"));
                                            if self.orbitals[orbital.0].euler.0 >= 360.0 {self.orbitals[orbital.0].euler.0 = self.orbitals[orbital.0].euler.0 % 360.0}
                                            else if self.orbitals[orbital.0].euler.0 < 0.0 {self.orbitals[orbital.0].euler.0 = self.orbitals[orbital.0].euler.0 % 360.0 + 360.0}

                                        ui.add_sized(self.vecter(0.05, 0.04), egui::DragValue::new(&mut self.orbitals[orbital.0].euler.1).speed(0).max_decimals(1).suffix("°"));
                                            if self.orbitals[orbital.0].euler.1 >= 360.0 {self.orbitals[orbital.0].euler.1 = self.orbitals[orbital.0].euler.1 % 360.0}
                                            else if self.orbitals[orbital.0].euler.1 < 0.0 {self.orbitals[orbital.0].euler.1 = self.orbitals[orbital.0].euler.1 % 360.0 + 360.0}

                                        ui.add_sized(self.vecter(0.05, 0.04), egui::DragValue::new(&mut self.orbitals[orbital.0].euler.2).speed(0).max_decimals(1).suffix("°"));
                                            if self.orbitals[orbital.0].euler.2 >= 360.0 {self.orbitals[orbital.0].euler.2 = self.orbitals[orbital.0].euler.2 % 360.0}
                                            else if self.orbitals[orbital.0].euler.2 < 0.0 {self.orbitals[orbital.0].euler.2 = self.orbitals[orbital.0].euler.2 % 360.0 + 360.0}

                                    //This converts the input angles to a quaternion, which asigns a unique value per angle combination
                                        let cr = (self.orbitals[orbital.0].euler.0/360.0 * PI as f32).cos();
                                        let sr = (self.orbitals[orbital.0].euler.0/360.0 * PI as f32).sin();
                                        let cp = (self.orbitals[orbital.0].euler.1/360.0 * PI as f32).cos();
                                        let sp = (self.orbitals[orbital.0].euler.1/360.0 * PI as f32).sin();
                                        let cy = (self.orbitals[orbital.0].euler.2/360.0 * PI as f32).cos();
                                        let sy = (self.orbitals[orbital.0].euler.2/360.0 * PI as f32).sin();

                                        self.orbitals[orbital.0].quaternion =  (cr * cp * cy + sr * sp * sy,
                                                                                sr * cp * cy - cr * sp * sy,
                                                                                cr * sp * cy + sr * cp * sy,
                                                                                cr * cp * sy - sr * sp * cy);
                                        });
                                    ui.end_row();

                                //Magnetic row – Here you gan change the last quantum number (when needed) so that you can get all the different orbitals
                                    if orbital.1.quantum.1 != 0 {

                                        //This is for p orbitals – the buttons don't actually change the magnetic's value, they just apply a rotation
                                            if orbital.1.quantum.1 == 1 {
                                                ui.horizontal( |ui| {
                                                    ui.small(RichText::new("Magnetic: ").family(FontFamily::Monospace));
                                                    
                                                    //px orbital
                                                    if ui.add_sized(self.vecter(0.08, 0.04), egui::Button::new(egui::RichText::new("x").text_style(TextStyle::Monospace))).clicked() {
                                                        self.orbitals[orbital.0].euler = (0.0, 0.0, 0.0)}
                                                    
                                                    //py orbital
                                                    if ui.add_sized(self.vecter(0.08, 0.04), egui::Button::new(egui::RichText::new("y").text_style(TextStyle::Monospace))).clicked() {
                                                        self.orbitals[orbital.0].euler = (0.0, 90.0, 0.0)}

                                                    //pz orbital
                                                    if ui.add_sized(self.vecter(0.08, 0.04), egui::Button::new(egui::RichText::new("z").text_style(TextStyle::Monospace))).clicked() {
                                                        self.orbitals[orbital.0].euler = (90.0, 0.0, 0.0)}
                                                });

                                        //This is for d orbitals – here you change both the magnetic's value and the rotation to create the acording orbital
                                            } else if orbital.1.quantum.1 == 2 {
                                                ui.horizontal( |ui| {
                                                    ui.small(RichText::new("Magnetic: ").family(FontFamily::Monospace));
                                                
                                                    //dz2 orbital – this one is the weird one
                                                    if ui.add_sized(self.vecter(0.08, 0.04), egui::Button::new(egui::RichText::new("z²").text_style(TextStyle::Monospace))).clicked() {
                                                        self.orbitals[orbital.0].magnetic = 0;
                                                        self.orbitals[orbital.0].euler = (0.0, 0.0, 0.0)}

                                                    //dxy orbital
                                                    if ui.add_sized(self.vecter(0.08, 0.04), egui::Button::new(egui::RichText::new("xy").text_style(TextStyle::Monospace))).clicked() {
                                                        self.orbitals[orbital.0].magnetic = 1;
                                                        self.orbitals[orbital.0].euler = (0.0, 0.0, 0.0)}

                                                    //dxz orbital
                                                    if ui.add_sized(self.vecter(0.08, 0.04), egui::Button::new(egui::RichText::new("xz").text_style(TextStyle::Monospace))).clicked() {
                                                        self.orbitals[orbital.0].magnetic = 1;
                                                        self.orbitals[orbital.0].euler = (90.0, 0.0, 90.0)}

                                                    //dyz orbital
                                                    if ui.add_sized(self.vecter(0.08, 0.04), egui::Button::new(egui::RichText::new("yz").text_style(TextStyle::Monospace))).clicked() {
                                                        self.orbitals[orbital.0].magnetic = 1;    
                                                        self.orbitals[orbital.0].euler = (90.0, 90.0, 0.0)}

                                                    //dx2-y2 orbital
                                                    if ui.add_sized(self.vecter(0.08, 0.04), egui::Button::new(egui::RichText::new("x²-y²").text_style(TextStyle::Monospace))).clicked() {
                                                        self.orbitals[orbital.0].magnetic = 1;    
                                                        self.orbitals[orbital.0].euler = (135.0, 0.0, 0.0)}
                                                });
                                            }
                                            if orbital.1.quantum.1 == 2 {
                                                
                                            }
                                        ui.end_row();
                                    }
                                
                                //Phase row – Here you change the orbital's phase with a button, nothing special
                                    ui.horizontal(|ui| {
                                        ui.small(RichText::new("Phase:    ").family(FontFamily::Monospace));
                                        if ui.add_sized(self.vecter(0.08, 0.04), egui::Button::new(if self.orbitals[orbital.0].phase == true {egui::RichText::new("+").text_style(TextStyle::Monospace)} else {egui::RichText::new("-").text_style(TextStyle::Monospace)})).clicked() {
                                            self.orbitals[orbital.0].phase = !self.orbitals[orbital.0].phase;
                                    }});

                                    ui.end_row();
                                })
                            });
                        });
                    });
                });

                /* if ui.put(self.recter(0.2625, 0.9, 0.425, 0.1),
                    egui::Button::new(RichText::new(status_symbol).text_style(TextStyle::Name("Play Button".into())).family(FontFamily::Proportional))).clicked() 
                    {self.status = !self.status};
                
                if ui.put(self.recter(0.7375, 0.9, 0.425, 0.1),
                    egui::Button::new(RichText::new('\u{27f3}').text_style(TextStyle::Name("Reload Button".into())))).clicked() 
                    {}; */

            });
        });
    }
//RECTER - This function translates a x and y position + a size into a coordinate which scales depending on the resolution
    fn recter (&mut self, x_pos: f32, y_pos: f32, x_size: f32, y_size: f32) -> Rect {
        let rectangle: Rect;
        rectangle = Rect::from_center_size(
            egui::pos2(self.window_size.0 / (1.0 / x_pos), self.window_size.1 / (1.0 / y_pos)),
            Vec2::new(self.window_size.0 / (1.0 / x_size), self.window_size.1 / (1.0 / y_size))
        );
        return rectangle;
    }
//VECTER – Like the RECTER, except this only needs a size input
    fn vecter (&mut self, x_size: f32, y_size: f32) -> Vec2 {
        let size: Vec2;
        size = Vec2::new(self.window_size.0 / (1.0 / x_size), self.window_size.1 / (1.0 / y_size));
        return size;
    }
}