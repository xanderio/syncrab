use std::time::Duration;

use color_eyre::Result;
use matrix_sdk::{
    room::Room,
    ruma::{
        events::room::{
            member::StrippedRoomMemberEvent,
            message::{
                MessageType, RoomMessageEventContent, SyncRoomMessageEvent, TextMessageEventContent,
            },
        },
        UserId,
    },
    Client,
};

use crate::command;

pub(super) async fn on_stripped_state_member(
    room_member: StrippedRoomMemberEvent,
    client: Client,
    room: Room,
) {
    if room_member.state_key != client.user_id().await.unwrap() {
        return;
    }

    if let Room::Invited(room) = room {
        tracing::info!("Autojoining room {}", room.room_id());
        let mut delay = 2;

        while let Err(err) = room.accept_invitation().await {
            // retry autojoin due to synapse sending invites, before the
            // invited user can join for more information see
            // https://github.com/matrix-org/synapse/issues/4345
            tracing::warn!(
                "Failed to join room {} ({err:?}), retrying in {delay}s",
                room.room_id(),
            );

            tokio::time::sleep(Duration::from_secs(delay)).await;
            delay *= 2;

            if delay > 3600 {
                tracing::warn!("Can't join room {} ({err:?})", room.room_id());
                break;
            }
        }
        tracing::info!("Successfully joined room {}", room.room_id());
    }
}

pub(super) async fn on_room_message(
    event: SyncRoomMessageEvent,
    room: Room,
    client: Client,
) -> Result<()> {
    if let Room::Joined(room) = room {
        if !is_admin(client.clone(), &event.sender).await? {
            tracing::info!("recieved message from non-admin user {}", event.sender);
            let content = RoomMessageEventContent::text_plain(
                "Only server admin are allowed to interact with me",
            );
            room.send(content, None).await.unwrap();
            return Ok(());
        }

        let msg_body = match event.content.msgtype {
            MessageType::Text(TextMessageEventContent { body, .. }) => body,
            _ => return Ok(()),
        };

        if let Some(cmd) = msg_body.strip_prefix("!") {
            tracing::info!("recieved command: {cmd}");
            command::handle(cmd, client, room).await?;
        }
    }

    Ok(())
}

async fn is_admin(client: Client, user_id: &UserId) -> Result<bool> {
    let request = synapse_admin_api::users::is_user_admin::v1::Request::new(user_id);
    let response = client.send(request, None).await?;
    Ok(response.admin)
}
