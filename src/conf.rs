pub(crate) mod config {
    use std::fs::File;
    use std::io::Read;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Deserialize, Serialize)]
    pub struct DB {
        #[serde(rename = "type")]
        pub db_type: String,
        pub host: String,
        pub port: u16,
        pub user: String,
        pub password: String,
        pub database: String,
    }
    #[derive(Debug, Deserialize, Serialize)]
    pub struct Config {
        pub db_src: DB,
        pub db_dst: DB,
        pub tb_only: Vec<String>,
        pub tb_ignore: Vec<String>,
    }
    impl Config {
        pub fn new() -> Self {
            let dir = std::env::current_dir().unwrap().to_str().unwrap().to_string() + "/";
            let mut file = File::open(dir + "config.yml").unwrap();
            let mut content = String::new();
            file.read_to_string(&mut content).unwrap();
            serde_yaml::from_str(&content).unwrap()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn test_config() {
            let config = Config::new();
            println!("{:?}", config);
        }
    }
}