use traversal::Bft;

use crate::app::data::*;
use crate::app::tree::*;
/**  Ui_helper
 *   Implements helper functions for the ui
 *   such as selecting on screen elements with the mouse
*/
///converts a screen coordinate into world space (2D)
fn screen_to_world(screen_size: egui::Rect, mouse_pos: egui::Pos2) -> egui::Pos2 {
    let mouse_zero = mouse_pos - screen_size.left_top().to_vec2();
    let screen_size = screen_size.right_bottom() - screen_size.left_top().to_vec2();
    let mut mouse_mirror = mouse_pos.clone();
    mouse_mirror.x = ((mouse_zero.x / screen_size.x) - 0.5) * 2.0;
    mouse_mirror.y = ((mouse_zero.y / screen_size.y) - 0.5) * -2.0;
    mouse_mirror
}
///retreive the nth node of a tree
fn fetch_nth_node(tree: &Tree, n: usize) -> Option<(usize, &Node)> {
    let mut iter = Bft::new(&tree.root, |tree| tree.children.iter());
    iter.nth(n)
}
///apply a matrix transformation to a vertex
fn transform_vert(vert: [f32; 3], matrix: cgmath::Matrix4<f32>) -> [f32; 3] {
    let t = matrix
        * cgmath::Vector4 {
            x: vert[0],
            y: vert[1],
            z: vert[2],
            w: 1.0,
        };
    [t.x, t.y, t.z]
}
///lookup the node the mouse is hovering over
pub fn inspector_lookup<'a>(
    verts: &Vec<[f32; 3]>,
    flamegraph: &'a Tree,
    screen_size: egui::Rect,
    mouse_pos: egui::Pos2,
    inspector_transform: cgmath::Matrix4<f32>,
) -> Option<(usize, &'a Node)> {
    let world = screen_to_world(screen_size, mouse_pos);
    let mut i = 0;
    while i + 5 < verts.len() {
        let lower_corner = transform_vert(verts[i], inspector_transform);
        let upper_corner = transform_vert(verts[i + 5], inspector_transform);
        if world.x > lower_corner[0]
            && world.x < upper_corner[0]
            && world.y > lower_corner[1]
            && world.y < upper_corner[1]
        {
            return fetch_nth_node(flamegraph, 1 + (i / VERTS_IN_RECT));
        }
        i += VERTS_IN_RECT;
    }
    None
}
///lookup the tree the mouse is hovering over
pub fn overview_lookup(
    num_flamegraphs: usize,
    screen_size: egui::Rect,
    mouse_pos: egui::Pos2,
) -> Option<usize> {
    let mut mouse_x = screen_to_world(screen_size, mouse_pos).x;
    mouse_x += BREDTH_OFFSET;
    let bredth_value = BREDTH_MOD / num_flamegraphs as f32;
    let x = mouse_x / bredth_value;
    if x > 0.0 && x < num_flamegraphs as f32 {
        return Some(x as usize);
    }
    None
}
