use relm4::{
    gtk::{prelude::*, Align, Box as GtkBox, Label, Orientation, Widget},
    ComponentParts, ComponentSender, SimpleComponent, component::ComponentSenderInner,
};
use std::{cell::RefCell, sync::Arc};

#[derive(Debug)]
pub(crate) enum LabeledItemMessage {
    Title(String),
    Desc(Option<String>),
    Align(Align),
    Child(Widget),
}

#[track]
pub(crate) struct LabeledItem {
    _title: String,
    _desc: Option<String>,
    _align: Align,
    _child: Option<Widget>,
    #[do_not_track]
    _remove_child: RefCell<Option<Widget>>,
    #[do_not_track]
    _sender: ComponentSender<Self>,
}

impl LabeledItem {
    pub fn title(&self) -> &str {
        &self._title
    }

    pub fn description(&self) -> Option<&String> {
        self._desc.as_ref()
    }

    pub fn alignment(&self) -> Align {
        self._align
    }

    pub fn child(&self) -> Option<&Widget> {
        self._child.as_ref()
    }

    pub fn set_title<S>(&self, title: S)
    where
        S: ToString,
    {
        self._sender
            .input(LabeledItemMessage::Title(title.to_string()));
    }

    pub fn set_description<'a, O>(&self, description: O)
    where
        O: Into<Option<&'a str>>,
    {
        let description = description.into();
        self._sender
            .input(LabeledItemMessage::Desc(description.map(|s| s.to_string())));
    }

    pub fn set_alignment(&self, align: Align) {
        self._sender.input(LabeledItemMessage::Align(align));
    }

    pub fn set_child(&self, child: Widget) {
        self._sender.input(LabeledItemMessage::Child(child));
    }
}

#[component(pub(crate))]
impl SimpleComponent for LabeledItem {
    type Widgets = AppWidgets;
    type InitParams = ();
    type Input = LabeledItemMessage;
    type Output = ();

    view! {
        base_box = GtkBox {
            add_css_class: "labeled-item",
            set_orientation: Orientation::Horizontal,
            set_hexpand: true,
            set_margin_start: 24,
            set_margin_end: 24,
            set_margin_top: 8,
            set_margin_bottom: 8,
            set_spacing: 16,
            append: labeled_item_info = &GtkBox {
                add_css_class: "labeled-item-info",
                set_orientation: Orientation::Vertical,
                set_hexpand: true,
                set_spacing: 8,
                set_valign: Align::Center,
                Label {
                    add_css_class: "labeled-item-title",
                    set_halign: Align::Start,
                    #[watch]
                    set_label: &model._title
                },
                Label {
                    add_css_class: "labeled-item-desc",
                    set_halign: Align::Start,
                    #[watch]
                    set_visible: model._desc.is_some(),
                    #[watch]
                    set_label: &model._desc.clone().unwrap_or_default()
                },
            }
        }
    }

    fn init(
        _init_params: Self::InitParams,
        root: &Self::Root,
        _sender: Arc<ComponentSenderInner<LabeledItemMessage, (), ()>>,
    ) -> ComponentParts<Self> {
        let model = LabeledItem {
            _title: String::default(),
            _desc: None,
            _align: Align::Start,
            _child: None,
            _remove_child: RefCell::new(None),
            _sender: _sender.clone(),
            tracker: 0,
        };
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(
        &mut self,
        msg: Self::Input,
        _sender: Arc<ComponentSenderInner<LabeledItemMessage, (), ()>>,
    ) {
        self.reset();
        match msg {
            LabeledItemMessage::Title(title) => self.set__title(title),
            LabeledItemMessage::Desc(desc) => self.set__desc(desc),
            LabeledItemMessage::Align(align) => self.set__align(align),
            LabeledItemMessage::Child(child) => {
                *self._remove_child.borrow_mut() = self._child.take();
                self.set__child(Some(child))
            }
        }
    }

    fn post_view() {
        if let Some(child) = self._remove_child.borrow_mut().take() {
            widgets.base_box.remove(&child);
        }
        if self.changed(LabeledItem::_child()) {
            let child = self._child.as_ref().expect("there's no child??");
            widgets.base_box.append(child);
        }
        if self.changed(LabeledItem::_align()) {
            let child = self._child.as_ref().expect("set alignment without child");
            match self._align {
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
