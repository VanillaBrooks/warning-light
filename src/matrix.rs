use anyhow::Context;
use anyhow::Result;

use ruma::client::Client;

use ruma::api::client::membership::joined_rooms;
use ruma::api::client::message::get_message_events;

use std::fmt;
use std::time::Duration;

pub type HyperClient = ruma::client::http_client::HyperNativeTls;

#[derive(serde::Deserialize)]
struct Configuration {
    homeserver_url: String,
    username: String,
    password: String
}

pub(crate) struct MatrixConnection {
    c: Client<HyperClient>,
}

impl MatrixConnection {
    pub(crate) async fn new() -> Result<Self> {
        let config_bytes = include_str!("../.config.json");
        let config : Configuration = serde_json::from_str(config_bytes)
            .with_context(|| format!("failed to parse configuration json"))?;

        let homeserver_url = config.homeserver_url.parse().unwrap();

        let https = hyper_tls::HttpsConnector::new();
        let hyper_client = hyper::client::Client::builder().build::<_, hyper::Body>(https);

        let client = Client::builder()
            .homeserver_url(homeserver_url)
            .http_client(hyper_client)
            .await
            .unwrap();

        let _session = client
            .log_in(&config.username, &config.password, None, None)
            .await
            .unwrap();

        let out = MatrixConnection { c: client };
        Ok(out)
    }

    pub(crate) async fn load_rooms(&self) -> Result<Vec<Room>> {
        let joined_rooms_req = joined_rooms::v3::Request::new();
        let joined_rooms_resp = self.c.send_request(joined_rooms_req).await.unwrap();

        let mut rooms = Vec::new();

        for room_id in joined_rooms_resp.joined_rooms {
            let room = Room::new(&self.c, room_id).await?;
            rooms.push(room);
        }

        Ok(rooms)
    }
}

pub(crate) struct Room {
    room_id: ruma::OwnedRoomId,
    last_event_id: ruma::OwnedEventId,
}

impl Room {
    async fn new(client: &Client<HyperClient>, room_id: ruma::OwnedRoomId) -> Result<Self> {
        let recent_message = recent_message_for_room(client, room_id.clone()).await?;

        let room = Room {
            room_id,
            last_event_id: recent_message.event_id,
        };

        Ok(room)
    }

    pub(crate) async fn poll_room(&mut self, client: &MatrixConnection) -> Result<Option<Seconds>> {
        let last_message = recent_message_for_room(&client.c, self.room_id.clone()).await?;

        if last_message.event_id != self.last_event_id {
            self.last_event_id = last_message.event_id;

            info!(
                "new message from room {}: - {}",
                self.room_id, last_message.and
            );

            Ok(Some(Seconds::from_str(last_message.and)?))
        } else {
            Ok(None)
        }
    }
}

pub(crate) async fn poll_all_rooms(
    conn: &MatrixConnection,
    rooms: &mut [Room],
) -> Result<Option<Seconds>> {
    let mut out = None;

    for room in rooms {
        let poll_result = room.poll_room(conn).await;

        match poll_result {
            Ok(seconds_measure) => {
                compare_and_store(&mut out, seconds_measure);
            }
            Err(e) => {
                error!("failed to poll room {}: {e}", room.room_id);
            }
        }
    }

    Ok(out)
}

/// check if the new seconds measure is larger than the current one. If it is,
/// store it in `curr_max_opt`
fn compare_and_store(curr_max_opt: &mut Option<Seconds>, new_opt: Option<Seconds>) {
    let new = match (*curr_max_opt, new_opt) {
        // do nothing, we have no new value
        (Some(x), None) => Some(x),
        (None, Some(x)) => Some(x),
        (Some(curr_max), Some(candidate)) => {
            if candidate > curr_max {
                Some(candidate)
            } else {
                Some(curr_max)
            }
        }
        // also do nothing
        (None, None) => None,
    };

    *curr_max_opt = new;
}

#[derive(Debug, PartialOrd, Ord, Eq, PartialEq, Copy, Clone)]
pub(crate) struct Seconds(u64);

impl Seconds {
    fn from_str(msg: String) -> Result<Self> {
        let num: u64 = msg
            .parse()
            .with_context(|| format!("failed to parse {msg} to a single integer"))?;

        // maximum number of minutes on at once is 5 minutes
        let num = num.min(60 * 5);

        Ok(Self(num))
    }
}

impl From<Seconds> for Duration {
    fn from(x: Seconds) -> Duration {
        Duration::from_secs(x.0)
    }
}

impl fmt::Display for Seconds {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

async fn recent_message_for_room(
    client: &Client<HyperClient>,
    room_id: ruma::OwnedRoomId,
) -> Result<filter::EventIdAnd<String>> {
    let req = get_message_events::v3::Request::backward(room_id);
    let resp = client.send_request(req).await.unwrap();

    let last_text_message = resp
        .chunk
        .into_iter()
        // deserialize the event
        .filter_map(|raw_event| raw_event.deserialize().ok())
        // filter events to find the event id and the text message associated with it
        .filter_map(|event| {
            filter::message(event)
                .and_then(filter::room_message)
                .and_then(filter::non_redacted)
                .and_then(filter::text_message)
                .map(filter::text_body)
        })
        .next()
        .unwrap();

    Ok(last_text_message)
}

mod filter {
    use ruma::events::room::message::MessageType;
    use ruma::events::room::message::RoomMessageEventContent;
    use ruma::events::room::message::TextMessageEventContent;
    use ruma::events::AnyMessageLikeEvent;
    use ruma::events::AnyTimelineEvent;
    use ruma::events::MessageLikeEvent;
    use ruma::events::OriginalMessageLikeEvent;
    use ruma::OwnedEventId;

    pub(super) struct EventIdAnd<T> {
        pub(super) event_id: OwnedEventId,
        pub(super) and: T,
    }

    pub(super) fn message(message_or_state: AnyTimelineEvent) -> Option<AnyMessageLikeEvent> {
        if let AnyTimelineEvent::MessageLike(message) = message_or_state {
            Some(message)
        } else {
            None
        }
    }

    pub(super) fn room_message(
        message_like: AnyMessageLikeEvent,
    ) -> Option<MessageLikeEvent<RoomMessageEventContent>> {
        if let AnyMessageLikeEvent::RoomMessage(message) = message_like {
            Some(message)
        } else {
            None
        }
    }

    pub(super) fn non_redacted(
        message: MessageLikeEvent<RoomMessageEventContent>,
    ) -> Option<OriginalMessageLikeEvent<RoomMessageEventContent>> {
        if let MessageLikeEvent::Original(unredacted) = message {
            Some(unredacted)
        } else {
            None
        }
    }

    pub(super) fn text_message(
        message: OriginalMessageLikeEvent<RoomMessageEventContent>,
    ) -> Option<EventIdAnd<TextMessageEventContent>> {
        if let MessageType::Text(text_message) = message.content.msgtype {
            let pair = EventIdAnd {
                event_id: message.event_id,
                and: text_message,
            };
            Some(pair)
        } else {
            None
        }
    }

    pub(super) fn text_body(
        text_message: EventIdAnd<TextMessageEventContent>,
    ) -> EventIdAnd<String> {
        EventIdAnd {
            event_id: text_message.event_id,
            and: text_message.and.body,
        }
    }
}
