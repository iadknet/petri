//! WebSocket session registry.

use std::collections::HashMap;

use crate::transport::protocol::ClientMessage;

use super::session::{PendingFrameDelivery, ProjectionNotice, TransportSession, ViewSubscription};

pub type SessionId = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionRegistryError {
    UnknownSession(SessionId),
}

#[derive(Debug, Default)]
pub struct SessionRegistry {
    next_session_id: SessionId,
    sessions: HashMap<SessionId, TransportSession>,
}

impl SessionRegistry {
    #[must_use]
    pub fn insert(&mut self) -> SessionId {
        let session_id = self.next_session_id;
        self.next_session_id = self.next_session_id.wrapping_add(1);
        self.sessions
            .insert(session_id, TransportSession::default());
        session_id
    }

    pub fn remove(&mut self, session_id: SessionId) -> Option<TransportSession> {
        self.sessions.remove(&session_id)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.sessions.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.sessions.is_empty()
    }

    pub fn apply_client_message(
        &mut self,
        session_id: SessionId,
        message: ClientMessage,
    ) -> Result<(), SessionRegistryError> {
        let session = self
            .sessions
            .get_mut(&session_id)
            .ok_or(SessionRegistryError::UnknownSession(session_id))?;
        let _ = session.apply_client_message(message);
        Ok(())
    }

    #[must_use]
    pub fn active_subscription(&self, session_id: SessionId) -> Option<ViewSubscription> {
        self.sessions
            .get(&session_id)
            .and_then(|session| session.active_subscription().copied())
    }

    #[must_use]
    pub fn prepare_frame_delivery(
        &mut self,
        session_id: SessionId,
        notice: ProjectionNotice,
    ) -> Option<PendingFrameDelivery> {
        self.sessions
            .get_mut(&session_id)
            .and_then(|session| session.prepare_frame_delivery(notice))
    }
}

#[cfg(test)]
mod tests {
    use crate::transport::protocol::{ClientMessage, ZoomTier};

    use super::SessionRegistry;
    use crate::transport::session::ProjectionNotice;

    #[test]
    fn registry_tracks_sessions_and_latest_request_semantics() {
        let mut registry = SessionRegistry::default();
        let session_id = registry.insert();

        registry
            .apply_client_message(
                session_id,
                ClientMessage::SubscribeView {
                    request_id: 2,
                    x: 0,
                    y: 0,
                    width: 20,
                    height: 20,
                    canvas_width: 400,
                    canvas_height: 400,
                    zoom_tier: ZoomTier::Detail,
                },
            )
            .expect("initial subscribe should succeed");
        registry
            .apply_client_message(
                session_id,
                ClientMessage::SubscribeView {
                    request_id: 1,
                    x: 5,
                    y: 5,
                    width: 10,
                    height: 10,
                    canvas_width: 100,
                    canvas_height: 100,
                    zoom_tier: ZoomTier::Overview,
                },
            )
            .expect("stale subscribe should not error");

        let delivery = registry
            .prepare_frame_delivery(
                session_id,
                ProjectionNotice {
                    projection_revision: 8,
                    dirty_rect: None,
                    world_static_changed: false,
                },
            )
            .expect("latest subscription should yield a delivery");
        assert_eq!(delivery.request_id, 2);
        assert_eq!(delivery.projection_revision, 8);
    }

    #[test]
    fn registry_unsubscribe_stops_future_delivery() {
        let mut registry = SessionRegistry::default();
        let session_id = registry.insert();
        registry
            .apply_client_message(
                session_id,
                ClientMessage::SubscribeView {
                    request_id: 4,
                    x: 0,
                    y: 0,
                    width: 20,
                    height: 20,
                    canvas_width: 400,
                    canvas_height: 400,
                    zoom_tier: ZoomTier::Detail,
                },
            )
            .expect("subscribe should succeed");

        registry
            .apply_client_message(session_id, ClientMessage::UnsubscribeView)
            .expect("unsubscribe should succeed");

        assert!(
            registry
                .prepare_frame_delivery(
                    session_id,
                    ProjectionNotice {
                        projection_revision: 9,
                        dirty_rect: None,
                        world_static_changed: false,
                    },
                )
                .is_none(),
            "unsubscribed session must not receive deliveries"
        );
    }
}
