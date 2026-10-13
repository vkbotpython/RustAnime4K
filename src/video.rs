//! GPU video surface (`video` feature).
//!
//! Renders libmpv frames into an OpenGL framebuffer and blits it onto the egui
//! surface through an `egui_glow` paint callback.
//!
//! The integration pattern (render-context lifetime handling, FBO management,
//! blit callback) follows the battle-tested `egui-sharkplayer` crate, but the
//! renderer is driven by **our** `Mpv` handle, so Anime4K `glsl-shaders`,
//! track selection and every other property keep working on the same core.

use std::ffi::{c_void, CString};
use std::sync::Arc;

use eframe::glow::{self, HasContext as _};
use libmpv2::render::{
    mpv_render_update, OpenGLInitParams, RenderContext, RenderParam, RenderParamApiType,
};
use libmpv2::Mpv;

type GpaFn = dyn Fn(&str) -> *mut c_void + Send + Sync;

/// Free function used as `OpenGLInitParams::get_proc_address`.
fn resolve_gl_proc(gpa: &Arc<GpaFn>, name: &str) -> *mut c_void {
    gpa(name)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FramebufferSize {
    width: i32,
    height: i32,
}

#[derive(Debug, Clone, Copy)]
struct GlResources {
    texture: glow::NativeTexture,
    framebuffer: glow::NativeFramebuffer,
    size: FramebufferSize,
}

/// UI-thread-only video renderer. `!Send + !Sync` (like `RenderContext`),
/// so it lives in the `eframe::App` struct, not in the shared backend.
pub struct VideoRenderer {
    // SAFETY: `ctx` must drop before `_mpv`; field order guarantees this.
    ctx: RenderContext<'static>,
    _mpv: Arc<Mpv>,
    gl: Arc<glow::Context>,
    _gpa: Arc<GpaFn>,
    resources: Option<GlResources>,
    /// Latest render failure, if any. The GUI build has no console
    /// (`windows_subsystem`), so `eprintln!` output is invisible — the
    /// message is surfaced in the video panel instead.
    last_error: Option<String>,
}

impl VideoRenderer {
    pub fn new(cc: &eframe::CreationContext<'_>, mpv: Arc<Mpv>) -> anyhow::Result<Self> {
        let gl = cc
            .gl
            .clone()
            .ok_or_else(|| anyhow::anyhow!("glow context unavailable (need the glow renderer)"))?;
        let proc_addr = cc.get_proc_address.clone().ok_or_else(|| {
            anyhow::anyhow!("get_proc_address unavailable despite glow being active")
        })?;
        let gpa: Arc<GpaFn> = Arc::new(move |name: &str| {
            CString::new(name)
                .ok()
                .map_or(std::ptr::null_mut(), |s| proc_addr(s.as_c_str()).cast_mut())
        });

        let render_ctx = mpv.create_render_context([
            RenderParam::ApiType(RenderParamApiType::OpenGl),
            RenderParam::InitParams(OpenGLInitParams {
                get_proc_address: resolve_gl_proc,
                ctx: Arc::clone(&gpa),
            }),
        ])?;

        // SAFETY: the transmuted lifetime is sound as long as the render
        // context drops before `mpv`. Both live in this struct (`ctx` is
        // declared first, so it drops first) and `_mpv` keeps the core alive.
        let mut render_ctx: RenderContext<'static> =
            unsafe { std::mem::transmute::<RenderContext<'_>, RenderContext<'static>>(render_ctx) };
        let egui_ctx = cc.egui_ctx.clone();
        render_ctx.set_update_callback(move || {
            egui_ctx.request_repaint();
        });

        Ok(Self {
            ctx: render_ctx,
            _mpv: mpv,
            gl,
            _gpa: gpa,
            resources: None,
            last_error: None,
        })
    }

    /// Latest render failure, if any (see `last_error`).
    pub fn last_error(&self) -> Option<&str> {
        self.last_error.as_deref()
    }

    /// Renders the current frame (if any) and schedules the blit for `rect`.
    pub fn show(&mut self, ui: &mut egui::Ui, rect: egui::Rect) {
        let ppp = ui.ctx().pixels_per_point();
        let size = FramebufferSize {
            width: (rect.width() * ppp).floor() as i32,
            height: (rect.height() * ppp).floor() as i32,
        };
        if size.width <= 0 || size.height <= 0 {
            return;
        }

        let res = match self.ensure_resources(size) {
            Ok(res) => res,
            Err(err) => {
                let msg = format!("video framebuffer allocation failed: {err:#}");
                eprintln!("{msg}");
                self.last_error = Some(msg);
                return;
            }
        };

        let flags = self.ctx.update().unwrap_or(0);
        if flags & mpv_render_update::Frame != 0 {
            match res.framebuffer.0.get().try_into() {
                Ok(fb) => {
                    if let Err(err) = self.ctx.render::<()>(fb, size.width, size.height, true) {
                        let msg = format!("mpv render failed: {err}");
                        eprintln!("{msg}");
                        self.last_error = Some(msg);
                    } else {
                        // Tell mpv the frame was presented: without this the
                        // core can't tune A-V timing ("expect bad video
                        // playback" otherwise, per the render API docs).
                        self.ctx.report_swap();
                        self.last_error = None;
                    }
                }
                Err(err) => {
                    let msg = format!("GL framebuffer id too large for libmpv: {err}");
                    eprintln!("{msg}");
                    self.last_error = Some(msg);
                    return;
                }
            }
        }

        let fb = res.framebuffer;
        let cb = eframe::egui_glow::CallbackFn::new(move |info, painter| {
            blit_frame(painter.gl(), rect, fb, size, &info);
        });
        ui.painter().add(egui::PaintCallback {
            rect,
            callback: Arc::new(cb),
        });
    }

    fn ensure_resources(&mut self, size: FramebufferSize) -> anyhow::Result<GlResources> {
        let needs_alloc = match &self.resources {
            Some(res) => res.size != size,
            None => true,
        };
        if needs_alloc {
            if let Some(old) = self.resources.take() {
                unsafe {
                    self.gl.delete_framebuffer(old.framebuffer);
                    self.gl.delete_texture(old.texture);
                }
            }
            self.resources = Some(unsafe { allocate_framebuffer(&self.gl, size)? });
        }
        Ok(self.resources.expect("just allocated"))
    }

    /// Frees GL resources; call from `App::on_exit`.
    pub fn destroy_gl_resources(&mut self) {
        if let Some(res) = self.resources.take() {
            unsafe {
                self.gl.delete_framebuffer(res.framebuffer);
                self.gl.delete_texture(res.texture);
            }
        }
    }
}

/// Allocates an RGBA8 texture + framebuffer.
///
/// # Safety
///
/// The GL context must be current on this thread (always true here: every
/// call happens on the eframe UI thread).
unsafe fn allocate_framebuffer(
    gl: &glow::Context,
    size: FramebufferSize,
) -> anyhow::Result<GlResources> {
    unsafe {
        let texture = gl
            .create_texture()
            .map_err(|e| anyhow::anyhow!("create_texture: {e}"))?;
        gl.bind_texture(glow::TEXTURE_2D, Some(texture));
        gl.tex_image_2d(
            glow::TEXTURE_2D,
            0,
            glow::RGBA8.cast_signed(),
            size.width,
            size.height,
            0,
            glow::RGBA,
            glow::UNSIGNED_BYTE,
            glow::PixelUnpackData::Slice(None),
        );
        for (name, val) in [
            (glow::TEXTURE_MIN_FILTER, glow::LINEAR.cast_signed()),
            (glow::TEXTURE_MAG_FILTER, glow::LINEAR.cast_signed()),
            (glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE.cast_signed()),
            (glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE.cast_signed()),
        ] {
            gl.tex_parameter_i32(glow::TEXTURE_2D, name, val);
        }
        gl.bind_texture(glow::TEXTURE_2D, None);

        let framebuffer = gl
            .create_framebuffer()
            .map_err(|e| anyhow::anyhow!("create_framebuffer: {e}"))?;
        gl.bind_framebuffer(glow::FRAMEBUFFER, Some(framebuffer));
        gl.framebuffer_texture_2d(
            glow::FRAMEBUFFER,
            glow::COLOR_ATTACHMENT0,
            glow::TEXTURE_2D,
            Some(texture),
            0,
        );

        let status = gl.check_framebuffer_status(glow::FRAMEBUFFER);
        if status != glow::FRAMEBUFFER_COMPLETE {
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
            gl.delete_framebuffer(framebuffer);
            gl.delete_texture(texture);
            anyhow::bail!("incomplete framebuffer (0x{status:x}); need RGBA8 support");
        }
        gl.bind_framebuffer(glow::FRAMEBUFFER, None);

        Ok(GlResources {
            texture,
            framebuffer,
            size,
        })
    }
}

/// Blits the rendered FBO onto the screen rect. Runs inside the
/// `egui_glow` paint callback, so it must only use `Copy` captures.
fn blit_frame(
    gl: &glow::Context,
    rect: egui::Rect,
    fb: glow::NativeFramebuffer,
    fb_size: FramebufferSize,
    info: &egui::PaintCallbackInfo,
) {
    unsafe {
        let prev_read_fb = gl
            .get_parameter_i32(eframe::glow::READ_FRAMEBUFFER_BINDING)
            .cast_unsigned();
        gl.bind_framebuffer(eframe::glow::READ_FRAMEBUFFER, Some(fb));

        let ppp = info.pixels_per_point;
        let screen_h = info.screen_size_px[1] as f32;
        gl.blit_framebuffer(
            0,
            0,
            fb_size.width,
            fb_size.height,
            (rect.min.x * ppp) as i32,
            (screen_h - rect.max.y * ppp) as i32,
            (rect.max.x * ppp) as i32,
            (screen_h - rect.min.y * ppp) as i32,
            eframe::glow::COLOR_BUFFER_BIT,
            eframe::glow::LINEAR,
        );

        let prev_fb = std::num::NonZeroU32::new(prev_read_fb).map(eframe::glow::NativeFramebuffer);
        gl.bind_framebuffer(eframe::glow::READ_FRAMEBUFFER, prev_fb);
    }
}
