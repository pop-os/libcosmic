use gio::DesktopAppInfo;
use glib::prelude::*;
use glib::subclass;
use glib::subclass::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, glib::GBoxed)]
#[gboxed(type_name = "MyBoxed")]
struct MyBoxed(DesktopAppInfo);

pub fn main() {
    let appinfo = DesktopAppInfo::new("firefox.desktop").expect("failed to get app info");
    assert!(MyBoxed::static_type().is_valid());

    let b = MyBoxed(appinfo);
    let v = b.to_value();
    let b2 = v.get::<&MyBoxed>().unwrap();
    assert_eq!(&b, b2);
    dbg!(&b2.0.filename());
}
