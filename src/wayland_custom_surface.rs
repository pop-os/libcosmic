use gdk4_wayland::{WaylandDisplay, WaylandPopup};
use gtk4::{
    gdk,
    glib::{self, translate::*},
};
use std::boxed::Box as Box_;
use std::fmt;

mod ffi {
    use gdk4_wayland::ffi::{GdkWaylandDisplay, GdkWaylandPopup};
    use gtk4::{
        gdk::ffi::GdkFrameClock,
        glib::ffi::{gboolean, gpointer, GDestroyNotify, GType},
    };
    use std::os::raw::c_int;

    pub type GdkWaylandCustomSurfaceGetPopupFunc = Option<
        unsafe extern "C" fn(
            *mut GdkWaylandCustomSurface,
            *mut GdkWaylandPopup,
            gpointer,
        ) -> gboolean,
    >;

    #[repr(C)]
    pub struct GdkWaylandCustomSurface {
        _data: [u8; 0],
        _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
    }

    impl ::std::fmt::Debug for GdkWaylandCustomSurface {
        fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            f.debug_struct(&format!("GdkWaylandCustomSurface @ {:p}", self))
                .finish()
        }
    }

    extern "C" {
        pub fn gdk_wayland_custom_surface_get_type() -> GType;

        pub fn gdk_wayland_custom_surface_new(
            display: *mut GdkWaylandDisplay,
        ) -> *mut GdkWaylandCustomSurface;

        pub fn gdk_wayland_custom_surface_present(
            custom_surface: *mut GdkWaylandCustomSurface,
            width: c_int,
            height: c_int,
        );

        pub fn gdk_wayland_custom_surface_set_get_popup_func(
            custom_surface: *mut GdkWaylandCustomSurface,
            get_popup_func: GdkWaylandCustomSurfaceGetPopupFunc,
            user_data: gpointer,
            destroy: GDestroyNotify,
        );

        pub fn _gdk_frame_clock_idle_new() -> *mut GdkFrameClock;
    }
}

glib::wrapper! {
    #[doc(alias = "GdkWaylandCustomSurface")]
    pub struct WaylandCustomSurface(Object<ffi::GdkWaylandCustomSurface>) @extends gdk4_wayland::WaylandSurface, gdk::Surface;

    match fn {
        type_ => || ffi::gdk_wayland_custom_surface_get_type(),
    }
}

impl WaylandCustomSurface {
    #[doc(alias = "gdk_wayland_custom_surface_new")]
    pub fn new(display: &WaylandDisplay) -> WaylandCustomSurface {
        unsafe {
            from_glib_full(ffi::gdk_wayland_custom_surface_new(
                display.to_glib_none().0,
            ))
        }
    }

    #[doc(alias = "gdk_wayland_custom_surface_present")]
    pub fn present(&self, width: i32, height: i32) {
        unsafe {
            ffi::gdk_wayland_custom_surface_present(self.to_glib_none().0, width, height);
        }
    }

    #[doc(alias = "gdk_wayland_custom_surface_set_get_popup_func")]
    pub fn set_get_popup_func(
        &self,
        get_popup_func: Option<
            Box_<dyn Fn(&WaylandCustomSurface, &WaylandPopup) -> bool + 'static>,
        >,
    ) {
        let get_popup_func_data: Box_<
            Option<Box_<dyn Fn(&WaylandCustomSurface, &WaylandPopup) -> bool + 'static>>,
        > = Box_::new(get_popup_func);
        unsafe extern "C" fn get_popup_func_func(
            custom_surface: *mut ffi::GdkWaylandCustomSurface,
            popup: *mut gdk4_wayland::ffi::GdkWaylandPopup,
            user_data: glib::ffi::gpointer,
        ) -> glib::ffi::gboolean {
            let custom_surface = from_glib_borrow(custom_surface);
            let popup = from_glib_borrow(popup);
            let callback: &Option<
                Box_<dyn Fn(&WaylandCustomSurface, &WaylandPopup) -> bool + 'static>,
            > = &*(user_data as *mut _);
            let res = if let Some(ref callback) = *callback {
                callback(&custom_surface, &popup)
            } else {
                panic!("cannot get closure...")
            };
            res.into_glib()
        }
        let get_popup_func = if get_popup_func_data.is_some() {
            Some(get_popup_func_func as _)
        } else {
            None
        };
        unsafe extern "C" fn destroy_func(data: glib::ffi::gpointer) {
            let _callback: Box_<
                Option<Box_<dyn Fn(&WaylandCustomSurface, &WaylandPopup) -> bool + 'static>>,
            > = Box_::from_raw(data as *mut _);
        }
        let destroy_call3 = Some(destroy_func as _);
        let super_callback0: Box_<
            Option<Box_<dyn Fn(&WaylandCustomSurface, &WaylandPopup) -> bool + 'static>>,
        > = get_popup_func_data;
        unsafe {
            ffi::gdk_wayland_custom_surface_set_get_popup_func(
                self.to_glib_none().0,
                get_popup_func,
                Box_::into_raw(super_callback0) as *mut _,
                destroy_call3,
            );
        }
    }
}

impl fmt::Display for WaylandCustomSurface {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("WaylandCustomSurface")
    }
}
