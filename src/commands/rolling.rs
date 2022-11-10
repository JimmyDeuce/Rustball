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
    model::{
        channel::Message, 
        id::{
            GuildId, 
            ChannelId
        }
    },
    prelude::*,
};
use std::collections::HashMap;
use crate::{
    dice::{
        command_translations,
        tray::Tray
    }, 
    sixball_errors::SixballError
};

pub type TrayMap = HashMap<TrayId, Tray>;

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum TrayId {
    Private(ChannelId),
    Guild(Option<GuildId>),
}

#[command]
#[description="The basic roll command! Currently under construction.\n
Use standard die roll notation of the form `XdY`. I can roll up to 255 dice with up to 255 sides at once!\n
I can also do math with dice! (　-\\`ω-)✧ﾄﾞﾔｯ Just plug your dice into any math expression, e.g. `1d20+5`. If the `calc` command can handle it, so can the `roll` command!\n
Additional dice operations to be added. Please wait warmly!"]
#[aliases("r", "rill", "rol", "rll")]
async fn roll(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let (roll_command, roll_comment) = extract_arguments(ctx, args).await;
    let in_command = &roll_command;

    let response = match new_roll_output(&ctx, &msg, &in_command, &roll_command, &roll_comment, true).await {
        Ok(res) => format!("{}", res),
        Err(why) => format!("{}", why),
    };
    msg.reply_ping(&ctx.http, response).await?;

    Ok(())
}

#[command]
#[description="Under construction. Please wait warmly!"]
async fn reroll(ctx: &Context, msg: &Message) -> CommandResult {
    // Get config data with write permission to manipulate the tray
    let mut tray_data = ctx.data.write().await;
    let mut tray_map = tray_data
        .get_mut::<crate::TrayKey>()
        .expect("Failed to retrieve tray map!")
        .lock().await;

    if let Some(tray) = tray_map.get_mut(&make_tray_id(msg)) {
        match tray.reroll_latest() {
            Ok(reroll) => {
                let message = format!("Reroll: {}", reroll);
                msg.reply_ping(&ctx.http, message).await?;
            },
            Err(why) => {
                let roll_error = format!("{}", why);
                msg.reply_ping(&ctx.http, roll_error).await?;
            }
        }
    } else {
        msg.reply_ping(&ctx.http, "There's nothing to reroll!").await?;
    }

    Ok(())
}

#[command]
#[aliases("tray")]
async fn pastrolls(ctx: &Context, msg: &Message) -> CommandResult {
    let tray_data = ctx.data.read().await;
    let tray_map = tray_data
        .get::<crate::TrayKey>()
        .expect("Failed to retrieve tray map!")
        .lock().await;

    if let Some(tray) = tray_map.get(&make_tray_id(msg)) {
        msg.channel_id.send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title("Currently Stored Rolls");
                for (i, roll) in tray.rolls().iter().enumerate() {
                    // Build the title here containing i, person who rolled, and maybe timestamp?
                    let title = format!("{}: By {} at {}", i, roll.roller(), roll.timestamp().format("%y/%m/%d %H:%M:%S"));
                    let text = format!("{}", roll);
                    e.field(title, text, false);
                }
                e
            });
            m
        }).await?;
    } else {
        msg.reply_ping(&ctx.http, "I haven't even set up a tray for this server yet!").await?;
    }

    Ok(())
}

#[command]
#[description="Under construction. Please wait warmly!"]
async fn verbose(ctx: &Context, msg: &Message) -> CommandResult {
    let tray_data = ctx.data.read().await;
    let tray_map = tray_data
        .get::<crate::TrayKey>()
        .expect("Failed to retrieve tray map!")
        .lock().await;

    if let Some(tray) = tray_map.get(&make_tray_id(msg)) {
        let latest_roll = match tray.get_newest_roll() {
            Ok(roll) => roll,
            Err(why) => {
                msg.reply_ping(&ctx.http, format!("{}", why)).await?;
                return Ok(());
            }
        };

        msg.channel_id.send_message(&ctx.http, |m| {
            m.embed(|e| {
                let annotation = match latest_roll.comment().trim() {
                    "" => "".into(),
                    other => format!(" ({})", other),
                };
                let title = format!("{}{}", latest_roll.command(), annotation);
                e.title(title);
                for operation in latest_roll.operations() {
                    let name = operation.description();
                    let value = operation.verbose();
                    e.field(name, value, false);
                }
                e.field("Total", latest_roll.result(), false);
                e
            });
            m
        }).await?;
    } else {
        msg.reply_ping(&ctx.http, "I haven't even set up a tray for this server yet!").await?;
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

#[command]
#[description="This command is for rolling Genesys narrative dice! Just tell me how many of which kinds to roll and I'll handle the rest under the hood! Σd(≧▽≦*)
Format the command like this: `[kind of die][number of dice]`. The different kinds of dice can be in any order, and you can put as many spaces as you want between them if it helps you organize the roll.
For example: `~genroll a2 p2 d3` -> 2 Ability dice, 2 Proficiency dice, 3 Difficulty dice
You can even have the same kind of die multiple times if you want, for example to keep track of different sources of dice! I'll add them all up for you.\n
The dice codes are:
\t• b: Boost (Blue d6)
\t• a: Ability (Green d8)
\t• p: Proficiency (Yellow d12)
\t• s: Setback (Black d6)
\t• d: Difficulty (Purple d8)
\t• c: Challenge (Red d12)"]
#[aliases("gr", "genesys", "groll")]
async fn genroll(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let (in_command, roll_comment) = extract_arguments(ctx, args).await;

    let response = match command_translations::genesys(&in_command) {
        Ok(roll_command) => match new_roll_output(&ctx, &msg, &in_command, &roll_command, &roll_comment, false).await {
            Ok(res) => format!("{}", res),
            Err(why) => format!("{}", why),
        },
        Err(why) => format!("{}", SixballError::RollError(why)),
    };
    msg.reply_ping(&ctx.http, response).await?;

    Ok(())
}

async fn extract_arguments(ctx: &Context, args: Args) -> (String, String) {
    // Get config data as read-only to look up the comment separator. It is then freed up when we move out of the function
    let config_data = ctx.data.read().await;
    let cfg = config_data.get::<crate::ConfigKey>().expect("Failed to retrieve config!");

    match args.message().split_once(&cfg.comment_separator) {
        Some((command, comment)) => (command.to_lowercase(), comment.into()),
        None => (args.message().to_lowercase(), "".into())
    }
}

async fn new_roll_output(ctx: &Context, msg: &Message, in_command: &str, roll_command: &str, roll_comment: &str, breakdown: bool) -> Result<String, SixballError> {
    // Get config data with write permission to manipulate the tray
    let mut tray_data = ctx.data.write().await;
    let mut tray_map = tray_data
        .get_mut::<crate::TrayKey>()
        .expect("Failed to retrieve tray map!")
        .lock().await;
    
    let tray = match tray_map.get_mut(&make_tray_id(msg)) {
        Some(extant_tray) => extant_tray,
        None => {
            let new_tray = Tray::new();
            tray_map.insert(make_tray_id(msg), new_tray);
            tray_map.get_mut(&make_tray_id(msg)).expect("Failed to get tray we literally just inserted!")
        }
    };
    let roller = msg.author_nick(&ctx).await.unwrap_or(msg.author.name.clone());

    let roll = tray.add_roll_from_command(roll_command, roll_comment, &roller)?;
    
    let annotation = match roll_comment.trim() {
        "" => "".to_owned(),
        other => format!(" ({})", other)
    };

    match breakdown {
        true => Ok(format!("`{}`{}:\n**{}** ({})", in_command.trim(), annotation, roll.result(), roll)),
        false => Ok(format!("`{}`{}:\n**{}** (use `verbose` or `tray` commands for details)", in_command.trim(), annotation, roll.result())),
    }
}

fn make_tray_id(msg: &Message) -> TrayId {
    let tray_id;
    if msg.is_private() {
        tray_id = TrayId::Private(msg.channel_id);
    } else {
        tray_id = TrayId::Guild(msg.guild_id);
    }

    tray_id
}
