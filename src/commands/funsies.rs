use serenity::{
    framework::standard::{
            CommandResult,
            macros::command,
        },
    model::channel::Message,
    prelude::*,
};
use crate::{funsies::funsies, scryfall::requests};

#[command]
async fn squid(ctx: &Context, msg: &Message) -> CommandResult {
    let squid = funsies::squid();
    msg.reply_ping(&ctx.http, squid).await?;

    Ok(())
}

#[command]
#[aliases("shadowruns", "fixalot", "rules")]
async fn shadow(ctx: &Context, msg: &Message) -> CommandResult {
    let rules = funsies::rules();
    msg.channel_id.say(&ctx.http, rules).await?;

    Ok(())
}

#[command]
async fn unyu(ctx: &Context, msg: &Message) -> CommandResult {
    let unyu =funsies::unyu();
    msg.reply_ping(&ctx.http, unyu).await?;

    Ok(())
}

#[command]
async fn atom(ctx: &Context, msg: &Message) -> CommandResult {
    let atom = funsies::atom();
    msg.channel_id.say(&ctx.http, atom).await?;

    Ok(())
}

#[command]
#[aliases("sway", "shimarin", "shima")]
async fn yuru(ctx: &Context, msg: &Message) -> CommandResult {
    let sway = funsies::yuru();
    msg.reply_ping(&ctx.http, sway).await?;

    Ok(())
}

#[command]
#[aliases("reiaq", "reiakyu", "brainrot", "dailydose")]
async fn them(ctx: &Context, msg: &Message) -> CommandResult {
    let dose = funsies::dailydose();
    msg.reply_ping(&ctx.http, dose).await?;

    Ok(())
}

#[command]
#[aliases("frostleaf", "frost", "her", "kee", "leaf")]
async fn dailyfox(ctx: &Context, msg: &Message) -> CommandResult {
    let client;
    let search_tags = ["frostleaf_(arknights)", "rating:g"];

    let mut config_data = ctx.data.write().await;
    let mut client_handler = config_data
        .get_mut::<crate::ClientHandlerKey>()
        .expect("Failed to retrieve client handler!")
        .lock()
        .await;
    if client_handler.client_available() {
        client = client_handler.client();
    } else {
        msg.reply_ping(&ctx.http, "☢ Not so fast! ☢\nThis command is rate-limited (100ms cooldown)! Please wait warmly and try again in a bit ❤").await?;
        return Ok(());
    }
    
    match requests::get_booru_random_json(client, &search_tags).await {
        Ok(booru_post) => {
            msg.reply_ping(&ctx.http, booru_post.post_url()).await?;
        },
        Err(why) => {
            msg.reply_ping(&ctx.http, format!("{}", why)).await?;
        }
    }

    Ok(())
}
