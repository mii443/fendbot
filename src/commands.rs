use crate::{context::save_context, Context, Error};

#[poise::command(prefix_command)]
pub async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

#[poise::command(prefix_command, slash_command)]
pub async fn calc(ctx: Context<'_>, expr: String) -> Result<(), Error> {
    let mut data = ctx.data().context.lock().await;
    let author = ctx.author().id.get();

    if !data.contains_key(&author) {
        data.insert(author, fend_core::Context::new());
    }
    let mut context = data.get_mut(&ctx.author().id.get()).unwrap();

    let result = fend_core::evaluate(&expr, &mut context).unwrap();
    let result = result.get_main_result();

    ctx.reply(result).await.unwrap();

    save_context(context, author);

    Ok(())
}

#[poise::command(prefix_command, slash_command)]
pub async fn reset_context(ctx: Context<'_>) -> Result<(), Error> {
    let mut data = ctx.data().context.lock().await;
    data.remove_entry(&ctx.author().id.get());
    ctx.reply("success").await.unwrap();
    Ok(())
}
