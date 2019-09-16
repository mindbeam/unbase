use crate::render::MeshRenderOpts;
use crate::render::NonSkinnedMesh;
use crate::render::Render;
use crate::render::SkinnedMesh;
use crate::render::WebRenderer;
use crate::shader::ShaderKind;
use crate::Assets;
use crate::State;
use web_sys::WebGlRenderingContext as GL;

impl WebRenderer {
    pub(in crate::render) fn render_meshes(
        &self,
        gl: &GL,
        state: &State,
        assets: &Assets,
        clip_plane: [f32; 4],
        flip_camera_y: bool,
    ) {
        if !state.show_scenery() {
            return;
        }

        let (skin, no_skin) = (ShaderKind::SkinnedMesh, ShaderKind::NonSkinnedMesh);
    }
}
