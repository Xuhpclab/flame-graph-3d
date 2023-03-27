use crate::app::tree::*;
use std::ops::Range;
use traversal::Bft;
pub const LENGTH_MOD: f32 = 1.9;
pub const LENGTH_OFFSET: f32 = 0.75;
pub const BREDTH_MOD: f32 = 1.8;
pub const BREDTH_OFFSET: f32 = 0.9;
pub const VERTS_IN_RECT: usize = 6;
pub const _VERTS_IN_CUBE: usize = 36;
const DEFUALT_DIVISONS: usize = 5;

/**  Data deals with the transformation of input (Currently JSON) to verticies
 *  
*/
#[derive(Debug, Clone, PartialEq)]
///DataChoices allows users to specify which type of data should be graphed
pub enum DataChoices {
    Duration,
    Value,
}
#[derive(Debug, Clone, PartialEq)]
///DataChoices allows users to specify which type of data should be graphed
pub enum AcrossMetric {
    Time,
    Thread,
}
///MeshOptions allow custom generation of the graphs
pub struct MeshOptions {
    pub bar_spacing: bool,
    pub has_changed: bool,
    pub time_range: Range<u64>,
    pub data_metric: DataChoices,
    pub across_metric: AcrossMetric,
    pub num_graphs: usize,
    pub num_threads: usize,
}
impl MeshOptions {
    pub fn new_3d(info: &TreeInfo) -> Self {
        MeshOptions {
            bar_spacing: false,
            has_changed: false,
            num_graphs: DEFUALT_DIVISONS,
            time_range: info.start..info.end,
            num_threads: info.num_threads,
            data_metric: DataChoices::Duration,
            across_metric: AcrossMetric::Time,
        }
    }
    pub fn new_2d(info: &TreeInfo) -> Self {
        MeshOptions {
            bar_spacing: false,
            has_changed: false,
            num_graphs: 1,
            num_threads: info.num_threads,
            time_range: info.start..(info.start + (info.end - info.start) / 5),
            data_metric: DataChoices::Duration,
            across_metric: AcrossMetric::Time,
        }
    }
}
pub fn get_mesh_from_tree(graph: &MasterTree, mesh_options: &MeshOptions) -> Mesh {
    let mut mesh = Mesh {
        verts: vec![],
        colors: vec![],
        indicies: vec![],
    };
    let root_overlaps = match mesh_options.across_metric {
        AcrossMetric::Time => graph
            .root
            .time_overlaps(mesh_options.num_graphs, mesh_options.time_range.clone()),
        AcrossMetric::Thread => graph.root.tread_overlaps(mesh_options.num_threads),
    };
    let Some(root_overlaps)  = root_overlaps else {
        return mesh;
    };
    let max_size = match mesh_options.data_metric {
        DataChoices::Duration => root_overlaps.iter().map(|i| i.dur).max().unwrap_or(0),
        DataChoices::Value => root_overlaps.iter().map(|i| i.value).max().unwrap_or(0),
    };

    let num_division = match mesh_options.across_metric {
        AcrossMetric::Time => mesh_options.num_graphs,
        AcrossMetric::Thread => mesh_options.num_threads,
    };
    let mut c = 0;
    tree_to_verts(&mut MeshBuilder {
        current_node: &graph.root,
        options: mesh_options,
        starting_dur_offset: vec![vec![0, 0]; num_division],
        starting_val_offset: vec![vec![0, 0]; num_division],
        depth: 1,
        max_bar_size: max_size,
        mesh: &mut mesh,
        counter: &mut c,
        num_divisions: num_division,
    });
    mesh
}
///called for every tree in the list, transforms the tree into a vec of verticies, indicies, and colors
struct MeshBuilder<'a> {
    current_node: &'a MasterNode,
    options: &'a MeshOptions,
    starting_dur_offset: Vec<Vec<u64>>,
    starting_val_offset: Vec<Vec<u64>>,
    depth: usize,
    max_bar_size: u64,
    mesh: &'a mut Mesh,
    counter: &'a mut usize,
    num_divisions: usize,
}
#[derive(Debug, Clone)]
pub struct Mesh {
    pub verts: Vec<[f32; 3]>,
    pub colors: Vec<[f32; 4]>,
    pub indicies: Vec<u32>,
}
fn tree_to_verts<'a>(builder: &mut MeshBuilder<'a>) {
    *builder.counter += 1;

    let overlaps = match builder.options.across_metric {
        AcrossMetric::Time => builder.current_node.time_overlaps(
            builder.options.num_graphs,
            builder.options.time_range.clone(),
        ),
        AcrossMetric::Thread => builder
            .current_node
            .tread_overlaps(builder.options.num_threads),
    };
    let Some(overlaps)  = overlaps else {
        return ;
    };

    //add this overlap to the layer below's offset
    for value in overlaps.iter().enumerate() {
        builder.starting_dur_offset[value.0][builder.depth - 1] += value.1.dur;
        builder.starting_val_offset[value.0][builder.depth - 1] += value.1.value;
    }
    //build the node to verts
    if builder.current_node.color.is_some() && builder.depth > 1 {
        let mut result = verts_from_overlaps(&builder, &overlaps);
        builder.mesh.verts.append(&mut result.0);
        builder.mesh.colors.append(&mut result.1);
    }
    for node in &builder.current_node.children {
        builder.current_node = node;
        builder.depth += 1;

        //push a duplicate on the offset stacks
        for graph_value in &mut builder.starting_dur_offset {
            graph_value.push(*graph_value.last().unwrap_or(&0));
        }
        for graph_value in &mut builder.starting_val_offset {
            graph_value.push(*graph_value.last().unwrap_or(&0));
        }
        tree_to_verts(builder);
        //if we need to pop, do so until we are at the right level
        while builder.starting_dur_offset[0].len() > builder.depth {
            for value in &mut builder.starting_dur_offset {
                value.pop();
            }
            for value in &mut builder.starting_val_offset {
                value.pop();
            }
        }
        builder.depth -= 1;
    }

    // let mut i: u32 = 0;
    // while i < (builder.mesh.verts.len() / 4) as u32 {
    //     let offset = i * 6;
    //     builder.mesh.indicies.extend([
    //         0 + offset,
    //         3 + offset,
    //         1 + offset,
    //         0 + offset,
    //         3 + offset,
    //         2 + offset,
    //     ]);
    //     i += 1;
    // }
}
///returns a single block for one node in the tree
fn verts_from_overlaps(
    builder: &MeshBuilder<'_>,
    overlaps: &Vec<TraceValues>,
) -> (Vec<[f32; 3]>, Vec<[f32; 4]>) {
    let mut colors: Vec<[f32; 4]> = vec![];
    let mut block_size = vec![];
    let mut offset = vec![];
    if builder.options.data_metric == DataChoices::Duration {
        for overlap in overlaps.iter().enumerate() {
            block_size.push(overlap.1.dur as f32 / builder.max_bar_size as f32);
            offset.push(
                builder.starting_dur_offset[overlap.0][builder.depth] as f32
                    / builder.max_bar_size as f32
                    - 0.9,
            );
        }
    }
    if builder.options.data_metric == DataChoices::Value {
        for overlap in overlaps.iter().enumerate() {
            block_size.push(overlap.1.value as f32 / builder.max_bar_size as f32);
            offset.push(
                builder.starting_val_offset[overlap.0][builder.depth] as f32
                    / builder.max_bar_size as f32
                    - 0.9,
            );
        }
    }
    let depth1 = 1.0 / -2.0_f32.powf(0.1 * builder.depth as f32) + 1.0;
    let depth2 = 1.0 / -2.0_f32.powf(0.1 * (builder.depth + 1) as f32) + 1.0;
    //these values help adjust the verts into a {-1,-1 to 1,1} cube
    let mut result = vec![];
    let spacing;
    if builder.options.bar_spacing {
        spacing = 1.0;
    } else {
        spacing = 0.0;
    }
    for overlap in 0..overlaps.len() {
        if block_size[overlap] > 0.0 {
            result.append(&mut get_points_for_cube(
                [
                    LENGTH_OFFSET + offset[overlap] * LENGTH_MOD,
                    depth1,
                    (overlap as f32 / builder.num_divisions as f32 * BREDTH_MOD) - BREDTH_OFFSET,
                ],
                [
                    LENGTH_OFFSET + offset[overlap] * LENGTH_MOD + block_size[overlap] * LENGTH_MOD,
                    depth2,
                    (overlap as f32 / builder.num_divisions as f32 * BREDTH_MOD) - BREDTH_OFFSET
                        + ((BREDTH_MOD - spacing) / builder.num_divisions as f32),
                ],
            ));
        }
    }

    for _vert in &result {
        colors.push(builder.current_node.color.unwrap());
    }
    (result, colors)
}
///returns a list of all the verticies and indicies for a cube's bounding points
fn get_points_for_cube(lower_corner: [f32; 3], upper_corner: [f32; 3]) -> Vec<[f32; 3]> {
    let mut verticies: Vec<[f32; 3]> = vec![];
    //define each corner of the 6 faces of the cube
    for i in 0..6 {
        let mut point1 = lower_corner;
        let mut point2 = upper_corner;
        if i < 3 {
            point2[i % 3] = lower_corner[i % 3];
        } else {
            point1[i % 3] = upper_corner[i % 3];
        }
        let mut result = get_rect_from_points(point1, point2, i % 3);
        verticies.append(&mut result);
    }
    verticies
}
///returns veticies for a rectangle from the two bounding corners
fn get_rect_from_points(
    lower_corner: [f32; 3],
    upper_corner: [f32; 3],
    axis: usize,
) -> Vec<[f32; 3]> {
    let mut verticies: Vec<[f32; 3]> = vec![];

    let mut point1 = [lower_corner[0], lower_corner[1], lower_corner[2]];
    point1[(axis + 1) % 3] = upper_corner[(axis + 1) % 3];
    let mut point2 = [lower_corner[0], lower_corner[1], lower_corner[2]];
    point2[(axis + 2) % 3] = upper_corner[(axis + 2) % 3];

    verticies.push(lower_corner);
    verticies.push(point1);
    verticies.push(upper_corner);
    verticies.push(lower_corner);
    verticies.push(point2);
    verticies.push(upper_corner);

    verticies
}
///this method could use conservative rasterazation, currently hides nodes smaller than 2 pixels wide or so
///This method is used to display flamegraphs in the inspector, similar to get_mesh_from_graphs but 2d
pub fn get_rects_from_tree(flamegraph: &Tree, options: &MeshOptions) -> Mesh {
    let mut max_bar_size = 0;
    let mut verticies: Vec<[f32; 3]> = vec![];
    let indicies: Vec<u32> = vec![];
    let mut colors: Vec<[f32; 4]> = vec![];
    let iter = Bft::new(&flamegraph.root, |tree| tree.children.iter());
    for block in iter.map(|(depth, node)| (depth, node)) {
        if block.1.color.is_some() {
            let mut block_size = 0.0;
            let mut off = 1;
            if options.data_metric == DataChoices::Duration {
                max_bar_size = flamegraph.root.values.dur;
                block_size = block.1.values.dur as f32 / max_bar_size as f32;
                off = block.1.offsets.dur;
            }
            if options.data_metric == DataChoices::Value {
                max_bar_size = flamegraph.root.values.value;
                block_size = block.1.values.value as f32 / max_bar_size as f32;
                off = block.1.offsets.value;
            }
            let offset = (off as f32 / max_bar_size as f32) - 0.9;
            //if the block size is too small, break (to prevent flickering)f

            // if block_size <= 1.0 / display_width {
            //     continue;
            // }
            let depth = block.0 as f32 - 1.0;
            let depth2 = block.0 as f32;
            //these values help adjust the verts into a {-1,-1 to 1,1} cube
            let block_length = LENGTH_OFFSET + offset * LENGTH_MOD;
            let mut result = get_rect_from_points(
                [block_length, depth, 0.0],
                [block_length + block_size * LENGTH_MOD, depth2, 0.0],
                2,
            );
            for _vert in &result {
                colors.push(block.1.color.unwrap());
            }
            verticies.append(&mut result);
        }
    }
    // let mut i: u32 = 0;
    // while i < (verticies.len() / 4) as u32 {
    //     let offset = i * 6;
    //     indicies.append(&mut vec![
    //         0 + offset,
    //         3 + offset,
    //         1 + offset,
    //         0 + offset,
    //         3 + offset,
    //         2 + offset,
    //     ]);
    //     i += 1;
    // }
    Mesh {
        verts: verticies,
        colors: colors,
        indicies: indicies,
    }
}
