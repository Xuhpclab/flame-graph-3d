use std::{
    collections::hash_map::DefaultHasher,
    fmt::Debug,
    hash::{Hash, Hasher},
    ops::Range,
};
#[derive(Debug, Clone, PartialEq, Copy)]
pub enum ColorScheme {
    Rainbow,
    Greyscale,
    Flame,
    Ice,
}
const DEFAULT_COLOR: ColorScheme = ColorScheme::Rainbow;
#[derive(Debug, Clone)]
pub struct TraceValues {
    pub start: u64,
    pub dur: u64,
    pub value: u64,
    pub thread: usize,
}
pub fn trace_zero() -> TraceValues {
    TraceValues {
        start: 0,
        dur: 0,
        value: 0,
        thread: 0,
    }
}
#[derive(Debug, Clone)]
pub struct Tree {
    pub root: Node,
    pub time_range: Range<u64>,
}
#[derive(Debug, Clone)]
pub struct Node {
    /// function name:
    pub name: String,
    pub values: TraceValues,
    pub children: Vec<Node>,
    pub color: Option<[f32; 4]>,
    pub offsets: TraceValues,
}
///This is a flamegraph, specifically one with all traces from the input
#[derive(Debug, Clone)]
pub struct MasterTree {
    pub root: MasterNode,
    pub color_scheme: ColorScheme,
    pub color_salt: u32,
}
#[derive(Debug, Clone)]
pub struct MasterNode {
    /// function name:
    pub name: String,
    pub values: Vec<TraceValues>,
    pub children: Vec<MasterNode>,
    pub color: Option<[f32; 4]>,
}
pub fn build_time_tree(node: &MasterTree, range: Range<u64>) -> Tree {
    let Some(mut root )= build_time_node_subtree(&node.root, range.clone()) else {
        return Tree{ root: Node{ name: "root".to_string(), values: trace_zero(), children: vec![], color: None, offsets: trace_zero() }, time_range: range };
    };
    root.calculate_offset(0, 0);
    Tree {
        root: root,
        time_range: range,
    }
}
fn build_time_node_subtree(node: &MasterNode, time_range: Range<u64>) -> Option<Node> {
    let overlaps = node.time_overlaps(1, time_range.clone());

    if overlaps.is_none() {
        return None;
    }
    let unwrapped_over = overlaps.unwrap();
    let mut new_node = Node {
        name: node.name.clone(),
        values: unwrapped_over[0].clone(),
        children: vec![],
        offsets: trace_zero(),
        color: node.color,
    };
    for child in &node.children {
        let n = build_time_node_subtree(child, time_range.clone());
        if n.is_some() {
            new_node.children.push(n.unwrap());
        }
    }
    Some(new_node)
}
pub fn build_thread_tree(node: &MasterTree, thread: usize) -> Tree {
    let Some(mut root )= build_thread_subtree(&node.root, thread) else {
        return Tree{ root: Node{ name: "root".to_string(), values: trace_zero(), children: vec![], color: None, offsets: trace_zero() }, time_range: 0..0 };
    };
    root.calculate_offset(0, 0);
    Tree {
        root: root,
        time_range: 0..0,
    }
}
fn build_thread_subtree(node: &MasterNode, thread: usize) -> Option<Node> {
    let overlaps = node.single_tread_overlap(thread);

    if overlaps.is_none() {
        return None;
    }
    let unwrapped_over = overlaps.unwrap();
    let mut new_node = Node {
        name: node.name.clone(),
        values: unwrapped_over[0].clone(),
        children: vec![],
        offsets: trace_zero(),
        color: node.color,
    };
    for child in &node.children {
        let n = build_thread_subtree(child, thread);
        if n.is_some() {
            new_node.children.push(n.unwrap());
        }
    }
    Some(new_node)
}
impl Node {
    ///once the tree is already constructed, loop through the whole thing and offset children
    fn calculate_offset(
        &mut self,
        starting_dur_offset: u64,
        starting_val_offset: u64,
    ) -> &mut Node {
        let mut dur_offset = starting_dur_offset;
        let mut val_offset = starting_val_offset;
        for node in &mut self.children {
            node.offsets.dur = dur_offset;
            node.offsets.value = val_offset;
            node.calculate_offset(dur_offset, val_offset);
            dur_offset += node.values.dur;
            val_offset += node.values.value;
        }
        self
    }
}
impl MasterNode {
    //maybe theres a fast overlap that doesnt require looking at each cell
    pub fn time_overlaps(&self, num_graphs: usize, range: Range<u64>) -> Option<Vec<TraceValues>> {
        //did we have overlaps
        let mut some = false;
        if num_graphs < 1 {
            return None;
        }
        //how big is the time slice we are considering
        let graph_size = (range.end - range.start) / num_graphs as u64;
        let mut overlaps = vec![];
        for i in 0..num_graphs {
            let time =
                (i as u64 * graph_size + range.start)..((i as u64 + 1) * graph_size + range.start);
            let mut dur_total = 0;
            let mut val_total = 0;
            //for each value find how much of the value fits into our considered range
            for value in &self.values {
                let dur = std::cmp::min(time.end, value.start + value.dur)
                    .saturating_sub(std::cmp::max(time.start, value.start));
                if value.dur != 0 {
                    val_total += (dur / value.dur) * value.value;
                    dur_total += dur;
                }
            }
            overlaps.push(TraceValues {
                start: 0,
                dur: dur_total,
                value: val_total,
                thread: 0,
            });
            if dur_total != 0 {
                some = true;
            }
        }
        if some {
            return Some(overlaps);
        }
        None
    }
    //maybe theres a fast overlap that doesnt require looking at each cell
    pub fn tread_overlaps(&self, num_threads: usize) -> Option<Vec<TraceValues>> {
        let mut overlaps = vec![];
        for i in 0..num_threads {
            let mut start = 0;
            let mut dur_total = 0;
            let mut val_total = 0;
            for value in &self.values {
                if value.thread == i {
                    val_total += value.value;
                    dur_total += value.dur;
                    start = std::cmp::max(start, value.start);
                }
            }
            overlaps.push(TraceValues {
                start: 0,
                dur: dur_total,
                value: val_total,
                thread: 0,
            });
        }
        return Some(overlaps);
    }
    pub fn single_tread_overlap(&self, thread_to_match: usize) -> Option<Vec<TraceValues>> {
        let mut overlaps = vec![];
        let mut start = 0;
        let mut dur_total = 0;
        let mut val_total = 0;
        for value in &self.values {
            if value.thread == thread_to_match {
                val_total += value.value;
                dur_total += value.dur;
                start = std::cmp::max(start, value.start);
            }
        }
        overlaps.push(TraceValues {
            start: 0,
            dur: dur_total,
            value: val_total,
            thread: 0,
        });

        return Some(overlaps);
    }
}
impl MasterTree {
    fn add_trace(&mut self, trace: &Trace) -> &mut MasterNode {
        let mut iter = &mut self.root;
        iter.values.push(TraceValues {
            start: trace.start,
            dur: trace.dur,
            value: trace.value,
            thread: trace.tid,
        });
        for stackframe in trace.stack.iter().rev() {
            iter = find_str(
                iter,
                stackframe.name.clone(),
                self.color_scheme,
                self.color_salt,
            );
            iter.values.push(TraceValues {
                start: trace.start,
                dur: trace.dur,
                value: trace.value,
                thread: trace.tid,
            });
        }
        iter = find_str(iter, trace.name.clone(), self.color_scheme, self.color_salt);
        iter.values.push(TraceValues {
            start: trace.start,
            dur: trace.dur,
            value: trace.value,
            thread: trace.tid,
        });
        fn find_str(
            iter: &mut MasterNode,
            str: String,
            color_scheme: ColorScheme,
            color_salt: u32,
        ) -> &mut MasterNode {
            if iter.children.iter().any(|i| i.name == str) {
                iter.children.iter_mut().find(|i| i.name == str).unwrap()
            } else {
                iter.children.push(master_root());
                iter.children.last_mut().unwrap().name = str.clone();
                iter.children.last_mut().unwrap().color =
                    Some(color_from_scheme(&str, color_scheme, color_salt));
                iter.children.last_mut().unwrap()
            }
        }
        iter
    }
    pub fn modify_color(&mut self, trace: String, color: [f32; 4]) {
        for child in &mut self.root.children {
            modify_color(child, &trace, color)
        }
    }
    pub fn new_color_scheme(&mut self) {
        for child in &mut self.root.children {
            new_color_scheme_nodes(child, self.color_scheme, self.color_salt)
        }
    }
}
fn modify_color(node: &mut MasterNode, trace: &String, color: [f32; 4]) {
    if node.name == trace.to_owned() {
        node.color = Some(color);
    }
    for child in &mut node.children {
        modify_color(child, trace, color)
    }
}
fn new_color_scheme_nodes(node: &mut MasterNode, scheme: ColorScheme, salt: u32) {
    node.color = Some(color_from_scheme(&node.name, scheme, salt));
    for child in &mut node.children {
        new_color_scheme_nodes(child, scheme, salt);
    }
}

fn master_root() -> MasterNode {
    MasterNode {
        name: "root".to_string(),
        values: vec![],
        children: vec![],
        color: None,
    }
}

fn generate_hash(name: &str, color_salt: u32) -> f64 {
    const MODULOUS: u32 = 11;
    const MAX_CHAR: usize = 6;
    let mut hash = 0.0;
    let mut max_hash = 0.0;
    let mut weight = 1.0;

    if !name.is_empty() {
        for (i, c) in name.chars().enumerate() {
            if i > MAX_CHAR {
                break;
            }
            hash += weight * ((c as u32 + color_salt) % MODULOUS) as f64;
            max_hash += weight * (MODULOUS - 1) as f64;
            weight = weight * 0.7;
        }
        if max_hash > 0. {
            hash / max_hash
        } else {
            0.0
        }
    } else {
        0.0
    }
}

pub fn generate_color_vector(name: &str, color_mod: u32) -> f64 {
    let mut vector = 0.0;
    let mut func_name = name;
    if !name.is_empty() {
        let name_arr: Vec<&str> = func_name.split('`').collect();
        if name_arr.len() > 1 {
            func_name = name_arr[name_arr.len() - 1];
        }
        let name_parts: Vec<&str> = func_name.split('(').collect();
        func_name = name_parts[0];
        vector = generate_hash(func_name, color_mod);
    }
    vector
}

pub fn color_from_scheme(name: &String, scheme: ColorScheme, color_mod: u32) -> [f32; 4] {
    match scheme {
        ColorScheme::Flame => color_scheme_flame(name, color_mod),
        ColorScheme::Ice => color_scheme_ice(name, color_mod),
        ColorScheme::Greyscale => color_scheme_greyscale(name, color_mod),
        ColorScheme::Rainbow => color_scheme_rainbow(name, color_mod),
    }
}
pub fn color_scheme_rainbow(name: &String, color_mod: u32) -> [f32; 4] {
    let mut s = DefaultHasher::new();
    name.hash(&mut s);
    color_mod.hash(&mut s);
    let u: u64 = s.finish();
    let b = u.to_be_bytes();
    [
        0.3 + b[0] as f32 / 500.0,
        0.3 + b[1] as f32 / 500.0,
        0.3 + b[2] as f32 / 500.0,
        1.0,
    ]
}
pub fn color_scheme_greyscale(name: &String, color_mod: u32) -> [f32; 4] {
    let vector = generate_color_vector(name, color_mod);

    let r = (0 + (255.0 * (1.0 - vector)).round() as u8) as f32;
    let g = (0 + (255.0 * (1.0 - vector)).round() as u8) as f32;
    let b = (0 + (255.0 * (1.0 - vector)).round() as u8) as f32;
    [r / 255.0, g / 255.0, b / 255.0, 1.0]
}
pub fn color_scheme_flame(name: &String, color_mod: u32) -> [f32; 4] {
    let vector = generate_color_vector(name, color_mod);

    let r = (200 + (55.0 * vector).round() as u8) as f32;
    let g = (0 + (230.0 * (1.0 - vector)).round() as u8) as f32;
    let b = (0 + (55.0 * (1.0 - vector)).round() as u8) as f32;
    [r / 255.0, g / 255.0, b / 255.0, 1.0]
}
pub fn color_scheme_ice(name: &String, color_mod: u32) -> [f32; 4] {
    let vector = generate_color_vector(name, color_mod);

    let r = (0 + (55.0 * (1.0 - vector)).round() as u8) as f32;
    let g = (0 + (230.0 * (1.0 - vector)).round() as u8) as f32;
    let b = (200 + (55.0 * vector).round() as u8) as f32;
    [r / 255.0, g / 255.0, b / 255.0, 1.0]
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct Trace {
    pub id: usize,
    pub name: String,
    pub stack: Vec<Stackframe>,
    pub start: u64,
    pub dur: u64,
    pub value: u64,
    pub tid: usize,
}
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct Stackframe {
    pub name: String,
}
pub fn grow_master_tree() -> MasterTree {
    let data = load_data();
    let mut tree = MasterTree {
        root: master_root(),
        color_scheme: DEFAULT_COLOR,
        color_salt: 1,
    };
    for trace in data {
        tree.add_trace(&trace);
    }
    tree
}

#[derive(Debug, Clone)]
pub struct TreeInfo {
    pub max_depth: usize,
    pub num_threads: usize,
    pub start: u64,
    pub end: u64,
    pub nodes: u64,
}
///gets relevent info from a list of traces
pub fn get_info<'a>(
    traces: impl Iterator<Item = &'a Trace> + Clone + ExactSizeIterator,
) -> TreeInfo {
    TreeInfo {
        max_depth: traces.clone().map(|i| i.stack.len()).max().unwrap_or(0),
        num_threads: traces.clone().map(|i| i.tid).max().unwrap_or(0) + 1,
        start: traces.clone().map(|i| i.start).min().unwrap_or(0),
        end: traces.clone().map(|i| i.start + i.dur).max().unwrap_or(0),
        nodes: traces.len() as u64,
    }
}
fn load_data() -> Vec<Trace> {
    serde_json::from_str(INPUT_TRACE).unwrap()
}
pub fn get_input_info() -> TreeInfo {
    let data = load_data();
    get_info(data.iter())
}

const INPUT_TRACE: &str = include_str!("./../../../lulesh.json");
