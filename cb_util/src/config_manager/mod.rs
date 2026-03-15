use kay::{World, Actor, TypedID};
use compact::{CHashMap, COption, CString, Compact};
use arrayvec::ArrayString;
use serde::de::DeserializeOwned;

pub type Name = ArrayString<[u8; 16]>;
pub trait Config: Compact + 'static {}

#[derive(Compact, Clone)]
pub struct ConfigManager<C: Config> {
    id: ConfigManagerID<C>,
    entries: CHashMap<Name, C>,
}

impl<C: Config> ConfigManager<C> {
    pub fn spawn(
        id: ConfigManagerID<C>,
        initial_entries: &CHashMap<Name, C>,
        _: &mut World,
    ) -> ConfigManager<C> {
        ConfigManager {
            id,
            entries: initial_entries.clone(),
        }
    }

    fn apply_entry_update(&mut self, name: Name, maybe_value: &COption<C>) {
        if let COption(Some(ref value)) = *maybe_value {
            self.entries.insert(name, value.clone());
        } else {
            self.entries.remove(name);
        }
    }

    fn replace_all_entries(&mut self, entries: &CHashMap<Name, C>) {
        self.entries = entries.clone();
    }

    pub fn request_current(&self, requester: ConfigUserID<C>, world: &mut World) {
        for (name, value) in self.entries.pairs() {
            requester.on_config_change(*name, COption(Some(value.clone())), world);
        }
    }

    pub fn update_entry(&mut self, name: Name, maybe_value: &COption<C>, world: &mut World) {
        if let COption(Some(ref value)) = *maybe_value {
            ConfigUserID::<C>::global_broadcast(world).on_config_change(
                name,
                COption(Some(value.clone())),
                world,
            );
        } else {
            ConfigUserID::<C>::global_broadcast(world).on_config_change(name, COption(None), world);
        }
        self.apply_entry_update(name, maybe_value);
    }

    pub fn update_all_entries(&mut self, entries: &CHashMap<Name, C>, world: &mut World) {
        // TODO: handle disappearing entries?
        self.replace_all_entries(entries);
        for (name, value) in self.entries.pairs() {
            ConfigUserID::<C>::global_broadcast(world).on_config_change(
                name.clone(),
                COption(Some(value.clone())),
                world,
            );
        }
    }
}

use std::{
    collections::HashMap,
    fs::File,
    io::BufReader,
};
use cb_time::actors::{Temporal, TemporalID};
use cb_time::units::{Instant};

#[derive(Compact, Clone)]
pub struct ConfigFileWatcher<CD: Config + DeserializeOwned> {
    id: ConfigFileWatcherID<CD>,
    target: ConfigManagerID<CD>,
    file: CString,
    loaded_once: bool,
}

impl<CD: Config + DeserializeOwned> ConfigFileWatcher<CD> {
    pub fn spawn(
        id: ConfigFileWatcherID<CD>,
        target: ConfigManagerID<CD>,
        file: &CString,
        _: &mut World,
    ) -> ConfigFileWatcher<CD> {
        ConfigFileWatcher {
            id,
            target,
            file: file.clone(),
            loaded_once: false,
        }
    }

    #[cfg(feature = "server")]
    fn read_entries(&self) -> CHashMap<Name, CD> {
        let file = File::open(&*(self.file))
            .expect(&format!("Couldn't find config file {:?}", &*self.file));
        let reader = BufReader::new(file);
        let new_entries: HashMap<Name, CD> =
            serde_yaml::from_reader(reader).expect("parsing failed");
        new_entries.into_iter().collect()
    }

    pub fn reload(&mut self, world: &mut World) {
        #[cfg(feature = "server")]
        {
            self.target.update_all_entries(self.read_entries(), world);
        }
    }

    fn load_if_needed<F>(&mut self, load: F) -> bool
    where
        F: FnOnce(&mut Self),
    {
        if self.loaded_once {
            return false;
        }

        load(self);
        self.loaded_once = true;
        true
    }
}

impl<CD: Config + DeserializeOwned> Temporal for ConfigFileWatcher<CD> {
    fn tick(&mut self, _dt: f32, _current_instant: Instant, world: &mut World) {
        #[cfg(feature = "server")]
        {
            self.load_if_needed(|watcher| watcher.reload(world));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use compact::{CHashMap, COption};
    use kay::{Actor, External, RawID, TypedID, World};

    #[derive(Compact, Clone, Deserialize)]
    struct TestConfig {
        value: u32,
    }

    impl Config for TestConfig {}

    fn raw_id(hex_type: &str) -> RawID {
        format!("{}_0.0@0", hex_type).parse().unwrap()
    }

    fn test_name(name: &str) -> Name {
        ArrayString::from(name).expect("test name too long")
    }

    fn test_config(value: u32) -> TestConfig {
        TestConfig { value }
    }

    fn test_entries(entries: &[(&str, u32)]) -> CHashMap<Name, TestConfig> {
        let mut map = CHashMap::new();
        for (name, value) in entries {
            map.insert(test_name(name), test_config(*value));
        }
        map
    }

    fn test_watcher(file: &str) -> ConfigFileWatcher<TestConfig> {
        ConfigFileWatcher {
            id: ConfigFileWatcherID::from_raw(raw_id("1")),
            target: ConfigManagerID::from_raw(raw_id("2")),
            file: file.to_owned().into(),
            loaded_once: false,
        }
    }

    unsafe fn inert_world() -> World {
        std::mem::zeroed()
    }

    fn temp_config_file(contents: &str) -> (std::path::PathBuf, std::path::PathBuf) {
        let temp_dir = std::env::temp_dir().join(format!(
            "citybound-config-manager-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&temp_dir).unwrap();
        let file_path = temp_dir.join("config.yaml");
        std::fs::write(&file_path, contents).unwrap();
        (temp_dir, file_path)
    }

    #[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
    struct LocalConfigUserID {
        raw_id: RawID,
    }

    impl TypedID for LocalConfigUserID {
        type Target = LocalConfigUser;

        fn from_raw(id: RawID) -> Self {
            LocalConfigUserID { raw_id: id }
        }

        fn as_raw(&self) -> RawID {
            self.raw_id
        }
    }

    #[derive(Compact, Clone)]
    struct LocalConfigUser {
        id: External<LocalConfigUserID>,
        cache: CHashMap<Name, TestConfig>,
    }

    fn local_config_user() -> LocalConfigUser {
        LocalConfigUser {
            id: External::new(LocalConfigUserID::from_raw(raw_id("5"))),
            cache: CHashMap::new(),
        }
    }

    impl Actor for LocalConfigUser {
        type ID = LocalConfigUserID;

        fn id(&self) -> Self::ID {
            *self.id
        }

        unsafe fn set_id(&mut self, id: RawID) {
            *self.id = LocalConfigUserID::from_raw(id);
        }
    }

    impl ConfigUser<TestConfig> for LocalConfigUser {
        fn local_cache(&mut self) -> &mut CHashMap<Name, TestConfig> {
            &mut self.cache
        }
    }

    #[test]
    fn load_if_needed_runs_only_once() {
        let mut watcher = test_watcher("test-config.yaml");
        let mut reloads = 0;

        assert!(watcher.load_if_needed(|_| reloads += 1));
        assert_eq!(reloads, 1);
        assert!(watcher.loaded_once);

        assert!(!watcher.load_if_needed(|_| reloads += 1));
        assert_eq!(reloads, 1);
    }

    #[test]
    fn new_watchers_start_unloaded() {
        let watcher = test_watcher("test-config.yaml");

        assert!(!watcher.loaded_once);
    }

    #[test]
    fn manager_spawn_clones_initial_entries() {
        let initial_entries = test_entries(&[("alpha", 1)]);
        let mut world = unsafe { inert_world() };

        let manager = ConfigManager::spawn(
            ConfigManagerID::from_raw(raw_id("4")),
            &initial_entries,
            &mut world,
        );

        assert_eq!(manager.entries.get(test_name("alpha")).unwrap().value, 1);
    }

    #[test]
    fn apply_entry_update_inserts_and_removes_entries() {
        let mut manager = ConfigManager {
            id: ConfigManagerID::from_raw(raw_id("4")),
            entries: CHashMap::new(),
        };

        manager.apply_entry_update(
            test_name("alpha"),
            &COption(Some(test_config(7))),
        );
        assert_eq!(manager.entries.get(test_name("alpha")).unwrap().value, 7);

        manager.apply_entry_update(test_name("alpha"), &COption(None));
        assert!(manager.entries.get(test_name("alpha")).is_none());
    }

    #[test]
    fn replace_all_entries_replaces_existing_entries() {
        let mut manager = ConfigManager {
            id: ConfigManagerID::from_raw(raw_id("4")),
            entries: test_entries(&[("old", 1)]),
        };

        manager.replace_all_entries(&test_entries(&[("alpha", 2), ("beta", 3)]));

        assert!(manager.entries.get(test_name("old")).is_none());
        assert_eq!(manager.entries.get(test_name("alpha")).unwrap().value, 2);
        assert_eq!(manager.entries.get(test_name("beta")).unwrap().value, 3);
    }

    #[test]
    fn config_user_apply_and_on_change_update_local_cache() {
        let mut user = local_config_user();
        let mut world = unsafe { inert_world() };

        <LocalConfigUser as ConfigUser<TestConfig>>::apply_config_change(
            &mut user,
            test_name("alpha"),
            &COption(Some(test_config(9))),
            &mut world,
        );
        assert_eq!(user.cache.get(test_name("alpha")).unwrap().value, 9);

        <LocalConfigUser as ConfigUser<TestConfig>>::on_config_change(
            &mut user,
            test_name("beta"),
            &COption(Some(test_config(11))),
            &mut world,
        );
        assert_eq!(user.cache.get(test_name("beta")).unwrap().value, 11);

        <LocalConfigUser as ConfigUser<TestConfig>>::apply_config_change(
            &mut user,
            test_name("alpha"),
            &COption(None),
            &mut world,
        );
        assert!(user.cache.get(test_name("alpha")).is_none());
    }

    #[cfg(feature = "server")]
    #[test]
    fn read_entries_parses_yaml_file() {
        let (temp_dir, file_path) = temp_config_file("alpha:\n  value: 11\nbeta:\n  value: 13\n");
        let watcher = test_watcher(file_path.to_string_lossy().as_ref());
        let entries = watcher.read_entries();

        assert_eq!(entries.get(test_name("alpha")).unwrap().value, 11);
        assert_eq!(entries.get(test_name("beta")).unwrap().value, 13);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}

pub trait ConfigUser<C: Config>: Actor {
    fn local_cache(&mut self) -> &mut CHashMap<Name, C>;
    fn apply_config_change(&mut self, name: Name, maybe_value: &COption<C>, _: &mut World) {
        if let COption(Some(ref value)) = *maybe_value {
            self.local_cache().insert(name, value.clone());
        } else {
            self.local_cache().remove(name);
        }
    }
    fn on_config_change(&mut self, name: Name, maybe_value: &COption<C>, world: &mut World) {
        self.apply_config_change(name, maybe_value, world);
    }
    fn get_initial_config(&self, world: &mut World) {
        ConfigManagerID::<C>::global_first(world).request_current(self.id_as(), world);
    }
}

mod kay_auto;
pub use self::kay_auto::*;
