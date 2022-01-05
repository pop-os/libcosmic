// TODO: scale-factor?

use derivative::Derivative;
use gdk4_wayland::prelude::*;
use gtk4::{
    cairo, gdk,
    glib::{self, clone, subclass::prelude::*, translate::*},
    gsk::{self, traits::RendererExt},
    prelude::*,
    subclass::prelude::*,
};
use std::{
    cell::{Cell, RefCell},
    os::raw::c_int,
    ptr,
    rc::Rc,
};
use wayland_client::{event_enum, sys::client::wl_proxy, Filter, GlobalManager, Main, Proxy};
use wayland_protocols::{
    wlr::unstable::layer_shell::v1::client::{zwlr_layer_shell_v1, zwlr_layer_surface_v1},
    xdg_shell::client::xdg_popup,
};

use crate::{deref_cell::DerefCell, wayland_custom_surface::WaylandCustomSurface};

pub use wayland_protocols::wlr::unstable::layer_shell::v1::client::{
    zwlr_layer_shell_v1::Layer,
    zwlr_layer_surface_v1::{Anchor, KeyboardInteractivity},
};

event_enum!(
    Events |
    LayerSurface => zwlr_layer_surface_v1::ZwlrLayerSurfaceV1
);

struct CosmicWaylandDisplay {
    event_queue: RefCell<wayland_client::EventQueue>,
    wayland_display: wayland_client::Display,
    wlr_layer_shell: Option<Main<zwlr_layer_shell_v1::ZwlrLayerShellV1>>,
}

impl CosmicWaylandDisplay {
    fn for_display(display: &gdk4_wayland::WaylandDisplay) -> Rc<Self> {
        const DATA_KEY: &str = "cosmic-wayland-display";

        // `GdkWaylandDisplay` already associated with a `CosmicWaylandDisplay`
        if let Some(data) = unsafe { display.data::<Rc<Self>>(DATA_KEY) } {
            return unsafe { data.as_ref() }.clone();
        }

        let wayland_display = unsafe {
            wayland_client::Display::from_external_display(
                display.wl_display().as_ref().c_ptr() as *mut _
            )
        }; // XXX is this sound?

        let mut event_queue = wayland_display.create_event_queue();
        let attached_display = wayland_display.attach(event_queue.token());
        let globals = GlobalManager::new(&attached_display);

        event_queue.sync_roundtrip(&mut (), |_, _, _| {}).unwrap();

        let wlr_layer_shell = globals
            .instantiate_exact::<zwlr_layer_shell_v1::ZwlrLayerShellV1>(1)
            .ok();

        let cosmic_wayland_display = Rc::new(Self {
            event_queue: RefCell::new(event_queue),
            wayland_display,
            wlr_layer_shell,
        });

        unsafe { display.set_data(DATA_KEY, cosmic_wayland_display.clone()) };

        // XXX Efficient way to poll?
        // XXX unwrap?
        glib::idle_add_local(
            clone!(@weak cosmic_wayland_display => @default-return Continue(false), move || {
                cosmic_wayland_display.wayland_display.flush().unwrap();
                let mut event_queue = cosmic_wayland_display.event_queue.borrow_mut();
                if let Some(guard) = event_queue.prepare_read() {
                    guard.read_events().unwrap();
                }
                event_queue.dispatch_pending(&mut (), |_, _, _| {}).unwrap();
                Continue(true)
            }),
        );

        cosmic_wayland_display
    }
}

#[derive(Derivative)]
#[derivative(Default)]
pub struct LayerShellWindowInner {
    display: DerefCell<gdk4_wayland::WaylandDisplay>,
    surface: RefCell<Option<WaylandCustomSurface>>,
    renderer: RefCell<Option<gsk::Renderer>>,
    wlr_layer_surface: RefCell<Option<Main<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1>>>,
    constraint_solver: DerefCell<ConstraintSolver>,
    child: RefCell<Option<gtk4::Widget>>,
    monitor: DerefCell<Option<gdk4_wayland::WaylandMonitor>>,
    #[derivative(Default(value = "Cell::new(Layer::Background)"))]
    layer: Cell<Layer>,
    #[derivative(Default(value = "Cell::new(Anchor::empty())"))]
    anchor: Cell<Anchor>,
    exclusive_zone: Cell<i32>,
    margin: Cell<(i32, i32, i32, i32)>,
    #[derivative(Default(value = "Cell::new(KeyboardInteractivity::None)"))]
    keyboard_interactivity: Cell<KeyboardInteractivity>,
    namespace: DerefCell<String>,
    focus_widget: RefCell<Option<gtk4::Widget>>,
}

#[glib::object_subclass]
impl ObjectSubclass for LayerShellWindowInner {
    const NAME: &'static str = "S76CosmicLayerShellWindow";
    type ParentType = gtk4::Widget;
    type Interfaces = (gtk4::Native, gtk4::Root);
    type Type = LayerShellWindow;
}

impl ObjectImpl for LayerShellWindowInner {
    fn constructed(&self, obj: &Self::Type) {
        self.display
            .set(gdk::Display::default().unwrap().downcast().unwrap()); // XXX any issue unwrapping?
        self.constraint_solver.set(glib::Object::new(&[]).unwrap());

        obj.add_css_class("background");
    }
}

impl WidgetImpl for LayerShellWindowInner {
    fn realize(&self, widget: &Self::Type) {
        let surface = WaylandCustomSurface::new(&*self.display);
        surface.set_get_popup_func(Some(Box::new(clone!(@strong widget => move |_surface, popup| {
            if let Some(wlr_layer_surface) = widget.inner().wlr_layer_surface.borrow().as_ref() {
                let xdg_popup: xdg_popup::XdgPopup =
                    unsafe { Proxy::from_c_ptr(gdk_wayland_popup_get_xdg_popup(popup.to_glib_none().0)).into() };
                wlr_layer_surface.get_popup(&xdg_popup);
            }
            true
        }))));
        widget.layer_shell_init(&surface);

        let widget_ptr: *mut _ = widget.to_glib_none().0;
        let surface_ptr: *mut _ = surface.to_glib_none().0;
        unsafe { gdk_surface_set_widget(surface_ptr as *mut _, widget_ptr as *mut _) };
        *self.surface.borrow_mut() = Some(surface.clone());
        surface.connect_render(move |_surface, region| {
            unsafe {
                gtk_widget_render(
                    widget_ptr as *mut _,
                    surface_ptr as *mut _,
                    region.to_glib_none().0,
                )
            };
            true
        });
        surface.connect_event(|_, event| {
            unsafe { gtk_main_do_event(event.to_glib_none().0) };
            true
        });

        self.parent_realize(widget);

        *self.renderer.borrow_mut() =
            Some(gsk::Renderer::for_surface(surface.upcast_ref()).unwrap()); // XXX unwrap?

        unsafe { gtk4::ffi::gtk_native_realize(widget_ptr as *mut _) };
    }

    fn unrealize(&self, widget: &Self::Type) {
        let widget_ptr: *mut Self::Instance = widget.to_glib_none().0;

        unsafe { gtk4::ffi::gtk_native_unrealize(widget_ptr as *mut _) };

        self.parent_unrealize(widget);

        if let Some(renderer) = self.renderer.borrow_mut().take() {
            renderer.unrealize();
        }

        if let Some(surface) = self.surface.borrow().as_ref() {
            let surface_ptr: *mut _ = surface.to_glib_none().0;
            unsafe { gdk_surface_set_widget(surface_ptr as *mut _, ptr::null_mut()) };
        }
    }

    fn map(&self, widget: &Self::Type) {
        if let Some(surface) = self.surface.borrow().as_ref() {
            let width = widget.measure(gtk4::Orientation::Horizontal, -1).1;
            let height = widget.measure(gtk4::Orientation::Vertical, width).1;
            widget.set_size(width as u32, height as u32);
            surface.present(width, height);
        }

        self.parent_map(widget);

        if let Some(child) = self.child.borrow().as_ref() {
            child.map();
        }
    }

    fn unmap(&self, widget: &Self::Type) {
        self.parent_unmap(widget);

        if let Some(surface) = self.surface.borrow().as_ref() {
            surface.hide();
        }

        if let Some(child) = self.child.borrow().as_ref() {
            child.unmap();
        }
    }

    fn measure(
        &self,
        _widget: &Self::Type,
        orientation: gtk4::Orientation,
        for_size: i32,
    ) -> (i32, i32, i32, i32) {
        if let Some(child) = self.child.borrow().as_ref() {
            child.measure(orientation, for_size)
        } else {
            (0, 0, 0, 0)
        }
    }

    fn size_allocate(&self, _widget: &Self::Type, width: i32, height: i32, baseline: i32) {
        if let Some(child) = self.child.borrow().as_ref() {
            child.allocate(width, height, baseline, None)
        }
    }

    fn show(&self, widget: &Self::Type) {
        widget.realize();
        self.parent_show(widget);
        widget.map();
    }

    fn hide(&self, widget: &Self::Type) {
        self.parent_hide(widget);
        widget.unmap();
    }
}

// TODO: Move into gtk4-rs when support merged/released in gtk
unsafe impl IsImplementable<LayerShellWindowInner> for gtk4::Native {
    fn interface_init(iface: &mut glib::Interface<Self>) {
        let iface = unsafe { &mut *(iface as *mut _ as *mut GtkNativeInterface) };
        iface.get_surface = Some(get_surface);
        iface.get_renderer = Some(get_renderer);
        iface.get_surface_transform = Some(get_surface_transform);
        iface.layout = Some(layout);
    }

    fn instance_init(_instance: &mut glib::subclass::InitializingObject<LayerShellWindowInner>) {}
}

// TODO: Move into gtk4-rs when support merged/released in gtk
unsafe impl IsImplementable<LayerShellWindowInner> for gtk4::Root {
    fn interface_init(iface: &mut glib::Interface<Self>) {
        let iface = unsafe { &mut *(iface as *mut _ as *mut GtkRootInterface) };
        iface.get_display = Some(get_display);
        iface.get_constraint_solver = Some(get_constraint_solver);
        iface.get_focus = Some(get_focus);
        iface.set_focus = Some(set_focus);
    }

    fn instance_init(_instance: &mut glib::subclass::InitializingObject<LayerShellWindowInner>) {}
}

glib::wrapper! {
    pub struct LayerShellWindow(ObjectSubclass<LayerShellWindowInner>)
        @extends gtk4::Widget, @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Native, gtk4::Root;
}
// TODO handle configure/destroy
// TODO presumably call destroy() when appropriate?
// What do wayland-client types do when associated connection is gone? Panic? UB?
impl LayerShellWindow {
    pub fn new(
        monitor: Option<&gdk4_wayland::WaylandMonitor>,
        layer: Layer,
        namespace: &str,
    ) -> Self {
        let obj: Self = glib::Object::new(&[]).unwrap();
        obj.inner().monitor.set(monitor.cloned());
        obj.inner().layer.set(layer);
        obj.inner().namespace.set(namespace.to_string());
        obj
    }

    fn inner(&self) -> &LayerShellWindowInner {
        LayerShellWindowInner::from_instance(self)
    }

    pub fn set_child<T: IsA<gtk4::Widget>>(&self, w: Option<&T>) {
        let mut child = self.inner().child.borrow_mut();
        if let Some(child) = child.take() {
            child.unparent();
        }
        if let Some(w) = w {
            w.set_parent(self);
        }
        *child = w.map(|x| x.clone().upcast());
    }

    fn layer_shell_init(&self, surface: &WaylandCustomSurface) {
        let width = self.measure(gtk4::Orientation::Horizontal, -1).1;
        let height = self.measure(gtk4::Orientation::Vertical, width).1;
        // XXX needed for wl_surface to exist
        surface.present(width, height);

        let wl_surface = surface.wl_surface();

        let cosmic_wayland_display = CosmicWaylandDisplay::for_display(&*self.inner().display);
        let wlr_layer_shell = match cosmic_wayland_display.wlr_layer_shell.as_ref() {
            Some(wlr_layer_shell) => wlr_layer_shell,
            None => {
                eprintln!("Error: Layer shell not supported by compositor");
                return;
            }
        };

        let output = self.inner().monitor.as_ref().map(|x| x.wl_output());
        let layer = self.layer();
        let namespace = self.namespace().to_string();
        let wlr_layer_surface =
            wlr_layer_shell.get_layer_surface(&wl_surface, output.as_ref(), layer, namespace);

        wlr_layer_surface.set_anchor(self.anchor());
        wlr_layer_surface.set_exclusive_zone(self.exclusive_zone());
        let margin = self.margin();
        wlr_layer_surface.set_margin(margin.0, margin.1, margin.2, margin.3);
        wlr_layer_surface.set_keyboard_interactivity(self.keyboard_interactivity());
        wlr_layer_surface.set_size(width as u32, height as u32);

        let filter = Filter::new(
            clone!(@strong self as self_ => move |event, _, _| match event {
                Events::LayerSurface { event, object } => match event {
                    zwlr_layer_surface_v1::Event::Configure {
                        serial,
                        width: _,
                        height: _,
                    } => {
                        // TODO: should size window to match `width`/`height`
                        object.ack_configure(serial);
                    }
                    zwlr_layer_surface_v1::Event::Closed => {}
                    _ => {}
                },
            }),
        );
        wlr_layer_surface.assign(filter);

        wl_surface.commit();

        cosmic_wayland_display
            .event_queue
            .borrow_mut()
            .sync_roundtrip(&mut (), |_, _, _| {})
            .unwrap();

        *self.inner().wlr_layer_surface.borrow_mut() = Some(wlr_layer_surface);
    }

    fn set_size(&self, width: u32, height: u32) {
        if let Some(wlr_layer_surface) = self.inner().wlr_layer_surface.borrow().as_ref() {
            wlr_layer_surface.set_size(width, height);
        };
    }

    pub fn anchor(&self) -> Anchor {
        self.inner().anchor.get()
    }

    pub fn set_anchor(&self, anchor: Anchor) {
        if let Some(wlr_layer_surface) = self.inner().wlr_layer_surface.borrow().as_ref() {
            wlr_layer_surface.set_anchor(anchor);
        };
        self.inner().anchor.set(anchor);
    }

    pub fn exclusive_zone(&self) -> i32 {
        self.inner().exclusive_zone.get()
    }

    pub fn set_exclusive_zone(&self, zone: i32) {
        if let Some(wlr_layer_surface) = self.inner().wlr_layer_surface.borrow().as_ref() {
            wlr_layer_surface.set_exclusive_zone(zone);
        };
        self.inner().exclusive_zone.set(zone);
    }

    pub fn margin(&self) -> (i32, i32, i32, i32) {
        self.inner().margin.get()
    }

    pub fn set_margin(&self, top: i32, right: i32, bottom: i32, left: i32) {
        if let Some(wlr_layer_surface) = self.inner().wlr_layer_surface.borrow().as_ref() {
            wlr_layer_surface.set_margin(top, right, bottom, left);
        };
        self.inner().margin.set((top, right, bottom, left));
    }

    pub fn keyboard_interactivity(&self) -> KeyboardInteractivity {
        self.inner().keyboard_interactivity.get()
    }

    pub fn set_keyboard_interactivity(&self, interactivity: KeyboardInteractivity) {
        if let Some(wlr_layer_surface) = self.inner().wlr_layer_surface.borrow().as_ref() {
            wlr_layer_surface.set_keyboard_interactivity(interactivity);
        };
        self.inner().keyboard_interactivity.set(interactivity);
    }

    pub fn layer(&self) -> Layer {
        self.inner().layer.get()
    }

    pub fn set_layer(&self, layer: Layer) {
        if let Some(wlr_layer_surface) = self.inner().wlr_layer_surface.borrow().as_ref() {
            wlr_layer_surface.set_layer(layer);
        };
        self.inner().layer.set(layer);
    }

    pub fn namespace(&self) -> &str {
        self.inner().namespace.as_str()
    }
}

pub struct GtkConstraintSolver {
    _private: [u8; 0],
}

// XXX needs to be public in gtk
#[link(name = "gtk-4")]
extern "C" {
    pub fn gtk_constraint_solver_get_type() -> glib::ffi::GType;

    pub fn gdk_surface_set_widget(surface: *mut gdk::ffi::GdkSurface, widget: glib::ffi::gpointer);

    pub fn _gtk_widget_set_visible_flag(
        widget: *mut gtk4::ffi::GtkWidget,
        visible: glib::ffi::gboolean,
    );

    pub fn gtk_widget_render(
        widget: *mut gtk4::ffi::GtkWidget,
        surface: *mut gdk::ffi::GdkSurface,
        region: *const cairo::ffi::cairo_region_t,
    );

    pub fn gtk_main_do_event(event: *mut gdk::ffi::GdkEvent);

    // Added API
    pub fn gdk_wayland_popup_get_xdg_popup(
        popup: *mut gdk4_wayland::ffi::GdkWaylandPopup,
    ) -> *mut wl_proxy;
}

glib::wrapper! {
    pub struct ConstraintSolver(Object<GtkConstraintSolver>);

    match fn {
        type_ => || gtk_constraint_solver_get_type(),
    }
}

pub struct GtkNativeInterface {
    pub g_iface: gobject_sys::GTypeInterface,
    pub get_surface:
        Option<unsafe extern "C" fn(self_: *mut gtk4::ffi::GtkNative) -> *mut gdk::ffi::GdkSurface>,
    pub get_renderer: Option<
        unsafe extern "C" fn(self_: *mut gtk4::ffi::GtkNative) -> *mut gsk::ffi::GskRenderer,
    >,
    pub get_surface_transform:
        Option<unsafe extern "C" fn(self_: *mut gtk4::ffi::GtkNative, x: *mut f64, y: *mut f64)>,
    pub layout:
        Option<unsafe extern "C" fn(self_: *mut gtk4::ffi::GtkNative, width: c_int, height: c_int)>,
}

pub struct GtkRootInterface {
    pub g_iface: gobject_sys::GTypeInterface,
    pub get_display:
        Option<unsafe extern "C" fn(self_: *mut gtk4::ffi::GtkRoot) -> *mut gdk::ffi::GdkDisplay>,
    pub get_constraint_solver:
        Option<unsafe extern "C" fn(self_: *mut gtk4::ffi::GtkRoot) -> *mut GtkConstraintSolver>,
    pub get_focus:
        Option<unsafe extern "C" fn(self_: *mut gtk4::ffi::GtkRoot) -> *mut gtk4::ffi::GtkWidget>,
    pub set_focus: Option<
        unsafe extern "C" fn(self_: *mut gtk4::ffi::GtkRoot, focus: *mut gtk4::ffi::GtkWidget),
    >,
}

unsafe extern "C" fn get_surface(native: *mut gtk4::ffi::GtkNative) -> *mut gdk::ffi::GdkSurface {
    let instance = &*(native as *mut <LayerShellWindowInner as ObjectSubclass>::Instance);
    let imp = instance.impl_();
    imp.surface.borrow().as_ref().map_or(ptr::null_mut(), |x| {
        x.upcast_ref::<gdk::Surface>().to_glib_none().0
    })
}

unsafe extern "C" fn get_renderer(native: *mut gtk4::ffi::GtkNative) -> *mut gsk::ffi::GskRenderer {
    let instance = &*(native as *mut <LayerShellWindowInner as ObjectSubclass>::Instance);
    let imp = instance.impl_();
    imp.renderer
        .borrow()
        .as_ref()
        .map_or(ptr::null_mut(), |x| x.to_glib_none().0)
}

unsafe extern "C" fn get_surface_transform(
    _native: *mut gtk4::ffi::GtkNative,
    x: *mut f64,
    y: *mut f64,
) {
    // TODO: add css logic like `GtkWindow` has

    *x = 0.;
    *y = 0.;
}

unsafe extern "C" fn layout(native: *mut gtk4::ffi::GtkNative, width: c_int, height: c_int) {
    // TODO: `GtkWindow` has more here
    gtk4::ffi::gtk_widget_allocate(native as *mut _, width, height, -1, ptr::null_mut());
}

unsafe extern "C" fn get_display(root: *mut gtk4::ffi::GtkRoot) -> *mut gdk::ffi::GdkDisplay {
    let instance = &*(root as *mut <LayerShellWindowInner as ObjectSubclass>::Instance);
    let imp = instance.impl_();
    imp.display.upcast_ref::<gdk::Display>().to_glib_none().0
}

unsafe extern "C" fn get_constraint_solver(
    root: *mut gtk4::ffi::GtkRoot,
) -> *mut GtkConstraintSolver {
    let instance = &*(root as *mut <LayerShellWindowInner as ObjectSubclass>::Instance);
    let imp = instance.impl_();
    imp.constraint_solver.to_glib_none().0
}

unsafe extern "C" fn get_focus(root: *mut gtk4::ffi::GtkRoot) -> *mut gtk4::ffi::GtkWidget {
    let instance = &*(root as *mut <LayerShellWindowInner as ObjectSubclass>::Instance);
    let imp = instance.impl_();
    imp.focus_widget
        .borrow()
        .as_ref()
        .map_or(ptr::null_mut(), |x| x.to_glib_none().0)
}

unsafe extern "C" fn set_focus(root: *mut gtk4::ffi::GtkRoot, focus: *mut gtk4::ffi::GtkWidget) {
    // TODO: `GtkWindow` does more here
    let instance = &*(root as *mut <LayerShellWindowInner as ObjectSubclass>::Instance);
    let imp = instance.impl_();
    *imp.focus_widget.borrow_mut() = if focus.is_null() {
        None
    } else {
        Some(gtk4::Widget::from_glib_none(focus))
    };
}
