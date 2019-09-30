use std::cell::RefCell;
use std::collections::HashMap;
use web_sys::WebGlRenderingContext as GL;
use web_sys::*;
use js_sys::{Reflect,WebAssembly};
use wasm_bindgen::JsCast;

pub mod texture_unit;
pub mod shader;
mod render_slabs;
//mod render_transmission;

pub use self::texture_unit::*;
use self::shader::{Shader,ShaderKind,ShaderSystem};
use self::render_slabs::SlabRenderer;
use super::State;
use super::Canvas;

pub trait Render<'a> {
    fn shader_kind() -> ShaderKind;

    fn shader(&'a self) -> &'a Shader;

    fn buffer_attributes(&self, gl: &GL);

    fn render(&self, gl: &GL, state: &State);

    fn buffer_f32_data(gl: &GL, data: &[f32], attrib: u32, size: i32) {


        let buffer = gl.create_buffer().ok_or("failed to create buffer").unwrap();

        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&buffer));

        // Note that `Float32Array::view` is somewhat dangerous (hence the
        // `unsafe`!). This is creating a raw view into our module's
        // `WebAssembly.Memory` buffer, but if we allocate more pages for ourself
        // (aka do a memory allocation in Rust) it'll cause the buffer to change,
        // causing the `Float32Array` to be invalid.
        //
        // As a result, after `Float32Array::view` we have to be very careful not to
        // do any memory allocations before it's dropped.
        unsafe {
            let vert_array = js_sys::Float32Array::view(&data);

            gl.buffer_data_with_array_buffer_view(
                WebGlRenderingContext::ARRAY_BUFFER,
                &vert_array,
                WebGlRenderingContext::STATIC_DRAW,
            );
        }

        gl.vertex_attrib_pointer_with_i32(attrib, size, GL::FLOAT, false, 0, 0);
//        gl.enable_vertex_attrib_array(0);

        // -----


//        let memory_buffer = wasm_bindgen::memory()
//            .dyn_into::<WebAssembly::Memory>()
//            .unwrap()
//            .buffer();
//
//        let data_location = data.as_ptr() as u32 / 4;
//
//        let data_array = js_sys::Float32Array::new(&memory_buffer)
//            .subarray(data_location, data_location + data.len() as u32);
//
//        let buffer = gl.create_buffer().unwrap();
//
//        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&buffer));
//        gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &data_array, GL::STATIC_DRAW);
//        gl.vertex_attrib_pointer_with_i32(attrib, size, GL::FLOAT, false, 0, 0);
    }

    fn buffer_u8_data(gl: &GL, data: &[u8], attrib: u32, size: i32) {
        let memory_buffer = wasm_bindgen::memory()
            .dyn_into::<WebAssembly::Memory>()
            .unwrap()
            .buffer();

        let data_location = data.as_ptr() as u32;

        let data_array = js_sys::Uint8Array::new(&memory_buffer)
            .subarray(data_location, data_location + data.len() as u32);

        let buffer = gl.create_buffer().unwrap();

        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&buffer));
        gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &data_array, GL::STATIC_DRAW);
        gl.vertex_attrib_pointer_with_i32(attrib, size, GL::UNSIGNED_BYTE, false, 0, 0);
    }

    fn buffer_u16_indices(gl: &GL, indices: &[u16]) {
        let memory_buffer = wasm_bindgen::memory()
            .dyn_into::<WebAssembly::Memory>()
            .unwrap()
            .buffer();

        let indices_location = indices.as_ptr() as u32 / 2;
        let indices_array = js_sys::Uint16Array::new(&memory_buffer)
            .subarray(indices_location, indices_location + indices.len() as u32);

        let index_buffer = gl.create_buffer().unwrap();
        gl.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(&index_buffer));
        gl.buffer_data_with_array_buffer_view(
            GL::ELEMENT_ARRAY_BUFFER,
            &indices_array,
            GL::STATIC_DRAW,
        );
    }
}


struct VaoExtension {
    oes_vao_ext: js_sys::Object,
    vaos: RefCell<HashMap<String, Vao>>,
}

struct Vao(js_sys::Object);

pub struct WebRenderer {
    shader_sys: ShaderSystem,
//    #[allow(unused)]
//    depth_texture_ext: Option<js_sys::Object>,
//    refraction_framebuffer: Framebuffer,
//    reflection_framebuffer: Framebuffer,
    vao_ext: VaoExtension,
}

impl WebRenderer {
    pub fn new(gl: &WebGlRenderingContext) -> WebRenderer {
        let shader_sys = ShaderSystem::new(&gl);
//
//        let depth_texture_ext = gl
//            .get_extension("WEBGL_depth_texture")
//            .expect("Depth texture extension");

        let oes_vao_ext = gl
            .get_extension("OES_vertex_array_object")
            .expect("Get OES vao ext")
            .expect("OES vao ext");

        let vao_ext = VaoExtension {
            oes_vao_ext,
            vaos: RefCell::new(HashMap::new()),
        };

//        let refraction_framebuffer = WebRenderer::create_refraction_framebuffer(&gl).unwrap();
//        let reflection_framebuffer = WebRenderer::create_reflection_framebuffer(&gl).unwrap();

        WebRenderer {
//            depth_texture_ext,
            shader_sys,
//            refraction_framebuffer,
//            reflection_framebuffer,
            vao_ext,
        }
    }

    pub fn render(&mut self, canvas: &Canvas, state: &State){ //}, assets: &Assets) {
        let gl = &canvas.gl;

        gl.clear_color(0.53, 0.8, 0.98, 1.);
        gl.clear(GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT);

        let above = 1000000.0;
        // Position is positive instead of negative for.. mathematical reasons..
        let clip_plane = [0., 1., 0., above];

        gl.viewport(0, 0, canvas.width() as i32, canvas.height() as i32);

        self.render_slabs(gl,state);
//        self.render_memos(gl,state);
    }

    fn render_slabs(&mut self, gl: &WebGlRenderingContext, state: &State) {
//        gl.bind_framebuffer(GL::FRAMEBUFFER, None);

        let slab_shader = self.shader_sys.get_shader(&ShaderKind::Slab).unwrap();
        self.shader_sys.use_program(gl, ShaderKind::Slab);

        let renderer = SlabRenderer::new(slab_shader);

//        renderer.buffer_attributes(gl);
        self.prepare_for_render(gl, &renderer, "slabs");

        renderer.render(gl, state);
    }
    fn render_memos(&mut self, gl: &WebGlRenderingContext, state: &State) {
        unimplemented!()
//        gl.bind_framebuffer(GL::FRAMEBUFFER, None);
//
//        let memo_shader = self.shader_sys.get_shader(&ShaderKind::Memo).unwrap();
//
//        self.shader_sys.use_program(gl, ShaderKind::Memo);
//
//        let renderer = SlabRenderer::new(memo_shader);
//
//        self.prepare_for_render(gl, &renderer, "memos");
//
//        renderer.render(gl, state);
    }

//    fn render_water(&mut self, gl: &WebGlRenderingContext, state: &State) {
//        gl.bind_framebuffer(GL::FRAMEBUFFER, None);
//
//        let water_shader = self.shader_sys.get_shader(&ShaderKind::Water).unwrap();
//        self.shader_sys.use_program(gl, ShaderKind::Water);
//
//        let water_tile = RenderableWaterTile::new(water_shader);
//
//        self.prepare_for_render(gl, &water_tile, "water");
//        water_tile.render(gl, state);
//    }

//    fn render_refraction_fbo(
//        &mut self,
//        gl: &WebGlRenderingContext,
//        state: &State,
//        assets: &Assets,
//    ) {
//        let Framebuffer { framebuffer, .. } = &self.refraction_framebuffer;
//        gl.bind_framebuffer(GL::FRAMEBUFFER, framebuffer.as_ref());
//
//        gl.viewport(0, 0, REFRACTION_TEXTURE_WIDTH, REFRACTION_TEXTURE_HEIGHT);
//
//        gl.clear_color(0.53, 0.8, 0.98, 1.);
//        gl.clear(GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT);
//
//        if state.water().use_refraction {
//            let clip_plane = [0., -1., 0., WATER_TILE_Y_POS];
//            self.render_meshes(gl, state, assets, clip_plane, false);
//        }
//    }

//    fn render_reflection_fbo(
//        &mut self,
//        gl: &WebGlRenderingContext,
//        state: &State,
//        assets: &Assets,
//    ) {
//        let Framebuffer { framebuffer, .. } = &self.reflection_framebuffer;
//        gl.bind_framebuffer(GL::FRAMEBUFFER, framebuffer.as_ref());
//
//        gl.viewport(0, 0, REFLECTION_TEXTURE_WIDTH, REFLECTION_TEXTURE_HEIGHT);
//
//        gl.clear_color(0.53, 0.8, 0.98, 1.);
//        gl.clear(GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT);
//
//        if state.water().use_reflection {
//            let clip_plane = [0., 1., 0., -WATER_TILE_Y_POS];
//            self.render_meshes(gl, state, assets, clip_plane, true);
//        }
//    }

//    fn render_refraction_visual(&self, gl: &WebGlRenderingContext, state: &State) {
//        let quad_shader = self
//            .shader_sys
//            .get_shader(&ShaderKind::TexturedQuad)
//            .unwrap();
//        self.shader_sys.use_program(gl, ShaderKind::TexturedQuad);
//        let textured_quad = TexturedQuad::new(
//            0,
//            CANVAS_HEIGHT as u16,
//            75,
//            75,
//            TextureUnit::Refraction as u8,
//            quad_shader,
//        );
//        self.prepare_for_render(gl, &textured_quad, "RefractionVisual");
//        textured_quad.render(gl, state);
//    }
//
//    fn render_reflection_visual(&self, gl: &WebGlRenderingContext, state: &State) {
//        let quad_shader = self
//            .shader_sys
//            .get_shader(&ShaderKind::TexturedQuad)
//            .unwrap();
//        self.shader_sys.use_program(gl, ShaderKind::TexturedQuad);
//        let textured_quad = TexturedQuad::new(
//            CANVAS_WIDTH as u16 - 75,
//            CANVAS_HEIGHT as u16,
//            75,
//            75,
//            TextureUnit::Reflection as u8,
//            quad_shader,
//        );
//
//        self.prepare_for_render(gl, &textured_quad, "ReflectionVisual");
//        textured_quad.render(gl, state);
//    }

    fn create_vao(&self) -> Vao {
        let oes_vao_ext = &self.vao_ext.oes_vao_ext;

        let create_vao_ext = Reflect::get(oes_vao_ext, &"createVertexArrayOES".into())
            .expect("Create vao func")
            .into();

        Vao(
            Reflect::apply(&create_vao_ext, oes_vao_ext, &js_sys::Array::new())
                .expect("Created vao")
                .into(),
        )

    }

    fn prepare_for_render<'a>(
        &self,
        gl: &WebGlRenderingContext,
        renderable: &impl Render<'a>,
        key: &str,
    ) {
        if self.vao_ext.vaos.borrow().get(key).is_none() {
            let vao = self.create_vao();
            self.bind_vao(&vao);
            renderable.buffer_attributes(gl);
            self.vao_ext.vaos.borrow_mut().insert(key.to_string(), vao);
            return;
        }

        let vaos = self.vao_ext.vaos.borrow();
        let vao = vaos.get(key).unwrap();
        self.bind_vao(vao);
    }

    fn bind_vao(&self, vao: &Vao) {
        let oes_vao_ext = &self.vao_ext.oes_vao_ext;

        let bind_vao_ext = Reflect::get(&oes_vao_ext, &"bindVertexArrayOES".into())
            .expect("Create vao func")
            .into();

        let args = js_sys::Array::new();
        args.push(&vao.0);

        Reflect::apply(&bind_vao_ext, oes_vao_ext, &args).expect("Bound VAO");
    }
}
