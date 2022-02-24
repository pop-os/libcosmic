use gtk4::{prelude::*, Align, Label, Orientation, Widget};
use relm4::{ComponentParts, Sender, SimpleComponent};

#[derive(Debug)]
pub enum LabeledItemMessage {
    Title(String),
    Desc(Option<String>),
    Align(Align),
    Child(Widget),
}

#[track]
struct LabeledItem {
    title: String,
    desc: Option<String>,
    align: Align,
    child: Option<Widget>,
}

#[component]
impl SimpleComponent for LabeledItem {
    type Widgets = AppWidgets;
    type InitParams = ();
    type Input = LabeledItemMessage;
    type Output = ();

    view! {
        base_box = gtk4::Box {
            add_css_class: "labeled-item",
            set_orientation: Orientation::Horizontal,
            set_hexpand: true,
            set_margin_start: 24,
            set_margin_end: 24,
            set_margin_top: 8,
            set_margin_bottom: 8,
            set_spacing: 16,
            append: labeled_item_info = &gtk4::Box {
                add_css_class: "labeled-item-info",
                set_orientation: Orientation::Vertical,
                set_hexpand: true,
                set_spacing: 8,
                set_valign: Align::Center,
                &Label {
                    add_css_class: "labeled-item-title",
                    set_halign: Align::Start,
                    set_label: watch! { &model.title }
                },
                &Label {
                    add_css_class: "labeled-item-desc",
                    set_halign: Align::Start,
                    set_visible: watch! { model.desc.is_some() },
                    set_label: watch! { &model.desc.clone().unwrap_or_default() }
                },
            }
        }
    }

    fn init_parts(
        _init_params: Self::InitParams,
        root: &Self::Root,
        _input: &Sender<Self::Input>,
        _output: &Sender<Self::Output>,
    ) -> ComponentParts<Self, Self::Widgets> {
        let model = LabeledItem {
            title: String::default(),
            desc: None,
            align: Align::Start,
            child: None,
            tracker: 0,
        };
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(
        &mut self,
        msg: Self::Input,
        _input: &Sender<Self::Input>,
        _ouput: &Sender<Self::Output>,
    ) {
        self.reset();
        match msg {
            LabeledItemMessage::Title(title) => self.set_title(title),
            LabeledItemMessage::Desc(desc) => self.set_desc(desc),
            LabeledItemMessage::Align(align) => self.set_align(align),
            LabeledItemMessage::Child(child) => self.set_child(Some(child)),
        }
    }

    fn post_view() {
        if self.changed(LabeledItem::child()) {
            let child = self.child.as_ref().expect("there's no child??");
            widgets.base_box.append(child);
        }
        if self.changed(LabeledItem::align()) {
            let child = self.child.as_ref().expect("set alignment without child");
            match self.align {
                Align::Start => {
                    widgets
                        .base_box
                        .reorder_child_after(&widgets.labeled_item_info, Some(child));
                }
                Align::End => {
                    widgets
                        .base_box
                        .reorder_child_after(child, Some(&widgets.labeled_item_info));
                }
                _ => unimplemented!(),
            }
        }
    }
}
