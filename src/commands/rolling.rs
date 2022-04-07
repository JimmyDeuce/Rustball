use serenity::{
    framework::{
        standard::{
            Args,
            CommandResult,
            macros::{
                command,
            },
        },
    },
    model::channel::Message,
    prelude::*,
};

#[command]
async fn roll(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let roll_command;
    let comment;
    // Get config data as read-only to look up the comment separator. It is then freed up at the end of the subscope
    {
        let config_data = ctx.data.read().await;
        let cfg = config_data.get::<crate::ConfigKey>().expect("Failed to retrieve config!");

        (roll_command, comment) = match args.message().split_once(&cfg.comment_separator) {
            Some(res) => res,
            None => (args.message(), "")
        };
    }

    if roll_command == "" {
        let no_args_error = format!("{} What do you want me to roll?", msg.author);
        msg.channel_id.say(&ctx.http, no_args_error).await?;
        return Ok(());
    }

    let verbose = false; // to be set inside the roll

    // Get config data with write permission to manipulate the tray
    let mut config_data = ctx.data.write().await;
    let tray = config_data
        .get_mut::<crate::TrayKey>()
        .expect("Failed to retrieve dice tray!");
    let roll = tray
        .lock().await
        .add_roll_from_string(roll_command);

    let result = match roll {
        Ok(_) => match tray.lock().await.get_newest_roll() {
            Ok(res) => format!("{}", res),
            Err(why) => format!("Sorry, I lost your dice (m´・ω・｀)m ｺﾞﾒﾝ… ({})", why)
        },
        Err(why) => format!("{}", why),
    };

    if verbose {
        let breakdown = "VERBOSE ROLL BREAKDOWN GOES HERE";
        let message = format!("{} rolled {}: {}", msg.author, roll_command, result);
        msg.channel_id.send_message(&ctx.http, |m| {
            m.content(message);
            m.embed(|e| {
                e.title(comment);
                e.description(breakdown);
    
                e
            });
            m
        }).await?;
    } else {
        let annotation = match comment {
            "" => "".to_owned(),
            _ => format!(" ({})", comment)
        };
        let breakdown = "COMPACT ROLL BREAKDOWN GOES HERE";
        let message = format!("{} rolled {}{}: {} ({})", msg.author, roll_command, annotation, result, breakdown);
        msg.channel_id.say(&ctx.http, message).await?;
    }

    Ok(())
}

#[command]
#[aliases("cod", "cofd")]
async fn wod(ctx: &Context, msg: &Message) -> CommandResult {
    let roll = format!("{} I'm not edgy enough for that yet!", msg.author);
    msg.channel_id.say(&ctx.http, roll).await?;

    Ok(())
}

#[command]
#[aliases("l5roll", "rings")]
async fn l5r(ctx: &Context, msg: &Message) -> CommandResult {
    let roll = format!("{} I'm not weeb enough for that yet!", msg.author);
    msg.channel_id.say(&ctx.http, roll).await?;

    Ok(())
}

#[command]
#[aliases("sroll")]
async fn sr(ctx: &Context, msg: &Message) -> CommandResult {
    let roll = format!("{} I'm not shady enough for that yet!", msg.author);
    msg.channel_id.say(&ctx.http, roll).await?;

    Ok(())
}

#[command]
#[aliases("ex")]
async fn exroll(ctx: &Context, msg: &Message) -> CommandResult {
    let roll = format!("{} I'm not epic enough for that yet!", msg.author);
    msg.channel_id.say(&ctx.http, roll).await?;

    Ok(())
}