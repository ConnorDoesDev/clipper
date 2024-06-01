use {
  crate::{
    composer::DirectorImplementation, handler::HandlerManager, voice::Handler as VoiceHandler,
  },
  serenity::{
    async_trait,
    framework::StandardFramework,
    model::prelude::Ready,
    prelude::{Client, Context, EventHandler, GatewayIntents},
    CacheAndHttp,
  },
  songbird::{driver::DecodeMode, Config, SerenityInit},
  std::sync::Arc,
};

#[derive(Debug)]
pub enum DiscordClientError {
  ClientCreation,
  ClientConnection,
}

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
  async fn ready(&self, _ctx: Context, data_about_bot: Ready) {
    println!("{} is ballin", data_about_bot.user.name);
  }
}

pub struct DiscordClient {
  pub director: DirectorImplementation,
  pub client: Arc<CacheAndHttp>,
}

impl DiscordClient {
  pub async fn new(
    token: &str,
    director: DirectorImplementation,
  ) -> Result<DiscordClient, DiscordClientError> {
    let mut handler_manager = HandlerManager::new();

    handler_manager.add_handler(Box::from(Handler));
    handler_manager.add_handler(Box::from(VoiceHandler::new(director.clone())));

    let intents = GatewayIntents::all();
    let songbird_config = Config::default().decode_mode(DecodeMode::Decode);

    // prc remove this since framework is legacy
    let framework = StandardFramework::new()
      .configure(|c| c.prefix("~"))
      .group(&crate::commands::GENERAL_GROUP);

    let mut client = match Client::builder(token, intents)
      .event_handler(handler_manager)
      .register_songbird_from_config(songbird_config)
      .framework(framework)
      .await
    {
      Ok(client) => client,
      Err(_) => return Err(DiscordClientError::ClientCreation),
    };

    let cache_and_http = client.cache_and_http.clone();

    tokio::spawn(async move {
      match client.start().await {
        Ok(_) => Ok(()),
        Err(e) => {
          println!("was unable to start client {}", e);
          Err(DiscordClientError::ClientConnection)
        }
      }
    });

    Ok(DiscordClient {
      director,
      client: cache_and_http,
    })
  }
}
