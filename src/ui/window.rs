use crate::database::Database;
use crate::database::ImageWrapper;

use super::processes::insert_new_folders;
use super::processes::scan_folders;
use super::processes::ScanFolderStatus;
use gtk;
use gtk::glib;
use gtk::prelude::*;
use gtk::Application;
use std::rc::Rc;
use std::thread;

pub struct MainWindow {
    left_image: gtk::Image,
    right_image: gtk::Image,
    remove_left_btn: gtk::Button,
    remove_right_btn: gtk::Button,
    not_duplicates_btn: gtk::Button,
    add_folder_btn: gtk::Button,
    scan_btn: gtk::Button,
    new_folder_chooser: gtk::FileChooserDialog,
    status_label: gtk::Label,

    last_search_index: i64,
}

impl MainWindow {
    fn new() -> Self {
        let left_image = gtk::Image::new();
        let right_image = gtk::Image::new();

        left_image.set_file(Some("test_image.jpg"));
        left_image.set_size_request(800, 600);
        right_image.set_file(Some("test_image.jpg"));
        right_image.set_size_request(800, 600);

        let remove_left_btn = gtk::Button::new();
        remove_left_btn.set_label("Remove left");

        let not_duplicates_btn = gtk::Button::new();
        not_duplicates_btn.set_label("Not duplicates");

        let remove_right_btn = gtk::Button::new();
        remove_right_btn.set_label("Remove right");

        let add_folder_btn = gtk::Button::builder().label("Add folder").build();

        let new_folder_chooser = gtk::FileChooserDialog::builder()
            .title("Choose folder")
            .select_multiple(true)
            .deletable(false)
            .action(gtk::FileChooserAction::SelectFolder)
            .create_folders(false)
            .build();

        let scan_btn = gtk::Button::builder().label("Scan").build();
        let status_label = gtk::Label::builder().label("").build();

        new_folder_chooser.add_button("Add", gtk::ResponseType::Accept);
        new_folder_chooser.add_button("Cancel", gtk::ResponseType::Cancel);

        {
            let new_folder_chooser = new_folder_chooser.clone();
            add_folder_btn.connect_clicked(move |_| {
                new_folder_chooser.show();
            });
        }

        let result = Self {
            last_search_index: 0,
            left_image,
            right_image,
            remove_left_btn,
            remove_right_btn,
            not_duplicates_btn,
            add_folder_btn,
            new_folder_chooser,
            scan_btn,
            status_label,
        };

        result.attach_handlers();

        result
    }

    fn attach_handlers(&self) {
        self.handle_add_folders_btn();
        self.handle_scan_btn();
    }

    fn handle_scan_btn(&self) {
        let blockable_widgets = Rc::new(self.get_blockable_widgets());
        let status_label = self.status_label.clone();

        self.scan_btn.connect_clicked(move |_| {
            status_label.set_label("Start scanning");
            blockable_widgets
                .iter()
                .for_each(|x| x.set_sensitive(false));

            let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

            thread::spawn(move || {
                scan_folders(sender);
            });

            let blockable_widgets_clone = blockable_widgets.clone();

            let status_label_clone = status_label.clone();
            receiver.attach(None, move |message| match message {
                ScanFolderStatus::Done => {
                    blockable_widgets_clone
                        .iter()
                        .for_each(|x| x.set_sensitive(true));
                    status_label_clone.set_label("Scan complete");
                    Continue(false)
                }
                ScanFolderStatus::ImageFound(image) => {
                    status_label_clone.set_label(format!("{}", image).as_str());
                    Continue(true)
                }
                ScanFolderStatus::HashCalculated(image) => {
                    status_label_clone.set_label(format!("Hash calculated: {}", image).as_str());
                    Continue(true)
                }
                _ => Continue(true),
            });
        });
    }

    fn handle_add_folders_btn(&self) {
        let blockable_widgets = Rc::new(self.get_blockable_widgets());
        self.new_folder_chooser
            .connect_response(move |dialog, response| {
                if response == gtk::ResponseType::Accept {
                    blockable_widgets
                        .iter()
                        .for_each(|x| x.set_sensitive(false));

                    let mut paths: Vec<String> = Vec::new();
                    for item in dialog.files().iter() {
                        let item_path: gtk::gio::File = item.unwrap();
                        let str = item_path.path().unwrap();
                        paths.push(str.to_str().unwrap().to_string());
                    }

                    let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

                    thread::spawn(move || {
                        insert_new_folders(paths);
                        sender
                            .send(true)
                            .expect("Can not send signal to main thread");
                    });

                    let blockable_widgets_clone = blockable_widgets.clone();
                    receiver.attach(None, move |x| {
                        blockable_widgets_clone
                            .iter()
                            .for_each(|x| x.set_sensitive(true));
                        Continue(!x)
                    });

                    dialog.hide();
                }

                if response == gtk::ResponseType::Cancel {
                    dialog.hide();
                }
            });
    }

    fn get_blockable_widgets(&self) -> Vec<impl WidgetExt> {
        return vec![
            self.remove_left_btn.clone(),
            self.remove_right_btn.clone(),
            self.not_duplicates_btn.clone(),
            self.add_folder_btn.clone(),
            self.scan_btn.clone(),
        ];
    }
}

pub fn build_ui(app: &Application) {
    let main_window = MainWindow::new();

    let window = gtk::ApplicationWindow::new(app);
    window.set_title(Some("Deduplicator"));
    let main_grid = gtk::Box::new(gtk::Orientation::Vertical, 0);

    let top_control_grid = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .build();

    top_control_grid.append(&main_window.add_folder_btn);
    top_control_grid.append(&main_window.scan_btn);
    top_control_grid.append(&main_window.status_label);
    main_grid.append(&top_control_grid);

    let image_grid = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    image_grid.set_homogeneous(true);
    image_grid.set_halign(gtk::Align::Fill);
    image_grid.set_vexpand(true);
    image_grid.set_spacing(32);

    image_grid.append(&main_window.left_image);
    image_grid.append(&main_window.right_image);

    main_grid.append(&image_grid);

    let buttons_grid = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    buttons_grid.set_homogeneous(true);
    buttons_grid.set_halign(gtk::Align::Center);
    buttons_grid.append(&main_window.remove_left_btn);
    buttons_grid.append(&main_window.not_duplicates_btn);
    buttons_grid.append(&main_window.remove_right_btn);
    main_grid.append(&buttons_grid);

    main_window
        .new_folder_chooser
        .set_transient_for(Some(&window));
    window.set_child(Some(&main_grid));

    window.show();
}

fn find_next_duplicates(last_index: Rc<i64>) -> Option<(ImageWrapper, ImageWrapper)> {
    let database = Database::connect_default();

    None
}
