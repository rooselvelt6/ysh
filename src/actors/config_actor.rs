use ractor::{async_trait, Actor, ActorProcessingErr, ActorRef};

pub struct ConfigActor;

pub struct ConfigActorState {
    pub config_path: String,
    pub lua_engine: crate::config::lua_engine::LuaEngine,
}

#[async_trait]
impl Actor for ConfigActor {
    type Msg = ConfigActorMsg;
    type State = ConfigActorState;
    type Arguments = String;

    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        config_path: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        let lua_engine = crate::config::lua_engine::LuaEngine::new()
            .map_err(|e| ActorProcessingErr::from(e.to_string()))?;
        tracing::info!("ConfigActor started, watching: {}", config_path);
        Ok(ConfigActorState {
            config_path,
            lua_engine,
        })
    }

    async fn handle(
        &self,
        _myself: ActorRef<Self::Msg>,
        msg: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match msg {
            ConfigActorMsg::Reload => {
                tracing::info!("Reloading config from: {}", state.config_path);
                match state.lua_engine.reload(&state.config_path) {
                    Ok(_config) => tracing::info!("Config reloaded successfully"),
                    Err(e) => tracing::error!("Config reload failed: {}", e),
                }
            }
            ConfigActorMsg::ConfigChanged(path) => {
                let content = std::fs::read(&path).unwrap_or_default();
                let hash = crate::security::password::hash_blake3(&content);
                tracing::info!("Config changed: {} (blake3: {})", path, hash);
                if crate::security::password::verify_blake3(&content, &hash.to_hex())
                    .unwrap_or(false)
                {
                    tracing::info!("Config integrity verified");
                }
            }
        }
        Ok(())
    }
}

pub enum ConfigActorMsg {
    Reload,
    ConfigChanged(String),
}
