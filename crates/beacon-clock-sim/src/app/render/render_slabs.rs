use crate::app::State;
use super::Render;
use crate::util::texture::TextureUnit;
use super::shader::Shader;
use super::shader::ShaderKind;
//use nalgebra;
//use nalgebra::{Isometry3, Matrix4, Vector3};
use web_sys::WebGlRenderingContext as GL;
use web_sys::*;

pub struct SlabRenderer<'a> {
    shader: &'a Shader,
}

impl<'a> SlabRenderer<'a> {
    pub fn new(shader: &'a Shader) -> SlabRenderer<'a> {
        SlabRenderer { shader }
    }
}

impl<'a> Render<'a> for SlabRenderer<'a> {
    fn shader_kind() -> ShaderKind {
        ShaderKind::Slab
    }

    fn shader(&'a self) -> &'a Shader {
        &self.shader
    }

    fn buffer_attributes(&self, gl: &WebGlRenderingContext, state: &State) {
        let shader = self.shader();

        let pos_attrib = gl.get_attrib_location(&shader.program, "position");
        gl.enable_vertex_attrib_array(pos_attrib as u32);

        let color_attrib = gl.get_attrib_location(&shader.program, "vRgbaColor");
        gl.enable_vertex_attrib_array(color_attrib as u32);


        let vertices: [f32; 9] = [-0.7, -0.7, 0.0, 0.7, -0.7, 0.0, 0.0, 0.7, 0.0];
//        let mut indices: [u16; 6] = [0, 1, 2, 0, 2, 3];

        let colors: [f32; 12] = [
            1.0, 0.0, 0.0, 1.0,
            0.7, 0.0, 0.0, 1.0,
            1.0, 0.0, 0.5, 1.0,
        ];

        Self::buffer_f32_data(&gl, &state.slabsystem.color[..], color_attrib as u32, 4);
        Self::buffer_f32_data(&gl, &state.slabsystem.position[..], pos_attrib as u32, 3);

//        Self::buffer_f32_data(&gl, &vertices, pos_attrib as u32, 3);
//        Self::buffer_u16_indices(&gl, &mut indices);
    }

    fn render(&self, gl: &WebGlRenderingContext, state: &State) {
        let shader = self.shader();

        //let time_uni = shader.get_uniform_location(gl, "time");
//        let view_uni = shader.get_uniform_location(gl, "modelViewMatrix");
//        let camera_pos_uni = shader.get_uniform_location(gl, "cameraPosition");
//        let perspective_uni = shader.get_uniform_location(gl, "projectionMatrix");
//        let disc_texture_uni = shader.get_uniform_location(gl, "disc_texture");


//        let pos = (0., 0.0, 0.);
//
//        let x_scale = 18.;
//        let z_scale = 18.;
//        let scale = Matrix4::new_nonuniform_scaling(&Vector3::new(x_scale, 1.0, z_scale));
//
//        let model = Isometry3::new(Vector3::new(pos.0, pos.1, pos.2), nalgebra::zero());
//        let model = model.to_homogeneous();
//        let model = scale * model;
//        let mut model_array = [0.; 16];
//        model_array.copy_from_slice(model.as_slice());
//        gl.uniform_matrix4fv_with_f32_array(model_uni.as_ref(), false, &mut model_array);

//        let mut view = state.camera.view();
//        gl.uniform_matrix4fv_with_f32_array(view_uni.as_ref(), false, &mut view);

//        gl.uniform1i(
//            disc_texture_uni.as_ref(),
//            TextureUnit::Disc.texture_unit(),
//        );

//        let seconds_elapsed = state.clock / 1000.;
//        gl.uniform1f(time_uni.as_ref(), seconds_elapsed);

//        let camera_pos = state.camera.get_eye_pos();
//        let mut camera_pos = [camera_pos.x, camera_pos.y, camera_pos.z];
//        gl.uniform3fv_with_f32_array(camera_pos_uni.as_ref(), &mut camera_pos);

//        let mut perspective = state.camera.projection();
//        gl.uniform_matrix4fv_with_f32_array(perspective_uni.as_ref(), false, &mut perspective);
//
        gl.enable(GL::BLEND);
        gl.blend_func(GL::SRC_ALPHA, GL::ONE_MINUS_SRC_ALPHA);
//
////        gl.draw_elements_with_i32(GL::POINTS, 3, GL::UNSIGNED_SHORT, 0);
//
//        gl.draw_arrays(
//            GL::POINTS,
//            0,
//            4 as i32//(vertices.len() / 3) as i32,
//        );

//        gl.enable(GL::BLEND);
//        gl.enable(GL::DEPTH_TEST);

        gl.disable(GL::DEPTH_TEST);
//        gl.blend_func(GL::SRC_ALPHA, GL::ONE_MINUS_SRC_ALPHA); // To disable the background color of the canvas element
//        gl.enable(GL::BLEND);

        gl.draw_arrays(
            WebGlRenderingContext::POINTS,
            0,
            state.slabsystem.len() as i32, //(vertices.len() / 3) as i32,
        );

        gl.disable(GL::BLEND);
    }
}
