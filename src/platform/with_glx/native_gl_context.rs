use std::ffi::CString;

use glx;
use x11::xlib::*;
use libc::*;
use glx::types::{GLXContext, GLXDrawable, GLXFBConfig, GLXPixmap};
use geom::Size2D;
use super::utils::{create_offscreen_pixmap_backed_context};

use platform::NativeGLContextMethods;

#[cfg(feature="texture_surface")]
use layers::platform::surface::NativeGraphicsMetadata;

pub struct NativeGLContext {
    native_context: GLXContext,
    native_display: *mut glx::types::Display,
    native_drawable: GLXDrawable,
}

impl NativeGLContext {
    pub fn new(share_context: Option<&NativeGLContext>,
               display: *mut glx::types::Display,
               drawable: GLXDrawable,
               framebuffer_config: GLXFBConfig)
        -> Result<NativeGLContext, &'static str> {

        let shared = match share_context {
            Some(ctx) => ctx.as_native_glx_context(),
            None      => 0 as GLXContext
        };

        let native = unsafe { glx::CreateNewContext(display, framebuffer_config, glx::RGBA_TYPE as c_int, shared, 1 as glx::types::Bool) };

        // FIXME: This should be:
        // if native == (0 as *const c_void) {
        // but that way compilation fails with error:
        //  expected `*const libc::types::common::c95::c_void`,
        //     found `*const libc::types::common::c95::c_void`
        // (expected enum `libc::types::common::c95::c_void`,
        //     found a different enum `libc::types::common::c95::c_void`) [E0308]
        if (native as *const c_void) == (0 as *const c_void) {
            unsafe { glx::DestroyPixmap(display, drawable as GLXPixmap) };
            return Err("Error creating native glx context");
        }

        Ok(NativeGLContext {
            native_context: native,
            native_display: display,
            native_drawable: drawable,
        })
    }

    pub fn as_native_glx_context(&self) -> GLXContext {
        self.native_context
    }
}

impl Drop for NativeGLContext {
    fn drop(&mut self) {
        self.make_current().unwrap();
        unsafe {
            glx::DestroyContext(self.native_display, self.native_context);
            glx::DestroyPixmap(self.native_display, self.native_drawable as GLXPixmap);
        }
    }
}

impl NativeGLContextMethods for NativeGLContext {
    fn get_proc_address(addr: &str) -> *const () {
        let addr = CString::new(addr.as_bytes()).unwrap();
        let addr = addr.as_ptr();
        unsafe {
            glx::GetProcAddress(addr as *const _) as *const ()
        }
    }

    fn create_headless() -> Result<NativeGLContext, &'static str> {
        // We create a context with a dummy size since in other platforms
        // a default framebuffer is not bound
        create_offscreen_pixmap_backed_context(Size2D::new(16, 16))
    }

    #[inline(always)]
    fn is_current(&self) -> bool {
        unsafe {
            glx::GetCurrentContext() == self.native_context
        }
    }

    fn make_current(&self) -> Result<(), &'static str> {
        unsafe {
            if !self.is_current()
                && glx::MakeCurrent(self.native_display,
                                    self.native_drawable,
                                    self.native_context) == 0 {
                Err("glx::MakeContextCurrent")
            } else {
                Ok(())
            }
        }
    }

    #[cfg(feature="texture_surface")]
    fn get_metadata(&self) -> NativeGraphicsMetadata {
        NativeGraphicsMetadata {
            display: self.native_display as *mut Display,
        }
    }
}

