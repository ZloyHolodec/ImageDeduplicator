use gtk;
use gtk::prelude::*;
use gtk::Application;

struct MainWindow {
    left_image: gtk::Image,
    right_image: gtk::Image,
    remove_left_btn: gtk::Button,
    remove_right_btn: gtk::Button,
    not_duplicates_btn: gtk::Button,
    add_folder_btn: gtk::Button,
    scan_btn: gtk::Button,
    new_folder_chooser: gtk::FileChooserDialog,
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

        new_folder_chooser.add_button("Add", gtk::ResponseType::Accept);
        new_folder_chooser.add_button("Cancel", gtk::ResponseType::Cancel);

        new_folder_chooser.connect_response(|dialog, response| match response {
            gtk::ResponseType::Accept => {
                for item in dialog.files().iter() {
                    let item_path: gtk::gio::File = item.unwrap();
                    println!("{}", item_path.path().unwrap().to_str().unwrap());
                }

                dialog.hide();
            }
            gtk::ResponseType::Cancel => dialog.hide(),
            _ => {}
        });

        {
            let new_folder_chooser = new_folder_chooser.clone();
            add_folder_btn.connect_clicked(move |_| {
                new_folder_chooser.show();
            });
        }

        Self {
            left_image,
            right_image,
            remove_left_btn,
            remove_right_btn,
            not_duplicates_btn,
            add_folder_btn,
            new_folder_chooser,
            scan_btn,
        }
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
