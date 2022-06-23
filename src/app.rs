use fltk::{prelude::*,
           app::{App, Receiver, Sender, channel, self}, 
           window::Window,
           menu::{MenuBar, MenuFlag}, enums::{Shortcut, Align}, dialog, group::{Tabs, Group, Tile}, tree::{Tree, TreeItem}, input::{IntInput, Input}, button::Button
           };

use crate::stagedef::StageDefInstance;

#[derive(Clone)]
enum Message {
    OpenStagedef,
    About,
    Quit,
}

pub fn screen_center() -> (i32, i32) {
    ((app::screen_size().0 / 2.0) as i32,
     (app::screen_size().1 / 2.0) as i32)
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
                     .with_size(800,600)
                     .with_pos(screen_center().0-400, screen_center().1-300)
                     .with_label("MKBViewer");


        let mut menu_bar = MenuBar::new(0,0, 800, 25, None);
        menu_bar.add_emit("File/Open...",
                     Shortcut::None, 
                     MenuFlag::Normal,
                     sender.clone(),
                     Message::OpenStagedef);
        menu_bar.add_emit("File/Quit",
                     Shortcut::None, 
                     MenuFlag::Normal,
                     sender.clone(),
                     Message::Quit);
        menu_bar.add_emit("Help/About",
                     Shortcut::None, 
                     MenuFlag::Normal,
                     sender.clone(),
                     Message::About);

        let tabs = Tabs::new(0, 25, 800, 600, None);
        
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

    pub fn create_stagedef_tile(window: &dyn WindowExt, stagedef: &StageDefInstance) -> Tile {
        let name = stagedef.file_path.file_stem().unwrap().to_str().unwrap();

        let mut tile = Tile::new(0, 50, window.width(), window.height()-25, None); 
        tile.set_label(name); 
        tile.end(); 

        let mut tree = Tree::new(0, 51, 200, window.height()-50, None);
        tree.set_connector_style(fltk::tree::TreeConnectorStyle::Dotted);
        tree.end();

        let test_input = IntInput::new(0, 0, 50, 25, "u32: ");
        let mut test_tree_item = TreeItem::new(&tree, ""); 
        test_tree_item.set_widget(&test_input);
        tree.add_item("Tree item", &test_tree_item);

        /*
        tree.set_root_label(name);
        tree.add("Magic Numbers");
        tree.add("Magic Numbers/No. 2");
        let test = format!("Magic Numbers/No. 2/Float: {:#}", stagedef.stagedef.magic_number_2);
        tree.add(test.as_str()); 
        let test2 = format!("Magic Numbers/No. 2/u32: {:#}", stagedef.stagedef.magic_number_2);
        tree.add(test2.as_str());
        tree.add("Start Position"); 
        */

        let main_group = Group::new(200, 51, window.width()-200, window.height()-50, None);
        main_group.end();

        tile.add(&tree);
        tile.add(&main_group);
        
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
        dialog::message(screen_center().0,screen_center().1,"MKBViewer v0.1.0 - by The BombSquad");
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
                    self.tabs.add(&Application::create_stagedef_tile(&self.main_window, &s));
                    self.main_window.redraw();
                    self.stagedef_instances.push(s);
                },
                Err(e) => {
                    dialog::message(screen_center().0, screen_center().1, &(e.to_string()));
                }
            }
        }

        else if ext == "lz" {
            dialog::message(screen_center().0,screen_center().1,"Compressed stagedefs not yet supported");
        }
        else {
            dialog::message(screen_center().0,screen_center().1,"No file selected");
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
}
