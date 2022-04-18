// Sets your background to a random (or specifiec by index) picture from a WallHaven
// collection.
//
// You'll need a ~/.config/wallhaven.json file, it needs to looklike
// {
//   "file_location": "/.wallpaper"   from $HOME directory on
//   "username": "",
//   "api_key": "",
//   "collection_id": "",
//   "filepath": "https://.....",
//   "current_selection": 0,
// }
//
// collections are specified by username/collection_id. If it is your own
// private collection, then you will also need to specify an api_key to have
// permissions to access it. If it is a public collection, then you shold be
// able to leave the API key blank

use error_chain::error_chain;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::Read;
use std::process::Command;

error_chain! {
    foreign_links {
        Io(std::io::Error);
        HttpRequest(reqwest::Error);
        Json(serde_json::Error);
        Parse(std::num::ParseIntError);
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct WpConfig {
    file_location: String,
    username: String,
    api_key: String,
    collection_id: String,
    current_selection: usize,
    filepath: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct WpResp {
    meta: WpMeta,
    data: Vec<WpData>,
}

#[derive(Debug, Deserialize, Serialize)]
struct WpMeta {
    per_page: usize,
    total: usize,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct WpData {
    path: String,
}

impl WpResp {}

impl WpConfig {
    fn parse_from_file(file_location: &str) -> Result<WpConfig> {
        let mut f = File::open(file_location)?;
        let mut input_str: String = String::new();
        f.read_to_string(&mut input_str)?;
        let config: WpConfig = serde_json::from_str(&input_str)?;
        Ok(config)
    }

    fn get_first(&self) -> Result<WpResp> {
        self.get_at_page(0)
    }

    fn get_at_page(&self, page: usize) -> Result<WpResp> {
        let url = format!(
            "https://wallhaven.cc/api/v1/collections/{}/{}/?apikey={}&page={}",
            self.username, self.collection_id, self.api_key, page
        );
        let mut res = reqwest::blocking::get(&url)?;
        let mut body = String::new();
        res.read_to_string(&mut body)?;
        let json: WpResp = serde_json::from_str(&body)?;
        Ok(json)
    }

    fn get_data_at_index_after_first_call(&mut self, index: usize, first: WpMeta) -> Result<()> {
        let page = 1 + index / first.per_page;
        let new_index = index % first.per_page;
        let next = self.get_at_page(page)?;
        self.current_selection = index;
        self.filepath = next.data[new_index].path.clone();
        Ok(())
    }

    fn get_data_at_index(&mut self, index: usize) -> Result<()> {
        let first = self.get_first()?;
        self.get_data_at_index_after_first_call(index, first.meta)
    }

    fn get_random_data(&mut self) -> Result<()> {
        let first = self.get_first()?;
        let index = rand::random::<usize>() % first.meta.total;
        self.get_data_at_index_after_first_call(index, first.meta)
    }

    fn set_wallpaper(&self) -> Result<()> {
        let data = reqwest::blocking::get(&self.filepath)?.bytes()?;
        let home = get_config_folder();
        let file_location = home + &self.file_location;
        let mut f = File::create(&file_location)?;
        f.write_all(&data)?;
        Command::new("feh")
            .args(["--bg-fill", &file_location])
            .output()?;

        Ok(())
    }
    fn get_data_based_on_args(&mut self) -> Result<()> {
        let args: Vec<String> = env::args().collect();
        if args.len() <= 1 {
            self.get_random_data()
        } else {
            self.get_data_at_index(args[1].parse::<usize>()?)
        }
    }

    fn write(&self, file_location: &str) -> Result<()> {
        let mut f = File::create(&file_location)?;
        f.write_all(serde_json::to_string(&self)?.as_bytes())?;
        Ok(())
    }
}

fn get_home_folder() -> String {
    match env::var("HOME") {
        Ok(val) => val + "/.config",
        Err(_) => "/home/miek".to_string(),
    }
}

fn get_config_folder() -> String {
    match env::var("XDG_CONFIG_HOME") {
        Ok(val) => val,
        Err(_) => get_home_folder(),
    }
}

fn main() -> Result<()> {
    let config_file_path = get_config_folder() + "/wallhaven.json";
    let mut config_obj = WpConfig::parse_from_file(&config_file_path)?;
    config_obj.get_data_based_on_args()?;
    config_obj.set_wallpaper()?;
    config_obj.write(&config_file_path)?;
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::WpConfig;

    #[test]
    fn test_parse_file() {
        // Test parsing the json file for what we need
        let config = WpConfig::parse_from_file("./testwallhaven.json").unwrap();
        assert_eq!(config.api_key, "test-api-key");
        assert_eq!(config.username, "test-user-name");
        assert_eq!(config.file_location, "/.wallpaper");
        assert_eq!(config.collection_id, "test-collection-id");
    }
}
