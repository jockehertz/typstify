mod typst_compile;

use poise::serenity_prelude as serenity;
use crate::typst_compile::CompileOutput;
use tokio;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

pub struct Data;

#[derive(Debug, Clone, Copy, Default, poise::ChoiceParameter)]
pub enum OutputType {
    #[name = "pdf"]
    Pdf,

    #[name = "png"]
    #[default]
    Png,

    #[name = "svg"]
    Svg,
}

#[poise::command(slash_command)]
async fn typst(
    ctx: Context<'_>,
    #[description = "Output format"]
    output: Option<OutputType>,
    #[description = "Typst code"]
    code: Option<String>,
) -> Result<(), Error> {
    process_typst_request(
        ctx,
        code,
        output.unwrap_or_default(),
        false,
    )
    .await
}

#[poise::command(slash_command)]
async fn math(
    ctx: Context<'_>,
    #[description = "Output format"]
    output: Option<OutputType>,
    #[description = "Math code"]
    code: Option<String>,
) -> Result<(), Error> {
    process_typst_request(
        ctx,
        code,
        output.unwrap_or_default(),
        true,
    )
    .await
}

#[poise::command(slash_command)]
async fn hello(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("hello").await?;
    Ok(())
}

async fn send_compiled_output(ctx: Context<'_>, output: CompileOutput) -> Result<(), Error> {
    match output {
        CompileOutput::Pdf(data) => {
            let attachment = serenity::CreateAttachment::bytes(data, "output.pdf");
            let reply = poise::CreateReply::default()
                .content("Compilation succeeded")
                .attachment(attachment);
            ctx.send(reply).await?;
        }

        CompileOutput::Png(pngs) => {
            let mut reply = poise::CreateReply::default()
                .content("Compilation succeeded");

            let mut i = 0;
            for png in pngs {
                let filename = format!("output-{}.png", i);
                let attachment = serenity::CreateAttachment::bytes(png, filename);
                reply = reply.attachment(attachment);
                i += 1;
            };
            ctx.send(reply).await?;
        },

        CompileOutput::Svg(svgs) => {
            let mut reply = poise::CreateReply::default()
                .content("Compilation succeeded");

            let mut i = 0;
            for svg in svgs {
                let filename = format!("output-{}.svg", i);
                let attachment = serenity::CreateAttachment::bytes(svg.into_bytes(), filename);
                reply = reply.attachment(attachment);
                i += 1;
            };
            ctx.send(reply).await?;
        },
    };
    Ok(())
}

async fn process_typst_request(
    ctx: Context<'_>,
    code: Option<String>,
    output_type: OutputType,
    math: bool
) -> Result<(), Error> {
    let Some(code) = code.filter(|code| !code.trim().is_empty()) else {
        ctx.say("modal here").await?;
        return Ok(());
    };

    let wrapped_code = match math {
        true => format!("$\n{}\n$", code),
        false => code
    };

    match typst_compile::typst_compile(&wrapped_code, output_type) {
        Ok(compiled_output) => {
            send_compiled_output(ctx, compiled_output).await?;
        }

        Err(typst_compile::TypstError::CompileError(message)) => {
            ctx.say(format!("```Error:\n{message}\n```")).await?;
        }

        Err(typst_compile::TypstError::RenderError) => {
            ctx.say("The render failed").await?;
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> () {
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![typst(), math(), hello()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();
}
