use std::{
    borrow::Cow,
    path::PathBuf,
    sync::{Arc, Mutex, MutexGuard, OnceLock},
};

use directories::ProjectDirs;
use eframe::egui::{
    self, Align2, Checkbox, DragValue, Grid, Hyperlink, Layout, MenuBar, ScrollArea,
    ThemePreference, ViewportBuilder, ViewportId, vec2,
};

use crate::{buffer::BufferWriter, types::Format};

const WINDOW_TITLE: &str = "Divine Tools";
const CONFIG_NAME: &str = "config.json";

static CONFIG: OnceLock<Arc<Mutex<Config>>> = OnceLock::new();

pub fn run_editor() -> crate::Result<()> {
    Config::load().map_err(|e| format!("Failed to load config: {e}"))?;

    let native_options = eframe::NativeOptions::default();
    let app = Editor::default();

    eframe::run_native(
        WINDOW_TITLE,
        native_options,
        Box::new(|ctx| {
            let config = Config::get();

            match config.theme {
                ColorTheme::System => ctx
                    .egui_ctx
                    .options_mut(|o| o.theme_preference = ThemePreference::System),
                ColorTheme::Light => ctx
                    .egui_ctx
                    .options_mut(|o| o.theme_preference = ThemePreference::Light),
                ColorTheme::Dark => ctx
                    .egui_ctx
                    .options_mut(|o| o.theme_preference = ThemePreference::Dark),
            }

            Ok(Box::new(app))
        }),
    )?;

    Ok(())
}

#[derive(Default)]
pub struct Editor {
    loaded_file: Option<Format>,
    message: Option<Message>,
    show_preferences: bool,
    show_world_editor_help: bool,
    show_about: bool,
}

impl Editor {
    fn show_message(&mut self, text: &str, severity: MessageSeverity) {
        self.message = Some(Message {
            text: text.to_owned(),
            severity,
        });
    }

    fn show_preferences(&mut self, ctx: &egui::Context) {
        let viewport_id = ViewportId::from_hash_of("preferences");
        ctx.show_viewport_immediate(
            viewport_id,
            ViewportBuilder::default().with_title("Preferences"),
            |ctx, _| {
                let mut config = Config::get();

                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.allocate_ui(ui.available_size() - vec2(0.0, 30.0), |ui| {
                        ScrollArea::vertical()
                            .auto_shrink([false; 2])
                            .show(ui, |ui| {
                                ui.heading("General");

                                Grid::new("prefs_general")
                                    .num_columns(2)
                                    .striped(true)
                                    .min_col_width(200.0)
                                    .show(ui, |ui| {
                                        ui.label("Path to Divine Divinity");

                                        ui.allocate_ui(ui.available_size(), |ui| {
                                            if ui.button("Browse").clicked() {
                                                let Some(path) =
                                                    rfd::FileDialog::new().pick_folder()
                                                else {
                                                    return;
                                                };

                                                config.dd_path = path;
                                            }

                                            let mut path = config.dd_path.to_string_lossy();
                                            ui.centered_and_justified(|ui| {
                                                ui.text_edit_singleline(&mut path);
                                            });

                                            if let Cow::Owned(new_path) = path {
                                                config.dd_path = new_path.into();
                                            }
                                        });

                                        ui.end_row();
                                    });

                                ui.heading("Appearance");

                                Grid::new("prefs_appearance")
                                    .num_columns(2)
                                    .striped(true)
                                    .min_col_width(200.0)
                                    .show(ui, |ui| {
                                        ui.label("Theme");

                                        ui.allocate_ui(ui.available_size(), |ui| {
                                            ui.radio_value(
                                                &mut config.theme,
                                                ColorTheme::System,
                                                "System",
                                            );
                                            ui.radio_value(
                                                &mut config.theme,
                                                ColorTheme::Light,
                                                "Light",
                                            );
                                            ui.radio_value(
                                                &mut config.theme,
                                                ColorTheme::Dark,
                                                "Dark",
                                            );
                                        });
                                    });
                            });
                    });

                    ui.vertical_centered(|ui| {
                        if ui.button("Apply").clicked() {
                            match config.save() {
                                Ok(_) => match config.theme {
                                    ColorTheme::System => ctx.options_mut(|o| {
                                        o.theme_preference = ThemePreference::System
                                    }),
                                    ColorTheme::Light => ctx.options_mut(|o| {
                                        o.theme_preference = ThemePreference::Light
                                    }),
                                    ColorTheme::Dark => ctx.options_mut(|o| {
                                        o.theme_preference = ThemePreference::Dark
                                    }),
                                },
                                Err(e) => {
                                    self.show_message(
                                        &format!("Failed to save preferences: {e}"),
                                        MessageSeverity::Error,
                                    );
                                }
                            }
                        }
                    });
                });

                ctx.input(|i| {
                    if i.viewport().close_requested() {
                        self.show_preferences = false;
                    }
                });
            },
        );
    }

    fn show_world_editor_help(&mut self, ctx: &egui::Context) {
        let viewport_id = ViewportId::from_hash_of("world_editor_help");
        ctx.show_viewport_immediate(
            viewport_id,
            ViewportBuilder::default()
                .with_title("World editor help")
                .with_inner_size(vec2(500.0, 400.0)),
            |ctx, _| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.heading("Navigation");

                    ui.label("To pan the view, press any mouse button and move the mouse. You can also use scroll bars on the sides.");

                    ui.heading("Painting");

                    ui.label("On the right side, select a tile to paint. Pressing left mouse \
                        button will select primary tile, and pressing right mouse button will \
                        select an overlay tile that will be painted on top of primary.");

                    ui.label("To place a primary tile, hold CTRL and press left mouse button. \
                        To place an overlay tile, hold CTRL and press right mouse button.");

                    ui.label("To erase an overlay tile under the cursor, hold X and press right mouse button. \
                        You can not erase primary tile.");

                    ui.heading("Objects");

                    ui.label("Holding ALT will highlight all objects. To select an object, click on it while holding ALT. \
                        When object is selected, you can drag it to move it to a different place.");

                    ui.separator();

                    ui.label("To undo changes, press CTRL+Z. To redo, press CTRL+SHIFT+Z.");

                    ctx.input(|i| {
                        if i.viewport().close_requested() {
                            self.show_world_editor_help = false;
                        }
                    });
                });
            },
        );
    }

    fn show_about(&mut self, ctx: &egui::Context) {
        let viewport_id = ViewportId::from_hash_of("about");
        ctx.show_viewport_immediate(
            viewport_id,
            ViewportBuilder::default()
                .with_title("About")
                .with_inner_size(vec2(200.0, 100.0)),
            |ctx, _| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.vertical_centered_justified(|ui| {
                        ui.label("Divine Tools");
                        ui.label(format!("Version {}", env!("CARGO_PKG_VERSION")));
                        ui.add(Hyperlink::from_label_and_url(
                            "GitHub",
                            "https://github.com/fstxz/divine_tools",
                        ));
                    });

                    ctx.input(|i| {
                        if i.viewport().close_requested() {
                            self.show_about = false;
                        }
                    });
                });
            },
        );
    }
}

struct Message {
    text: String,
    severity: MessageSeverity,
}

enum MessageSeverity {
    // Info,
    // Warning,
    Error,
}

impl eframe::App for Editor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut message_closed = false;
        if let Some(message) = &self.message {
            let severity = match message.severity {
                // MessageSeverity::Info => "Info",
                // MessageSeverity::Warning => "Warning",
                MessageSeverity::Error => "Error",
            };

            egui::Window::new(severity)
                .collapsible(false)
                .resizable(false)
                .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(&message.text);
                        if ui.button("Close").clicked() {
                            message_closed = true;
                        }
                    });
                });
        }

        if message_closed {
            self.message = None;
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open").clicked() {
                        let file_dialog = rfd::FileDialog::new();

                        let Some(file_path) = file_dialog.pick_file() else {
                            return;
                        };

                        match Format::from_file(&file_path).and_then(|mut v| {
                            v.binary.init(&Context { ui })?;
                            Ok(v)
                        }) {
                            Ok(v) => {
                                ctx.send_viewport_cmd(egui::ViewportCommand::Title(format!(
                                    "{WINDOW_TITLE} - {}",
                                    file_path.display()
                                )));
                                self.loaded_file = Some(v)
                            }
                            Err(e) => {
                                self.show_message(&format!("{e}"), MessageSeverity::Error);
                            }
                        }
                    }

                    ui.separator();

                    ui.add_enabled_ui(self.loaded_file.is_some(), |ui| {
                        let save_clicked = ui.button("Save").clicked();
                        let save_as_clicked = ui.button("Save As...").clicked();

                        ui.separator();

                        let export_clicked = ui.button("Export as JSON").clicked();

                        if let Some(loaded_file) = &self.loaded_file {
                            if export_clicked {
                                let file_dialog = rfd::FileDialog::new()
                                    .set_directory(
                                        std::env::current_dir()
                                            .expect("must be able to get current directory"),
                                    )
                                    .set_file_name(
                                        loaded_file
                                            .file_name
                                            .as_ref()
                                            .unwrap_or(&PathBuf::from("file"))
                                            .with_added_extension("json")
                                            .to_string_lossy(),
                                    );

                                let Some(path) = file_dialog.save_file() else {
                                    return;
                                };

                                let Ok(serialized) = serde_json::to_string_pretty(&loaded_file)
                                else {
                                    self.show_message(
                                        "Failed to serialize the file",
                                        MessageSeverity::Error,
                                    );
                                    return;
                                };

                                if let Err(e) = std::fs::write(path, serialized) {
                                    self.show_message(
                                        &format!("Failed to write to file: {e}"),
                                        MessageSeverity::Error,
                                    );
                                    return;
                                }
                            }

                            if save_clicked || save_as_clicked {
                                let mut writer = BufferWriter::new();
                                loaded_file.binary.to_bytes(&mut writer);
                                let bytes = writer.finish();

                                let path = if save_as_clicked {
                                    let file_dialog = rfd::FileDialog::new();

                                    let Some(path) = file_dialog.save_file() else {
                                        return;
                                    };

                                    path
                                } else {
                                    match &loaded_file.path {
                                        Some(p) => p.clone(),
                                        None => {
                                            let file_dialog = rfd::FileDialog::new();

                                            let Some(path) = file_dialog.save_file() else {
                                                return;
                                            };

                                            path
                                        }
                                    }
                                };

                                if let Err(e) = std::fs::write(path, bytes) {
                                    eprintln!("Failed to write to a file: {e}");
                                }
                            }
                        }
                    });

                    let import_clicked = ui.button("Import from JSON").clicked();

                    if import_clicked {
                        let file_dialog = rfd::FileDialog::new();

                        let Some(path) = file_dialog.pick_file() else {
                            return;
                        };

                        let file = match std::fs::read_to_string(&path) {
                            Ok(f) => f,
                            Err(e) => {
                                self.show_message(
                                    &format!("Failed to open file at {}: {e}", path.display()),
                                    MessageSeverity::Error,
                                );
                                return;
                            }
                        };

                        let Ok(deserialized) = serde_json::from_str::<Format>(&file) else {
                            self.show_message("Failed to load file", MessageSeverity::Error);
                            return;
                        };

                        self.loaded_file = Some(deserialized);
                    }

                    ui.separator();

                    if ui.button("Quit").clicked() {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.menu_button("Edit", |ui| {
                    if ui.button("Preferences").clicked() {
                        self.show_preferences = true;
                    }
                });

                ui.menu_button("Help", |ui| {
                    if ui.button("World editor").clicked() {
                        self.show_world_editor_help = true;
                    }

                    ui.separator();

                    if ui.button("About").clicked() {
                        self.show_about = true;
                    }
                });
            });
        });

        if self.show_preferences {
            self.show_preferences(ctx);
        }

        if self.show_world_editor_help {
            self.show_world_editor_help(ctx);
        }

        if self.show_about {
            self.show_about(ctx);
        }

        egui::CentralPanel::default().show(ctx, |ui| match &mut self.loaded_file {
            Some(file) => {
                egui::ScrollArea::vertical()
                    .auto_shrink(false)
                    .show(ui, |ui| {
                        file.binary.show(ui);
                    });
            }
            _ => {
                ui.centered_and_justified(|ui| {
                    ui.label("To load a file, select File -> Open");
                });
            }
        });
    }
}

pub trait Inspector: 'static {
    fn init(&mut self, _ctx: &Context) -> crate::Result<()> {
        Ok(())
    }

    fn show(&mut self, ui: &mut egui::Ui);
}

pub struct Context<'a> {
    pub ui: &'a egui::Ui,
}

pub fn struct_ui(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui)) {
    egui::Grid::new(ui.id())
        .num_columns(2)
        .striped(true)
        .spacing([40.0, 8.0])
        .show(ui, add_contents);
}

pub fn property<T: Inspector>(name: &str, property: &mut T, ui: &mut egui::Ui) {
    ui.label(name);
    ui.push_id(name, |ui| {
        property.show(ui);
    });
    ui.end_row();
}

pub fn property_read_only<T: Inspector>(name: &str, property: &mut T, ui: &mut egui::Ui) {
    ui.label(name);
    ui.add_enabled_ui(false, |ui| {
        ui.push_id(name, |ui| {
            property.show(ui);
        });
    });
    ui.end_row();
}

pub fn property_tooltip<T: Inspector>(
    name: &str,
    tooltip_text: &str,
    property: &mut T,
    ui: &mut egui::Ui,
) {
    ui.allocate_ui(ui.available_size(), |ui| {
        ui.label(name);
        ui.label("(?)").on_hover_text(tooltip_text);
    });
    ui.allocate_ui(ui.available_size(), |ui| {
        property.show(ui);
    });
    ui.end_row();
}

impl Inspector for String {
    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        ui.text_edit_multiline(self);
    }
}

impl Inspector for u32 {
    fn show(&mut self, ui: &mut egui::Ui) {
        ui.add(DragValue::new(self));
    }
}

impl Inspector for u8 {
    fn show(&mut self, ui: &mut egui::Ui) {
        ui.add(DragValue::new(self));
    }
}

impl Inspector for f32 {
    fn show(&mut self, ui: &mut egui::Ui) {
        ui.add(DragValue::new(self));
    }
}

impl<T: Inspector + Default> Inspector for Vec<T> {
    fn show(&mut self, ui: &mut egui::Ui) {
        let mut index_to_delete = None;

        // (from, to)
        let mut index_to_swap = None;

        egui::CollapsingHeader::new(format!("Array ({})", self.len()))
            .id_salt(ui.id())
            .show_background(true)
            .show(ui, |ui| {
                ui.with_layout(Layout::left_to_right(egui::Align::Min), |ui| {
                    if ui.button("Add element").clicked() {
                        self.push(T::default());
                    }
                });

                ui.separator();

                let len = self.len();

                egui::Grid::new(ui.id())
                    .num_columns(1)
                    .spacing([40.0, 8.0])
                    .show(ui, |ui| {
                        for (i, element) in self.iter_mut().enumerate() {
                            ui.push_id(i, |ui| {
                                ui.vertical(|ui| {
                                    ui.add_enabled_ui(i > 0, |ui| {
                                        if ui.button("🔼").clicked() {
                                            index_to_swap = Some((i, i - 1))
                                        }
                                    });

                                    if ui.button("❌").clicked() {
                                        index_to_delete = Some(i);
                                    }

                                    ui.add_enabled_ui(i < len - 1, |ui| {
                                        if ui.button("🔽").clicked() {
                                            index_to_swap = Some((i, i + 1))
                                        }
                                    });
                                });

                                egui::collapsing_header::CollapsingState::load_with_default_open(
                                    ui.ctx(),
                                    ui.id(),
                                    true,
                                )
                                .show_header(ui, |ui| {
                                    ui.label(format!("{i}"));
                                })
                                .body_unindented(|ui| {
                                    element.show(ui);
                                });
                            });
                            ui.end_row();
                        }
                    });
            });

        if let Some(index) = index_to_delete {
            self.remove(index);
        }

        if let Some((from, to)) = index_to_swap {
            self.swap(from, to);
        }
    }
}

impl Inspector for char {
    fn show(&mut self, ui: &mut egui::Ui) {
        let mut s = self.to_string();
        ui.text_edit_singleline(&mut s);
        match s.parse() {
            Ok(v) => *self = v,
            Err(e) => eprintln!("failed to parse char: {e}"),
        }
    }
}

impl Inspector for i16 {
    fn show(&mut self, ui: &mut egui::Ui) {
        ui.add(DragValue::new(self));
    }
}

impl Inspector for u16 {
    fn show(&mut self, ui: &mut egui::Ui) {
        ui.add(DragValue::new(self));
    }
}

impl Inspector for bool {
    fn show(&mut self, ui: &mut egui::Ui) {
        ui.add(Checkbox::without_text(self));
    }
}

impl Inspector for i32 {
    fn show(&mut self, ui: &mut egui::Ui) {
        ui.add(DragValue::new(self));
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub(crate) struct Config {
    #[serde(default)]
    pub dd_path: PathBuf,
    #[serde(default)]
    pub theme: ColorTheme,
}

impl Config {
    const QUALIFIER: &str = "org";
    const ORGANIZATION: &str = "Divine Tools";
    const APPLICATION: &str = "Divine Tools";

    pub fn get<'a>() -> MutexGuard<'a, Self> {
        CONFIG.get().unwrap().lock().unwrap()
    }

    fn load() -> crate::Result<()> {
        let Some(dirs) = Self::get_project_dirs() else {
            return Err("couldn't retrieve config directory".into());
        };

        let path = dirs.config_dir().join(CONFIG_NAME);

        if !std::fs::exists(&path)? {
            std::fs::create_dir_all(dirs.config_dir())?;
            std::fs::write(&path, serde_json::to_string_pretty(&Config::default())?)?;
        }

        let file = std::fs::read_to_string(&path)?;

        let config = serde_json::from_str(&file)?;
        let _ = CONFIG.set(Arc::new(Mutex::new(config)));

        Ok(())
    }

    fn save(&self) -> crate::Result<()> {
        let serialized = serde_json::to_string_pretty(self)?;

        let Some(dirs) = Self::get_project_dirs() else {
            return Err("couldn't retrieve config directory".into());
        };

        std::fs::create_dir_all(dirs.config_dir())?;
        std::fs::write(dirs.config_dir().join(CONFIG_NAME), serialized)?;

        Ok(())
    }

    fn get_project_dirs() -> Option<ProjectDirs> {
        directories::ProjectDirs::from(Self::QUALIFIER, Self::ORGANIZATION, Self::APPLICATION)
    }
}

#[derive(Default, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
pub(crate) enum ColorTheme {
    #[default]
    System,
    Light,
    Dark,
}
