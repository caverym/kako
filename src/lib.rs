use futures::stream::StreamExt;
use std::error::Error;
use std::fmt::{Display, Formatter};
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_gateway::cluster::Events;
use twilight_gateway::{
    cluster::{Cluster, ShardScheme},
    Event,
};
use twilight_http::{Client as HttpClient, Client};
use twilight_model::gateway::Intents;
use twilight_model::id::{ChannelId, UserId};

pub type Er = Box<dyn Error + Send + Sync>;

pub struct Bot {
    name: String,
    token: String,
    owner: Option<UserId>,
    shard: u64,
    cache: InMemoryCache,
    events: Events,
    http: HttpClient,
}

impl Bot {
    pub async fn connect(
        token: String,
        owner: Option<UserId>,
    ) -> Result<Bot, Box<dyn Error + Send + Sync>> {
        let name: String = String::new();
        let shard: u64 = 0;
        let scheme: ShardScheme = ShardScheme::Auto;
        let (cluster, events) = Cluster::builder(token.clone(), Intents::GUILD_MESSAGES)
            .shard_scheme(scheme)
            .build()
            .await?;

        tokio::spawn(async move {
            cluster.up().await;
        });

        let http: Client = HttpClient::new(token.clone());
        let cache: InMemoryCache = InMemoryCache::builder()
            .resource_types(ResourceType::MESSAGE)
            .build();

        Ok(Self {
            name,
            token,
            owner,
            shard,
            cache,
            events,
            http,
        })
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name
    }

    pub async fn run(&mut self) -> Result<Event, Er> {
        if let Some((shard, event)) = self.events.next().await {
            self.shard = shard;
            self.cache.update(&event);
            Ok(event)
        } else {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "failed to wait for event",
            )))
        }
    }

    pub async fn create_message<T: ToString>(
        &self,
        channel: ChannelId,
        content: T,
    ) -> Result<(), Er> {
        let content: String = content.to_string();

        self.http
            .create_message(channel)
            .content(&content)?
            .exec()
            .await?;

        Ok(())
    }

    pub fn owner(&self) -> Option<UserId> {
        self.owner
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub async fn token(&mut self) -> String {
        self.token.clone()
    }
}

impl Display for Bot {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
