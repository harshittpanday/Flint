use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};

pub struct Presence {
    client: Option<DiscordIpcClient>,
    started_at: i64,
}

impl Presence {
    pub fn new() -> Self {
        Self {
            client: None,
            started_at: chrono::Utc::now().timestamp(),
        }
    }

    pub fn update(&mut self, enabled: bool, details: &str, state: &str) {
        if !enabled {
            if let Some(mut client) = self.client.take() {
                let _ = client.close();
            }
            return;
        }
        let Some(client_id) = option_env!("FLINT_DISCORD_CLIENT_ID") else {
            tracing::debug!("Discord presence is enabled but no client ID was built into Flint");
            return;
        };
        if self.client.is_none() {
            let mut client = DiscordIpcClient::new(client_id);
            if let Err(error) = client.connect() {
                tracing::warn!(%error, "Discord RPC is unavailable");
                return;
            }
            self.client = Some(client);
        }
        let activity = activity::Activity::new()
            .details(details)
            .state(state)
            .timestamps(activity::Timestamps::new().start(self.started_at))
            .assets(
                activity::Assets::new()
                    .large_image("flint")
                    .large_text("Flint Launcher"),
            )
            .buttons(vec![activity::Button::new(
                "Join Discord",
                "https://discord.gg/atWfHfwjYy",
            )]);
        if let Some(client) = &mut self.client {
            if let Err(error) = client.set_activity(activity) {
                tracing::warn!(%error, "Discord presence update failed");
            }
        }
    }
}

impl Default for Presence {
    fn default() -> Self {
        Self::new()
    }
}
