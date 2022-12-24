use eframe::egui_glow;
use three_d::{Camera, Viewport, vec3, degrees, Gm, Color, Mesh, ColorMaterial, ClearState, Context};
use three_d::renderer::geometry::CpuMesh;
use std::cell::RefCell;
use std::sync::Arc;

/*
fn get_paint_callback(rect: rect) -> egui::PaintCallback {
    egui::PaintCallback {
        rect,
        callback: Arc::new(egui_glow::CallbackFn::new(move |info, painter| {
            with_three_d(painter.gl(), |three_d| three_d.frame())
        })),
    }
}*/

/// Gives us a [Renderer] object to do render-y stuff with 
/// src: https://github.com/emilk/egui/blob/master/examples/custom_3d_three-d/src/main.rs
pub fn with_three_d<R>(gl: &std::sync::Arc<glow::Context>, f: impl FnOnce(&mut Renderer) -> R) -> R {
    thread_local! {
        pub static THREE_D: RefCell<Option<Renderer>> = RefCell::new(None);
    }

    THREE_D.with(|three_d| {
        let mut three_d = three_d.borrow_mut();
        let three_d = three_d.get_or_insert_with(|| Renderer::new(gl.clone()));
        f(three_d)
    })
}

///
/// Translates from egui input to three-d input
/// src: https://github.com/emilk/egui/blob/master/examples/custom_3d_three-d/src/main.rs
///
pub struct FrameInput<'a> {
    screen: three_d::RenderTarget<'a>,
    viewport: three_d::Viewport,
    scissor_box: three_d::ScissorBox,
}

impl FrameInput<'_> {
    pub fn new(
        context: &three_d::Context,
        info: &egui::PaintCallbackInfo,
        painter: &egui_glow::Painter,
    ) -> Self {
        use three_d::*;

        // Disable sRGB textures for three-d
        #[cfg(not(target_arch = "wasm32"))]
        #[allow(unsafe_code)]
        unsafe {
            use glow::HasContext as _;
            context.disable(glow::FRAMEBUFFER_SRGB);
        }

        // Constructs a screen render target to render the final image to
        let screen = painter.intermediate_fbo().map_or_else(
            || {
                RenderTarget::screen(
                    context,
                    info.viewport.width() as u32,
                    info.viewport.height() as u32,
                )
            },
            |fbo| {
                RenderTarget::from_framebuffer(
                    context,
                    info.viewport.width() as u32,
                    info.viewport.height() as u32,
                    fbo,
                )
            },
        );

        // Set where to paint
        let viewport = info.viewport_in_pixels();
        let viewport = Viewport {
            x: viewport.left_px.round() as _,
            y: viewport.from_bottom_px.round() as _,
            width: viewport.width_px.round() as _,
            height: viewport.height_px.round() as _,
        };

        // Respect the egui clip region (e.g. if we are inside an `egui::ScrollArea`).
        let clip_rect = info.clip_rect_in_pixels();
        let scissor_box = ScissorBox {
            x: clip_rect.left_px.round() as _,
            y: clip_rect.from_bottom_px.round() as _,
            width: clip_rect.width_px.round() as _,
            height: clip_rect.height_px.round() as _,
        };
        Self {
            screen,
            scissor_box,
            viewport,
        }
    }
}

pub struct Renderer {
    pub context: Context,
    camera: Camera,
    test_model: Gm<Mesh, ColorMaterial>,
} 

impl Renderer {
    fn new(ctx: Arc<glow::Context>) -> Self {
        let three_d_ctx = three_d::Context::from_gl_context(ctx).unwrap();
        let camera = Camera::new_perspective(
            Viewport { x: 0, y: 0, width: 0, height: 0 },
            vec3(0.0, 0.0, 0.0),
            vec3(0.0, 0.0, 0.0),
            vec3(0.0, 1.0, 0.0),
            degrees(90.0),
            0.1,
            20000.0,
        );

        let pos = vec![
            vec3(0.5, -0.5, 0.0),
            vec3(-0.5, -0.5, 0.0),
            vec3(0.0, 0.5, 0.0),
        ];

        let col = vec![
            Color::new(255, 0, 0, 255),
            Color::new(0, 255, 0, 255),
            Color::new(0, 0, 255, 255),
        ];
        
        let trimesh = CpuMesh {
            positions: three_d::Positions::F32(pos),
            colors: Some(col),
            ..Default::default()
        };

        let model = Gm::new(Mesh::new(&three_d_ctx, &trimesh), ColorMaterial::default());

        Self {
            context: three_d_ctx,
            camera,
            test_model: model,
        }

    }
    
    pub fn render(&mut self, frame_input: FrameInput<'_>) -> Option<glow::Framebuffer> {
        self.camera.set_viewport(frame_input.viewport);

        frame_input.screen.clear_partially(frame_input.scissor_box, ClearState::depth(1.0));
        frame_input.screen.render_partially(frame_input.scissor_box, &self.camera, [&self.test_model], &[]); 
        frame_input.screen.into_framebuffer()
    }
}
