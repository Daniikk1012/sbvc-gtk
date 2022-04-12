use std::{cell::RefCell, rc::Rc, sync::Arc, time::Duration};

use async_std::{
    sync::RwLock,
    task::{self, JoinHandle},
};
use futures::{channel::mpsc, join};
use gtk4::{
    glib::{self, clone, MainContext},
    prelude::*,
    Align, Application, ApplicationWindow, Button, Entry, FileChooserAction,
    FileChooserNative, Grid, Inhibit, Label, MessageDialog, MessageType,
    Orientation, ResponseType, ScrolledWindow, Widget,
};
use sbvc_lib::{Database, Version};

fn main() {
    let application =
        Application::builder().application_id("com.wgsoft.app.sbvc").build();

    application.connect_activate(build_ui);

    application.run();
}

fn build_ui(application: &Application) {
    let database: Arc<RwLock<Option<Database>>> = Arc::new(RwLock::new(None));
    let version: Arc<RwLock<Option<Version>>> = Arc::new(RwLock::new(None));
    let handle: Rc<RefCell<Option<JoinHandle<()>>>> =
        Rc::new(RefCell::new(None));

    let window = ApplicationWindow::builder()
        .application(application)
        .title("SBVC")
        .default_width(960)
        .default_height(540)
        .build();

    window.connect_close_request(
        clone!(@strong database, @strong handle => move |_| {
            if let Some(handle) = handle.borrow_mut().as_mut() {
                task::block_on(handle);
            }

            task::block_on(clone!(@weak database => async move {
                let mut write = database.write().await;

                if let Some(database) = write.take() {
                    database.close().await;
                }
            }));

            Inhibit(false)
        }),
    );

    let main_box =
        gtk4::Box::builder().orientation(Orientation::Vertical).build();

    let top_box = gtk4::Box::builder()
        .orientation(Orientation::Horizontal)
        .hexpand(true)
        .build();

    let file_label = Label::builder()
        .label("No file selected")
        .hexpand(true)
        .xalign(0.0)
        .build();
    file_label.set_margin_default();
    top_box.append(&file_label);

    let file_chooser = FileChooserNative::builder()
        .title("Select file to open")
        .transient_for(&window)
        .modal(true)
        .action(FileChooserAction::Open)
        .select_multiple(false)
        .accept_label("Open")
        .cancel_label("Cancel")
        .build();

    let select_file_button = Button::builder().label("Browse").build();
    select_file_button.set_margin_default();
    select_file_button.connect_clicked(
        clone!(@strong file_chooser => move |_| {
            file_chooser.show();
        }),
    );
    top_box.append(&select_file_button);

    main_box.append(&top_box);

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

    let callback = clone!(@strong version => move || {
        let (id, base_id, base_name, name, date, deletions, insertions) =
            task::block_on(async {
                let read = version.read().await;
                let version = read.as_ref().unwrap();
                let base = version.base().await;

                let (base_id, base_name) = if let Some(base) = base {
                    let (base_id, base_name) = join!(base.id(), base.name());
                    (Some(base_id), Some(base_name))
                } else {
                    (None, None)
                };

                let (id, name, date, deletions, insertions) = join!(
                    version.id(),
                    version.name(),
                    version.date(),
                    version.deletions(),
                    version.insertions(),
                );

                (id, base_id, base_name, name, date, deletions, insertions)
            });

        id_label.set_label(&format!("Version ID: {}", id));

        if let (Some(base_id), Some(base_name)) = (base_id, base_name) {
            base_label.set_label(
                &format!("Base version: {}: {}", base_id, base_name),
            );
        } else {
            base_label.set_label(&format!("Base version: None"));
        }

        name_label.set_label(&format!("Version name: {}", name));
        date_label.set_label(&format!("Commit date: {}", date));
        deletions_label.set_label(&format!("Deleteion count: {}", deletions));
        insertions_label.set_label(&format!("Insertion count: {}", insertions));
    });

    file_chooser.connect_response(clone!(
        @strong callback,
        @weak handle,
        @weak database,
        @weak version,
        @weak scrolled_grid,
    => move |file_chooser, response| {
        if let ResponseType::Accept = response {
            let path = file_chooser.file()
                .unwrap()
                .path()
                .unwrap();

            file_label.set_label(&format!(
                "Selected file: {}",
                path.to_string_lossy(),
            ));

            if let Some(handle) = handle.borrow_mut().as_mut() {
                task::block_on(handle);
            }

            for child in scrolled_grid.observe_children() {
                scrolled_grid.remove(
                    &child.dynamic_cast::<Widget>().unwrap(),
                );
            }

            let (mut sender, mut reciever) = mpsc::channel(4);

            *handle.borrow_mut() = Some(task::spawn(
                clone!(@weak database, @weak version => async move {
                    let mut write = database.write().await;

                    if let Some(database) = write.take() {
                        if database.path() != path {
                            database.close().await;

                            *write = Some(Database::new(path).await.unwrap());
                        } else {
                            *write = Some(database);
                        }
                    } else {
                        *write = Some(
                            Database::new(path).await.unwrap(),
                        );
                    }

                    let database = write.as_ref().unwrap().clone();
                    *version.write().await = Some(database.versions());
                    sender.start_send(database.clone()).unwrap();
                })
            ));

            MainContext::default().spawn_local(clone!(
                @strong callback,
                @weak version,
                @weak scrolled_grid,
            => async move {
                loop {
                    if let Ok(Some(database))
                        = reciever.try_next()
                    {
                        build_tree(
                            &scrolled_grid,
                            0,
                            &mut 0,
                            database.versions(),
                            version.clone(),
                            &callback,
                        );

                        callback();

                        break;
                    }

                    task::sleep(Duration::from_millis(10)).await;
                }
            }));
        }
    }));

    let commit_button =
        Button::builder().label("Commit").halign(Align::Fill).build();
    commit_button.set_margin_default();
    commit_button.connect_clicked(clone!(
        @strong callback,
        @weak handle,
        @weak database,
        @weak version,
        @weak scrolled_grid
    => move |_| {
            if let Some(handle) = handle.borrow_mut().as_mut() {
                task::block_on(handle);
            }

            let (mut sender, mut receiver) = mpsc::channel(4);

            *handle.borrow_mut() = Some(task::spawn(
                clone!(@weak database, @weak version => async move {
                    let read = version.read().await;

                    if let Some(version) = read.as_ref() {
                        version.commit().await.unwrap();

                        sender
                            .start_send(
                                database.read().await.as_ref().unwrap().clone(),
                            )
                            .unwrap();
                    }
                }),
            ));

            MainContext::default().spawn_local(
                clone!(@strong callback, @weak version, @weak scrolled_grid =>
                    async move {
                        loop {
                            if let Ok(option) = receiver.try_next() {
                                if let Some(database) = option {
                                    for child
                                        in scrolled_grid.observe_children()
                                    {
                                        scrolled_grid.remove(
                                            &child
                                                .dynamic_cast::<Widget>()
                                                .unwrap(),
                                        );
                                    }

                                    build_tree(
                                        &scrolled_grid,
                                        0,
                                        &mut 0,
                                        database.versions(),
                                        version.clone(),
                                        &callback,
                                    );
                                }

                                break;
                            }

                            task::sleep(Duration::from_millis(10)).await;
                        }
                    }
                ),
            );
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
        @strong callback,
        @weak handle,
        @weak database,
        @weak version,
        @weak scrolled_grid,
    => move |rename_dialog, response| {
        if response == ResponseType::Ok {
            if let Some(handle) = handle.borrow_mut().as_mut() {
                task::block_on(handle);
            }

            let (mut sender, mut receiver) = mpsc::channel(4);

            let name = rename_entry.text().to_string();

            *handle.borrow_mut() =
                Some(task::spawn(clone!(@weak version => async move {
                    let read = version.read().await;

                    if let Some(version) = read.as_ref() {
                        version.rename(name).await.unwrap();
                        sender
                            .start_send(
                                database.read().await.as_ref().unwrap().clone(),
                            )
                            .unwrap();
                    }
                })));

            MainContext::default().spawn_local(
                clone!(@strong callback, @weak version, @weak scrolled_grid =>
                    async move {
                        loop {
                            if let Ok(option) = receiver.try_next() {
                                if let Some(database) = option {
                                    for child
                                        in scrolled_grid.observe_children()
                                    {
                                        scrolled_grid.remove(
                                            &child
                                                .dynamic_cast::<Widget>()
                                                .unwrap(),
                                        );
                                    }

                                    build_tree(
                                        &scrolled_grid,
                                        0,
                                        &mut 0,
                                        database.versions(),
                                        version.clone(),
                                        &callback,
                                    );

                                    callback();
                                }

                                break;
                            }

                            task::sleep(Duration::from_millis(10)).await;
                        }
                    }
                ),
            );
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

    let rollback_button =
        Button::builder().label("Roll back").halign(Align::Fill).build();
    rollback_button.set_margin_default();
    rollback_button.connect_clicked(clone!(@weak handle, @weak version =>
        move |_| {
            if let Some(handle) = handle.borrow_mut().as_mut() {
                task::block_on(handle);
            }

            *handle.borrow_mut() = Some(task::spawn(clone!(@weak version =>
                async move {
                    let read = version.read().await;

                    if let Some(version) = read.as_ref() {
                        version.rollback().await.unwrap();
                    }
                }
            )));
        }
    ));
    side_box.append(&rollback_button);

    let delete_dialog = MessageDialog::builder()
        .title("Delete version")
        .text("Are you sure you want to delete the selected version?")
        .transient_for(&window)
        .modal(true)
        .message_type(MessageType::Warning)
        .buttons(gtk4::ButtonsType::YesNo)
        .build();
    delete_dialog.connect_response(clone!(
        @strong callback,
        @weak handle,
        @weak database,
        @weak version,
        @weak scrolled_grid,
    => move |delete_dialog, response| {
        if response == ResponseType::Yes {
            if let Some(handle) = handle.borrow_mut().as_mut() {
                task::block_on(handle);
            }

            let (mut sender, mut receiver) = mpsc::channel(4);

            *handle.borrow_mut() = Some(task::spawn(
                clone!(@weak database, @weak version => async move {
                    let mut write = version.write().await;

                    if let Some(version) = write.as_mut() {
                        if let Some(base) = version.base().await {
                            version.delete().await.unwrap();
                            *version = base;
                            sender
                                .start_send(
                                    database.read()
                                        .await
                                        .as_ref()
                                        .unwrap()
                                        .clone(),
                                )
                                .unwrap();
                        }
                    }
                }),
            ));

            MainContext::default().spawn_local(
                clone!(@strong callback, @weak version, @weak scrolled_grid =>
                    async move {
                        loop {
                            if let Ok(option) = receiver.try_next() {
                                if let Some(database) = option {
                                    for child
                                        in scrolled_grid.observe_children()
                                    {
                                        scrolled_grid.remove(
                                            &child
                                                .dynamic_cast::<Widget>()
                                                .unwrap(),
                                        );
                                    }

                                    build_tree(
                                        &scrolled_grid,
                                        0,
                                        &mut 0,
                                        database.versions(),
                                        version.clone(),
                                        &callback,
                                    );

                                    callback();
                                }

                                break;
                            }

                            task::sleep(Duration::from_millis(10)).await;
                        }
                    }
                ),
            );
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

fn build_tree<F: 'static + Fn() + Clone>(
    grid: &Grid,
    grid_width: i32,
    grid_height: &mut i32,
    version: Version,
    selected: Arc<RwLock<Option<Version>>>,
    callback: &F,
) {
    let (id, name, children) = task::block_on(async {
        join!(
            version.id(),
            async { version.name().await.to_string() },
            version.children(),
        )
    });

    let button = Button::builder().label(&format!("{}: {}", id, name)).build();

    grid.attach(&button, grid_width, *grid_height, 1, 1);

    if children.len() > 0 {
        for child in children {
            build_tree(
                grid,
                grid_width + 1,
                grid_height,
                child,
                selected.clone(),
                callback,
            );
        }
    } else {
        *grid_height += 1;
    }

    button.connect_clicked(clone!(@strong callback => move |_| {
        *task::block_on(selected.write()) = Some(version.clone());
        callback();
    }));
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
