mod typst_compile;

use poise::serenity_prelude as serenity;




mod typst_compile;

use poise::serenity_prelude as serenity;
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
    )
    .await
}

async fn process_typst_request(
    ctx: Context<'_>,
    code: Option<String>,
    output: OutputType,
) -> Result<(), Error> {
    let Some(code) = code.filter(|code| !code.trim().is_empty()) else {
        ctx.say("modal here").await?;
        return Ok(());
    };

    match typst_compile::typst_compile(&code, output) {
        Ok(_compiled_output) => {
            ctx.say("Compilation succeeded.").await?;
        }

        Err(typst_compile::TypstError::CompileError(message)) => {
            ctx.say(format!("```text\n{message}\n```")).await?;
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> () {


}
