use std::sync::Arc;

use poise::{serenity_prelude::CreateAttachment, CreateReply};
use tokio::sync::Mutex;
use tracing::{info, trace};

use crate::{
    context::{create_context, save_context},
    Context, Error,
};

#[poise::command(prefix_command)]
pub async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

#[poise::command(
    prefix_command,
    slash_command,
    subcommands("context"),
    subcommand_required
)]
pub async fn fend(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(
    prefix_command,
    slash_command,
    subcommands("reset", "define_custom_unit"),
    subcommand_required
)]
pub async fn context(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(prefix_command, slash_command)]
pub async fn reset(ctx: Context<'_>) -> Result<(), Error> {
    info!("/fend context reset by {}", ctx.author().id);
    let id = ctx.author().id.get();

    let mut data = ctx.data().context.lock().await;

    let context = create_context();
    save_context(&context, id);

    data.insert(id, context);

    ctx.reply("success").await.unwrap();
    Ok(())
}

#[poise::command(prefix_command, slash_command)]
pub async fn define_custom_unit(
    ctx: Context<'_>,
    singular: String,
    definition: String,
    plural: Option<String>,
) -> Result<(), Error> {
    info!(
        "/fend context define_custom_unit {} {} {:?} by {}",
        singular,
        definition,
        plural,
        ctx.author().id
    );
    let plural = plural.unwrap_or_default();
    let author = ctx.author().id.get();

    let mut data = ctx.data().context.lock().await;

    if !data.contains_key(&author) {
        let context = create_context();
        data.insert(author, context);
    }

    let context = data.get_mut(&ctx.author().id.get()).unwrap();

    context.define_custom_unit_v1(
        &singular,
        &plural,
        &definition,
        &fend_core::CustomUnitAttribute::None,
    );

    ctx.reply(format!(
        "Custom unit defined.\n{}({}): `{}`",
        singular, plural, definition,
    ))
    .await
    .unwrap();

    Ok(())
}

#[poise::command(prefix_command, slash_command)]
pub async fn calc(ctx: Context<'_>, expr: String) -> Result<(), Error> {
    info!("/calc {} by {}", expr, ctx.author().id);
    ctx.defer().await.unwrap();

    let author = ctx.author().id.get();

    let context = {
        let mut data = ctx.data().context.lock().await;

        if !data.contains_key(&author) {
            let context = create_context();
            data.insert(author, context);
        }
        data.get(&ctx.author().id.get()).unwrap().clone()
    };
    let context = Arc::new(Mutex::new(context));

    trace!("Evaluating {}", expr);
    let result = tokio::task::spawn_blocking({
        let context = context.clone();
        let expr = expr.clone();
        move || async move {
            let mut context = context.lock().await;
            let main_result = match fend_core::evaluate(&expr, &mut context) {
                Ok(eval_result) => eval_result.get_main_result().to_string(),
                Err(err) => err.to_string(),
            };
            main_result
        }
    })
    .await?
    .await;

    if let Err(_) = ctx.reply(format!("> {}\n{}", expr, result)).await {
        trace!("Reply using file");
        ctx.reply("Sending result...").await.unwrap();
        ctx.send(CreateReply::default().attachment(CreateAttachment::bytes(
            format!("> {}\n{}", expr, result),
            "result.txt",
        )))
        .await
        .unwrap();
    }

    {
        let mut data = ctx.data().context.lock().await;
        let context = context.lock().await;
        data.insert(author, context.clone());

        save_context(&context, author);
    }

    Ok(())
}
