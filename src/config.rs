use std::collections::HashMap;

use configparser::ini::Ini;
use homedir::get_my_home;

#[derive(Default)]
pub struct Config {
    pub core: CoreConfig,
    pub user: UserConfig,
}
impl Config {
    pub fn from_ini(ini: Ini) -> Self {
        let mut config = Config::default();
        if let Some(ini_config) = ini.get_map() {
            if let Some(hashmap) = ini_config.get("core") {
                config.core = CoreConfig::from_hashmap(hashmap.clone());
            }
            if let Some(hashmap) = ini_config.get("user") {
                config.user = UserConfig::from_hashmap(hashmap.clone());
            }
        };
        config
    }
    /// Creates a [configparser::ini::Ini] from the current Config
    pub fn to_ini(&self) -> Ini {
        let mut ini = Ini::new();
        for (k, v) in self.core.to_hashmap() {
            ini.set("core", k, Some(v));
        }
        for (k, v) in self.user.to_hashmap() {
            ini.set("user", k, Some(v));
        }
        ini
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
}

#[derive(Clone, Default)]
pub struct CoreConfig {
    pub repositoryformatversion: u8,
    pub filemode: bool,
    pub bare: bool,
}
impl CoreConfig {
    fn from_hashmap(hashmap: HashMap<String, Option<String>>) -> Self {
        let mut config = Self::default();
        if let Some(Some(val)) = hashmap.get("repositoryformatversion") {
            config.repositoryformatversion = val.parse::<u8>().unwrap();
        }
        if let Some(Some(val)) = hashmap.get("filemode") {
            config.filemode = val.parse::<bool>().unwrap();
        }
        if let Some(Some(val)) = hashmap.get("bare") {
            config.bare = val.parse::<bool>().unwrap();
        }
        config
    }
    fn to_hashmap(&self) -> HashMap<&str, String> {
        let mut hm = HashMap::new();
        hm.insert(
            "repositoryformatversion",
            self.repositoryformatversion.to_string(),
        );
        hm.insert("filemode", self.filemode.to_string());
        hm.insert("bare", self.bare.to_string());
        hm
    }
}

#[derive(Clone)]
pub struct UserConfig {
    pub name: String,
    pub email: String,
}
impl UserConfig {
    fn from_hashmap(hashmap: HashMap<String, Option<String>>) -> Self {
        let mut config = Self::default();
        if let Some(Some(val)) = hashmap.get("name") {
            config.name = val.to_owned();
        }
        if let Some(Some(val)) = hashmap.get("email") {
            config.email = val.to_owned();
        }
        config
    }
    fn to_hashmap(&self) -> HashMap<&str, String> {
        let mut hm = HashMap::new();
        hm.insert("name", self.name.clone());
        hm.insert("email", self.email.clone());
        hm
    }
}
impl Default for UserConfig {
    fn default() -> Self {
        Self {
            name: String::from("username"),
            email: String::from("username@test.com"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_to_ini() {
        let config = Config::default();
        let ini = config.to_ini();
        assert_eq!(
            ini.get("core", "repositoryformatversion"),
            Some(String::from("0"))
        );
    }
}
