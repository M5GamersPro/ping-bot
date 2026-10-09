use poise::serenity_prelude as serenity;
use std::time::Instant;

// Custom user data struct required by Poise framework
struct Data {} 
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

/// Ultimate Ping command supporting both Slash and Prefix execution!
#[poise::command(slash_command, prefix_command)]
async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    // 1. Calculate API Latency (Websocket Heartbeat)
    let shard_manager = ctx.framework().shard_manager();
    let runners = shard_manager.runners.lock().await;
    let api_latency = match runners.get(&ctx.serenity_context().shard_id) {
        Some(runner) => match runner.latency {
            Some(duration) => format!("{}ms", duration.as_millis()),
            None => "Calculating...".to_string(),
        },
        None => "Unavailable".to_string(),
    };
    drop(runners); // Release lock safely

    let start_time = Instant::now();

    // 2. Send loading placeholder message
    let reply = ctx.say("🏓 Pinging server stats...").await?;
    
    // 3. Calculate Client/Host latency 
    let client_latency = start_time.elapsed().as_millis();

    // 4. Determine status tier and card layout coloring based on speed
    let (color, status) = if client_latency < 150 {
        (serenity::Color::from_rgb(46, 204, 113), "🚀 Excellent") // Green
    } else if client_latency < 300 {
        (serenity::Color::from_rgb(230, 126, 34), "🟡 Average")   // Orange
    } else {
        (serenity::Color::from_rgb(231, 76, 60), "🔴 Lagging")     // Red
    };

    let author_name = ctx.author().name.clone();
    let author_avatar = ctx.author().face();

    // 5. Structure updated layout into a crisp Embed display card
    reply.edit(ctx, poise::CreateReply::default()
        .content("") // Clear out the loading text
        .embed(serenity::CreateEmbed::new()
            .title("🏓 Pong!")
            .description(format!("Bot performance is currently **{}**.", status))
            .color(color)
            .field("API Latency", format!("`{}`", api_latency), true)
            .field("Client Latency", format!("`{}ms`", client_latency), true)
            .footer(serenity::CreateEmbedFooter::new(format!("Requested by {}", author_name))
                .icon_url(author_avatar))
            .timestamp(serenity::Timestamp::now())
        )
    ).await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");

    // Framework builder orchestration
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![ping()],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("!".to_string()), // Sets "!ping" prefix variant
                ..Default::default()
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                // Instantly registers your slash command globally across your profile
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT)
        .framework(framework)
        .await;

    println!("🚀 Rust Bot initialized. Listening for !ping or /ping...");
    client.unwrap().start().await.unwrap();
}

