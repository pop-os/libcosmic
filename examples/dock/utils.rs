use std::path::PathBuf;

use gtk4::glib;
use std::future::Future;

use crate::DockObject;
use crate::Item;

#[derive(Clone, Debug, Default, glib::GBoxed)]
#[gboxed(type_name = "BoxedWindowList")]
pub struct BoxedWindowList(pub Vec<Item>);

#[derive(Clone, Debug, Default, glib::GBoxed)]
#[gboxed(type_name = "BoxedDockObject")]
pub struct BoxedDockObject(pub Option<DockObject>);

pub fn data_path() -> PathBuf {
    let mut path = glib::user_data_dir();
    path.push("com.cosmic.dock");
    std::fs::create_dir_all(&path).expect("Could not create directory.");
    path.push("data.json");
    path
}

pub fn thread_context() -> glib::MainContext {
    glib::MainContext::thread_default().unwrap_or_else(|| {
        let ctx = glib::MainContext::new();
        ctx.push_thread_default();
        ctx
    })
}

pub fn block_on<F>(future: F) -> F::Output
where
    F: Future,
{
    thread_context().block_on(future)
}
