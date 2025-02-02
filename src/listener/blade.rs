use anyhow::Result;
use serenity::{
    all::CommandInteraction,
    builder::{CreateCommand, CreateInteractionResponse, CreateInteractionResponseMessage},
    client::Context,
};
use sqlx::SqlitePool;

pub fn register(name: &str, commands: &mut Vec<CreateCommand>) {
    commands.push(CreateCommand::new(name).description("Blade"));
}

pub async fn command(ctx: &Context, command: &CommandInteraction, _: &SqlitePool) -> Result<()> {
    command
        .create_response(
            &ctx,
            CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().content("https://cdn.discordapp.com/attachments/1316430234421166082/1333818715971653686/image.png?ex=679f8cfb&is=679e3b7b&hm=4a690f047e7ef2649210570408a7bd501b06b2520dbc1cea698d9e545af737fb&"))
        )
        .await?;

    Ok(())
}
