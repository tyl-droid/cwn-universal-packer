use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};

#[derive(Debug)]
pub enum GuiMessage {
    Started(String),

    Success {
        message: String,
        archive: Option<PathBuf>,
    },

    Error(String),
}

pub struct GuiTask {
    pub sender: Sender<GuiMessage>,
    pub receiver: Receiver<GuiMessage>,
}

impl GuiTask {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();

        Self { sender, receiver }
    }
}

impl Default for GuiTask {
    fn default() -> Self {
        Self::new()
    }
}
