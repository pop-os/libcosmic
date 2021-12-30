use gtk4::glib;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use gtk4::CompositeTemplate;

#[derive(Debug, Default, CompositeTemplate)]
#[template(file = "dock_item.ui")]
pub struct DockItem {
    #[template_child]
    pub image: TemplateChild<gtk4::Image>,
    #[template_child]
    pub dots: TemplateChild<gtk4::Label>,
}

#[glib::object_subclass]
impl ObjectSubclass for DockItem {
    const NAME: &'static str = "DockItem";
    type Type = super::DockItem;
    type ParentType = gtk4::Box;

    fn class_init(klass: &mut Self::Class) {
        Self::bind_template(klass);
    }

    fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for DockItem {}

impl WidgetImpl for DockItem {}

impl BoxImpl for DockItem {}
