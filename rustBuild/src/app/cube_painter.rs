use cgmath::Matrix4;
use glow::{Buffer, HasContext, Program, VertexArray};

///the cube painter is responsible for interfacing with WebGL to draw the graphs
pub struct CubePainter {
    program: glow::Program,
    vertex_array: glow::VertexArray,
    _vertex_buffer: glow::Buffer,
    vertex_count: usize,
}
impl CubePainter {
    pub fn new(gl: &glow::Context, verticies: &Vec<[f32; 3]>, colors: &Vec<[f32; 4]>) -> Self {
        unsafe {
            let program = create_program(gl, VERTEX_SHADER_SOURCE, FRAGMENT_SHADER_SOURCE);
            let (_vertex_buffer, vertex_array) = create_vertex_buffer(gl, verticies, colors);
            Self {
                vertex_count: verticies.len(),
                program,
                vertex_array,
                _vertex_buffer,
            }
        }
    }
    pub fn destroy(&self, gl: &glow::Context) {
        use glow::HasContext as _;
        unsafe {
            gl.delete_program(self.program);
            gl.delete_vertex_array(self.vertex_array);
        }
    }
    pub fn paint(&self, gl: &glow::Context, matrix: Matrix4<f32>) {
        use glow::HasContext as _;
        let f: &[f32; 16] = matrix.as_ref();
        unsafe {
            gl.enable(glow::DEPTH_TEST);
            // Set the backbuffer's alpha to 1.0
            gl.clear(glow::DEPTH_BUFFER_BIT);
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
            gl.depth_func(glow::LESS);
            gl.use_program(Some(self.program));
            gl.uniform_matrix_4_f32_slice(
                gl.get_uniform_location(self.program, "transform").as_ref(),
                false,
                f,
            );
            gl.bind_vertex_array(Some(self.vertex_array));
            gl.draw_arrays(glow::TRIANGLES, 0, self.vertex_count as i32);
        }
    }
}
unsafe fn create_program(
    gl: &glow::Context,
    vertex_shader_source: &str,
    fragment_shader_source: &str,
) -> Program {
    let program = gl.create_program().expect("Cannot create program");
    let shader_sources = [
        (glow::VERTEX_SHADER, vertex_shader_source),
        (glow::FRAGMENT_SHADER, fragment_shader_source),
    ];
    let mut shaders = Vec::with_capacity(shader_sources.len());

    for (shader_type, shader_source) in shader_sources.iter() {
        let shader = gl
            .create_shader(*shader_type)
            .expect("Cannot create shader");
        gl.shader_source(shader, shader_source);
        gl.compile_shader(shader);
        if !gl.get_shader_compile_status(shader) {
            panic!("{}", gl.get_shader_info_log(shader));
        }
        gl.attach_shader(program, shader);
        shaders.push(shader);
    }

    gl.link_program(program);
    if !gl.get_program_link_status(program) {
        panic!("{}", gl.get_program_info_log(program));
    }

    for shader in shaders {
        gl.detach_shader(program, shader);
        gl.delete_shader(shader);
    }

    program
}
unsafe fn create_vertex_buffer(
    gl: &glow::Context,
    verticies: &[[f32; 3]],
    color: &[[f32; 4]],
) -> (Buffer, VertexArray) {
    // We construct a buffer and upload the data
    let a = [verticies.concat(), color.concat()].concat();
    let vbo = gl.create_buffer().unwrap();
    gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
    gl.buffer_data_u8_slice(
        glow::ARRAY_BUFFER,
        bytemuck::cast_slice(&a),
        glow::STATIC_DRAW,
    );
    // We now construct a vertex array to describe the format of the input buffer
    let vao = gl.create_vertex_array().unwrap();
    gl.bind_vertex_array(Some(vao));

    gl.enable_vertex_attrib_array(0);
    gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 12, 0);
    gl.enable_vertex_attrib_array(1);
    gl.vertex_attrib_pointer_f32(1, 4, glow::FLOAT, false, 16, (verticies.len() * 12) as i32);
    (vbo, vao)
}
// unsafe fn create_color_buffer(
//     gl: &glow::Context,
//     verticies: &[[f32; 3]],
//     color: &[[f32; 3]],
// ) -> (Buffer, VertexArray) {
//     // We construct a buffer and upload the data
//     let a = [verticies, color].concat();
//     let vbo = gl.create_buffer().unwrap();
//     gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
//     gl.buffer_data_u8_slice(
//         glow::ARRAY_BUFFER,
//         bytemuck::cast_slice(&a),
//         glow::STATIC_DRAW,
//     );

//     // We now construct a vertex array to describe the format of the input buffer
//     let vao = gl.create_vertex_array().unwrap();
//     gl.bind_vertex_array(Some(vao));

//     gl.enable_vertex_attrib_array(0);
//     gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 12, 0);
//     gl.enable_vertex_attrib_array(1);
//     gl.vertex_attrib_pointer_f32(1, 3, glow::FLOAT, false, 12, (verticies.len() * 12) as i32);
//     (vbo, vao)
// }
const VERTEX_SHADER_SOURCE: &str = include_str!("../shaders/triangle.vert");
const FRAGMENT_SHADER_SOURCE: &str = include_str!("../shaders/triangle.frag");
