mod client;
mod commands;
mod composer;
mod handler;
mod voice;
use {
  composer::Director,
  serenity::model::prelude::{AttachmentType, GuildId},
  std::{
    env,
    sync::{Arc, Mutex},
    time::Duration,
  },
  warp::Filter,
};

#[tokio::main]
async fn main() {
  let clip_duration: Option<Duration> = match env::var("DURATION") {
    Ok(v) => Some(Duration::from_millis(
      v.parse().expect("wass unable to parse duration!"),
    )),
    Err(_) => None,
  };

  let director = Arc::new(Mutex::new(Director::new(48_000, clip_duration)));

  let client = client::DiscordClient::new(
    &env::var("TOKEN").expect("no token brother you cant even fill out a config file????,,"),
    director,
  )
  .await
  .expect("error starting discord client");

  let clip = warp::path!("clip" / u64).map(move |guild_id: u64| {
    let parsed_id = GuildId(guild_id);
    let data = client.director.lock().unwrap().clip(&parsed_id);

    let path = voice::save_clip(&parsed_id, &data);
    // Get the name of the file that was saved (usually the milliseconds since the Unix epoch) and convert them to human-readable time
    let path = path.replace("output/", "");
    let path = path.replace(".wav", "");
    let human_readable_time = chrono::NaiveDateTime::from_timestamp(
      path.parse::<i64>().expect("unable to parse path to i64"),
      0,
    );
    let path = human_readable_time.format("%Y-%m-%d %H:%M:%S").to_string();

    let client = client.client.clone();

    tokio::spawn(async move {
      for (_channel_id, guild_channel) in client
        .http
        .get_guild(guild_id)
        .await?
        .channels(client.http.clone())
        .await?
      {
        if guild_channel.name == "clips" {
          guild_channel
            .send_files(
              client.http.clone(),
              vec![AttachmentType::from(std::path::Path::new(&path))],
              |m| m.content("new clip made!"),
            )
            .await?;
        }
      }

      Ok::<(), serenity::Error>(())
    });

    "Clip!"
  });
  // This is a warp filter that will match any request to /clip/{guild_id} and call the closure with the guild_id as a u64

  println!("runnin up dat webserver on port {} chief", env::var("PORT").unwrap());
  warp::serve(clip)
    .run((
      [127, 0, 0, 1],
      env::var("PORT")
        .expect("PORT not a thing bro lmfaooooooo")
        .parse()
        .expect("Port is not a number ??????? LOL"),
    ))
    .await;

  println!("webserver is kill qqqqq");
}
