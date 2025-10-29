use crate::commands::types::{Context, Error};
use poise::serenity_prelude as serenity;

/// Truncates the message with a given truncation message if the
/// text is too long. "Too long" means, it either goes beyond Discord's 2000 char message limit,
/// or if the `text_body` has too many lines.
pub async fn trim_text(
    text_body: &str,
    truncation_msg_future: impl std::future::Future<Output = String>,
) -> String {
    const MAX_OUTPUT_LINES: usize = 45;
    const MAX_OUTPUT_LENGTH: usize = 2000;

    let needs_truncating = text_body.len() > MAX_OUTPUT_LENGTH
        || text_body.lines().count() > MAX_OUTPUT_LINES;

    if needs_truncating {
        let truncation_msg = truncation_msg_future.await;

        // truncate for length
        let text_body: String = text_body
            .chars()
            .take(MAX_OUTPUT_LENGTH - truncation_msg.len())
            .collect();

        // truncate for lines
        let text_body = text_body
            .lines()
            .take(MAX_OUTPUT_LINES)
            .collect::<Vec<_>>()
            .join("\n");

        format!("{text_body}{truncation_msg}")
    } else {
        format!("{text_body}")
    }
}

pub async fn send_reply(
    ctx: Context<'_>,
    raw_text: &str,
) -> Result<(), Error> {
    println!("sending reply {:?}", ctx);

    let text = trim_text(raw_text, async {
        "Output too large".to_string()
    }).await;

    let image_url = "https://raw.githubusercontent.com/serenity-rs/serenity/current/logo.png";

    let reply = {
        let embed = serenity::CreateEmbed::default()
            .description("embed 1")
            .image(image_url);

        /*let components = vec![serenity::CreateActionRow::Buttons(vec![
            serenity::CreateButton::new("1")
                .label("button 1")
                .style(serenity::ButtonStyle::Primary),
        ])];*/

        poise::CreateReply::default()
            .content(text)
            .embed(embed)
            //.components(components)
    };

    ctx.send(reply).await?;

    Ok(())
}