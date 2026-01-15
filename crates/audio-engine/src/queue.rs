use std::collections::VecDeque;

use crate::types::TrackInfo;

#[derive(Debug, Clone)]
pub struct QueueItem {
    pub track: TrackInfo,
}

#[derive(Debug, Default)]
pub struct PlaybackQueue {
    items: VecDeque<QueueItem>,
    current_index: Option<usize>,
}

impl PlaybackQueue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn play_now(&mut self, track: TrackInfo) {
        self.items.clear();
        self.items.push_back(QueueItem { track });
        self.current_index = Some(0);
    }

    pub fn add_to_queue(&mut self, track: TrackInfo) {
        self.items.push_back(QueueItem { track });

        if self.current_index.is_none() {
            self.current_index = Some(0);
        }
    }

    pub fn next(&mut self) -> Option<&QueueItem> {
        let len = self.items.len();
        let current_index = self.current_index?;

        let next_index = current_index + 1;
        if next_index >= len {
            self.current_index = None;
            return None;
        }

        self.current_index = Some(next_index);
        self.items.get(next_index)
    }

    pub fn previous(&mut self, position_ms: u64) -> PreviousAction {
        let current_index = match self.current_index {
            Some(index) => index,
            None => return PreviousAction::AtStart,
        };

        if position_ms > 3_000 {
            return PreviousAction::RestartCurrent;
        }

        if current_index == 0 {
            return PreviousAction::AtStart;
        }

        let previous_index = current_index - 1;
        self.current_index = Some(previous_index);
        PreviousAction::MoveToPrevious(previous_index)
    }

    pub fn current(&self) -> Option<&QueueItem> {
        let index = self.current_index?;
        self.items.get(index)
    }

    pub fn clear(&mut self) {
        self.items.clear();
        self.current_index = None;
    }

    pub fn items(&self) -> &VecDeque<QueueItem> {
        &self.items
    }

    pub fn current_index(&self) -> Option<usize> {
        self.current_index
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviousAction {
    RestartCurrent,
    MoveToPrevious(usize),
    AtStart,
}
