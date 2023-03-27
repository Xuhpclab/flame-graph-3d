use std::sync::Arc;

use super::{App, ClearOptions, CubePainter};
use super::{Graph3D, OptionsMenu};
use crate::app::data::*;
use crate::app::tree::*;
use crate::app::ui_helper::*;

use cgmath::{Deg, Euler};
use egui::mutex::Mutex;
use wasm_bindgen::prelude::wasm_bindgen;

lazy_static! {
    static ref DARK_MODE: Mutex<bool> = Mutex::new(true);
}
#[derive(Debug, Clone, PartialEq)]
pub enum RegenWhat {
    Left,
    Right,
    Inspector,
    All,
}
#[derive(Debug, Clone, PartialEq)]

pub enum WhatGraph {
    LeftGraph,
    RightGraph,
    Inspector,
}
impl Graph3D {
    pub fn interactive_view(&mut self, ui: &mut egui::Ui) {
        let (rect, response) = ui.allocate_exact_size(
            egui::Vec2::splat(ui.available_size().y.min(ui.available_size().x)),
            egui::Sense::union(egui::Sense::drag(), egui::Sense::hover()),
        );

        if ui.input().modifiers.shift {
            self.view_options.displacement += cgmath::vec3(
                response.drag_delta().x * 0.01,
                response.drag_delta().y * -0.01,
                0.0,
            );
        } else {
            self.view_options.angle.x += Deg(response.drag_delta().y * 0.2);
            self.view_options.angle.y += Deg(response.drag_delta().x * 0.2);
        }
        if response.hovered() {
            let x = ui.input().scroll_delta.y;
            self.view_options.zoom += x;
            self.view_options.zoom = self.view_options.zoom.max(0.01);
        }
        ui.horizontal(|ui| {
            ui.label(self.view_options.angle.x.0.to_string());
            ui.label(self.view_options.angle.y.0.to_string());
            ui.label(self.view_options.angle.z.0.to_string());
        });
        ui.horizontal(|ui| {
            ui.label(self.view_options.displacement.x.to_string());
            ui.label(self.view_options.displacement.y.to_string());
            ui.label(self.view_options.displacement.z.to_string());
        });
        // Clone locals so we can move them into the paint callback:
        let angle: Euler<Deg<f32>> = self.view_options.angle;
        let cube_painter = self.painter.clone();
        let m = cgmath::perspective(cgmath::Rad(1.5), 1.0, 0.01, 20.0)
            * cgmath::Matrix4::from_translation(self.view_options.displacement)
            * cgmath::Matrix4::from_scale((self.view_options.zoom + 1000.0) / 1000.0)
            * cgmath::Matrix4::from(angle);

        let callback = egui::PaintCallback {
            rect,
            callback: std::sync::Arc::new(egui_glow::CallbackFn::new(move |_info, painter| {
                cube_painter.lock().paint(painter.gl(), m);
            })),
        };
        if ui.add(egui::Button::new("Reset button")).clicked() {
            self.view_options = Default::default();
        }
        ui.painter().add(callback);
    }

    pub fn topdown_view(
        &mut self,
        ui: &mut egui::Ui,
        _frame: &mut eframe::Frame,
        inspector: &mut OptionsMenu,
    ) {
        // Clone locals so we can move them into the paint callback:
        //TODO maybe at some point have the graphs rotate based on what metrics are tracked for visual clarity
        // let angle = match self.mesh_options.across_metric {
        //     AcrossMetric::Time => Euler {
        //         x: Deg(0.0),
        //         y: Deg(-90.0),
        //         z: Deg(-90.0),
        //     },
        //     AcrossMetric::Thread => Euler {
        //         x: Deg(-90.0),
        //         y: Deg(00.0),
        //         z: Deg(180.0),
        //     },
        // };
        let angle = Euler {
            x: Deg(0.0),
            y: Deg(-90.0),
            z: Deg(-90.0),
        };
        let m = cgmath::Matrix4::from(angle) * cgmath::Matrix4::from_scale(-1.0);
        let cube_painter = self.painter.clone();
        let (rect, response) = ui.allocate_at_least(
            egui::Vec2::splat(ui.available_size().x.min(ui.available_size().y)),
            egui::Sense::union(egui::Sense::hover(), egui::Sense::click()),
        );
        let callback = egui::PaintCallback {
            rect,
            callback: std::sync::Arc::new(egui_glow::CallbackFn::new(move |_info, painter| {
                cube_painter.lock().paint(painter.gl(), m);
            })),
        };
        if response.hover_pos().is_some() {
            let num_graphs = match self.mesh_options.across_metric {
                AcrossMetric::Time => self.mesh_options.num_graphs,
                AcrossMetric::Thread => self.mesh_options.num_threads,
            };
            let text = match self.mesh_options.across_metric {
                AcrossMetric::Time => "Select Time Slice: ",
                AcrossMetric::Thread => "Select Thread: ",
            };
            let n = overview_lookup(num_graphs, rect, response.hover_pos().unwrap());
            if n.is_some() {
                if response.clicked() {
                    let m_opt = &self.mesh_options;
                    let graph_size =
                        (m_opt.time_range.end - m_opt.time_range.start) as f64 / num_graphs as f64;
                    inspector.inspector_options.time_range = (n.unwrap() as f64 * graph_size
                        + m_opt.time_range.start as f64)
                        as u64
                        ..((n.unwrap() as f64 + 1.0) * graph_size + m_opt.time_range.start as f64)
                            as u64;
                    inspector.inspector_options.data_metric = self.mesh_options.data_metric.clone();
                    inspector.inspector_options.across_metric =
                        self.mesh_options.across_metric.clone();
                    inspector.inspector_options.num_threads = n.unwrap() - 1;
                    inspector.inspector_options.has_changed = true;
                }
                response.on_hover_text_at_pointer(text.to_string() + &(n.unwrap()).to_string());
            }
        }
        ui.painter().add(callback);
    }
}
impl App {
    pub fn inspector_veiw(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let min_size = f32::min(ui.available_size().y, ui.available_size().x);
        let (rect, response) = ui.allocate_exact_size(
            egui::Vec2::splat(min_size),
            egui::Sense::union(egui::Sense::click(), egui::Sense::hover()),
        );
        let m = cgmath::Matrix4::from_translation(cgmath::vec3(0.0, -1.0, 0.0))
            * cgmath::Matrix4::from_nonuniform_scale(
                1.0,
                1.0 / self.ui_state.inspector_height as f32,
                1.0,
            );
        if response.hover_pos().is_some() {
            let n = inspector_lookup(
                &self.inspector_mesh.verts,
                &self.inspector_graph,
                rect,
                response.hover_pos().unwrap(),
                m,
            );
            if n.is_some() {
                let u_n = n.unwrap().1;

                let percent_dur =
                    u_n.values.dur as f32 / self.inspector_graph.root.values.dur as f32 / 0.01;
                let percent_val =
                    u_n.values.value as f32 / self.inspector_graph.root.values.value as f32 / 0.01;
                if response.clicked() {
                    self.ui_state.modify_options.open = true;
                    self.ui_state.modify_options.new_popup = true;
                    self.ui_state.modify_options.node = u_n.clone();
                    self.ui_state.modify_options.window_position = response.hover_pos().unwrap();
                } else {
                    response.on_hover_text_at_pointer(
                        " Name:        ".to_owned()
                            + &u_n.name.clone()
                            + "\n Duration:     ".clone()
                            + &u_n.values.dur.to_string()
                            + "    "
                            + &percent_dur.to_string()
                            + "% of this slice"
                            + "\n Value:        ".clone()
                            + &u_n.values.value.to_string()
                            + "    "
                            + &percent_val.to_string()
                            + "% of this slice",
                    );
                }
            }
        }
        let rect_painter = self.rect_painter.clone();

        let callback = egui::PaintCallback {
            rect,
            callback: std::sync::Arc::new(egui_glow::CallbackFn::new(move |_info, painter| {
                rect_painter.lock().paint(painter.gl(), m);
            })),
        };
        ui.painter().add(callback);
    }

    pub fn set_web_options(&mut self, ctx: &egui::Context) {
        if *DARK_MODE.lock() {
            self.ui_state.visual_options = egui::Visuals::dark()
        } else {
            self.ui_state.visual_options = egui::Visuals::light()
        }
        ctx.set_visuals(self.ui_state.visual_options.clone());
    }
    pub fn modify_trace(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let window = egui::Window::new("Modify Trace")
            .default_width(600.0)
            .default_height(400.0)
            .vscroll(false)
            .resizable(false);

        let mut color = egui::epaint::Hsva::from_rgb([0.0, 0.0, 0.0]);
        let state = self.ui_state.modify_options.new_popup;
        self.ui_state.modify_options.new_popup = false;
        let pos = self.ui_state.modify_options.window_position;
        let show_window = |ui: &mut egui::Ui| {
            ui.label("Selected Trace: ".to_owned() + &self.ui_state.modify_options.node.name);

            if self.ui_state.modify_options.node.color.is_some() {
                let c = self.ui_state.modify_options.node.color.unwrap();
                color = egui::epaint::Hsva::from_rgb([c[0], c[1], c[2]]);
            }

            let alpha = egui::widgets::color_picker::Alpha::Opaque;
            let _e = egui::widgets::color_picker::color_edit_button_hsva(ui, &mut color, alpha);
            if ui.button("Apply").clicked() {
                if self.ui_state.modify_options.node.color.is_some() {
                    self.master_graph.modify_color(
                        self.ui_state.modify_options.node.name.clone(),
                        self.ui_state.modify_options.node.color.unwrap(),
                    );
                    self.regen_all(frame)
                }
            }
            ui.horizontal(|ui| {
                if ui.button("Hide others").clicked() {
                    self.master_graph.modify_color(
                        self.ui_state.modify_options.node.name.clone(),
                        self.ui_state.modify_options.node.color.unwrap(),
                    );
                    let mut left_m =
                        get_mesh_from_tree(&self.master_graph, &self.graph_left.mesh_options);
                    let mut right_m =
                        get_mesh_from_tree(&self.master_graph, &self.graph_right.mesh_options);
                    for color in left_m.colors.iter_mut() {
                        if *color != self.ui_state.modify_options.node.color.unwrap() {
                            color[3] = 0.05;
                        }
                    }
                    for color in right_m.colors.iter_mut() {
                        if *color != self.ui_state.modify_options.node.color.unwrap() {
                            color[3] = 0.05;
                        }
                    }
                    self.graph_left.painter = Arc::new(Mutex::new(CubePainter::new(
                        &frame.gl().unwrap(),
                        &left_m.verts,
                        &left_m.colors,
                    )));
                    self.graph_right.painter = Arc::new(Mutex::new(CubePainter::new(
                        &frame.gl().unwrap(),
                        &right_m.verts,
                        &right_m.colors,
                    )));
                }
                if ui.button("Darken Others").clicked() {
                    self.ui_state.modify_options.highlighting_node =
                        !self.ui_state.modify_options.highlighting_node;
                    if self.ui_state.modify_options.highlighting_node {
                        self.master_graph.modify_color(
                            self.ui_state.modify_options.node.name.clone(),
                            self.ui_state.modify_options.node.color.unwrap(),
                        );
                        let mut left_m =
                            get_mesh_from_tree(&self.master_graph, &self.graph_left.mesh_options);
                        let mut right_m =
                            get_mesh_from_tree(&self.master_graph, &self.graph_right.mesh_options);
                        for color in left_m.colors.iter_mut() {
                            if *color != self.ui_state.modify_options.node.color.unwrap() {
                                *color = [color[0] * 0.1, color[1] * 0.1, color[2] * 0.1, color[3]];
                            }
                        }
                        for color in right_m.colors.iter_mut() {
                            if *color != self.ui_state.modify_options.node.color.unwrap() {
                                *color = [color[0] * 0.1, color[1] * 0.1, color[2] * 0.1, color[3]];
                            }
                        }
                        self.graph_left.painter = Arc::new(Mutex::new(CubePainter::new(
                            &frame.gl().unwrap(),
                            &left_m.verts,
                            &left_m.colors,
                        )));
                        self.graph_right.painter = Arc::new(Mutex::new(CubePainter::new(
                            &frame.gl().unwrap(),
                            &right_m.verts,
                            &right_m.colors,
                        )));
                    } else {
                        self.regen_left_mesh(frame);
                        self.regen_right_mesh(frame);
                    }
                }
            });
        };
        if state {
            window.current_pos(pos).show(ctx, show_window);
        } else {
            window.show(ctx, show_window);
        }
        self.ui_state.modify_options.node.color =
            Some([color.to_rgb()[0], color.to_rgb()[1], color.to_rgb()[2], 1.0]);
    }
    pub fn regen_left_mesh(&mut self, frame: &mut eframe::Frame) {
        let mesh_left = get_mesh_from_tree(&self.master_graph, &self.graph_left.mesh_options);
        self.graph_left.painter = Arc::new(Mutex::new(CubePainter::new(
            &frame.gl().unwrap(),
            &mesh_left.verts,
            &mesh_left.colors,
        )));
    }
    pub fn regen(&mut self, frame: &mut eframe::Frame, what: RegenWhat) {
        if what == RegenWhat::Inspector {
            self.regen_inspector(frame);
        } else if what == RegenWhat::Left {
            self.regen_left_mesh(frame);
        } else if what == RegenWhat::Right {
            self.regen_right_mesh(frame);
        } else if what == RegenWhat::All {
            self.regen_all(frame);
        }
    }
    pub fn regen_right_mesh(&mut self, frame: &mut eframe::Frame) {
        let mesh_right = get_mesh_from_tree(&self.master_graph, &self.graph_right.mesh_options);
        self.graph_right.painter = Arc::new(Mutex::new(CubePainter::new(
            &frame.gl().unwrap(),
            &mesh_right.verts,
            &mesh_right.colors,
        )));
    }
    pub fn regen_inspector(&mut self, frame: &mut eframe::Frame) {
        self.inspector_graph = match self.ui_state.inspector_options.across_metric {
            AcrossMetric::Thread => build_thread_tree(
                &self.master_graph,
                self.ui_state.inspector_options.num_threads + 1,
            ),
            AcrossMetric::Time => build_time_tree(
                &self.master_graph,
                self.ui_state.inspector_options.time_range.start
                    ..self.ui_state.inspector_options.time_range.end,
            ),
        };

        let mesh = get_rects_from_tree(&self.inspector_graph, &self.ui_state.inspector_options);

        self.rect_painter = Arc::new(Mutex::new(CubePainter::new(
            &frame.gl().unwrap(),
            &mesh.verts,
            &mesh.colors,
        )));
        self.inspector_mesh = mesh;
    }
    pub fn regen_all(&mut self, frame: &mut eframe::Frame) {
        if self.clear_options != ClearOptions::HasBeenCleared {
            self.regen_left_mesh(frame);
            self.regen_right_mesh(frame);
            self.regen_inspector(frame);
        }
        self.clear_options = ClearOptions::HasBeenCleared
    }

    pub fn slider_num_graphs(
        &mut self,
        frame: &mut eframe::Frame,
        ui: &mut egui::Ui,
        what_graph: WhatGraph,
    ) {
        let counter;
        let what;
        if what_graph == WhatGraph::LeftGraph {
            counter = &mut self.graph_left.mesh_options.num_graphs;
            what = RegenWhat::Left;
        } else if what_graph == WhatGraph::RightGraph {
            counter = &mut self.graph_right.mesh_options.num_graphs;
            what = RegenWhat::Right;
        } else {
            panic!();
        }
        let old_counter = counter.clone();
        // Put the buttons and label on the same row:
        ui.horizontal(|ui| {
            ui.add(egui::Slider::new(counter, 1..=100).logarithmic(true));
            ui.add(egui::widgets::Label::new("Divisions"));
        });
        if old_counter != counter.clone() {
            self.regen(frame, what)
        }
    }
    pub fn metric_dropdown(
        &mut self,
        frame: &mut eframe::Frame,
        ui: &mut egui::Ui,
        what_graph: WhatGraph,
    ) {
        //there are two sets of options, one for the left graph, the other for the right
        let options;
        if what_graph == WhatGraph::LeftGraph {
            options = &mut self.graph_left.mesh_options.data_metric;
        } else if what_graph == WhatGraph::RightGraph {
            options = &mut self.graph_right.mesh_options.data_metric;
        } else {
            panic!();
        }
        let past_option = options.clone();
        ui.horizontal(|ui| {
            ui.label("Metric");
            egui::containers::ComboBox::from_label("")
                .selected_text(format!("{:?}", options))
                .show_ui(ui, |ui| {
                    ui.selectable_value(options, DataChoices::Duration, "Duration");
                    ui.selectable_value(options, DataChoices::Value, "Value");
                });
        });
        if past_option != options.clone() {
            self.regen_all(frame);
        }
    }
    pub fn across_dropdown(
        &mut self,
        frame: &mut eframe::Frame,
        ui: &mut egui::Ui,
        what_graph: WhatGraph,
    ) {
        //there are two sets of options, one for the left graph, the other for the right
        let options;
        if what_graph == WhatGraph::LeftGraph {
            options = &mut self.graph_left.mesh_options.across_metric;
        } else if what_graph == WhatGraph::RightGraph {
            options = &mut self.graph_right.mesh_options.across_metric;
        } else {
            panic!();
        }
        let past_option = options.clone();
        ui.horizontal(|ui| {
            ui.label("Across");
            egui::containers::ComboBox::from_label(" ")
                .selected_text(format!("{:?}", options))
                .show_ui(ui, |ui| {
                    ui.selectable_value(options, AcrossMetric::Time, "Time");
                    ui.selectable_value(options, AcrossMetric::Thread, "Thread");
                });
        });

        if past_option != options.clone() {
            self.regen_all(frame);
        }
    }
    pub fn salt_drag_value(&mut self, frame: &mut eframe::Frame, ui: &mut egui::Ui) {
        let salt = &mut self.master_graph.color_salt;
        let past_salt = salt.clone();
        ui.horizontal(|ui| {
            ui.add(egui::DragValue::new(salt).speed(1));
            ui.add(egui::widgets::Label::new("Seed"));
        });
        if past_salt != salt.clone() {
            self.master_graph.new_color_scheme();
            self.regen_all(frame);
        }
    }
    pub fn color_dropdown(&mut self, frame: &mut eframe::Frame, ui: &mut egui::Ui) {
        //there are two sets of options, one for the left graph, the other for the right
        let color = &mut self.master_graph.color_scheme;
        let past_color = color.clone();
        ui.horizontal(|ui| {
            ui.label("Color Scheme");
            egui::containers::ComboBox::from_label("")
                .selected_text(format!("{:?}", color))
                .show_ui(ui, |ui| {
                    ui.selectable_value(color, ColorScheme::Flame, "Flame");
                    ui.selectable_value(color, ColorScheme::Ice, "Ice");
                    ui.selectable_value(color, ColorScheme::Greyscale, "Greyscale");
                    ui.selectable_value(color, ColorScheme::Rainbow, "Rainbow");
                });
        });
        if past_color != color.clone() {
            self.master_graph.new_color_scheme();
            self.regen_all(frame);
        }
    }
    pub fn division_checkbox(
        &mut self,
        frame: &mut eframe::Frame,
        ui: &mut egui::Ui,
        what_graph: WhatGraph,
    ) {
        let counter;
        let what;
        if what_graph == WhatGraph::LeftGraph {
            counter = &mut self.graph_left.mesh_options.bar_spacing;
            what = RegenWhat::Left;
        } else if what_graph == WhatGraph::RightGraph {
            counter = &mut self.graph_right.mesh_options.bar_spacing;
            what = RegenWhat::Right;
        } else {
            panic!();
        }
        let old_counter = counter.clone();
        ui.checkbox(counter, "Spacing");
        if old_counter != *counter {
            self.regen(frame, what);
        }
    }
    pub fn slider_start_time(
        &mut self,
        frame: &mut eframe::Frame,
        ui: &mut egui::Ui,
        what_graph: WhatGraph,
    ) {
        let counter;
        let what;
        if what_graph == WhatGraph::LeftGraph {
            counter = &mut self.graph_left.mesh_options.time_range;
            what = RegenWhat::Left;
        } else if what_graph == WhatGraph::RightGraph {
            counter = &mut self.graph_right.mesh_options.time_range;
            what = RegenWhat::Right;
        } else {
            // what_graph == WhatGraph::Inspector
            counter = &mut self.ui_state.inspector_options.time_range;
            what = RegenWhat::Inspector;
        }
        let old_counter = counter.clone();
        ui.horizontal(|ui| {
            ui.add(egui::Slider::new(
                &mut counter.start,
                self.data_info.start..=self.data_info.end - 1,
            ));
            ui.add(egui::widgets::Label::new("Start Time"));
        });
        counter.start = std::cmp::min(counter.start, counter.end - 1000);
        if old_counter != counter.clone() {
            self.regen(frame, what)
        }
    }
    pub fn slider_end_time(
        &mut self,
        frame: &mut eframe::Frame,
        ui: &mut egui::Ui,
        what_graph: WhatGraph,
    ) {
        let counter;
        let what;
        if what_graph == WhatGraph::LeftGraph {
            counter = &mut self.graph_left.mesh_options.time_range;
            what = RegenWhat::Left;
        } else if what_graph == WhatGraph::RightGraph {
            counter = &mut self.graph_right.mesh_options.time_range;
            what = RegenWhat::Right;
        } else {
            // what_graph == WhatGraph::Inspector
            counter = &mut self.ui_state.inspector_options.time_range;
            what = RegenWhat::Inspector;
        }
        let old_counter = counter.clone();
        ui.horizontal(|ui| {
            ui.add(egui::Slider::new(
                &mut counter.end,
                self.data_info.start..=self.data_info.end - 1,
            ));
            ui.add(egui::widgets::Label::new("End Time"));
        });

        if old_counter != counter.clone() {
            self.regen(frame, what)
        }
    }
}
//used in xperflab.org to toggle dark mode
#[wasm_bindgen]
pub fn toggle_dark_mode() {
    *DARK_MODE.lock() ^= true;
}
