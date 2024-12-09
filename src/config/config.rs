// add configs here
#[derive(Debug, Deserialize)]
pub struct Config {
    pub server: Server,
    pub database: Database,
}
