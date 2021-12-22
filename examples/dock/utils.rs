use crate::Item;

#[derive(Clone, Debug, Default, glib::GBoxed)]
#[gboxed(type_name = "BoxedWindowList")]
pub struct BoxedWindowList(pub Vec<Item>);
