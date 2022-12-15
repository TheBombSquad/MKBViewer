//! Handles all the UI-related activities
use crate::stagedef::{StageDef, StageDefInstance};
use egui::{Button, Frame, Label, Window};
use egui::{CentralPanel, Separator, TopBottomPanel};
use futures::executor::block_on;
use poll_promise::Promise;
use rfd::AsyncFileDialog;
use rfd::FileHandle;
use std::io::Cursor;
use std::vec::Vec;
use tracing::{event, instrument, trace, Level};

/// Our root window.
#[derive(Default)]
pub struct MkbViewerApp {
    /// A file pending to load, which we will split off into a new window to handle once the
    /// promise has a result.
    pending_file_to_load: Option<Promise<Option<FileHandleWrapper>>>,
    /// A collection of all the ['stagedef instances'](MkbViewerApp::StageDefInstance) that are
    /// currently loaded.
    stagedef_viewers: Vec<StageDefInstance>,
    /// The state of the central widget, used to display a message indicating the status.
    state: CentralWidgetState,
}

impl MkbViewerApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self::default()
    }

    /// Open a file dialog with the given restriction on file type.
    // TODO: Support for WSMod configs
    fn open_file_dialog(&mut self, file_type: MkbFileType) {
        self.pending_file_to_load = Some(MkbViewerApp::get_promise_from_file_dialog(file_type));
    }

    /// Poll [`pending_file_to_load`](MkbViewerApp::pending_file_to_load) for a file to load, handle it based on the assigned type.
    ///
    /// This is run every frame.
    fn poll_pending_file(&mut self) {
        let pending_file_to_load = self.pending_file_to_load.take();

        // Checks if we even have a promise to wait on
        let Some(promise) = pending_file_to_load else {
            trace!("No file open promise check"); 
            return;
        };

        self.state = CentralWidgetState::Loading;

        // If we do, checks if that promise has completed yet
        let filehandle_opt = match promise.try_take() {
            Ok(o) => {
                trace!("Promise completed");
                o
            }
            Err(o) => {
                trace!("Promise has not completed yet");
                self.pending_file_to_load = Some(o);
                return;
            }
        };

        // If it has completed, check to see if it returned anything
        let Some(filehandle) = filehandle_opt else {
            event!(Level::INFO, "No file was selected");
            self.state = self.get_non_loading_state();
            self.pending_file_to_load = None;
            return;
        };

        // Construct the new StageDefInstance since we've loaded the file
        event!(
            Level::INFO,
            "Loading pending file: {}...",
            filehandle.file_name
        );

        let new_instance_test = StageDefInstance::new(filehandle).unwrap();

        event!(
            Level::INFO,
            "Instance test: {:?}",
            new_instance_test.stagedef.magic_number_2
        );

        self.stagedef_viewers.push(new_instance_test);

        self.state = self.get_non_loading_state();
        self.pending_file_to_load = None;
    }

    /// Creates a promise for loading of files from a file picker.
    ///
    /// Spawns a new thread on native, otherwise handles asyncronously on Wasm32.
    fn get_promise_from_file_dialog(
        filter_type: MkbFileType,
    ) -> Promise<Option<FileHandleWrapper>> {
        let filter = MkbFileType::get_rfd_extension_filter(&filter_type);

        #[cfg(target_arch = "wasm32")]
        let promise = Promise::spawn_async(async {
            let file_dialog = AsyncFileDialog::new()
                .add_filter(filter.0, filter.1)
                .pick_file()
                .await;
            if let Some(f) = file_dialog {
                Some(FileHandleWrapper::new(f, filter_type).await)
            } else {
                None
            }
        });

        #[cfg(not(target_arch = "wasm32"))]
        let promise = Promise::spawn_thread("get_file_from_dialog_native", || {
            let file_dialog_future = async {
                let file_dialog = AsyncFileDialog::new()
                    .add_filter(filter.0, filter.1)
                    .pick_file()
                    .await;
                if let Some(f) = file_dialog {
                    Some(FileHandleWrapper::new(f, filter_type).await)
                } else {
                    None
                }
            };
            block_on(file_dialog_future)
        });

        promise
    }

    /// Handle the central widget's panel, which will display something depending on whether or not
    /// a stagedef is loaded.
    pub fn get_central_widget_frame(&mut self, ctx: &egui::Context) {
        let state = self.state;
        let panel = egui::CentralPanel::default();
        panel.show(ctx, |ui| {
            ui.centered_and_justified(|ui| {
                match state {
                    CentralWidgetState::NoStagedefLoaded => {
                        ui.label("No stagedef currently loaded - go to File->Open to add one")
                    }
                    CentralWidgetState::Loading => ui.label("Loading file..."),
                    CentralWidgetState::StagedefLoaded => ui.label(""),
                };
            });
        });
    }

    /// Get the appropriate (CentralWidgetState)[MkbViewerApp::CentralWidgetState] based on the
    /// number of loaded StageDefInstances.
    fn get_non_loading_state(&self) -> CentralWidgetState {
        let loaded_stagedef_count = self.stagedef_viewers.len();
        if loaded_stagedef_count > 0 {
            CentralWidgetState::StagedefLoaded
        } else {
            CentralWidgetState::NoStagedefLoaded
        }
    }
}

#[derive(Clone, Copy)]
pub enum CentralWidgetState {
    NoStagedefLoaded,
    Loading,
    StagedefLoaded,
}

impl Default for CentralWidgetState {
    fn default() -> Self {
        CentralWidgetState::NoStagedefLoaded
    }
}

impl eframe::App for MkbViewerApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.poll_pending_file();

        // Menubar
        TopBottomPanel::top("mkbviewer_menubar").show(ctx, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button(" Open...").clicked() {
                    event!(Level::INFO, "Opening file");
                    self.open_file_dialog(MkbFileType::StagedefType);
                }

                // Can't quit on web...
                #[cfg(not(target_arch = "wasm32"))]
                ui.add(Separator::default().spacing(0.0));

                #[cfg(not(target_arch = "wasm32"))]
                if ui.button(" Quit").clicked() {
                    event!(Level::INFO, "Quitting...");
                    frame.close();
                }
            });
        });

        // Toolbar
        TopBottomPanel::top("mkbviewer_toolbar")
            .min_height(32.0)
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.label("Toolbar goes here...");
                });
            });

        // Central panel
        MkbViewerApp::get_central_widget_frame(self, ctx);

        for viewer in self.stagedef_viewers.iter() {
            //event!(Level::INFO, "{:?}", viewer.stagedef_instance.stagedef.magic_number_1);
        }
    }
}

/// Represents the contents, file name, and type of a specific file.
///
/// We use this in place of `FileHandle` because it cannot be polled every frame easily on Wasm32.
#[derive(Debug, Default)]
pub struct FileHandleWrapper {
    pub buffer: Vec<u8>,
    pub file_name: String,
    pub file_type: MkbFileType,
}

impl FileHandleWrapper {
    pub async fn new(fh: FileHandle, file_type: MkbFileType) -> Self {
        trace!("Constructing new FileHandleWrapper...");
        let buffer = fh.read().await;
        trace!("Read buffer");

        Self {
            buffer,
            // TODO: Verify that this works with non-UTF8 filenames
            file_name: fh.file_name(),
            file_type,
        }
    }

    pub fn with_buffer(mut self, buffer: Vec<u8>) -> FileHandleWrapper {
        self.buffer = buffer;
        self
    }

    pub fn get_cursor(&self) -> Cursor<Vec<u8>> {
        Cursor::new(self.buffer.clone())
    }
}

/// Represents which type of file we are expecting from a file picker.
///
/// By default, this will be a [StagedefType](MkbFileType::StagedefType).
#[derive(Debug)]
pub enum MkbFileType {
    StagedefType,
    WsModConfigType,
}

impl Default for MkbFileType {
    fn default() -> Self {
        MkbFileType::StagedefType
    }
}

impl MkbFileType {
    pub fn get_rfd_extension_filter(
        filter: &MkbFileType,
    ) -> (&'static str, &'static [&'static str]) {
        match filter {
            MkbFileType::StagedefType => (&("Stagedef files"), &["lz", "lz.raw"]),
            MkbFileType::WsModConfigType => (&("Workshop Mod config files"), &["txt"]),
        }
    }
}

/*
trait FancyTreeFmt {
    fn as_tree_item(&self, tree: &mut Tree, label: Option<&str>);
}

impl FancyTreeFmt for f32 {
    fn as_tree_item(&self, tree: &mut Tree, label: Option<&str>) -> () {
        let mut input_widget = FloatInput::new(300, 300, 50, 25, "f32");
        input_widget.set_align(Align::Right);
        let val_str = self.to_string();
        input_widget.set_value(&val_str);

        let mut input_widget_item = TreeItem::new(&tree, "");
        input_widget_item.set_widget(&input_widget);
        tree.add_item("Test/", &input_widget_item);
    }
}


    // Stagedef file - so we can have multiple stagedefs open at once
    pub fn create_stagedef_tile(window: &dyn WindowExt, stagedef: &StageDefInstance) -> Tile {
        let name = stagedef.file_path.file_stem().unwrap().to_str().unwrap();

        let mut tile = Tile::default()
            .with_pos(0, 50)
            .with_size(window.width(), window.height() - 25);

        tile.set_label(name);

        let mut tree = Tree::default()
            .with_pos(0, 51)
            .with_size(200, window.height() - 50);


        let name = stagedef.file_path.file_stem().unwrap_or(OsStr::new("STAGEDEF")).to_str().unwrap_or("STAGEDEF");

        // TEST TREE STUFF
        let mut input_widget = IntInput::new(300, 300, 50, 25, "u32");
        input_widget.set_align(Align::Right);

        let mut input_widget_item = TreeItem::new(&tree, "");
        input_widget_item.set_widget(&input_widget);
        tree.add_item("Test/", &input_widget_item);

        let mut dropdown_widget = InputChoice::new(0, 0, 100, 25, "Type");
        dropdown_widget.set_align(Align::Right);
        dropdown_widget.add("Blue");
        dropdown_widget.add("Green");
        dropdown_widget.add("Red");
        dropdown_widget.set_value_index(0);

        let mut dropdown_widget_item = TreeItem::new(&tree, "");
        dropdown_widget_item.set_widget(&dropdown_widget);
        tree.add_item("Test/", &dropdown_widget_item);

        tree.end();

        let viewer = Group::default()
            .with_pos(200, 51)
            .with_size(window.width() - 200, window.height() - 50);
        viewer.end();

        tile.end();

        tile
    }


    // Handle 'quit' selection from menu
    fn on_quit(&self) {
        println!("Quitting...");
        self.app.quit();
    }

    // Handle 'about' selection from menu
    fn on_about(&self) {
        println!("{}, {}", app::screen_size().0, app::screen_size().1);
        dialog::message_title("About MKBViewer");
        dialog::message(
            screen_center().0,
            screen_center().1,
            "MKBViewer v0.1.0 - by The BombSquad",
        );
    }

    // Handle 'open' selection from menu
    fn on_open(&mut self) {
        let mut dialog = dialog::NativeFileChooser::new(dialog::NativeFileChooserType::BrowseFile);
        dialog.set_filter("*.{lz,lz.raw}");
        dialog.show();

        let filename = dialog.filename();
        let ext = filename.extension().unwrap_or_default();

        if ext == "raw" {
            match StageDefInstance::new(filename) {
                Ok(s) => {
                    let stagedef_tile = &Application::create_stagedef_tile(&self.main_window, &s);
                    self.tabs.add(stagedef_tile);
                    self.main_window.resizable(stagedef_tile);
                    self.main_window.redraw();
                    self.stagedef_instances.push(s);
                }
                Err(e) => {
                    dialog::message(screen_center().0, screen_center().1, &(e.to_string()));
                }
            }
        } else if ext == "lz" {
            dialog::message(
                screen_center().0,
                screen_center().1,
                "Compressed stagedefs not yet supported",
            );
        } else {
            dialog::message(screen_center().0, screen_center().1, "No file selected");
        }
    }

}*/
