use cascade::cascade;
use gio::DesktopAppInfo;
use gio::Icon;
use gio::Menu;
use gio::MenuItem;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use gtk4::Align;
use gtk4::Box;
use gtk4::Image;
use gtk4::Label;
use gtk4::Orientation;
use gtk4::PopoverMenu;

use crate::dock_object::DockObject;
use crate::utils::BoxedWindowList;

mod imp;

glib::wrapper! {
    pub struct DockItem(ObjectSubclass<imp::DockItem>)
        @extends gtk4::Button, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Actionable, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl Default for DockItem {
    fn default() -> Self {
        Self::new()
    }
}

impl DockItem {
    pub fn new() -> Self {
        let self_: DockItem = glib::Object::new(&[]).expect("Failed to create DockItem");

        let item_box = Box::new(Orientation::Vertical, 0);
        self_.set_child(Some(&item_box));

        let image = cascade! {
            Image::new();
            ..set_hexpand(true);
            ..set_halign(Align::Center);
            ..set_pixel_size(64);
        };
        let dots = cascade! {
            Label::new(Some(""));
            ..set_hexpand(true);
            ..set_halign(Align::Center);
        };
        item_box.append(&image);
        item_box.append(&dots);
        let popover_menu = cascade! {
            PopoverMenu::from_model_full(&Menu::new(), gtk4::PopoverMenuFlags::NESTED);
            ..set_autohide(true);
        };
        item_box.append(&popover_menu);
        let self_clone = self_.clone();
        popover_menu.connect_closed(move |_| {
            self_clone
                .emit_by_name::<&str>("popover-closed", &[])
                .unwrap();
        });

        let imp = imp::DockItem::from_instance(&self_);
        imp.image.replace(image);
        imp.dots.replace(dots);
        imp.item_box.replace(item_box);
        imp.popover_menu.replace(Some(popover_menu));

        self_
    }

    // refactor to emit event for removing the item?
    pub fn set_app_info(&self, dock_object: &DockObject) {
        let self_ = imp::DockItem::from_instance(self);
        if let Ok(app_info_value) = dock_object.property("appinfo") {
            if let Ok(Some(app_info)) = app_info_value.get::<Option<DesktopAppInfo>>() {
                self_
                    .image
                    .borrow()
                    .set_tooltip_text(Some(&app_info.name()));

                let icon = app_info.icon().unwrap_or(
                    Icon::for_string("image-missing").expect("Failed to set default icon"),
                );

                self_.image.borrow().set_from_gicon(&icon);
            }
        } else {
            println!("initializing dock item failed...");
        }
        if let Ok(active_value) = dock_object.property("active") {
            if let Ok(active) = active_value.get::<BoxedWindowList>() {
                let dots = self_.dots.borrow();
                dots.set_text("");
                for _ in active.0 {
                    dots.set_text(format!("{}{}", dots.text(), " · ").as_str());
                }
            }
        }
        if let Ok(popover_value) = dock_object.property("popover") {
            if let Ok(popover) = popover_value.get::<bool>() {
                dbg!(popover);
                dbg!(dock_object);
                if popover {
                    self.add_popover(dock_object);
                } else {
                    self.clear_popover();
                }
            }
        }
    }

    pub fn add_popover(&self, item: &DockObject) {
        let imp = imp::DockItem::from_instance(self);
        if let Some(popover_menu) = imp.popover_menu.borrow().as_ref() {
            let menu = if let Some(m) = popover_menu.menu_model() {
                m.downcast::<Menu>().unwrap()
            } else {
                Menu::new()
            };
            menu.remove_all();

            let appinfo = item
                .property("appinfo")
                .expect("property appinfo missing from DockObject")
                .get::<Option<DesktopAppInfo>>();
            let window_list = item
                .property("active")
                .expect("property appinfo missing from DockObject")
                .get::<BoxedWindowList>();

            if let Ok(Some(_appinfo)) = appinfo {
                let launch_menu = Menu::new();
                let launch_subsection = MenuItem::new_section(None, &launch_menu);
                launch_menu.append(Some("New Window"), None);
                menu.append_item(&launch_subsection);
            }

            let favorites_menu = Menu::new();
            let favorites_subsection = MenuItem::new_section(None, &favorites_menu);
            match item.property("saved").unwrap().get::<bool>().unwrap() {
                true => favorites_menu.append(Some("Remove from Favorites"), None),
                false => favorites_menu.append(Some("Add to Favorites"), None),
            };
            menu.append_item(&favorites_subsection);

            if let Ok(window_list) = window_list {
                let window_list = window_list.0;
                if window_list.len() > 0 {
                    let all_windows_submenu_menu = Menu::new();
                    for w in window_list {
                        all_windows_submenu_menu.append(Some(w.name.as_str()), None);
                    }

                    let all_windows_menu = Menu::new();
                    let all_windows_submenu =
                        MenuItem::new_submenu(Some("All Windows"), &all_windows_submenu_menu);
                    all_windows_menu.append_item(&all_windows_submenu);
                    let all_windows_subsection = MenuItem::new_section(None, &all_windows_menu);

                    let quit_windows_menu = Menu::new();
                    quit_windows_menu.append(Some("Quit All"), None);
                    let quit_windows_subsection = MenuItem::new_section(None, &quit_windows_menu);

                    menu.prepend_item(&all_windows_subsection);
                    menu.append_item(&quit_windows_subsection);
                }
            }
            popover_menu.popup();
        }
    }

    pub fn clear_popover(&self) {
        let imp = imp::DockItem::from_instance(self);
        let popover_menu = imp.popover_menu.borrow();

        if let Some(popover) = popover_menu.as_ref() {
            popover.popdown();
        }
    }
}
