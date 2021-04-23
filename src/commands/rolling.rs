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
async fn roll(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let mut roll = "".to_owned();
    let mut part_of_roll = true;
    let mut verbose = false;
    let mut comment = "".to_owned();
    for arg in args.iter::<String>() {
        match arg {
            Err(why) => {
                let arg_error = format!("☢ I don't know how to roll that! ☢\nError parsing argument: {}", why);
                msg.channel_id.say(&ctx.http, arg_error).await?;
                return Ok(());
            }
            Ok(arg_string) => {
                if arg_string == "!" {
                    part_of_roll = false;
                    continue
                }
                if arg_string == "-verbose" && part_of_roll {
                    verbose = true;
                    part_of_roll = false;
                    continue
                }
                match part_of_roll {
                    true => roll += &arg_string,
                    false => comment += &arg_string
                }
            }
        }
    }

    if verbose {
        let result = "RESULT GOES HERE";
        let breakdown = "VERBOSE ROLL BREAKDOWN GOES HERE";
        let message = format!("{} rolled {}: **{}**", msg.author, roll, result);
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
        if comment != "" {comment = format!(" ({})", comment)}
        let result = "RESULT GOES HERE";
        let breakdown = "COMPACT ROLL BREAKDOWN GOES HERE";
        let message = format!("{} rolled {}{}: **{}** ({})", msg.author, roll, comment, result, breakdown);
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