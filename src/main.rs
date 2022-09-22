use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use chrono::{DateTime, Utc};
use gtk4::{
    glib::{self, clone},
    prelude::{ApplicationExt, ApplicationExtManual, Cast, FileExt},
    traits::{
        BoxExt, ButtonExt, DialogExt, EditableExt, EntryExt, FileChooserExt,
        GridExt, GtkWindowExt, NativeDialogExt, WidgetExt,
    },
    Align, Application, ApplicationWindow, Button, Entry, FileChooserAction,
    FileChooserNative, FileFilter, Grid, Label, MessageDialog, MessageType,
    Orientation, ResponseType, ScrolledWindow, Widget,
};
use sbvc_lib::{Sbvc, Version};

fn main() {
    let application =
        Application::builder().application_id("com.wgsoft.app.sbvc").build();
    application.connect_activate(build_ui);
    application.run();
}

fn build_ui(application: &Application) {
    let sbvc: Rc<RefCell<Option<Sbvc>>> = Rc::new(RefCell::new(None));
    let selected = Rc::new(Cell::new(0u32));

    let window = ApplicationWindow::builder()
        .application(application)
        .title("SBVC")
        .default_width(960)
        .default_height(540)
        .build();

    let main_box =
        gtk4::Box::builder().orientation(Orientation::Vertical).build();

    let top_box = gtk4::Box::builder()
        .orientation(Orientation::Horizontal)
        .hexpand(true)
        .build();

    let path_label = Label::builder()
        .label("No file selected")
        .hexpand(true)
        .xalign(0.0)
        .build();
    path_label.set_margin_default();
    top_box.append(&path_label);

    let filter = FileFilter::new();
    filter.add_pattern("*.sbvc");

    let path_chooser = FileChooserNative::builder()
        .title("Select SBVC file to open")
        .transient_for(&window)
        .modal(true)
        .action(FileChooserAction::Open)
        .select_multiple(false)
        .accept_label("Open")
        .cancel_label("Cancel")
        .filter(&filter)
        .build();

    let select_path_button = Button::builder().label("Open").build();
    select_path_button.set_margin_default();
    select_path_button.connect_clicked(
        clone!(@strong path_chooser => move |_| {
            path_chooser.show();
        }),
    );
    top_box.append(&select_path_button);

    let file_chooser = FileChooserNative::builder()
        .title("Select file to be tracked")
        .transient_for(&window)
        .modal(true)
        .action(FileChooserAction::Open)
        .select_multiple(false)
        .accept_label("Open")
        .cancel_label("Cancel")
        .build();

    let select_file_button = Button::builder().label("New").build();
    select_file_button.set_margin_default();
    select_file_button.connect_clicked(
        clone!(@strong file_chooser => move |_| {
            file_chooser.show();
        }),
    );
    top_box.append(&select_file_button);

    main_box.append(&top_box);

    let file_box = gtk4::Box::builder()
        .orientation(Orientation::Horizontal)
        .hexpand(true)
        .build();

    let file_label = Label::builder()
        .label("No file selected")
        .hexpand(true)
        .xalign(0.0)
        .build();
    file_label.set_margin_default();
    file_box.append(&file_label);

    let rename_chooser = FileChooserNative::builder()
        .title("Select file to be tracked")
        .transient_for(&window)
        .modal(true)
        .action(FileChooserAction::Open)
        .select_multiple(false)
        .accept_label("Open")
        .cancel_label("Cancel")
        .build();

    let select_rename_button =
        Button::builder().label("Change tracked file").build();
    select_rename_button.set_margin_default();
    select_rename_button.connect_clicked(
        clone!(@strong sbvc, @strong rename_chooser => move |_| {
            if sbvc.borrow().is_some() {
                rename_chooser.show();
            }
        }),
    );
    file_box.append(&select_rename_button);

    main_box.append(&file_box);

    let bottom_box = gtk4::Box::builder()
        .orientation(Orientation::Horizontal)
        .vexpand(true)
        .hexpand(true)
        .build();

    let scrolled_window =
        ScrolledWindow::builder().vexpand(true).hexpand(true).build();
    scrolled_window.set_margin_default();

    let scrolled_grid = Grid::builder().build();
    scrolled_window.set_child(Some(&scrolled_grid));

    bottom_box.append(&scrolled_window);

    let side_box = gtk4::Box::builder()
        .orientation(Orientation::Vertical)
        .width_request(300)
        .build();
    side_box.set_margin_default();

    let side_label =
        Label::builder().label("Version info").halign(Align::Start).build();
    side_label.set_margin_default();
    side_box.append(&side_label);

    let id_label =
        Label::builder().label("Version ID:").halign(Align::Start).build();
    id_label.set_margin_default();
    side_box.append(&id_label);

    let base_label =
        Label::builder().label("Base version:").halign(Align::Start).build();
    base_label.set_margin_default();
    side_box.append(&base_label);

    let name_label =
        Label::builder().label("Version name:").halign(Align::Start).build();
    name_label.set_margin_default();
    side_box.append(&name_label);

    let date_label =
        Label::builder().label("Commit date:").halign(Align::Start).build();
    date_label.set_margin_default();
    side_box.append(&date_label);

    let deletions_label =
        Label::builder().label("Deletion count:").halign(Align::Start).build();
    deletions_label.set_margin_default();
    side_box.append(&deletions_label);

    let insertions_label =
        Label::builder().label("Insertion count:").halign(Align::Start).build();
    insertions_label.set_margin_default();
    side_box.append(&insertions_label);

    let rollback_dialog = MessageDialog::builder()
        .title("Rollback?")
        .text(
            "You have uncommited changes in your file. \
            Do you wish to discard them?",
        )
        .transient_for(&window)
        .modal(true)
        .message_type(MessageType::Warning)
        .buttons(gtk4::ButtonsType::YesNo)
        .build();
    rollback_dialog.connect_response(clone!(
        @weak sbvc,
        @strong selected,
        @weak scrolled_grid,
    => move |delete_dialog, response| {
        sbvc.borrow_mut()
            .as_mut()
            .unwrap()
            .checkout(
                selected.get(),
                response == ResponseType::Yes,
            )
            .unwrap();

        delete_dialog.hide();
    }));

    let callback = clone!(
        @weak sbvc,
        @weak id_label,
        @weak base_label,
        @weak name_label,
        @weak date_label,
        @weak deletions_label,
        @weak insertions_label,
    => move || {
        let borrow = sbvc.borrow();
        let version = borrow.as_ref().unwrap().current();
        id_label.set_text(&format!("Version ID: {}", version.id()));
        base_label.set_text(&format!("Version base: {}", version.base()));
        name_label.set_text(&format!("Version name: {}", version.name()));
        date_label.set_text(&format!(
            "Commit date: {}",
            DateTime::<Utc>::from(version.date()).to_rfc2822(),
        ));
        deletions_label.set_text(&format!(
            "Deletion count: {}",
            version.difference().deletions.len()
        ));
        insertions_label.set_text(&format!(
            "Insertion count: {}",
            version.difference().insertions.len(),
        ));
    });

    path_chooser.connect_response(clone!(
        @weak sbvc,
        @weak selected,
        @strong rollback_dialog,
        @strong callback,
        @weak path_label,
        @weak file_label,
        @weak scrolled_grid
    => move |path_chooser, response| {
        if let ResponseType::Accept = response {
            let path = path_chooser.file().unwrap().path().unwrap();

            path_label.set_label(
                &format!("Selected SBVC file: {}", path.to_string_lossy()),
            );

            *sbvc.borrow_mut() = Sbvc::open(path).ok();

            file_label.set_label(&format!(
                "Tracked file: {}",
                sbvc.borrow().as_ref().unwrap().file().to_string_lossy(),
            ));

            for child in scrolled_grid.observe_children() {
                scrolled_grid.remove(
                    &child.dynamic_cast::<Widget>().unwrap(),
                );
            }

            build_tree(
                &scrolled_grid,
                0,
                &mut 0,
                sbvc.clone(),
                selected,
                &rollback_dialog,
                callback.clone(),
                sbvc.borrow()
                    .as_ref()
                    .unwrap()
                    .versions()
                    .iter()
                    .find(|&version| version.id() == version.base())
                    .unwrap(),
            );

            callback();
        }
    }));

    file_chooser.connect_response(clone!(
        @weak sbvc,
        @weak selected,
        @weak rollback_dialog,
        @strong callback,
        @weak path_label,
        @weak file_label,
        @weak scrolled_grid,
    => move |file_chooser, response| {
        if let ResponseType::Accept = response {
            let file = file_chooser.file().unwrap().path().unwrap();

            path_label.set_label(&format!(
                "Selected SBVC file: {}",
                file.with_extension("sbvc").to_string_lossy(),
            ));
            file_label.set_label(
                &format!("Tracked file: {}", file.to_string_lossy()),
            );

            *sbvc.borrow_mut() =
                Sbvc::new(file.with_extension("sbvc"), file).ok();

            for child in scrolled_grid.observe_children() {
                scrolled_grid.remove(
                    &child.dynamic_cast::<Widget>().unwrap(),
                );
            }

            build_tree(
                &scrolled_grid,
                0,
                &mut 0,
                sbvc.clone(),
                selected,
                &rollback_dialog,
                callback.clone(),
                sbvc.borrow()
                    .as_ref()
                    .unwrap()
                    .versions()
                    .iter()
                    .find(|&version| version.id() == version.base())
                    .unwrap(),
            );

            callback();
        }
    }));

    rename_chooser.connect_response(clone!(
        @weak sbvc,
        @weak path_label,
        @weak file_label,
        @weak scrolled_grid
    => move |rename_chooser, response| {
        if let ResponseType::Accept = response {
            let file = rename_chooser.file().unwrap().path().unwrap();

            file_label.set_label(
                &format!("Tracked file: {}", file.to_string_lossy()),
            );

            sbvc.borrow_mut().as_mut().unwrap().set_file(file).unwrap();
        }
    }));

    let commit_button =
        Button::builder().label("Commit").halign(Align::Fill).build();
    commit_button.set_margin_default();
    commit_button.connect_clicked(clone!(
        @weak sbvc,
        @weak selected,
        @weak rollback_dialog,
        @strong callback,
        @weak scrolled_grid,
    => move |_| {
        if sbvc.borrow().is_some() {
            let mut borrow = sbvc.borrow_mut();

            if let Some(sbvc) = borrow.as_mut() {
                sbvc.commit().unwrap();
            }

            for child in scrolled_grid.observe_children() {
                scrolled_grid.remove(
                    &child.dynamic_cast::<Widget>().unwrap(),
                );
            }

            drop(borrow);

            build_tree(
                &scrolled_grid,
                0,
                &mut 0,
                sbvc.clone(),
                selected,
                &rollback_dialog,
                callback.clone(),
                sbvc.borrow()
                    .as_ref()
                    .unwrap()
                    .versions()
                    .iter()
                    .find(|&version| version.id() == version.base())
                    .unwrap(),
            );

            callback();
        }
    }));
    side_box.append(&commit_button);

    let rename_dialog = MessageDialog::builder()
        .title("Rename version")
        .text("Choose a new name for the version")
        .transient_for(&window)
        .modal(true)
        .message_type(MessageType::Question)
        .buttons(gtk4::ButtonsType::OkCancel)
        .build();

    let rename_entry = Entry::builder().placeholder_text("New name").build();
    rename_entry.set_margin_default();
    rename_entry.connect_activate(clone!(@strong rename_dialog => move |_| {
        rename_dialog.response(ResponseType::Ok);
    }));
    rename_dialog.content_area().append(&rename_entry);

    rename_dialog.connect_response(clone!(
        @weak sbvc,
        @weak selected,
        @weak rollback_dialog,
        @strong callback,
        @weak scrolled_grid,
    => move |rename_dialog, response| {
        if response == ResponseType::Ok && sbvc.borrow().is_some() {
            if let Some(sbvc) = sbvc.borrow_mut().as_mut() {
                sbvc.rename(&rename_entry.text()).unwrap();
            }

            for child in scrolled_grid.observe_children() {
                scrolled_grid.remove(
                    &child.dynamic_cast::<Widget>().unwrap(),
                );
            }

            build_tree(
                &scrolled_grid,
                0,
                &mut 0,
                sbvc.clone(),
                selected,
                &rollback_dialog,
                callback.clone(),
                sbvc.borrow()
                    .as_ref()
                    .unwrap()
                    .versions()
                    .iter()
                    .find(|&version| version.id() == version.base())
                    .unwrap(),
            );

            callback();
        }

        rename_entry.set_text("");
        rename_dialog.hide();
    }));

    let rename_button =
        Button::builder().label("Rename").halign(Align::Fill).build();
    rename_button.set_margin_default();
    rename_button.connect_clicked(clone!(@weak rename_dialog => move |_| {
        rename_dialog.show();
    }));
    side_box.append(&rename_button);

    let delete_dialog = MessageDialog::builder()
        .title("Delete version")
        .text(
            "Are you sure you want to delete the selected version? Your file \
            content will be set to the one of the base of the deleted version",
        )
        .transient_for(&window)
        .modal(true)
        .message_type(MessageType::Warning)
        .buttons(gtk4::ButtonsType::YesNo)
        .build();
    delete_dialog.connect_response(clone!(
        @weak sbvc,
        @weak selected,
        @weak rollback_dialog,
        @strong callback,
        @weak scrolled_grid,
    => move |delete_dialog, response| {
        if response == ResponseType::Yes && sbvc.borrow().is_some() {
            if let Some(sbvc) = sbvc.borrow_mut().as_mut() {
                sbvc.delete().unwrap();
            }

            for child in scrolled_grid.observe_children() {
                scrolled_grid.remove(
                    &child.dynamic_cast::<Widget>().unwrap(),
                );
            }

            build_tree(
                &scrolled_grid,
                0,
                &mut 0,
                sbvc.clone(),
                selected,
                &rollback_dialog,
                callback.clone(),
                sbvc.borrow()
                    .as_ref()
                    .unwrap()
                    .versions()
                    .iter()
                    .find(|&version| version.id() == version.base())
                    .unwrap(),
            );

            callback();
        }

        delete_dialog.hide();
    }));

    let delete_button =
        Button::builder().label("Delete").halign(Align::Fill).build();
    delete_button.set_margin_default();
    delete_button.connect_clicked(move |_| {
        delete_dialog.show();
    });
    side_box.append(&delete_button);

    bottom_box.append(&side_box);

    main_box.append(&bottom_box);

    window.set_child(Some(&main_box));

    window.present();
}

#[allow(clippy::too_many_arguments)]
fn build_tree<F: 'static + Fn() + Clone>(
    grid: &Grid,
    grid_width: i32,
    grid_height: &mut i32,
    sbvc: Rc<RefCell<Option<Sbvc>>>,
    selected: Rc<Cell<u32>>,
    rollback_dialog: &MessageDialog,
    callback: F,
    version: &Version,
) {
    let id = version.id();

    let button =
        Button::builder().label(&format!("{}: {}", id, version.name())).build();
    button.connect_clicked(clone!(
        @weak sbvc,
        @weak selected,
        @weak rollback_dialog,
        @strong callback
    => move |_| {
        if sbvc.borrow()
            .as_ref()
            .unwrap()
            .is_changed()
            .unwrap()
        {
            selected.set(id);
            rollback_dialog.show();
        } else {
            sbvc.borrow_mut().as_mut().unwrap().checkout(id, true).unwrap();
            callback();
        }
    }));

    grid.attach(&button, grid_width, *grid_height, 1, 1);

    let mut found = false;

    for child in sbvc
        .borrow()
        .as_ref()
        .unwrap()
        .versions()
        .iter()
        .filter(|&child| child.base() == id && child.id() != id)
    {
        build_tree(
            grid,
            grid_width + 1,
            grid_height,
            sbvc.clone(),
            selected.clone(),
            rollback_dialog,
            callback.clone(),
            child,
        );
        found = true;
    }

    if !found {
        *grid_height += 1;
    }
}

trait SetMargin {
    fn set_margin(&self, margin: i32);

    fn set_margin_default(&self) {
        self.set_margin(12);
    }
}

impl<T: WidgetExt> SetMargin for T {
    fn set_margin(&self, margin: i32) {
        self.set_margin_top(margin);
        self.set_margin_bottom(margin);
        self.set_margin_start(margin);
        self.set_margin_end(margin);
    }
}
