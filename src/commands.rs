use std::sync::Arc;

use tokio::sync::Mutex;

use crate::{
    context::{create_context, save_context},
    Context, Error,
};

#[poise::command(prefix_command)]
pub async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

#[poise::command(prefix_command, slash_command)]
pub async fn calc(ctx: Context<'_>, expr: String) -> Result<(), Error> {
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

    ctx.reply(format!("> {}\n{}", expr, result)).await.unwrap();

    {
        let mut data = ctx.data().context.lock().await;
        let context = context.lock().await;
        data.insert(author, context.clone());

        save_context(&context, author);
    }

    Ok(())
}

#[poise::command(prefix_command, slash_command)]
pub async fn reset_context(ctx: Context<'_>) -> Result<(), Error> {
    let id = ctx.author().id.get();

    let mut data = ctx.data().context.lock().await;

    let context = create_context();
    save_context(&context, id);

    data.insert(id, context);

    ctx.reply("success").await.unwrap();
    Ok(())
}
