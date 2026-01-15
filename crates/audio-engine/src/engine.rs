use std::time::Instant;

use crate::queue::{PlaybackQueue, PreviousAction};
use crate::types::{PlaySession, PlaybackState, TrackInfo};

pub struct EngineState {
    pub state: PlaybackState,
    pub session: Option<PlaySession>,
    pub queue: PlaybackQueue,
    pub volume: f32,
}

impl EngineState {
    pub fn new() -> Self {
        Self {
            state: PlaybackState::Stopped,
            session: None,
            queue: PlaybackQueue::new(),
            volume: 1.0,
        }
    }

    pub fn play_now(&mut self, track: TrackInfo) {
        self.queue.play_now(track.clone());
        self.start_session(track, Instant::now());
    }

    pub fn add_to_queue(&mut self, track: TrackInfo) {
        self.queue.add_to_queue(track);
    }

    pub fn pause(&mut self) {
        if self.state != PlaybackState::Playing {
            return;
        }

        self.update_time_at(Instant::now());

        if let Some(session) = &mut self.session {
            session.last_play_start = None;
        }

        self.state = PlaybackState::Paused;
    }

    pub fn resume(&mut self) {
        if self.state != PlaybackState::Paused {
            return;
        }

        let now = Instant::now();
        if let Some(session) = &mut self.session {
            session.last_play_start = Some(now);
            self.state = PlaybackState::Playing;
        }
    }

    pub fn stop(&mut self) {
        if self.state == PlaybackState::Playing {
            self.update_time_at(Instant::now());
        }

        self.state = PlaybackState::Stopped;
        self.session = None;
    }

    pub fn seek(&mut self, position_ms: u64) {
        if self.session.is_none() {
            return;
        }

        if self.state == PlaybackState::Playing {
            self.update_time_at(Instant::now());
        }

        if let Some(session) = &mut self.session {
            session.position_ms = position_ms;
        }
    }

    pub fn next(&mut self) {
        if self.state == PlaybackState::Playing {
            self.update_time_at(Instant::now());
        }

        let now = Instant::now();
        let next_item = self.queue.next().cloned();

        match next_item {
            Some(item) => self.start_session(item.track, now),
            None => self.stop(),
        }
    }

    pub fn previous(&mut self) {
        let (position_ms, state) = match &self.session {
            Some(session) => (session.position_ms, self.state),
            None => return,
        };

        if state == PlaybackState::Playing {
            self.update_time_at(Instant::now());
        }

        let now = Instant::now();
        match self.queue.previous(position_ms) {
            PreviousAction::RestartCurrent | PreviousAction::AtStart => {
                if let Some(session) = &mut self.session {
                    session.position_ms = 0;
                    session.last_play_start = Some(now);
                    self.state = PlaybackState::Playing;
                }
            }
            PreviousAction::MoveToPrevious(_) => {
                let Some(track) = self.queue.current().map(|item| item.track.clone()) else {
                    self.stop();
                    return;
                };
                self.start_session(track, now);
            }
        }
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
    }

    pub fn volume(&self) -> f32 {
        self.volume
    }

    pub fn update_time(&mut self) {
        self.update_time_at(Instant::now());
    }

    fn start_session(&mut self, track: TrackInfo, now: Instant) {
        self.session = Some(PlaySession::new(track, now));
        self.state = PlaybackState::Playing;
    }

    fn update_time_at(&mut self, now: Instant) {
        if self.state != PlaybackState::Playing {
            return;
        }

        let Some(session) = &mut self.session else {
            return;
        };

        let Some(last_play_start) = session.last_play_start else {
            return;
        };

        let elapsed = now.saturating_duration_since(last_play_start);
        let elapsed_ms: u64 = elapsed.as_millis().min(u64::MAX as u128) as u64;

        session.played_ms = session.played_ms.saturating_add(elapsed_ms);
        session.position_ms = session.position_ms.saturating_add(elapsed_ms);
        session.last_play_start = Some(now);
    }
}

impl Default for EngineState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    fn track(id: i64) -> TrackInfo {
        TrackInfo {
            id,
            path: format!("/music/{id}.flac"),
            title: Some(format!("Track {id}")),
            artist: None,
            album: None,
            duration_ms: None,
            sample_rate: None,
            bit_depth: None,
            channels: None,
            codec: None,
            container: None,
        }
    }

    #[test]
    fn play_pause_time_accounting_increments_only_while_playing() {
        let mut engine = EngineState::new();
        engine.play_now(track(1));

        let start = engine.session.as_ref().unwrap().started_at;

        engine.update_time_at(start + Duration::from_millis(1500));
        assert_eq!(engine.session.as_ref().unwrap().played_ms, 1500);
        assert_eq!(engine.session.as_ref().unwrap().position_ms, 1500);

        engine.pause();
        let played_at_pause = engine.session.as_ref().unwrap().played_ms;

        engine.update_time_at(start + Duration::from_millis(5000));
        assert_eq!(engine.session.as_ref().unwrap().played_ms, played_at_pause);

        engine.resume();
        let resume_time = engine.session.as_ref().unwrap().last_play_start.unwrap();
        engine.update_time_at(resume_time + Duration::from_millis(700));

        assert_eq!(
            engine.session.as_ref().unwrap().played_ms,
            played_at_pause + 700
        );
    }

    #[test]
    fn seek_updates_position_without_incrementing_played_ms() {
        let mut engine = EngineState::new();
        engine.play_now(track(1));

        let start = engine.session.as_ref().unwrap().started_at;
        engine.update_time_at(start + Duration::from_millis(1000));

        let played_before = engine.session.as_ref().unwrap().played_ms;
        engine.seek(10_000);

        assert_eq!(engine.session.as_ref().unwrap().played_ms, played_before);
        assert_eq!(engine.session.as_ref().unwrap().position_ms, 10_000);
    }

    #[test]
    fn play_now_generates_new_play_id() {
        let mut engine = EngineState::new();
        engine.play_now(track(1));

        let first_id = engine.session.as_ref().unwrap().play_id;
        engine.play_now(track(2));
        let second_id = engine.session.as_ref().unwrap().play_id;

        assert_ne!(first_id, second_id);
        assert_eq!(engine.session.as_ref().unwrap().track.id, 2);
    }

    #[test]
    fn queue_operations_add_next_and_stop_at_end() {
        let mut engine = EngineState::new();
        engine.play_now(track(1));
        engine.add_to_queue(track(2));
        engine.add_to_queue(track(3));

        assert_eq!(engine.queue.items().len(), 3);
        assert_eq!(engine.queue.current_index(), Some(0));

        engine.next();
        assert_eq!(engine.session.as_ref().unwrap().track.id, 2);

        engine.next();
        assert_eq!(engine.session.as_ref().unwrap().track.id, 3);

        engine.next();
        assert_eq!(engine.state, PlaybackState::Stopped);
        assert!(engine.session.is_none());
        assert_eq!(engine.queue.current_index(), None);
    }

    #[test]
    fn previous_restarts_when_position_over_3s() {
        let mut engine = EngineState::new();
        engine.play_now(track(1));
        engine.add_to_queue(track(2));

        engine.seek(3500);
        engine.pause();
        engine.previous();

        assert_eq!(engine.session.as_ref().unwrap().track.id, 1);
        assert_eq!(engine.session.as_ref().unwrap().position_ms, 0);
        assert_eq!(engine.state, PlaybackState::Playing);
    }

    #[test]
    fn previous_moves_to_previous_when_position_under_3s() {
        let mut engine = EngineState::new();
        engine.play_now(track(1));
        engine.add_to_queue(track(2));

        engine.next();
        let play_id_on_second = engine.session.as_ref().unwrap().play_id;

        engine.seek(2000);
        engine.pause();
        engine.previous();

        assert_eq!(engine.session.as_ref().unwrap().track.id, 1);
        assert_eq!(engine.state, PlaybackState::Playing);
        assert_ne!(engine.session.as_ref().unwrap().play_id, play_id_on_second);
    }

    #[test]
    fn next_while_paused_starts_playback() {
        let mut engine = EngineState::new();
        engine.play_now(track(1));
        engine.add_to_queue(track(2));
        engine.pause();

        engine.next();
        assert_eq!(engine.session.as_ref().unwrap().track.id, 2);
        assert_eq!(engine.state, PlaybackState::Playing);
    }

    #[test]
    fn next_changes_play_id() {
        let mut engine = EngineState::new();
        engine.play_now(track(1));
        engine.add_to_queue(track(2));

        let first = engine.session.as_ref().unwrap().play_id;
        engine.next();
        let second = engine.session.as_ref().unwrap().play_id;

        assert_ne!(first, second);
    }

    #[test]
    fn previous_at_start_restarts_current() {
        let mut engine = EngineState::new();
        engine.play_now(track(1));
        engine.seek(2000);
        engine.pause();

        engine.previous();
        assert_eq!(engine.session.as_ref().unwrap().track.id, 1);
        assert_eq!(engine.session.as_ref().unwrap().position_ms, 0);
        assert_eq!(engine.state, PlaybackState::Playing);
        assert_eq!(engine.queue.current_index(), Some(0));
    }

    #[test]
    fn volume_is_clamped_to_0_1() {
        let mut engine = EngineState::new();
        engine.set_volume(2.0);
        assert_eq!(engine.volume(), 1.0);
        engine.set_volume(-1.0);
        assert_eq!(engine.volume(), 0.0);
    }
}
