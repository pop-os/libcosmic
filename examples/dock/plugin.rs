use gtk4::glib;

#[derive(Clone, Debug, Default, gtk4::glib::Boxed)]
#[boxed_type(name = "BoxedDockPlugin")]
pub struct BoxedDockPlugin {
    pub path: String,
    pub name: String,
    pub image: gtk4::Image,
    pub popover_menu: gtk4::Box,
}
