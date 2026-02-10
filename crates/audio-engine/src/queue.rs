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

    /// Insert tracks immediately after the current track (Queue Next)
    /// If queue is empty or no current track, behaves like add_to_queue
    pub fn insert_next(&mut self, tracks: Vec<TrackInfo>) {
        if tracks.is_empty() {
            return;
        }

        match self.current_index {
            Some(idx) => {
                // Insert after current track, preserving order
                let insert_pos = idx + 1;
                for (i, track) in tracks.into_iter().enumerate() {
                    self.items.insert(insert_pos + i, QueueItem { track });
                }
            }
            None => {
                // Queue empty or nothing playing - append and set current to first
                for track in tracks {
                    self.items.push_back(QueueItem { track });
                }
                self.current_index = Some(0);
            }
        }
    }

    /// Replace entire queue with new tracks and start playing from given index
    pub fn set_and_play(&mut self, tracks: Vec<TrackInfo>, start_index: usize) {
        self.items.clear();
        for track in tracks {
            self.items.push_back(QueueItem { track });
        }
        if !self.items.is_empty() {
            self.current_index = Some(start_index.min(self.items.len() - 1));
        } else {
            self.current_index = None;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_track(id: i64) -> TrackInfo {
        TrackInfo {
            id,
            path: format!("/test/{}.flac", id),
            title: Some(format!("Track {}", id)),
            artist: Some("Test Artist".to_string()),
            album: Some("Test Album".to_string()),
            duration_ms: Some(180000),
            sample_rate: Some(44100),
            channels: Some(2),
            bit_depth: Some(16),
            codec: Some("flac".to_string()),
            container: Some("flac".to_string()),
            dsd_rate_hz: None,
            dsd_channels: None,
        }
    }

    #[test]
    fn test_insert_next_with_current_index() {
        let mut queue = PlaybackQueue::new();
        queue.add_to_queue(make_track(1));
        queue.add_to_queue(make_track(2));
        queue.add_to_queue(make_track(3));

        queue.insert_next(vec![make_track(10), make_track(11)]);

        let ids: Vec<i64> = queue.items().iter().map(|i| i.track.id).collect();
        assert_eq!(ids, vec![1, 10, 11, 2, 3]);
        assert_eq!(queue.current_index(), Some(0));
    }

    #[test]
    fn test_insert_next_empty_queue() {
        let mut queue = PlaybackQueue::new();
        queue.insert_next(vec![make_track(1), make_track(2)]);

        let ids: Vec<i64> = queue.items().iter().map(|i| i.track.id).collect();
        assert_eq!(ids, vec![1, 2]);
        assert_eq!(queue.current_index(), Some(0));
    }

    #[test]
    fn test_insert_next_empty_tracks() {
        let mut queue = PlaybackQueue::new();
        queue.add_to_queue(make_track(1));
        queue.insert_next(vec![]);

        assert_eq!(queue.items().len(), 1);
    }

    #[test]
    fn test_insert_next_mid_queue() {
        let mut queue = PlaybackQueue::new();
        queue.add_to_queue(make_track(1));
        queue.add_to_queue(make_track(2));
        queue.add_to_queue(make_track(3));
        queue.next();

        queue.insert_next(vec![make_track(20)]);

        let ids: Vec<i64> = queue.items().iter().map(|i| i.track.id).collect();
        assert_eq!(ids, vec![1, 2, 20, 3]);
        assert_eq!(queue.current_index(), Some(1));
    }

    #[test]
    fn test_play_now_replaces_existing_queue_and_resets_current_index() {
        let mut queue = PlaybackQueue::new();
        queue.add_to_queue(make_track(1));
        queue.add_to_queue(make_track(2));
        let _ = queue.next();
        assert_eq!(queue.current_index(), Some(1));

        queue.play_now(make_track(99));

        let ids: Vec<i64> = queue.items().iter().map(|i| i.track.id).collect();
        assert_eq!(ids, vec![99]);
        assert_eq!(queue.current_index(), Some(0));
        assert_eq!(queue.current().map(|item| item.track.id), Some(99));
    }

    #[test]
    fn test_set_and_play_clamps_start_index_and_uses_given_order() {
        let mut queue = PlaybackQueue::new();

        queue.set_and_play(vec![make_track(3), make_track(1), make_track(2)], 99);

        let ids: Vec<i64> = queue.items().iter().map(|i| i.track.id).collect();
        assert_eq!(ids, vec![3, 1, 2]);
        assert_eq!(queue.current_index(), Some(2));
        assert_eq!(queue.current().map(|item| item.track.id), Some(2));
    }

    #[test]
    fn test_set_and_play_empty_tracks_clears_queue() {
        let mut queue = PlaybackQueue::new();
        queue.add_to_queue(make_track(1));
        queue.add_to_queue(make_track(2));

        queue.set_and_play(Vec::new(), 0);

        assert!(queue.items().is_empty());
        assert_eq!(queue.current_index(), None);
        assert!(queue.current().is_none());
    }

    #[test]
    fn test_clear_removes_items_and_next_is_none() {
        let mut queue = PlaybackQueue::new();
        queue.add_to_queue(make_track(1));
        queue.add_to_queue(make_track(2));

        queue.clear();

        assert!(queue.items().is_empty());
        assert!(queue.current().is_none());
        assert_eq!(queue.current_index(), None);
        assert!(queue.next().is_none());
    }

    #[test]
    fn test_previous_on_empty_queue_returns_at_start() {
        let mut queue = PlaybackQueue::new();

        let action = queue.previous(0);
        assert_eq!(action, PreviousAction::AtStart);
    }
}
