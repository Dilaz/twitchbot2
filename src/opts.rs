use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about, long_about = None)]
pub struct Opts {
    #[clap(long, env, hide_env = true)]
    pub twitch_client_id: twitch_api::twitch_oauth2::ClientId,
    
    #[clap(long, env, hide_env = true)]
    pub twitch_client_secret: twitch_api::twitch_oauth2::ClientSecret,
    
    #[clap(long, env, hide_env = true)]
    pub twitch_token: String,

    #[clap(long, env, hide_env = true)]
    pub twitch_username: String,
    
    #[clap(long, env, hide_env = true)]
    pub database_url: String,
}