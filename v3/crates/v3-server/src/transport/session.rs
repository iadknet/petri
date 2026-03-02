//! Transitional websocket-session boundary.

use crate::query::cache::DirtyRect;
use crate::transport::protocol::{ClientMessage, ZoomTier};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProjectionNotice {
    pub projection_revision: u64,
    pub dirty_rect: Option<DirtyRect>,
    pub world_static_changed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ViewSubscription {
    pub request_id: u64,
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
    pub canvas_width: u16,
    pub canvas_height: u16,
    pub zoom_tier: ZoomTier,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PendingFrameDelivery {
    pub request_id: u64,
    pub projection_revision: u64,
    pub subscription: ViewSubscription,
}

#[derive(Clone, Debug, Default)]
pub struct TransportSession {
    active_subscription: Option<ViewSubscription>,
    last_delivery: Option<PendingFrameDelivery>,
}

impl TransportSession {
    pub fn apply_client_message(
        &mut self,
        message: ClientMessage,
    ) -> Result<(), std::convert::Infallible> {
        match message {
            ClientMessage::SubscribeView {
                request_id,
                x,
                y,
                width,
                height,
                canvas_width,
                canvas_height,
                zoom_tier,
            } => {
                if self
                    .active_subscription
                    .is_some_and(|active| request_id < active.request_id)
                {
                    return Ok(());
                }

                self.active_subscription = Some(ViewSubscription {
                    request_id,
                    x,
                    y,
                    width,
                    height,
                    canvas_width,
                    canvas_height,
                    zoom_tier,
                });
                self.last_delivery = None;
            }
            ClientMessage::UnsubscribeView => {
                self.active_subscription = None;
                self.last_delivery = None;
            }
        }

        Ok(())
    }

    #[must_use]
    pub fn active_subscription(&self) -> Option<&ViewSubscription> {
        self.active_subscription.as_ref()
    }

    pub fn prepare_frame_delivery(
        &mut self,
        notice: ProjectionNotice,
    ) -> Option<PendingFrameDelivery> {
        let subscription = self.active_subscription?;
        if self.last_delivery.is_some_and(|last| {
            last.request_id == subscription.request_id
                && last.projection_revision >= notice.projection_revision
        }) {
            return None;
        }

        let delivery = PendingFrameDelivery {
            request_id: subscription.request_id,
            projection_revision: notice.projection_revision,
            subscription,
        };
        self.last_delivery = Some(delivery);
        Some(delivery)
    }
}

#[cfg(test)]
mod tests {
    use crate::transport::protocol::{ClientMessage, ZoomTier};

    use super::{ProjectionNotice, TransportSession};

    #[test]
    fn latest_subscribe_request_wins() {
        let mut session = TransportSession::default();

        session
            .apply_client_message(ClientMessage::SubscribeView {
                request_id: 5,
                x: 0,
                y: 0,
                width: 10,
                height: 10,
                canvas_width: 100,
                canvas_height: 100,
                zoom_tier: ZoomTier::Overview,
            })
            .expect("initial subscribe should succeed");
        session
            .apply_client_message(ClientMessage::SubscribeView {
                request_id: 4,
                x: 1,
                y: 1,
                width: 11,
                height: 11,
                canvas_width: 101,
                canvas_height: 101,
                zoom_tier: ZoomTier::Detail,
            })
            .expect("stale subscribe should be safely ignored");

        let active = session
            .active_subscription()
            .expect("session should stay subscribed");
        assert_eq!(active.request_id, 5);
        assert_eq!(active.zoom_tier, ZoomTier::Overview);
    }

    #[test]
    fn delivery_cursor_coalesces_old_projection_revisions() {
        let mut session = TransportSession::default();
        session
            .apply_client_message(ClientMessage::SubscribeView {
                request_id: 9,
                x: 0,
                y: 0,
                width: 5,
                height: 5,
                canvas_width: 50,
                canvas_height: 50,
                zoom_tier: ZoomTier::Inspect,
            })
            .expect("subscribe should succeed");

        let first = session
            .prepare_frame_delivery(ProjectionNotice {
                projection_revision: 12,
                dirty_rect: None,
                world_static_changed: false,
            })
            .expect("first projection should be deliverable");
        assert_eq!(first.request_id, 9);
        assert_eq!(first.projection_revision, 12);
        assert!(
            session
                .prepare_frame_delivery(ProjectionNotice {
                    projection_revision: 12,
                    dirty_rect: None,
                    world_static_changed: false,
                })
                .is_none(),
            "same revision must not be redelivered"
        );
        assert!(
            session
                .prepare_frame_delivery(ProjectionNotice {
                    projection_revision: 11,
                    dirty_rect: None,
                    world_static_changed: false,
                })
                .is_none(),
            "older revisions must be dropped"
        );

        let next = session
            .prepare_frame_delivery(ProjectionNotice {
                projection_revision: 13,
                dirty_rect: None,
                world_static_changed: false,
            })
            .expect("newer revision should be delivered");
        assert_eq!(next.request_id, 9);
        assert_eq!(next.projection_revision, 13);
    }

    #[test]
    fn dirty_rect_metadata_does_not_change_cursor_behavior() {
        let mut session = TransportSession::default();
        session
            .apply_client_message(ClientMessage::SubscribeView {
                request_id: 7,
                x: 50,
                y: 50,
                width: 10,
                height: 10,
                canvas_width: 100,
                canvas_height: 100,
                zoom_tier: ZoomTier::Detail,
            })
            .expect("subscribe should succeed");

        let delivery = session
            .prepare_frame_delivery(ProjectionNotice {
                projection_revision: 1,
                dirty_rect: None,
                world_static_changed: false,
            })
            .expect("initial delivery should succeed");
        assert_eq!(delivery.request_id, 7);

        assert!(
            session
                .prepare_frame_delivery(ProjectionNotice {
                    projection_revision: 1,
                    dirty_rect: Some(crate::query::cache::DirtyRect {
                        x: 0,
                        y: 0,
                        width: 5,
                        height: 5,
                    }),
                    world_static_changed: false,
                })
                .is_none(),
            "older revisions must still be coalesced even when dirty rect metadata is present"
        );
    }
}
