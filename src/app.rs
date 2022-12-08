use egui::{TopBottomPanel, CentralPanel, Separator};
use tracing::{Level, event, span};

use crate::stagedef::StageDefInstance;


#[derive(Default)]
pub struct MkbViewerApp {}

impl MkbViewerApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self::default()
    }
}

impl eframe::App for MkbViewerApp {
   fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
       TopBottomPanel::top("mkbviewer_menubar").show(ctx, |ui| {
           ui.menu_button("File", |ui| {
                if ui.button (" Open...").clicked() {
                    event!(Level::INFO, "Opening file");
                }
                #[cfg(not(target_arch = "wasm32"))]
                ui.add(Separator::default().spacing(0.0));

                #[cfg(not(target_arch = "wasm32"))]
                if ui.button (" Quit").clicked() {
                    event!(Level::INFO, "Quitting...");
                    frame.close();
                }
           });
       });

       TopBottomPanel::top("mkbviewer_toolbar").min_height(32.0).show(ctx, |ui| {
           ui.horizontal_centered(|ui| {
            ui.label("Toolbar goes here...");
           });
       });

       CentralPanel::default().show(ctx, |ui| {
           ui.centered_and_justified(|ui| {
               ui.label("No stagedef currently loaded - go to File->Open to add one!");
           });
       });
   }
}

/*
#[derive(Clone)]
enum Message {
    OpenStagedef,
    About,
    Quit,
} */

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

pub fn screen_center() -> (i32, i32) {
    (
        (app::screen_size().0 / 2.0) as i32,
        (app::screen_size().1 / 2.0) as i32,
    )
}


pub struct Application {
    app: App,
    main_window: Window,
    menu_bar: MenuBar,
    tabs: Tabs,
    stagedef_instances: Vec<StageDefInstance>,
    sender: Sender<Message>,
    receiver: Receiver<Message>,
}

impl Application {
    
    pub fn new() -> Self {
        let app = App::default();
        let (sender, receiver) = channel::<Message>();
        let mut main_window = Window::default()
            .with_size(800, 600)
            .with_pos(screen_center().0 - 400, screen_center().1 - 300)
            .with_label("MKBViewer");

        main_window.make_resizable(true);

        let mut menu_bar = MenuBar::new(0, 0, 800, 25, None);
        menu_bar.add_emit(
            "File/Open...",
            Shortcut::None,
            MenuFlag::Normal,
            sender.clone(),
            Message::OpenStagedef,
        );
        menu_bar.add_emit(
            "File/Quit",
            Shortcut::None,
            MenuFlag::Normal,
            sender.clone(),
            Message::Quit,
        );
        menu_bar.add_emit(
            "Help/About",
            Shortcut::None,
            MenuFlag::Normal,
            sender.clone(),
            Message::About,
        );

        let tabs = Tabs::new(0, 25, 800, 575, None);

        main_window.end();

        let stagedef_instances: Vec<StageDefInstance> = Vec::new();
        
        main_window.show();

        Self {
            app,
            main_window,
            menu_bar,
            tabs,
            stagedef_instances,
            sender,
            receiver,
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

    pub fn run(mut self) {
        while self.app.wait() {
            if let Some(msg) = self.receiver.recv() {
                match msg {
                    Message::Quit => self.on_quit(),
                    Message::About => self.on_about(),
                    Message::OpenStagedef => self.on_open(),
                }
            }
        }
    }
    
}*/
