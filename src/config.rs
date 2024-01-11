use std::collections::HashMap;

use configparser::ini::Ini;
use homedir::get_my_home;

#[derive(Default)]
struct Config {
    core: CoreConfig,
}
impl Config {
    fn from_ini(ini: Ini) -> Self {
        let mut config = Config::default();
        if let Some(ini_config) = ini.get_map() {
            if let Some(hashmap) = ini_config.get("core") {
                config.core = CoreConfig::from_hashmap(hashmap.clone());
            }
        };
        config
    }
    /// Helper method to get the user's system wide config, returns default if it fails to find it
    fn get_system_config() -> Self {
        if let Ok(Some(user_home)) = get_my_home() {
            let mut config_path = user_home.clone();
            config_path.push(".ritconfig");
            let mut ini = Ini::new();
            if ini.load(config_path).is_err() {
                return Self::default();
            };
            Self::from_ini(ini)
        } else {
            Self::default()
        }
    }
    fn merge(&self, other: &Self) -> Self {
        Self {
            core: other.core.clone(),
            ..*self
        }
    }
}

#[derive(Clone, Default)]
struct CoreConfig {
    repositoryformatversion: u8,
}
impl CoreConfig {
    fn from_hashmap(hashmap: HashMap<String, Option<String>>) -> Self {
        let mut config = Self::default();
        if let Some(Some(val)) = hashmap.get("repositoryformatversion") {
            config.repositoryformatversion = val.parse::<u8>().unwrap();
        }
        config
    }
}
