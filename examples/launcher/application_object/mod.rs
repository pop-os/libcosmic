mod imp;

use gdk4::glib::Object;
use glib::ObjectExt;
use glib::ToVariant;
use gtk4::glib;

glib::wrapper! {
    pub struct ApplicationObject(ObjectSubclass<imp::ApplicationObject>);
}

impl ApplicationObject {
    pub fn new(application_search_result: &pop_launcher::SearchResult) -> Self {
        let self_: Self = Object::new(&[
            ("id", &application_search_result.id),
            ("name", &application_search_result.name),
            ("description", &application_search_result.description),
        ])
        .expect("Failed to create `ApplicationObject`.");
        if let Some(icon) = &application_search_result.icon {
            if let Err(e) = self_.set_property(
                "icon",
                match icon {
                    pop_launcher::IconSource::Name(name) => {
                        (pop_launcher::IconSource::Name as i32, name.to_string()).to_variant()
                    }
                    pop_launcher::IconSource::Mime(name) => {
                        (pop_launcher::IconSource::Mime as i32, name.to_string()).to_variant()
                    }
                },
            ) {
                println!("failed to set icon property");
                dbg!(e);
            };
        }
        if let Some(icon) = &application_search_result.category_icon {
            if let Err(e) = self_.set_property(
                "categoryicon",
                match icon {
                    pop_launcher::IconSource::Name(name) => {
                        (pop_launcher::IconSource::Name as i32, name.to_string()).to_variant()
                    }
                    pop_launcher::IconSource::Mime(name) => {
                        (pop_launcher::IconSource::Mime as i32, name.to_string()).to_variant()
                    }
                },
            ) {
                println!("failed to set category icon property");
                dbg!(e);
            };
        }

        self_
    }
}

// Object holding the state
pub struct ApplicationData(pop_launcher::SearchResult);

impl Default for ApplicationData {
    fn default() -> Self {
        let default_application = pop_launcher::SearchResult {
            id: 0,
            name: String::default(),
            description: String::default(),
            icon: None,
            category_icon: None,
            window: None,
        };
        Self(default_application)
    }
}
