use crate::app::*;

impl App {
    ///central panel containing the flamegraph inspector
    pub fn central_panel(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        ui.collapsing("Options", |ui| {
            // ui.horizontal(|ui| {
            if self.ui_state.inspector_options.across_metric == AcrossMetric::Time {
                self.slider_start_time(frame, ui, WhatGraph::Inspector);
                self.slider_end_time(frame, ui, WhatGraph::Inspector);
            } else {
                ui.label(
                    "Inspecting Thread: ".to_owned()
                        + &(self.ui_state.inspector_options.num_threads + 1).to_string(),
                );
            }
            ui.horizontal(|ui| {
                self.color_dropdown(frame, ui);
                self.salt_drag_value(frame, ui);
            });
            // });
        });
        egui::Frame::canvas(ui.style()).show(ui, |ui| {
            self.inspector_veiw(ui, frame);
        });
    }
    ///the left panel contains the 3d inspector and overview
    pub fn left_panel(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        ui.collapsing("Options", |ui| {
            // slider_num_graphs(ui, &mut self.ui_state.mesh_options.num_graphs);
            self.metric_dropdown(frame, ui, WhatGraph::LeftGraph);
            self.across_dropdown(frame, ui, WhatGraph::LeftGraph);
            if self.graph_left.mesh_options.across_metric == AcrossMetric::Time {
                self.slider_start_time(frame, ui, WhatGraph::LeftGraph);
                self.slider_end_time(frame, ui, WhatGraph::LeftGraph);
                self.slider_num_graphs(frame, ui, WhatGraph::LeftGraph);
            }
            self.division_checkbox(frame, ui, WhatGraph::LeftGraph);
        });

        // });
        egui::Frame::canvas(ui.style()).show(ui, |ui| {
            self.graph_left.interactive_view(ui);
        });
        egui::Frame::canvas(ui.style()).show(ui, |ui| {
            self.graph_left.topdown_view(ui, frame, &mut self.ui_state);
        });
    }
    pub fn right_panel(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        // slider_num_graphs(ui, &mut self.ui_state.mesh_options.num_graphs);
        ui.collapsing("Options", |ui| {
            self.metric_dropdown(frame, ui, WhatGraph::RightGraph);
            self.across_dropdown(frame, ui, WhatGraph::RightGraph);
            if self.graph_right.mesh_options.across_metric == AcrossMetric::Time {
                self.slider_start_time(frame, ui, WhatGraph::RightGraph);
                self.slider_end_time(frame, ui, WhatGraph::RightGraph);
                self.slider_num_graphs(frame, ui, WhatGraph::RightGraph);
            }
            self.division_checkbox(frame, ui, WhatGraph::RightGraph);
        });
        egui::Frame::canvas(ui.style()).show(ui, |ui| {
            self.graph_right.interactive_view(ui);
        });
        egui::Frame::canvas(ui.style()).show(ui, |ui| {
            self.graph_right.topdown_view(ui, frame, &mut self.ui_state);
        });
    }
}
