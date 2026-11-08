// 計測データ
#[derive(Clone, Copy, Debug)]
pub struct Sample {
    pub time: f64,
    pub voltage_ratio: f64,
}

// GUIから計測スレッドへの命令
#[derive(Debug)]
pub enum Command {
    Start(RecordingMode),
    Stop,
    Exit,
}

// 計測スレッドからGUIへの通知
#[derive(Debug)]
pub enum UiEvent {
    Started,
    Stopped,
    Progress {
        count: usize,
        ratio: f64,
        new_samples: Vec<Sample>,
    },
    Saved(String),
    Error(String),
}

// 計測モード
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RecordingMode {
    Raw,
    Round,
    Truncate,
}
