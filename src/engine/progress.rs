#[derive(Debug, Clone)]
pub enum ProgressEvent {
    Started {
        operation: String,
        total_items: Option<usize>,
    },

    Item {
        current: usize,
        total: usize,
        path: String,
    },

    Bytes {
        processed: u64,
        total: u64,
    },

    Message(String),

    Finished,
}
