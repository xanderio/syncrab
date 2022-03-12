use color_eyre::Result;
use matrix_sdk::{room::Joined, ruma::events::room::message::RoomMessageEventContent, Client};

pub async fn handle(cmd: &str, client: Client, room: Joined) -> Result<()> {
    match cmd {
        "party" => party(room).await,
        "list_users" => list_users(client, room).await,
        _ => help(room).await,
    }?;

    Ok(())
}

async fn help(room: Joined) -> Result<()> {
    let help = r#"Available commands:

!list_users: list all users on this server
!party: are you ready to party?
!help: list all available commands (this message)"#;
    let content = RoomMessageEventContent::text_plain(help);
    room.send(content, None).await?;
    Ok(())
}

async fn party(room: Joined) -> Result<()> {
    let content = RoomMessageEventContent::text_plain("🎉🎊🥳 let's PARTY!! 🥳🎊🎉");
    room.send(content, None).await?;
    Ok(())
}

async fn list_users(client: Client, room: Joined) -> Result<()> {
    const TABLE_START: &str = "<table><tr><th>User Id</th><th>Displayname</th><th>Admin</th></tr>";
    const TABLE_END: &str = "</table>";

    let request = synapse_admin_api::users::list_users::v2::Request::new();
    let response = client.send(request, None).await?;

    let list: String = response
        .users
        .into_iter()
        .map(|user| {
            format!(
                "<tr><td>{name}</td><td>{display}</td><td>{admin}</td></tr>",
                name = user.name,
                display = user.displayname,
                admin = user.admin
            )
        })
        .collect();

    let table: String = [TABLE_START, &list, TABLE_END].into_iter().collect();
    let content = RoomMessageEventContent::text_html("", table);

    room.send(content, None).await?;

    Ok(())
}
