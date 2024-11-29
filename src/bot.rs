use color_eyre::{Result, Report};
use entity::urls;
use regex::Regex;
use tokio::try_join;
use tracing::{debug, error, info, warn};
use twitch_api::helix::users::GetUsersRequest;
use twitch_api::{twitch_oauth2::AppAccessToken, HelixClient};
use twitch_irc::{TwitchIRCClient, SecureTCPTransport, login::StaticLoginCredentials, ClientConfig, irc};
use std::fmt;
use tokio::sync::mpsc::{self, Receiver, Sender};
use crate::{errors::TwitchbotError, opts::Opts};
use entity::channels::{self, Entity as Channel};
use entity::banned_words::{self, Entity as BannedWord};
use sea_orm::{prelude::*, DatabaseConnection, EntityTrait, Set};
use std::collections::HashSet;
use entity::users::{self, Entity as User};
use entity::urls::{Entity as Url};

#[aliri_braid::braid(display = "owned", debug = "owned", serde)]
pub struct PostgressDatabaseUrl;

impl fmt::Debug for PostgressDatabaseUrlRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[redacted postgres url]")
    }
}
impl fmt::Display for PostgressDatabaseUrlRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[redacted postgres url]")
    }
}

#[derive(Debug, Clone)]
enum BannedWordSimple {
    Word(String),
    Regex(Regex),
}

pub enum BotEvent {
    TwitchMessage(twitch_irc::message::ServerMessage),
    // Add other event types here
}

pub struct Bot {
    name: String,
    channels: Vec<channels::Model>,
    helix_client: Option<HelixClient<'static, reqwest::Client>>,
    helix_client_token: Option<AppAccessToken>,
    twitch_client: Option<TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>>,
    twitch_token: String,
    twitch_client_id: twitch_api::twitch_oauth2::ClientId,
    twitch_client_secret: twitch_api::twitch_oauth2::ClientSecret,
    database_url: PostgressDatabaseUrl,
    banned_words: Vec<BannedWordSimple>,
    db: Option<DatabaseConnection>,
    event_sender: Option<Sender<BotEvent>>,
    event_receiver: Option<Receiver<BotEvent>>,
    urls: HashSet<String>,
    seen_users: HashSet<String>,
    banned_users: HashSet<String>,
}

impl Bot {
    /**
     * Create a new bot
     */
    pub fn new(opts: Opts) -> Bot {
        let (event_sender, event_receiver) = mpsc::channel(100);
        Bot {
            name: opts.twitch_username,
            channels: vec![],
            helix_client: None,
            helix_client_token: None,
            twitch_client: None,
            twitch_token: opts.twitch_token,
            twitch_client_id: opts.twitch_client_id,
            twitch_client_secret: opts.twitch_client_secret,
            database_url: PostgressDatabaseUrl::new(opts.database_url),
            banned_words: vec![],
            db: None,
            event_sender: Some(event_sender),
            event_receiver: Some(event_receiver),
            urls: HashSet::new(),
            seen_users: HashSet::new(),
            banned_users: HashSet::new(),
        }
    }

    /**
     * Run the bot
     */
    pub async fn run(&mut self) -> Result<(), TwitchbotError> {
        info!("Bot is running!");

        self.init_seaorm().await.expect("Failed to connect to Postgres");
        self.load_channels().await.expect("Failed to load channels");
        self.load_banned_words().await.expect("Failed to load banned words");
        self.load_urls().await.expect("Failed to load URLs");
        self.load_users().await.expect("Failed to load users");
        self.init_twitch().await.expect("Failed to connect to Twitch");
        self.init_helix().await.expect("Failed to connect to Twitch Helix");


        self.main_loop().await.expect("Main loop failed");

        Ok(())
    }

    /**
     * Main loop
     */
    async fn main_loop(&mut self) -> Result<(), TwitchbotError> {
        let mut event_receiver = self.event_receiver.take().unwrap();

        while let Some(event) = event_receiver.recv().await {
            match event {
                BotEvent::TwitchMessage(message) => {
                    match message {
                        twitch_irc::message::ServerMessage::Privmsg(msg) => self.handle_privmsg(&msg).await,
                        twitch_irc::message::ServerMessage::Join(msg) => self.handle_join(&msg),
                        twitch_irc::message::ServerMessage::Part(msg) => self.handle_part(&msg),
                        twitch_irc::message::ServerMessage::Ping(_) => {
                            debug!("Ping? Pong!");
                        },
                        twitch_irc::message::ServerMessage::Pong(_) => {
                            // debug!("Pong!");
                        },
                        _ => {
                            info!("Received message: {:?}", message);
                        }
                    }
                }
                // Handle other event types here
            }
        }

        Ok(())
    }

    /**
     * Connect to Twitch Helix
     */
    async fn init_helix(&mut self) -> Result<()>{
        let client: HelixClient<reqwest::Client> = HelixClient::default();
        let token: std::result::Result<AppAccessToken, twitch_api::twitch_oauth2::tokens::errors::AppAccessTokenError<twitch_api::client::CompatError<reqwest::Error>>> = AppAccessToken::get_app_access_token(
            &client,
            self.twitch_client_id.to_owned(),
            self.twitch_client_secret.to_owned(),
            vec![],
        ).await;

        if token.is_err() {
            error!("Failed to get token");
            return Err(Report::new(token.err().unwrap()));
        }

        self.helix_client = Some(client);
        self.helix_client_token = token.ok();

        Ok(())
    }

    /**
     * Connect to Twitch
     */
    async fn init_twitch<'a>(&'a mut self) -> Result<()> {
        let config = ClientConfig {
            login_credentials: StaticLoginCredentials::new(
                self.name.to_owned(),
                Some(self.twitch_token.to_owned()),
            ),
            ..Default::default()
        };
        
        let (mut incoming_messages, twitch_client)
            = TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(config);

        let event_sender = self.event_sender.clone().unwrap();

        tokio::spawn(async move {
            while let Some(message) = incoming_messages.recv().await {
                event_sender.send(BotEvent::TwitchMessage(message)).await.unwrap();
            }
        });

        self.twitch_client = Some(twitch_client);

        // Request join info
        let commands = self.twitch_client.as_ref().unwrap().send_message(irc!["CAP", "REQ", "twitch.tv/commands"]);
        let tags = self.twitch_client.as_ref().unwrap().send_message(irc!["CAP", "REQ", "twitch.tv/tags"]);
        let membership = self.twitch_client.as_ref().unwrap().send_message(irc!["CAP", "REQ", "twitch.tv/membership"]);
        try_join!(commands, tags, membership)?;

        // Join to channels
        for channel in self.channels.iter() {
            match self.twitch_client.as_ref().unwrap().join(channel.name.to_lowercase()) {
                Ok(_) => {
                    info!("Joined channel: {}", channel.name);
                },
                Err(e) => {
                    warn!("Failed to join channel: {}", e);
                }
            }
        }
        Ok(())
    }

    /**
     * Connect to Postgres
     */
    async fn init_seaorm(&mut self) -> Result<(), ()> {
        info!("Connecting to Postgres");

        let db = sea_orm::Database::connect(self.database_url.as_str()).await.map_err(|_| ())?;
        self.db = Some(db);

        Ok(())
    }

    /**
     * Load channels from Postgres
     */
    async fn load_channels(&mut self) -> Result<()> {
        info!("Loading channels");

        if let Some(db) = &self.db {
            let channels: Vec<channels::Model> = Channel::find().all(db).await?;
            self.channels = channels;
            info!("Loaded {} channels", self.channels.len());
        } else {
            error!("Database connection not initialized");
        }

        Ok(())
    }

    /**
     * Load banned words from Postgres
     */
    async fn load_banned_words(&mut self) -> Result<()> {
        info!("Loading banned words");
        if let Some(db) = &self.db {
            let banned_words: Vec<banned_words::Model> = BannedWord::find()
                .filter(banned_words::Column::ChannelId.is_null())
                .all(db)
                .await
                .map_err(|e| {
                    error!("Failed to load banned words: {:?}", e);
                    Report::new(e)
                })?;

            // Convert banned words to simple struct with precompiled regex
            self.banned_words = banned_words.into_iter().filter_map(|bw| {
                if bw.is_regex {
                    Some(BannedWordSimple::Word(bw.word.clone()))
                } else {
                    let regex = Regex::new(&bw.word).ok().or_else(|| {
                        warn!("Invalid regex: {}", bw.word);
                        None
                    })?;

                    Some(BannedWordSimple::Regex(regex))
                }
            }).collect();
            info!("Loaded {} banned words", self.banned_words.len());
        } else {
            error!("Database connection not initialized");
        }
        Ok(())
    }

    /**
     * Load URLs from Postgres
     */
    async fn load_urls(&mut self) -> Result<()> {
        info!("Loading URLs");
        if let Some(db) = &self.db {
            let urls: Vec<urls::Model> = Url::find().all(db).await.map_err(|e| {
                error!("Failed to load URLs: {:?}", e);
                Report::new(e)
            })?;
            self.urls = urls.into_iter().map(|url| url.url).collect();
            info!("Loaded {} URLs", self.urls.len());
        } else {
            error!("Database connection not initialized");
        }
        Ok(())
    }

    /**
     * Load users from Postgres
     */
    async fn load_users(&mut self) -> Result<()> {
        info!("Loading users");

        if let Some(db) = &self.db {
            let users: Vec<users::Model> = User::find().all(db).await?;
            let (banned_users, seen_users): (Vec<_>, Vec<_>) = users.into_iter().partition(|user| user.is_bot);
            self.seen_users = seen_users.into_iter().map(|user| user.username.clone()).collect();
            self.banned_users = banned_users.into_iter().map(|user| user.username.clone()).collect();
            info!("Loaded {} seen users and {} banned users ", self.seen_users.len(), self.banned_users.len());
        } else {
            error!("Database connection not initialized");
        }

        Ok(())
    }

    /**
     * Handle a privmsg
     */
    async fn handle_privmsg(&mut self, msg: &twitch_irc::message::PrivmsgMessage) {
        let is_mod = match &msg.source.tags.0.get("mod") {
            Some(Some(n)) if n == "1" => true,
            _ => false,
        };
        let is_vip = match &msg.source.tags.0.get("vip") {
            Some(Some(n)) if n == "1" => true,
            _ => false,
        };
        let from_prefix = if is_mod { "@" } else if is_vip { "+" } else { "" };

        let from = &msg.sender.name;
        let to = &msg.channel_login;

        info!("<{}{} -> #{}>: {}", from_prefix, from, to, msg.message_text);

        // Mark user as seen
        let seen = self.seen_users.contains(from);

        // Check for banned words
        let banned = self.banned_words.iter().any(|bw| {
            match &bw {
                BannedWordSimple::Regex(re) => re.is_match(&msg.message_text),
                BannedWordSimple::Word(word) => msg.message_text.contains(word),
            }
        });

        if banned && !seen && !is_mod && !is_vip {
            info!("Message contains banned words or patterns and user has not been seen before");
            self.ban_user(&msg.sender.id, &msg.channel_id).await;
        } else if !seen {
            self.add_new_user(&from).await;
        }
    }

    /**
     * Add a new user to seen users list and database
     */
    async fn add_new_user(&mut self, from: &str) {
        self.seen_users.insert(from.to_string());
        info!("User {} seen for the first time", from);

        // Add new user to the db
        if let Some(db) = &self.db {
            let user = users::ActiveModel {
                username: Set(from.to_string()),
                is_bot: Set(false),
                ..Default::default()
            };

            let result = User::insert(user).exec(db).await;

            match result {
                Ok(_) => info!("Added user {} to the database", from),
                Err(e) => error!("Failed to add user {} to the database: {:?}", from, e),
            }
        } else {
            error!("Database connection not initialized");
        }
    }
    
    /**
     * Ban a user
     */
    async fn ban_user(&mut self, user: &str, channel: &str) {
        if let Some(client) = &self.helix_client {
            let result = client.ban_user(
                            user,
                            "spam",
                            None,
                            channel,
                            "Banned for using banned words",
                            self.helix_client_token.as_ref().unwrap(),
                        ).await;

            match result {
                Ok(_) => info!("Banned user {} in channel {}", user, channel),
                Err(e) => error!("Failed to ban user {} in channel {}: {:?}", user, channel, e),
            }
        } else {
            error!("Helix client not initialized");
        }
    }

    /**
     * Handle a join
     */
    async fn handle_join(&self, msg: &twitch_irc::message::JoinMessage) {
        info!("{} joined channel #{}", msg.user_login, msg.channel_login);

        // Check if user is a bot
        if self.banned_users.contains(&msg.user_login) {
            info!("{} is a bot", msg.user_login);
            GetUsersRequest::builder()
                .login(vec![msg.user_login.to_string()])
                .build()
                .execute(&self.helix_client, self.helix_client_token.as_ref().unwrap())
                .await
                .map(|response| {
                    if let Some(user) = response.data.first() {
                        if let Some(id) = &user.id {
                            self.ban_user(id, &msg.channel_login);
                        }
                    }
                })
                .unwrap_or_else(|e| {
                    error!("Failed to get user info: {:?}", e);
                });
        }
    }

    /**
     * Handle a part
     */
    fn handle_part(&self, msg: &twitch_irc::message::PartMessage) {
        info!("{} left channel #{}", msg.user_login, msg.channel_login);
    }
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult};

    async fn setup_mock_db() -> DatabaseConnection {
        MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results(vec![vec![channels::Model {
                id: 1,
                name: "test_channel".to_string(),
                settings: serde_json::json!({}),
                created_at: chrono::Utc::now().with_timezone(&chrono::FixedOffset::east_opt(0).unwrap()),
                updated_at: chrono::Utc::now().with_timezone(&chrono::FixedOffset::east_opt(0).unwrap()),
            }]])
            .append_query_results(vec![vec![banned_words::Model {
                id: 1,
                word: "test_word".to_string(),
                channel_id: None,
                is_regex: false,
                created_at: chrono::Utc::now().with_timezone(&chrono::FixedOffset::east_opt(0).unwrap()),
                updated_at: chrono::Utc::now().with_timezone(&chrono::FixedOffset::east_opt(0).unwrap()),
            }]])
            .append_exec_results(vec![MockExecResult {
                last_insert_id: 1,
                rows_affected: 1,
            }])
            .into_connection()
    }

    #[tokio::test]
    async fn test_init_seaorm() {
        let opts = Opts {
            twitch_username: "test_user".to_string(),
            twitch_token: "test_token".to_string(),
            twitch_client_id: twitch_api::twitch_oauth2::ClientId::new("test_client_id".to_string()),
            twitch_client_secret: twitch_api::twitch_oauth2::ClientSecret::new("test_client_secret".to_string()),
            database_url: "postgres://user:password@localhost/test_db".to_string(),
        };
        let mut bot = Bot::new(opts);
        bot.db = Some(setup_mock_db().await);
        assert!(bot.db.is_some());
    }

    #[tokio::test]
    async fn test_load_channels() {
        let opts = Opts {
            twitch_username: "test_user".to_string(),
            twitch_token: "test_token".to_string(),
            twitch_client_id: twitch_api::twitch_oauth2::ClientId::new("test_client_id".to_string()),
            twitch_client_secret: twitch_api::twitch_oauth2::ClientSecret::new("test_client_secret".to_string()),
            database_url: "postgres://user:password@localhost/test_db".to_string(),
        };
        let mut bot = Bot::new(opts);
        bot.db = Some(setup_mock_db().await);
        assert!(bot.load_channels().await.is_ok());
    }

    #[tokio::test]
    async fn test_load_banned_words() {
        let opts = Opts {
            twitch_username: "test_user".to_string(),
            twitch_token: "test_token".to_string(),
            twitch_client_id: twitch_api::twitch_oauth2::ClientId::new("test_client_id".to_string()),
            twitch_client_secret: twitch_api::twitch_oauth2::ClientSecret::new("test_client_secret".to_string()),
            database_url: "postgres://user:password@localhost/test_db".to_string(),
        };
        let mut bot = Bot::new(opts);
        bot.db = Some(setup_mock_db().await);
        assert!(bot.load_banned_words().await.is_ok());
    }

    #[tokio::test]
    async fn test_load_urls() {
        let opts = Opts {
            twitch_username: "test_user".to_string(),
            twitch_token: "test_token".to_string(),
            twitch_client_id: twitch_api::twitch_oauth2::ClientId::new("test_client_id".to_string()),
            twitch_client_secret: twitch_api::twitch_oauth2::ClientSecret::new("test_client_secret".to_string()),
            database_url: "postgres://user:password@localhost/test_db".to_string(),
        };
        let mut bot = Bot::new(opts);
        bot.db = Some(setup_mock_db().await);
        assert!(bot.load_urls().await.is_ok());
    }

    #[tokio::test]
    async fn test_load_users() {
        let opts = Opts {
            twitch_username: "test_user".to_string(),
            twitch_token: "test_token".to_string(),
            twitch_client_id: twitch_api::twitch_oauth2::ClientId::new("test_client_id".to_string()),
            twitch_client_secret: twitch_api::twitch_oauth2::ClientSecret::new("test_client_secret".to_string()),
            database_url: "postgres://user:password@localhost/test_db".to_string(),
        };
        let mut bot = Bot::new(opts);
        bot.db = Some(setup_mock_db().await);
        assert!(bot.load_users().await.is_ok());
    }
}
