use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use gtk4 as gtk;

#[derive(Debug, Default, CompositeTemplate)]
#[template(file = "dock_item.ui")]
pub struct DockItem {
    #[template_child]
    pub image: TemplateChild<gtk::Image>,
    #[template_child]
    pub dots: TemplateChild<gtk::Label>,
}

#[glib::object_subclass]
impl ObjectSubclass for DockItem {
    const NAME: &'static str = "DockItem";
    type Type = super::DockItem;
    type ParentType = gtk::Box;

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
