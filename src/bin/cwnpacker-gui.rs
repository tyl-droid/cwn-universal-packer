#[cfg(target_os = "android")]
fn main() {
    println!(
        "CWN Universal Packer GUI is a desktop application.\n\
         The CLI remains fully supported in Termux."
    );
}

#[cfg(not(target_os = "android"))]
mod desktop {
    use eframe::egui;
    use std::path::PathBuf;

    #[derive(Default)]
    pub struct CwnPackerApp {
        inputs: Vec<PathBuf>,
        archive: Option<PathBuf>,
        output: Option<PathBuf>,
        extract_to: Option<PathBuf>,

        compression_level: i32,

        status: String,
    }

    impl CwnPackerApp {
        pub fn new() -> Self {
            Self {
                compression_level: 10,
                status: "Ready.".to_string(),
                ..Default::default()
            }
        }

        fn add_files(&mut self) {
            if let Some(files) = rfd::FileDialog::new().pick_files() {
                for file in files {
                    if !self.inputs.contains(&file) {
                        self.inputs.push(file);
                    }
                }

                self.status = format!("{} input item(s) selected.", self.inputs.len());
            }
        }

        fn add_folder(&mut self) {
            if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                if !self.inputs.contains(&folder) {
                    self.inputs.push(folder);
                }

                self.status = format!("{} input item(s) selected.", self.inputs.len());
            }
        }

        fn choose_output(&mut self) {
            if let Some(path) = rfd::FileDialog::new()
                .set_file_name("Package.CWN")
                .save_file()
            {
                self.output = Some(path);
            }
        }

        fn open_archive(&mut self) {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("CWN Container", &["CWN", "cwn"])
                .pick_file()
            {
                self.archive = Some(path);
                self.status = "CWN container selected.".to_string();
            }
        }

        fn choose_extract_folder(&mut self) {
            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                self.extract_to = Some(path);
            }
        }

        fn pack(&mut self) {
            let Some(output) = self.output.clone() else {
                self.status = "Choose an output .CWN file first.".to_string();
                return;
            };

            if self.inputs.is_empty() {
                self.status = "Add at least one file or folder.".to_string();
                return;
            }

            match cwn_universal_packer::commands::pack::run(
                self.inputs.clone(),
                output.clone(),
                self.compression_level,
            ) {
                Ok(()) => {
                    self.status = format!("Created {} successfully.", output.display());

                    self.archive = Some(output);
                }

                Err(error) => {
                    self.status = format!("Pack failed: {error}");
                }
            }
        }

        fn verify(&mut self) {
            let Some(archive) = self.archive.clone() else {
                self.status = "Open a .CWN container first.".to_string();
                return;
            };

            match cwn_universal_packer::commands::verify::run(archive) {
                Ok(()) => {
                    self.status = "Container integrity VERIFIED.".to_string();
                }

                Err(error) => {
                    self.status = format!("Verification failed: {error}");
                }
            }
        }

        fn test_container(&mut self) {
            let Some(archive) = self.archive.clone() else {
                self.status = "Open a .CWN container first.".to_string();
                return;
            };

            match cwn_universal_packer::commands::test::run(archive) {
                Ok(()) => {
                    self.status = "Container structure VALID.".to_string();
                }

                Err(error) => {
                    self.status = format!("Container test failed: {error}");
                }
            }
        }

        fn extract(&mut self) {
            let Some(archive) = self.archive.clone() else {
                self.status = "Open a .CWN container first.".to_string();
                return;
            };

            let Some(output) = self.extract_to.clone() else {
                self.status = "Choose an extraction destination first.".to_string();
                return;
            };

            match cwn_universal_packer::commands::unpack::run(archive, output.clone()) {
                Ok(()) => {
                    self.status = format!("Extracted to {}.", output.display());
                }

                Err(error) => {
                    self.status = format!("Extraction failed: {error}");
                }
            }
        }
    }

    impl eframe::App for CwnPackerApp {
        fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
            egui::TopBottomPanel::top("header").show(ctx, |ui| {
                ui.add_space(10.0);

                ui.heading("CWN Universal Packer");

                ui.label("Community Watch Network • Universal .CWN Container");

                ui.add_space(10.0);
            });

            egui::CentralPanel::default().show(ctx, |ui| {
                ui.heading("Create .CWN Package");
                ui.separator();

                ui.horizontal(|ui| {
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

                ui.add_space(8.0);

                egui::ScrollArea::vertical()
                    .max_height(180.0)
                    .show(ui, |ui| {
                        if self.inputs.is_empty() {
                            ui.label("No files or folders selected.");
                        } else {
                            for input in &self.inputs {
                                ui.label(input.display().to_string());
                            }
                        }
                    });

                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.label("Compression level:");

                    ui.add(egui::Slider::new(&mut self.compression_level, 1..=22).text("Zstd"));
                });

                ui.horizontal(|ui| {
                    if ui.button("Choose Output").clicked() {
                        self.choose_output();
                    }

                    if let Some(output) = &self.output {
                        ui.label(output.display().to_string());
                    }
                });

                ui.add_space(8.0);

                if ui.button("PACK TO .CWN").clicked() {
                    self.pack();
                }

                ui.add_space(20.0);

                ui.heading("Open Existing .CWN");
                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("Open CWN").clicked() {
                        self.open_archive();
                    }

                    if let Some(archive) = &self.archive {
                        ui.label(archive.display().to_string());
                    }
                });

                ui.horizontal(|ui| {
                    if ui.button("Test Structure").clicked() {
                        self.test_container();
                    }

                    if ui.button("Verify SHA-256").clicked() {
                        self.verify();
                    }
                });

                ui.horizontal(|ui| {
                    if ui.button("Choose Extract Folder").clicked() {
                        self.choose_extract_folder();
                    }

                    if ui.button("Extract").clicked() {
                        self.extract();
                    }
                });

                if let Some(folder) = &self.extract_to {
                    ui.label(format!("Extraction destination: {}", folder.display()));
                }

                ui.add_space(20.0);

                ui.separator();
                ui.heading("Status");
                ui.label(&self.status);
            });
        }
    }

    pub fn run() -> eframe::Result<()> {
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_title("CWN Universal Packer")
                .with_inner_size([900.0, 700.0])
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
