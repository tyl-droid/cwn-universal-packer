#[cfg(target_os = "android")]
fn main() {
    println!(
        "CWN Universal Packer GUI is a desktop application.\n\
         The CLI remains fully supported in Termux."
    );
}

#[cfg(not(target_os = "android"))]
mod desktop {
    use cwn_universal_packer::container::manifest::PackageMetadata;
    use cwn_universal_packer::engine::types::ContainerInfo;
    use cwn_universal_packer::filesystem::size::human_size;
    use cwn_universal_packer::gui_task::{GuiMessage, GuiTask};
    use eframe::egui;
    use egui::{Color32, CornerRadius, Stroke, Vec2};

    use std::path::PathBuf;
    use std::thread;
    use std::time::Duration;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Workspace {
        Pack,
        Archive,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum StatusKind {
        Info,
        Success,
        Warning,
        Error,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum ArchiveFilter {
        All,
        Files,
        Directories,
        Zstd,
        Stored,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum ArchiveSortColumn {
        Type,
        OriginalSize,
        PackedSize,
        Compression,
        Path,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum SortDirection {
        Ascending,
        Descending,
    }

    pub struct CwnPackerApp {
        workspace: Workspace,

        inputs: Vec<PathBuf>,

        archive: Option<PathBuf>,
        archive_info: Option<ContainerInfo>,
        archive_search: String,
        archive_filter: ArchiveFilter,
        archive_sort_column: ArchiveSortColumn,
        archive_sort_direction: SortDirection,
        selected_archive_entry: Option<usize>,
        output: Option<PathBuf>,
        extract_to: Option<PathBuf>,

        package_name: String,
        package_version: String,
        publisher: String,

        compression_level: i32,

        status: String,
        status_kind: StatusKind,
        busy: bool,
        current_operation: Option<String>,

        progress_current: usize,
        progress_total: usize,
        progress_path: Option<String>,

        task: GuiTask,
    }

    impl Default for CwnPackerApp {
        fn default() -> Self {
            Self::new()
        }
    }

    impl CwnPackerApp {
        fn apply_cwn_theme(ctx: &egui::Context) {
            let mut visuals = egui::Visuals::dark();

            visuals.panel_fill = Color32::from_rgb(10, 13, 18);
            visuals.window_fill = Color32::from_rgb(15, 19, 26);
            visuals.extreme_bg_color = Color32::from_rgb(7, 10, 14);

            visuals.faint_bg_color = Color32::from_rgb(18, 24, 32);
            visuals.code_bg_color = Color32::from_rgb(9, 13, 18);

            visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(18, 23, 31);

            visuals.widgets.noninteractive.bg_stroke =
                Stroke::new(1.0, Color32::from_rgb(42, 53, 67));

            visuals.widgets.inactive.bg_fill = Color32::from_rgb(21, 28, 37);

            visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(48, 62, 78));

            visuals.widgets.hovered.bg_fill = Color32::from_rgb(30, 40, 52);

            visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, Color32::from_rgb(95, 190, 255));

            visuals.widgets.active.bg_fill = Color32::from_rgb(34, 50, 65);

            visuals.widgets.active.bg_stroke = Stroke::new(1.0, Color32::from_rgb(110, 205, 255));

            visuals.selection.bg_fill = Color32::from_rgb(28, 104, 150);

            visuals.selection.stroke = Stroke::new(1.0, Color32::from_rgb(160, 225, 255));

            visuals.window_corner_radius = CornerRadius::same(10);

            ctx.set_visuals(visuals);

            let mut style = (*ctx.style()).clone();

            style.spacing.item_spacing = Vec2::new(10.0, 8.0);
            style.spacing.button_padding = Vec2::new(14.0, 8.0);

            ctx.set_style(style);
        }

        fn accent() -> Color32 {
            Color32::from_rgb(92, 196, 255)
        }

        fn success() -> Color32 {
            Color32::from_rgb(95, 220, 150)
        }

        fn warning() -> Color32 {
            Color32::from_rgb(245, 190, 85)
        }

        fn error() -> Color32 {
            Color32::from_rgb(245, 95, 105)
        }

        fn status_color(&self) -> Color32 {
            match self.status_kind {
                StatusKind::Info => Self::accent(),
                StatusKind::Success => Self::success(),
                StatusKind::Warning => Self::warning(),
                StatusKind::Error => Self::error(),
            }
        }

        fn muted() -> Color32 {
            Color32::from_rgb(145, 155, 170)
        }

        pub fn new() -> Self {
            Self {
                workspace: Workspace::Pack,

                inputs: Vec::new(),

                archive: None,
                archive_info: None,
                archive_search: String::new(),
                archive_filter: ArchiveFilter::All,
                archive_sort_column: ArchiveSortColumn::Path,
                archive_sort_direction: SortDirection::Ascending,
                selected_archive_entry: None,
                output: None,
                extract_to: None,

                package_name: "CWN Package".to_string(),
                package_version: env!("CARGO_PKG_VERSION").to_string(),
                publisher: "Community Watch Network".to_string(),

                compression_level: 10,

                status: "Ready.".to_string(),
                status_kind: StatusKind::Info,
                busy: false,
                current_operation: None,

                progress_current: 0,
                progress_total: 0,
                progress_path: None,

                task: GuiTask::new(),
            }
        }

        fn set_archive_sort(&mut self, column: ArchiveSortColumn) {
            if self.archive_sort_column == column {
                self.archive_sort_direction = match self.archive_sort_direction {
                    SortDirection::Ascending => SortDirection::Descending,
                    SortDirection::Descending => SortDirection::Ascending,
                };
            } else {
                self.archive_sort_column = column;
                self.archive_sort_direction = SortDirection::Ascending;
            }
        }

        fn archive_sort_label(&self, column: ArchiveSortColumn, label: &str) -> String {
            if self.archive_sort_column != column {
                return format!("{label} ↕");
            }

            let indicator = match self.archive_sort_direction {
                SortDirection::Ascending => "↑",
                SortDirection::Descending => "↓",
            };

            format!("{label} {indicator}")
        }

        fn add_input(&mut self, path: PathBuf) {
            if !self.inputs.contains(&path) {
                self.inputs.push(path);
            }
        }

        fn add_files(&mut self) {
            if self.busy {
                return;
            }

            if let Some(files) = rfd::FileDialog::new().pick_files() {
                for file in files {
                    self.add_input(file);
                }

                self.status = format!("{} input item(s) selected.", self.inputs.len());
            }
        }

        fn add_folder(&mut self) {
            if self.busy {
                return;
            }

            if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                self.add_input(folder);

                self.status = format!("{} input item(s) selected.", self.inputs.len());
            }
        }

        fn choose_output(&mut self) {
            if self.busy {
                return;
            }

            if let Some(mut path) = rfd::FileDialog::new()
                .set_file_name("Package.CWN")
                .save_file()
            {
                if path.extension().is_none() {
                    path.set_extension("CWN");
                }

                if self.package_name.trim().is_empty() || self.package_name == "CWN Package" {
                    if let Some(name) = path.file_stem().and_then(|name| name.to_str()) {
                        self.package_name = name.to_string();
                    }
                }

                self.output = Some(path);
            }
        }

        fn open_archive(&mut self) {
            if self.busy {
                return;
            }

            let Some(path) = rfd::FileDialog::new()
                .add_filter("CWN Container", &["CWN", "cwn"])
                .pick_file()
            else {
                return;
            };

            self.archive = Some(path.clone());
            self.archive_info = None;
            self.archive_search.clear();
            self.archive_filter = ArchiveFilter::All;
            self.archive_sort_column = ArchiveSortColumn::Path;
            self.archive_sort_direction = SortDirection::Ascending;
            self.selected_archive_entry = None;
            self.busy = true;
            self.current_operation = Some("Inspecting".to_string());

            let sender = self.task.sender.clone();

            thread::spawn(move || {
                let _ = sender.send(GuiMessage::Started(
                    "Inspecting CWN container...".to_string(),
                ));

                match cwn_universal_packer::engine::inspect::inspect_container(&path) {
                    Ok(info) => {
                        let _ = sender.send(GuiMessage::Inspected {
                            message: "CWN container loaded.".to_string(),
                            info,
                        });
                    }

                    Err(error) => {
                        let _ = sender.send(GuiMessage::Error(format!(
                            "Failed to inspect container: {error}"
                        )));
                    }
                }
            });
        }

        fn choose_extract_folder(&mut self) {
            if self.busy {
                return;
            }

            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                self.extract_to = Some(path);
            }
        }

        fn start_pack(&mut self) {
            if self.busy {
                return;
            }

            if self.inputs.is_empty() {
                self.status = "Add at least one file or folder.".to_string();
                self.status_kind = StatusKind::Warning;
                return;
            }

            let Some(output) = self.output.clone() else {
                self.status = "Choose an output .CWN file first.".to_string();
                self.status_kind = StatusKind::Warning;
                return;
            };

            let inputs = self.inputs.clone();
            let level = self.compression_level;

            let metadata = match PackageMetadata::new(
                self.package_name.clone(),
                self.package_version.clone(),
                self.publisher.clone(),
            )
            .validate()
            {
                Ok(metadata) => metadata,
                Err(error) => {
                    self.status = format!("Invalid package metadata: {error}");
                    self.status_kind = StatusKind::Warning;
                    return;
                }
            };

            let sender = self.task.sender.clone();

            self.busy = true;
            self.current_operation = Some("Packing".to_string());

            self.progress_current = 0;
            self.progress_total = 0;
            self.progress_path = None;

            thread::spawn(move || {
                let _ = sender.send(GuiMessage::Started(
                    "Packing files into CWN container...".to_string(),
                ));

                match cwn_universal_packer::commands::pack::run_with_metadata_and_progress(
                    inputs,
                    output.clone(),
                    level,
                    metadata,
                    |event| {
                        let _ = sender.send(GuiMessage::Progress(event));
                    },
                ) {
                    Ok(()) => {
                        let _ = sender.send(GuiMessage::Success {
                            message: format!("Created {} successfully.", output.display()),
                            archive: Some(output),
                        });
                    }

                    Err(error) => {
                        let _ = sender.send(GuiMessage::Error(format!("Pack failed: {error}")));
                    }
                }
            });
        }

        fn start_verify(&mut self) {
            if self.busy {
                return;
            }

            let Some(archive) = self.archive.clone() else {
                self.status = "Open a .CWN container first.".to_string();
                self.status_kind = StatusKind::Warning;
                return;
            };

            let sender = self.task.sender.clone();

            self.busy = true;
            self.current_operation = Some("Verification".to_string());

            self.progress_current = 0;
            self.progress_total = 0;
            self.progress_path = None;

            thread::spawn(move || {
                let _ = sender.send(GuiMessage::Started(
                    "Verifying SHA-256 integrity...".to_string(),
                ));

                match cwn_universal_packer::commands::verify::run_with_progress(archive, |event| {
                    let _ = sender.send(GuiMessage::Progress(event));
                }) {
                    Ok(()) => {
                        let _ = sender.send(GuiMessage::Success {
                            message: "Container integrity VERIFIED.".to_string(),
                            archive: None,
                        });
                    }

                    Err(error) => {
                        let _ =
                            sender.send(GuiMessage::Error(format!("Verification failed: {error}")));
                    }
                }
            });
        }

        fn start_test(&mut self) {
            if self.busy {
                return;
            }

            let Some(archive) = self.archive.clone() else {
                self.status = "Open a .CWN container first.".to_string();
                self.status_kind = StatusKind::Warning;
                return;
            };

            let sender = self.task.sender.clone();

            self.busy = true;
            self.current_operation = Some("Structure test".to_string());

            thread::spawn(move || {
                let _ = sender.send(GuiMessage::Started(
                    "Testing CWN container structure...".to_string(),
                ));

                match cwn_universal_packer::commands::test::run(archive) {
                    Ok(()) => {
                        let _ = sender.send(GuiMessage::Success {
                            message: "Container structure VALID.".to_string(),
                            archive: None,
                        });
                    }

                    Err(error) => {
                        let _ = sender
                            .send(GuiMessage::Error(format!("Container test failed: {error}")));
                    }
                }
            });
        }

        fn start_extract(&mut self) {
            if self.busy {
                return;
            }

            let Some(archive) = self.archive.clone() else {
                self.status = "Open a .CWN container first.".to_string();
                self.status_kind = StatusKind::Warning;
                return;
            };

            let Some(output) = self.extract_to.clone() else {
                self.status = "Choose an extraction destination first.".to_string();
                self.status_kind = StatusKind::Warning;
                return;
            };

            let sender = self.task.sender.clone();

            self.busy = true;
            self.current_operation = Some("Extraction".to_string());

            self.progress_current = 0;
            self.progress_total = 0;
            self.progress_path = None;

            thread::spawn(move || {
                let _ = sender.send(GuiMessage::Started(
                    "Extracting CWN container...".to_string(),
                ));

                match cwn_universal_packer::commands::unpack::run_with_progress(
                    archive,
                    output.clone(),
                    |event| {
                        let _ = sender.send(GuiMessage::Progress(event));
                    },
                ) {
                    Ok(()) => {
                        let _ = sender.send(GuiMessage::Success {
                            message: format!("Extracted to {}.", output.display()),
                            archive: None,
                        });
                    }

                    Err(error) => {
                        let _ =
                            sender.send(GuiMessage::Error(format!("Extraction failed: {error}")));
                    }
                }
            });
        }

        fn start_extract_selected(&mut self) {
            if self.busy {
                return;
            }

            let Some(archive) = self.archive.clone() else {
                self.status = "Open a .CWN container first.".to_string();
                self.status_kind = StatusKind::Warning;
                return;
            };

            let Some(output) = self.extract_to.clone() else {
                self.status = "Choose an extraction destination first.".to_string();
                self.status_kind = StatusKind::Warning;
                return;
            };

            let Some(index) = self.selected_archive_entry else {
                self.status = "Select an archive entry first.".to_string();
                self.status_kind = StatusKind::Warning;
                return;
            };

            let Some(info) = self.archive_info.as_ref() else {
                self.status = "Archive information is not available.".to_string();
                self.status_kind = StatusKind::Warning;
                return;
            };

            let Some(entry) = info.entries.get(index) else {
                self.status = "The selected archive entry is no longer available.".to_string();
                self.status_kind = StatusKind::Warning;
                self.selected_archive_entry = None;
                return;
            };

            if entry.is_directory {
                self.status = "Select a file to extract, not a directory.".to_string();
                self.status_kind = StatusKind::Warning;
                return;
            }

            let selected_path = entry.path.clone();
            let display_path = selected_path.clone();

            let sender = self.task.sender.clone();

            self.busy = true;
            self.status_kind = StatusKind::Info;
            self.current_operation = Some("Selected extraction".to_string());

            self.progress_current = 0;
            self.progress_total = 0;
            self.progress_path = None;

            thread::spawn(move || {
                let _ = sender.send(GuiMessage::Started(format!(
                    "Extracting selected entry: {display_path}"
                )));

                match cwn_universal_packer::commands::unpack::run_selected_with_progress(
                    archive,
                    output.clone(),
                    selected_path,
                    |event| {
                        let _ = sender.send(GuiMessage::Progress(event));
                    },
                ) {
                    Ok(()) => {
                        let _ = sender.send(GuiMessage::Success {
                            message: format!("Extracted selected entry to {}.", output.display()),
                            archive: None,
                        });
                    }

                    Err(error) => {
                        let _ = sender.send(GuiMessage::Error(format!(
                            "Selected extraction failed: {error}"
                        )));
                    }
                }
            });
        }

        fn process_messages(&mut self) {
            while let Ok(message) = self.task.receiver.try_recv() {
                match message {
                    GuiMessage::Started(message) => {
                        self.status = message;
                        self.status_kind = StatusKind::Info;
                    }

                    GuiMessage::Progress(event) => match event {
                        cwn_universal_packer::engine::progress::ProgressEvent::Started {
                            operation,
                            total_items,
                        } => {
                            self.current_operation = Some(operation);
                            self.progress_current = 0;
                            self.progress_total = total_items.unwrap_or(0);
                            self.progress_path = None;
                        }

                        cwn_universal_packer::engine::progress::ProgressEvent::Item {
                            current,
                            total,
                            path,
                        } => {
                            self.progress_current = current;
                            self.progress_total = total;
                            self.progress_path = Some(path);
                        }

                        cwn_universal_packer::engine::progress::ProgressEvent::Bytes {
                            processed: _,
                            total: _,
                        } => {}

                        cwn_universal_packer::engine::progress::ProgressEvent::Message(message) => {
                            self.status = message;
                        }

                        cwn_universal_packer::engine::progress::ProgressEvent::Finished => {}
                    },

                    GuiMessage::Success { message, archive } => {
                        self.status = message;
                        self.status_kind = StatusKind::Success;

                        if let Some(archive) = archive {
                            self.archive = Some(archive.clone());

                            match cwn_universal_packer::engine::inspect::inspect_container(&archive)
                            {
                                Ok(info) => {
                                    self.archive_info = Some(info);
                                }

                                Err(error) => {
                                    self.archive_info = None;

                                    self.status =
                                        format!("Created archive, but inspection failed: {error}");
                                }
                            }
                        }

                        self.busy = false;
                        self.current_operation = None;
                        self.progress_current = 0;
                        self.progress_total = 0;
                        self.progress_path = None;
                    }

                    GuiMessage::Inspected { message, info } => {
                        self.status = message;
                        self.status_kind = StatusKind::Success;
                        self.archive = Some(info.path.clone());
                        self.archive_info = Some(info);

                        self.busy = false;
                        self.current_operation = None;
                    }

                    GuiMessage::Error(message) => {
                        self.status = message;
                        self.status_kind = StatusKind::Error;

                        self.busy = false;
                        self.current_operation = None;
                        self.progress_current = 0;
                        self.progress_total = 0;
                        self.progress_path = None;
                    }
                }
            }
        }

        fn handle_drag_and_drop(&mut self, ctx: &egui::Context) {
            if self.busy {
                return;
            }

            let dropped_files = ctx.input(|input| input.raw.dropped_files.clone());

            if dropped_files.is_empty() {
                return;
            }

            for dropped in dropped_files {
                if let Some(path) = dropped.path {
                    self.add_input(path);
                }
            }

            self.status = format!("{} input item(s) selected.", self.inputs.len());
        }
    }

    fn show_pack_workspace(&mut self, ui: &mut egui::Ui) {
        ui.heading(egui::RichText::new("Create Package").size(22.0).strong());

        ui.label(
                egui::RichText::new(
                    "Build a secure CWN container from files, folders, binaries, scripts and other content."
                )
                .color(Self::muted()),
            );

        ui.add_space(14.0);
        ui.separator();
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            ui.add_enabled_ui(!self.busy, |ui| {
                if ui.button("Add Files").clicked() {
                    self.add_files();
                }

                if ui.button("Add Folder").clicked() {
                    self.add_folder();
                }

                if ui.button("Clear").clicked() {
                    self.inputs.clear();
                    self.status = "Input list cleared.".to_string();
                }
            });
        });

        ui.add_space(10.0);

        egui::Frame::group(ui.style())
            .corner_radius(CornerRadius::same(8))
            .show(ui, |ui| {
                ui.set_min_height(190.0);

                ui.label(
                    egui::RichText::new("PACKAGE CONTENTS")
                        .small()
                        .strong()
                        .color(Self::accent()),
                );

                ui.add_space(6.0);

                ui.label(
                    egui::RichText::new("Drop files and folders here, or use the buttons above.")
                        .color(Self::muted()),
                );

                ui.add_space(8.0);

                egui::ScrollArea::vertical()
                    .max_height(170.0)
                    .show(ui, |ui| {
                        if self.inputs.is_empty() {
                            ui.centered_and_justified(|ui| {
                                ui.label(
                                    egui::RichText::new("No files or folders selected.")
                                        .color(Self::muted()),
                                );
                            });
                        } else {
                            let mut remove = None;

                            for (index, input) in self.inputs.iter().enumerate() {
                                ui.horizontal(|ui| {
                                    ui.label("▣");

                                    ui.label(
                                        egui::RichText::new(input.display().to_string())
                                            .monospace(),
                                    );

                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            if !self.busy && ui.small_button("Remove").clicked() {
                                                remove = Some(index);
                                            }
                                        },
                                    );
                                });

                                ui.separator();
                            }

                            if let Some(index) = remove {
                                self.inputs.remove(index);
                            }
                        }
                    });
            });

        ui.add_space(16.0);

        ui.label(
            egui::RichText::new("PACKAGE METADATA")
                .small()
                .strong()
                .color(Self::accent()),
        );

        ui.add_space(6.0);

        egui::Frame::group(ui.style())
            .corner_radius(CornerRadius::same(8))
            .show(ui, |ui| {
                ui.add_enabled_ui(!self.busy, |ui| {
                    egui::Grid::new("package_metadata")
                        .num_columns(2)
                        .spacing([18.0, 8.0])
                        .show(ui, |ui| {
                            ui.label("Package name");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.package_name)
                                    .desired_width(320.0),
                            );
                            ui.end_row();

                            ui.label("Package version");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.package_version)
                                    .desired_width(320.0),
                            );
                            ui.end_row();

                            ui.label("Publisher");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.publisher)
                                    .desired_width(320.0),
                            );
                            ui.end_row();

                            ui.label("Producer");
                            ui.label(
                                egui::RichText::new(format!(
                                    "CWN Universal Packer {}",
                                    env!("CARGO_PKG_VERSION")
                                ))
                                .monospace()
                                .color(Self::muted()),
                            );
                            ui.end_row();
                        });
                });
            });

        ui.add_space(16.0);

        ui.label(
            egui::RichText::new("COMPRESSION")
                .small()
                .strong()
                .color(Self::accent()),
        );

        ui.add_space(6.0);

        ui.add_enabled_ui(!self.busy, |ui| {
            ui.horizontal(|ui| {
                ui.label("Zstandard level");

                ui.add(egui::Slider::new(&mut self.compression_level, 1..=22).show_value(true));
            });
        });

        ui.add_space(14.0);

        ui.label(
            egui::RichText::new("OUTPUT")
                .small()
                .strong()
                .color(Self::accent()),
        );

        ui.add_space(6.0);

        ui.add_enabled_ui(!self.busy, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Choose Output").clicked() {
                    self.choose_output();
                }

                match &self.output {
                    Some(output) => {
                        ui.label(egui::RichText::new(output.display().to_string()).monospace());
                    }

                    None => {
                        ui.label(egui::RichText::new("No output selected.").color(Self::muted()));
                    }
                }
            });
        });

        ui.add_space(18.0);

        if ui
            .add_enabled(
                !self.busy,
                egui::Button::new(egui::RichText::new("PACK TO .CWN").strong())
                    .min_size(egui::vec2(210.0, 42.0)),
            )
            .clicked()
        {
            self.start_pack();
        }
    }

    fn show_archive_workspace(&mut self, ui: &mut egui::Ui) {
        ui.heading(egui::RichText::new("Archive Browser").size(22.0).strong());

        ui.label(
            egui::RichText::new("Inspect, validate, verify and extract CWN containers.")
                .color(Self::muted()),
        );

        ui.add_space(14.0);
        ui.separator();
        ui.add_space(10.0);

        ui.add_enabled_ui(!self.busy, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Open CWN").clicked() {
                    self.open_archive();
                }

                match &self.archive {
                    Some(archive) => {
                        ui.label(egui::RichText::new(archive.display().to_string()).monospace());
                    }

                    None => {
                        ui.label(
                            egui::RichText::new("No CWN container selected.").color(Self::muted()),
                        );
                    }
                }
            });
        });

        ui.add_space(12.0);

        ui.add_enabled_ui(!self.busy && self.archive.is_some(), |ui| {
            ui.horizontal(|ui| {
                if ui.button("Test Structure").clicked() {
                    self.start_test();
                }

                if ui.button("Verify SHA-256").clicked() {
                    self.start_verify();
                }

                if ui.button("Choose Extract Folder").clicked() {
                    self.choose_extract_folder();
                }

                if ui.button("Extract All").clicked() {
                    self.start_extract();
                }

                let can_extract_selected = self.extract_to.is_some()
                    && self
                        .selected_archive_entry
                        .and_then(|index| {
                            self.archive_info
                                .as_ref()
                                .and_then(|info| info.entries.get(index))
                        })
                        .is_some_and(|entry| !entry.is_directory);

                if ui
                    .add_enabled(can_extract_selected, egui::Button::new("Extract Selected"))
                    .clicked()
                {
                    self.start_extract_selected();
                }
            });
        });

        if let Some(folder) = &self.extract_to {
            ui.add_space(6.0);

            ui.label(
                egui::RichText::new(format!("Extraction destination: {}", folder.display()))
                    .small()
                    .color(Self::muted()),
            );
        }

        let Some(info) = &self.archive_info else {
            ui.add_space(24.0);

            egui::Frame::group(ui.style())
                .corner_radius(CornerRadius::same(8))
                .show(ui, |ui| {
                    ui.set_min_height(150.0);

                    ui.centered_and_justified(|ui| {
                        ui.vertical_centered(|ui| {
                            ui.label(egui::RichText::new("◫").size(30.0).color(Self::accent()));

                            ui.label(
                                egui::RichText::new("Open a .CWN container to inspect it.")
                                    .color(Self::muted()),
                            );
                        });
                    });
                });

            return;
        };

        ui.add_space(18.0);

        ui.label(
            egui::RichText::new("CONTAINER OVERVIEW")
                .small()
                .strong()
                .color(Self::accent()),
        );

        ui.add_space(8.0);

        egui::Grid::new("container_info")
            .num_columns(2)
            .striped(true)
            .spacing([20.0, 8.0])
            .show(ui, |ui| {
                ui.label("Package");
                ui.strong(&info.package_name);
                ui.end_row();

                ui.label("Package Version");
                ui.label(&info.package_version);
                ui.end_row();

                ui.label("Publisher");
                ui.label(&info.publisher);
                ui.end_row();

                ui.label("Producer");
                ui.label(&info.producer);
                ui.end_row();

                ui.label("Format");
                ui.label(format!("CWN v{}", info.format_version));
                ui.end_row();

                ui.label("Files");
                ui.label(info.files.to_string());
                ui.end_row();

                ui.label("Directories");
                ui.label(info.directories.to_string());
                ui.end_row();

                ui.label("Original Size");
                ui.label(human_size(info.original_size));
                ui.end_row();

                ui.label("Payload Size");
                ui.label(human_size(info.payload_size));
                ui.end_row();

                ui.label("Container Size");
                ui.label(human_size(info.container_size));
                ui.end_row();

                ui.label("Payload Saved");

                let saved = info.original_size.saturating_sub(info.payload_size);

                let saved_percent = if info.original_size == 0 {
                    0.0
                } else {
                    saved as f64 / info.original_size as f64 * 100.0
                };

                ui.label(format!("{} ({:.2}%)", human_size(saved), saved_percent));
                ui.end_row();

                ui.label("Zstandard Files");
                ui.label(info.zstd_files.to_string());
                ui.end_row();

                ui.label("Stored Files");
                ui.label(info.stored_files.to_string());
                ui.end_row();
            });

        ui.add_space(18.0);

        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("ARCHIVE CONTENTS")
                    .small()
                    .strong()
                    .color(Self::accent()),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut self.archive_search)
                        .hint_text("Search files...")
                        .desired_width(240.0),
                );
            });
        });

        ui.add_space(8.0);

        ui.horizontal_wrapped(|ui| {
            ui.label(
                egui::RichText::new("FILTER")
                    .small()
                    .strong()
                    .color(Self::muted()),
            );

            if ui
                .selectable_label(self.archive_filter == ArchiveFilter::All, "All")
                .clicked()
            {
                self.archive_filter = ArchiveFilter::All;
            }

            if ui
                .selectable_label(self.archive_filter == ArchiveFilter::Files, "Files")
                .clicked()
            {
                self.archive_filter = ArchiveFilter::Files;
            }

            if ui
                .selectable_label(
                    self.archive_filter == ArchiveFilter::Directories,
                    "Directories",
                )
                .clicked()
            {
                self.archive_filter = ArchiveFilter::Directories;
            }

            if ui
                .selectable_label(self.archive_filter == ArchiveFilter::Zstd, "Zstd")
                .clicked()
            {
                self.archive_filter = ArchiveFilter::Zstd;
            }

            if ui
                .selectable_label(self.archive_filter == ArchiveFilter::Stored, "Stored")
                .clicked()
            {
                self.archive_filter = ArchiveFilter::Stored;
            }
        });

        ui.add_space(8.0);

        if let Some(index) = self.selected_archive_entry {
            if let Some(entry) = info.entries.get(index) {
                egui::Frame::group(ui.style())
                    .corner_radius(CornerRadius::same(8))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new("SELECTED ENTRY")
                                .small()
                                .strong()
                                .color(Self::accent()),
                        );

                        ui.add_space(6.0);

                        egui::Grid::new("selected_entry")
                            .num_columns(2)
                            .spacing([18.0, 6.0])
                            .show(ui, |ui| {
                                ui.label("Path");
                                ui.label(egui::RichText::new(&entry.path).monospace());
                                ui.end_row();

                                ui.label("Type");
                                ui.label(if entry.is_directory {
                                    "Directory"
                                } else {
                                    &entry.file_type
                                });
                                ui.end_row();

                                ui.label("Original");
                                ui.label(if entry.is_directory {
                                    "-".to_string()
                                } else {
                                    human_size(entry.original_size)
                                });
                                ui.end_row();

                                ui.label("Stored");
                                ui.label(if entry.is_directory {
                                    "-".to_string()
                                } else {
                                    human_size(entry.packed_size)
                                });
                                ui.end_row();

                                ui.label("Compression");
                                ui.label(if entry.is_directory {
                                    "-".to_string()
                                } else {
                                    entry.compression.to_uppercase()
                                });
                                ui.end_row();

                                ui.label("Space Saved");

                                let saved_text = if entry.is_directory || entry.original_size == 0 {
                                    "-".to_string()
                                } else {
                                    let saved =
                                        entry.original_size.saturating_sub(entry.packed_size);

                                    let percent = saved as f64 / entry.original_size as f64 * 100.0;

                                    format!("{} ({:.2}%)", human_size(saved), percent)
                                };

                                ui.label(saved_text);
                                ui.end_row();
                            });
                    });

                ui.add_space(10.0);
            }
        }

        let query = self.archive_search.trim().to_ascii_lowercase();

        let entry_matches = |entry: &cwn_universal_packer::engine::types::ContainerEntryInfo| {
            let search_matches = query.is_empty()
                || entry.path.to_ascii_lowercase().contains(&query)
                || entry.file_type.to_ascii_lowercase().contains(&query)
                || entry.compression.to_ascii_lowercase().contains(&query);

            let filter_matches = match self.archive_filter {
                ArchiveFilter::All => true,
                ArchiveFilter::Files => !entry.is_directory,
                ArchiveFilter::Directories => entry.is_directory,
                ArchiveFilter::Zstd => !entry.is_directory && entry.compression == "zstd",
                ArchiveFilter::Stored => !entry.is_directory && entry.compression == "none",
            };

            search_matches && filter_matches
        };

        let mut visible_indices: Vec<usize> = info
            .entries
            .iter()
            .enumerate()
            .filter_map(|(index, entry)| entry_matches(entry).then_some(index))
            .collect();

        visible_indices.sort_by(|left_index, right_index| {
            let left = &info.entries[*left_index];
            let right = &info.entries[*right_index];

            let ordering = match self.archive_sort_column {
                ArchiveSortColumn::Type => {
                    let left_type = if left.is_directory {
                        "directory"
                    } else {
                        left.file_type.as_str()
                    };

                    let right_type = if right.is_directory {
                        "directory"
                    } else {
                        right.file_type.as_str()
                    };

                    left_type
                        .to_ascii_lowercase()
                        .cmp(&right_type.to_ascii_lowercase())
                }

                ArchiveSortColumn::OriginalSize => left.original_size.cmp(&right.original_size),

                ArchiveSortColumn::PackedSize => left.packed_size.cmp(&right.packed_size),

                ArchiveSortColumn::Compression => left
                    .compression
                    .to_ascii_lowercase()
                    .cmp(&right.compression.to_ascii_lowercase()),

                ArchiveSortColumn::Path => left
                    .path
                    .to_ascii_lowercase()
                    .cmp(&right.path.to_ascii_lowercase()),
            };

            let ordering = ordering.then_with(|| {
                left.path
                    .to_ascii_lowercase()
                    .cmp(&right.path.to_ascii_lowercase())
            });

            match self.archive_sort_direction {
                SortDirection::Ascending => ordering,
                SortDirection::Descending => ordering.reverse(),
            }
        });

        let visible_entries = visible_indices.len();

        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(format!(
                    "Showing {} of {} entries",
                    visible_entries,
                    info.entries.len()
                ))
                .small()
                .color(Self::muted()),
            );

            if visible_entries == 0 {
                ui.label(
                    egui::RichText::new("No matches")
                        .small()
                        .color(Self::warning()),
                );
            }
        });

        ui.add_space(6.0);

        let mut requested_sort = None;

        egui::ScrollArea::both().max_height(360.0).show(ui, |ui| {
            egui::Grid::new("archive_entries")
                .striped(true)
                .min_col_width(90.0)
                .spacing([16.0, 7.0])
                .show(ui, |ui| {
                    if ui
                        .button(self.archive_sort_label(ArchiveSortColumn::Type, "Type"))
                        .clicked()
                    {
                        requested_sort = Some(ArchiveSortColumn::Type);
                    }

                    if ui
                        .button(
                            self.archive_sort_label(ArchiveSortColumn::OriginalSize, "Original"),
                        )
                        .clicked()
                    {
                        requested_sort = Some(ArchiveSortColumn::OriginalSize);
                    }

                    if ui
                        .button(self.archive_sort_label(ArchiveSortColumn::PackedSize, "Stored"))
                        .clicked()
                    {
                        requested_sort = Some(ArchiveSortColumn::PackedSize);
                    }

                    if ui
                        .button(self.archive_sort_label(ArchiveSortColumn::Compression, "Method"))
                        .clicked()
                    {
                        requested_sort = Some(ArchiveSortColumn::Compression);
                    }

                    if ui
                        .button(self.archive_sort_label(ArchiveSortColumn::Path, "Path"))
                        .clicked()
                    {
                        requested_sort = Some(ArchiveSortColumn::Path);
                    }

                    ui.end_row();

                    for index in visible_indices.iter().copied() {
                        let entry = &info.entries[index];

                        let selected = self.selected_archive_entry == Some(index);

                        let response = ui.selectable_label(
                            selected,
                            if entry.is_directory {
                                "Directory"
                            } else {
                                &entry.file_type
                            },
                        );

                        if response.clicked() {
                            self.selected_archive_entry = Some(index);
                        }

                        if entry.is_directory {
                            ui.label("-");
                            ui.label("-");
                            ui.label("-");
                        } else {
                            ui.label(human_size(entry.original_size));
                            ui.label(human_size(entry.packed_size));
                            ui.label(entry.compression.to_uppercase());
                        }

                        let path_response = ui.selectable_label(
                            selected,
                            egui::RichText::new(&entry.path).monospace(),
                        );

                        if path_response.clicked() {
                            self.selected_archive_entry = Some(index);
                        }

                        ui.end_row();
                    }
                });
        });

        if let Some(column) = requested_sort {
            self.set_archive_sort(column);
        }
    }

    impl eframe::App for CwnPackerApp {
        fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
            Self::apply_cwn_theme(ctx);

            self.process_messages();
            self.handle_drag_and_drop(ctx);

            if self.busy {
                ctx.request_repaint_after(Duration::from_millis(100));
            }

            egui::TopBottomPanel::top("header")
                .exact_height(82.0)
                .show(ctx, |ui| {
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(
                                egui::RichText::new("CWN")
                                    .size(14.0)
                                    .strong()
                                    .color(Self::accent()),
                            );

                            ui.label(egui::RichText::new("UNIVERSAL PACKER").size(24.0).strong());

                            ui.label(
                                egui::RichText::new("Secure universal .CWN container workspace")
                                    .color(Self::muted()),
                            );
                        });

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                egui::RichText::new(env!("CARGO_PKG_VERSION"))
                                    .monospace()
                                    .color(Self::accent()),
                            );

                            ui.label(
                                egui::RichText::new("COMMUNITY WATCH NETWORK")
                                    .small()
                                    .color(Self::muted()),
                            );
                        });
                    });
                });

            egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
                ui.add_space(6.0);
                ui.separator();

                ui.horizontal(|ui| {
                    if self.busy {
                        ui.spinner();

                        if let Some(operation) = &self.current_operation {
                            ui.strong(operation);
                        }
                    } else {
                        ui.label(
                            egui::RichText::new("● READY")
                                .strong()
                                .color(Self::success()),
                        );
                    }

                    ui.separator();
                    ui.label(egui::RichText::new(&self.status).color(self.status_color()));
                });

                if self.progress_total > 0 {
                    let fraction = self.progress_current as f32 / self.progress_total as f32;

                    ui.add_space(4.0);

                    ui.add(
                        egui::ProgressBar::new(fraction)
                            .show_percentage()
                            .text(format!(
                                "{} / {} files",
                                self.progress_current, self.progress_total
                            )),
                    );

                    if let Some(path) = &self.progress_path {
                        ui.small(path);
                    }
                }

                ui.add_space(6.0);
            });

            egui::SidePanel::left("navigation")
                .exact_width(190.0)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.add_space(14.0);

                    ui.label(
                        egui::RichText::new("WORKSPACE")
                            .small()
                            .strong()
                            .color(Self::muted()),
                    );

                    ui.add_space(8.0);

                    if ui
                        .selectable_label(self.workspace == Workspace::Pack, "▣  Create Package")
                        .clicked()
                    {
                        self.workspace = Workspace::Pack;
                    }

                    if ui
                        .selectable_label(
                            self.workspace == Workspace::Archive,
                            "◫  Archive Browser",
                        )
                        .clicked()
                    {
                        self.workspace = Workspace::Archive;
                    }

                    ui.add_space(18.0);
                    ui.separator();
                    ui.add_space(12.0);

                    ui.label(
                        egui::RichText::new("ENGINE")
                            .small()
                            .strong()
                            .color(Self::muted()),
                    );

                    ui.add_space(6.0);

                    ui.label("CWN Container v1");
                    ui.label("Zstandard");
                    ui.label("SHA-256");

                    ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                        ui.label(
                            egui::RichText::new("CWN Universal Packer")
                                .small()
                                .color(Self::muted()),
                        );
                    });
                });

            egui::CentralPanel::default().show(ctx, |ui| {
                ui.add_space(14.0);

                match self.workspace {
                    Workspace::Pack => {
                        self.show_pack_workspace(ui);
                    }

                    Workspace::Archive => {
                        self.show_archive_workspace(ui);
                    }
                }

                ui.add_space(24.0);
                ui.separator();

                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(format!(
                            "CWN Universal Packer {}",
                            env!("CARGO_PKG_VERSION")
                        ))
                        .small()
                        .color(Self::muted()),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new("Community Watch Network")
                                .small()
                                .color(Self::muted()),
                        );
                    });
                });
            });
        }
    }

    pub fn run() -> eframe::Result<()> {
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_title("CWN Universal Packer")
                .with_inner_size([920.0, 720.0])
                .with_min_inner_size([720.0, 560.0]),

            ..Default::default()
        };

        eframe::run_native(
            "CWN Universal Packer",
            options,
            Box::new(|_| Ok(Box::new(CwnPackerApp::new()))),
        )
    }
}

#[cfg(not(target_os = "android"))]
fn main() -> eframe::Result<()> {
    desktop::run()
}
