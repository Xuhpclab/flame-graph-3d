mod app_panels;
mod app_widgets;
mod cube_painter;
mod data;
mod tree;
mod ui_helper;

use cgmath::Deg;
use cgmath::Euler;

use cube_painter::*;
use data::*;
use egui::mutex::Mutex;
use std::sync::Arc;
use tree::*;

use self::app_widgets::WhatGraph;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
struct OptionsMenu {
    inspector_options: MeshOptions,
    inspector_height: usize,
    modify_options: ModifyOptions,
    visual_options: egui::Visuals,
}
struct InteractiveViewOptions {
    zoom: f32,
    displacement: cgmath::Vector3<f32>,
    angle: cgmath::Euler<Deg<f32>>,
}
struct Graph3D {
    painter: Arc<Mutex<CubePainter>>,
    mesh_options: MeshOptions,
    view_options: InteractiveViewOptions,
}
struct ModifyOptions {
    highlighting_node: bool,
    open: bool,
    node: Node,
    new_popup: bool,
    window_position: egui::Pos2,
}
#[derive(Debug, Clone, PartialEq)]
enum ClearOptions {
    NeedsFirstClear,
    HasBeenCleared,
    HasntBeenCleared,
}
impl Default for InteractiveViewOptions {
    fn default() -> Self {
        InteractiveViewOptions {
            zoom: 350.0,
            displacement: cgmath::Vector3 {
                x: 0.175,
                y: 0.0,
                z: -2.4,
            },
            angle: Euler {
                x: Deg(35.0),
                y: Deg(-120.0),
                z: Deg(0.0),
            },
        }
    }
}
pub struct App {
    /// Behind an `Arc<Mutex<…>>` so we can pass it to [`egui::PaintCallback`] and paint later.
    graph_left: Graph3D,
    graph_right: Graph3D,
    rect_painter: Arc<Mutex<CubePainter>>,
    inspector_mesh: Mesh,
    master_graph: MasterTree,
    inspector_graph: Tree,
    data_info: TreeInfo,
    ui_state: OptionsMenu,
    clear_options: ClearOptions,
}
impl App {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let gl = cc
            .gl
            .as_ref()
            .expect("You need to run eframe with the glow backend");
        let info = get_input_info();
        let l_options = MeshOptions::new_3d(&info);
        let r_options = MeshOptions::new_3d(&info);
        let master = grow_master_tree();
        let mesh = get_mesh_from_tree(&master, &l_options);

        let ins_options = MeshOptions::new_2d(&info);
        let tree = build_time_tree(&master, info.start..info.end);
        let insm = get_rects_from_tree(&tree, &ins_options);

        let ui = OptionsMenu {
            inspector_height: 12,
            inspector_options: ins_options,
            modify_options: ModifyOptions {
                highlighting_node: false,
                open: false,
                node: Node {
                    name: "null".to_string(),
                    values: trace_zero(),
                    children: vec![],
                    color: None,
                    offsets: trace_zero(),
                },
                new_popup: false,
                window_position: egui::pos2(0.0, 0.0),
            },
            visual_options: egui::Visuals::dark(),
        };

        Self {
            inspector_graph: tree,
            ui_state: ui,
            graph_left: Graph3D {
                painter: Arc::new(Mutex::new(CubePainter::new(gl, &mesh.verts, &mesh.colors))),
                mesh_options: l_options,
                view_options: Default::default(),
            },
            graph_right: Graph3D {
                painter: Arc::new(Mutex::new(CubePainter::new(gl, &mesh.verts, &mesh.colors))),
                mesh_options: r_options,
                view_options: Default::default(),
            },
            rect_painter: Arc::new(Mutex::new(CubePainter::new(gl, &insm.verts, &insm.colors))),
            inspector_mesh: insm,
            data_info: info,
            master_graph: master,
            clear_options: ClearOptions::NeedsFirstClear,
        }
    }
}
impl eframe::App for App {
    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if self.clear_options == ClearOptions::NeedsFirstClear {
            self.regen_all(frame);
            self.clear_options = ClearOptions::HasBeenCleared;
        }
        // self.option_menu(ctx, frame);
        self.set_web_options(ctx);
        if self.ui_state.modify_options.open {
            self.modify_trace(ctx, frame);
        }
        if self.ui_state.inspector_options.has_changed {
            self.regen_inspector(frame);
            self.ui_state.inspector_options.has_changed = false;
        }
        egui::TopBottomPanel::top("top_panel")
            .resizable(true)
            .min_height(32.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("EasyView Inspector");
                    });
                });
            });
        egui::SidePanel::left("left")
            .resizable(true)
            .show(ctx, |ui| {
                self.left_panel(ui, frame);
            });
        egui::SidePanel::right("right")
            .resizable(true)
            .show(ctx, |ui| {
                self.right_panel(ui, frame);
            });
        egui::TopBottomPanel::bottom("bottom")
            .resizable(true)
            .show(ctx, |_ui| {});
        egui::CentralPanel::default().show(ctx, |ui| {
            self.central_panel(ui, frame);
        });
        egui::Context::request_repaint_after(ctx, std::time::Duration::from_millis(100));
        self.clear_options = ClearOptions::HasntBeenCleared;
    }
    fn on_exit(&mut self, gl: Option<&glow::Context>) {
        if let Some(gl) = gl {
            self.graph_left.painter.lock().destroy(gl);
            self.graph_right.painter.lock().destroy(gl);
        }
    }
}
