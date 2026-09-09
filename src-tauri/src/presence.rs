use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};

const JOIN_DISCORD_URL: &str = "https://discord.gg/atWfHfwjYy";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PresenceState {
    Browsing,
    Preparing,
    Downloading(String),
    Launching(String),
    Playing(String),
}

impl PresenceState {
    fn label(&self) -> String {
        match self {
            Self::Browsing => "Browsing Flint".to_string(),
            Self::Preparing => "Preparing Minecraft".to_string(),
            Self::Downloading(version) => format!("Downloading Minecraft {version}"),
            Self::Launching(version) => format!("Launching Minecraft {version}"),
            Self::Playing(version) => format!("Playing Minecraft {version}"),
        }
    }
}

pub struct Presence {
    client: Option<DiscordIpcClient>,
    connection_attempted: bool,
    started_at: i64,
}

impl Presence {
    pub fn new() -> Self {
        Self {
            client: None,
            connection_attempted: false,
            started_at: chrono::Utc::now().timestamp(),
        }
    }

    pub fn update(&mut self, enabled: bool, state: PresenceState) {
        if !enabled {
            if let Some(mut client) = self.client.take() {
                let _ = client.close();
            }
            self.connection_attempted = false;
            return;
        }
        let Some(client_id) = option_env!("FLINT_DISCORD_CLIENT_ID") else {
            if !self.connection_attempted {
                tracing::warn!(
                    "Discord presence is enabled but FLINT_DISCORD_CLIENT_ID was not built into Flint"
                );
                self.connection_attempted = true;
            }
            return;
        };
        if !client_id
            .chars()
            .all(|character| character.is_ascii_digit())
            || client_id.is_empty()
        {
            if !self.connection_attempted {
                tracing::warn!("Discord presence Application ID is invalid");
                self.connection_attempted = true;
            }
            return;
        }
        if self.client.is_none() {
            if self.connection_attempted {
                return;
            }
            self.connection_attempted = true;
            let mut client = DiscordIpcClient::new(client_id);
            if let Err(error) = client.connect() {
                tracing::warn!(%error, "Discord RPC is unavailable; presence is disabled for this session");
                return;
            }
            self.client = Some(client);
        }
        let state_label = state.label();
        let activity = activity::Activity::new()
            .details("Flint Launcher")
            .state(&state_label)
            .timestamps(activity::Timestamps::new().start(self.started_at))
            .assets(
                activity::Assets::new()
                    .large_image("flint")
                    .large_text("Flint Launcher"),
            )
            .buttons(vec![activity::Button::new(
                "Join Discord",
                JOIN_DISCORD_URL,
            )]);
        if let Some(client) = &mut self.client {
            if let Err(error) = client.set_activity(activity) {
                tracing::warn!(%error, "Discord presence update failed; presence is disabled for this session");
                let mut failed_client = self.client.take().expect("presence client existed");
                let _ = failed_client.close();
            }
        }
    }
}

impl Default for Presence {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{Presence, PresenceState};

    #[test]
    fn activity_labels_are_privacy_safe_and_product_consistent() {
        assert_eq!(PresenceState::Browsing.label(), "Browsing Flint");
        assert_eq!(
            PresenceState::Downloading("26.2".into()).label(),
            "Downloading Minecraft 26.2"
        );
        assert_eq!(
            PresenceState::Playing("26.2".into()).label(),
            "Playing Minecraft 26.2"
        );
    }

    #[test]
    fn disabled_or_unconfigured_presence_is_non_blocking() {
        let mut presence = Presence::new();
        presence.update(false, PresenceState::Browsing);
        if option_env!("FLINT_DISCORD_CLIENT_ID").is_none() {
            presence.update(true, PresenceState::Preparing);
        }
    }
}
