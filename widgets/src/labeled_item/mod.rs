mod imp;

use gtk4::{glib::IsA, prelude::*, Align, Orientation, Widget};
use relm4::{Component, ComponentController, ComponentParts, Controller};
use std::{cell::Ref, ops::Deref};

pub struct LabeledItem {
    root: gtk4::Box,
    controller: Controller<imp::LabeledItem>,
}

impl LabeledItem {
    fn inner(&self) -> Ref<'_, ComponentParts<imp::LabeledItem>> {
        self.controller.state().get()
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn widget(&self) -> gtk4::Box {
        self.root.clone()
    }

    pub fn title(&self) -> String {
        self.inner().model.title().to_owned()
    }

    pub fn description(&self) -> Option<String> {
        self.inner().model.description().cloned()
    }

    pub fn alignment(&self) -> Align {
        self.inner().model.alignment()
    }

    pub fn child(&self) -> Option<Widget> {
        self.inner().model.child().cloned()
    }

    pub fn set_title<S>(&self, title: S)
    where
        S: ToString,
    {
        self.inner().model.set_title(title)
    }

    pub fn set_description<S>(&self, description: Option<S>)
    where
        S: ToString,
    {
        self.inner().model.set_description(description)
    }

    pub fn set_alignment(&self, align: Align) {
        self.inner().model.set_alignment(align)
    }

    pub fn set_child(&self, child: &impl IsA<Widget>) {
        let widget = child.upcast_ref();
        self.inner().model.set_child(widget.clone());
    }
}

impl Default for LabeledItem {
    fn default() -> Self {
        let root = gtk4::Box::new(Orientation::Horizontal, 0);
        let controller = imp::LabeledItem::init()
            .attach_to(&root)
            .launch(())
            .detach();
        Self { root, controller }
    }
}

impl AsRef<Widget> for LabeledItem {
    fn as_ref(&self) -> &Widget {
        self.root.upcast_ref()
    }
}

impl Deref for LabeledItem {
    type Target = gtk4::Box;

    fn deref(&self) -> &Self::Target {
        &self.root
    }
}
