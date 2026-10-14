//! GPU video surface (`video` feature).
//!
//! Renders libmpv frames into an OpenGL framebuffer and blits it onto the egui
//! surface through an `egui_glow` paint callback.
//!
//! The integration pattern (render-context lifetime handling, FBO management,
//! blit callback) follows the battle-tested `egui-sharkplayer` crate, but the
//! renderer is driven by **our** `Mpv` handle, so Anime4K `glsl-shaders`,
//! track selection and every other property keep working on the same core.

use std::ffi::{CStr, CString, c_void};
use std::os::raw::c_char;
use std::sync::Arc;

use eframe::glow::{self, HasContext as _};
use libmpv2::Mpv;
use libmpv2::render::{
    OpenGLInitParams, RenderContext, RenderParam, RenderParamApiType, mpv_render_update,
};

type GpaFn = dyn Fn(&str) -> *mut c_void + Send + Sync;

/// Upper bound for the presentation FBO (~8.3MP = 4K). On wider displays
/// (5K/8K) the FBO renders at 4K and the blit upscales with LINEAR: saves
/// tens of MB VRAM for 480p-in-8K-window cases with no visible loss.
const MAX_FBO_PIXELS: i64 = 3840 * 2160;

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
            // Zero-alloc fast path: GL entry names are short ASCII, so a
            // stack buffer avoids a `CString` heap alloc per lookup (dozens
            // per shader compile). Falls back to `CString` for odd names.
            let bytes = name.as_bytes();
            if bytes.len() + 1 < 256 && !bytes.contains(&0) {
                let mut buf = [0 as c_char; 256];
                for (i, b) in bytes.iter().enumerate() {
                    buf[i] = *b as c_char;
                }
                // SAFETY: `buf` is NUL-terminated by construction (`len+1 <
                // 256`, rest zeros) and lives for the call; `get_proc_address`
                // only reads it synchronously.
                let cstr = unsafe { CStr::from_ptr(buf.as_ptr()) };
                return proc_addr(cstr).cast_mut();
            }
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
        let mut w = (rect.width() * ppp).floor() as i32;
        let mut h = (rect.height() * ppp).floor() as i32;
        if w <= 0 || h <= 0 {
            return;
        }
        // Cap VRAM: huge windows render at most 4K into the FBO; the blit
        // below upscales to the full rect with LINEAR.
        let pixels = w as i64 * h as i64;
        if pixels > MAX_FBO_PIXELS {
            let scale = (MAX_FBO_PIXELS as f64 / pixels as f64).sqrt();
            w = ((w as f64 * scale).floor() as i32).max(1);
            h = ((h as f64 * scale).floor() as i32).max(1);
        }
        let size = FramebufferSize {
            width: w,
            height: h,
        };

        let (res, fresh) = match self.ensure_resources(size) {
            Ok(res) => res,
            Err(err) => {
                let msg = format!("video framebuffer allocation failed: {err:#}");
                eprintln!("{msg}");
                self.last_error = Some(msg);
                return;
            }
        };

        let flags = self.ctx.update().unwrap_or(0);
        // While paused mpv sends no new frames, but a fresh (just
        // reallocated) framebuffer starts empty: opening the side panel
        // resizes the rect, reallocates, and the paused picture would go
        // black until the next frame. Re-render unconditionally then.
        if fresh || (flags & mpv_render_update::Frame != 0) {
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

    /// Returns the framebuffer plus whether it was (re)allocated by this
    /// call (see `show`: a fresh target must be re-rendered even when mpv
    /// reports no new frame, e.g. while paused).
    fn ensure_resources(&mut self, size: FramebufferSize) -> anyhow::Result<(GlResources, bool)> {
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
        Ok((self.resources.expect("just allocated"), needs_alloc))
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
