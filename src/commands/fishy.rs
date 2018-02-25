use serenity::framework::standard::CommandError;
use serenity::model::id::UserId;
use chrono::Duration;
use chrono_humanize::HumanTime;

use std::fmt::Write;
use utils::config::get_pool;
use utils::time::now_utc;
use utils::user::get_id;

command!(fishy(ctx, msg, args) {
    let pool = get_pool(&ctx);

    if let Some(last_fishy) = pool.get_last_fishies(msg.author.id.0) {
        let now = now_utc();
        let next_rep = last_fishy + Duration::hours(12);

        let diff = next_rep.signed_duration_since(now);
        // precise humanized time 
        let ht = format!("{:#}", HumanTime::from(diff));

        if next_rep > now {
            return Err(CommandError::from(get_msg!("error/fishy_too_soon", ht)))
        }
    };

    let mut fishies_self = false;

    let target = if !args.is_empty() {
        // fishies for someone else
        match args.single::<String>().ok().and_then(|x| get_id(&x)) {
            Some(val) => val,
            None => return Err(CommandError::from(get_msg!("error/invalid_user"))),
        }
    } else {
        msg.author.id.0
    };

    // check if fishy for self
    if target == msg.author.id.0 {
        fishies_self = true
    }

    let target_user = match UserId(target).get() {
        Ok(val) => val,
        Err(_) => return Err(CommandError::from(get_msg!("error/failed_get_user"))),
    };

    // disallow bots fishy
    if target_user.bot {
        return Err(CommandError::from(get_msg!("error/fishy_bot")));
    }


    let (num_fishies, is_golden) = pool.get_fishies(msg.author.id.0, target, fishies_self);
    pool.update_stat("fishies", "fishies_given", num_fishies);

    let s = if fishies_self && !is_golden {
        get_msg!("info/fishies_received", num_fishies)
    } else if fishies_self && is_golden {
        get_msg!("info/fishies_received_golden", num_fishies)        
    } else if !fishies_self && !is_golden {
        get_msg!("info/fishies_given", num_fishies, target_user.tag())
    } else {
        get_msg!("info/fishies_given_golden", num_fishies, target_user.tag())
    };

    let _ = msg.channel_id.say(&s);
});

fn get_pos_emoji(pos: i64) -> String {
    match pos {
        0 => ":first_place:",
        1 => ":second_place:",
        2 => ":third_place:",
        _ => ":medal:",
    }.to_owned()
}

command!(fishies_top(ctx, msg, _args) {
    let pool = get_pool(&ctx);

    if let Some(users) = pool.get_top_fishies() {
        let mut s = String::new();
        for (i, user) in users.iter().enumerate() {
            let _ = write!(s, "{} {} fishies - <@{}>\n", get_pos_emoji(i as i64), user.fishies, user.id);
        }

        let _ = msg.channel_id.send_message(|m|
            m.embed(|e| e
                .author(|a| a
                    .name("Top Fishies - Global")
                )
                .color(0x2ecc71)
                .description(&s)
            )
        );
    }
});