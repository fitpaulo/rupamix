/// A simple wrapper aound ServerInfo. Since our only access to ServerInfo is through a ref in a callback,
/// We will just make this object to store the data we want
use pulse::channelmap::Map;
use pulse::context::introspect::ServerInfo;

pub struct MyServerInfo {
    pub user_name: String,
    pub host_name: String,
    pub server_version: String,
    pub server_name: String,
    pub default_sink_name: String,
    pub default_source_name: String,
    pub cookie: u32,
    pub channel_map: Map,
}

impl MyServerInfo {
    pub fn new(info: &ServerInfo) -> MyServerInfo {
        let default_source_name = info.default_source_name.as_ref().unwrap().to_string();
        let default_sink_name = info.default_sink_name.as_ref().unwrap().to_string();
        let server_name = info.server_name.as_ref().unwrap().to_string();
        let server_version = info.server_version.as_ref().unwrap().to_string();
        let user_name = info.user_name.as_ref().unwrap().to_string();
        let host_name = info.host_name.as_ref().unwrap().to_string();
        let cookie = info.cookie;
        let channel_map = info.channel_map;

        MyServerInfo {
            default_sink_name,
            default_source_name,
            server_version,
            server_name,
            user_name,
            host_name,
            cookie,
            channel_map,
        }
    }
}
