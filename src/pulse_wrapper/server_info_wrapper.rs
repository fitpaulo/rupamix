/// A simple wrapper aound ServerInfo. Since our only access to ServerInfo is through a ref in a callback,
/// We will just make this object to store the data we want
use pulse::context::introspect::ServerInfo;

#[derive(Default)]
pub struct PulseServerInfo {
    pub default_source_name: String,
    pub default_sink_name: String,
}

impl PulseServerInfo {
    pub fn update(&mut self, info: &'_ ServerInfo<'_>) {
        self.default_source_name = info.default_source_name.as_ref().unwrap().to_string();
        self.default_sink_name = info.default_sink_name.as_ref().unwrap().to_string();
    }
}

impl From<&'_ ServerInfo<'_>> for PulseServerInfo {
    fn from(info: &ServerInfo) -> Self {
        let default_source_name = info.default_source_name.as_ref().unwrap().to_string();
        let default_sink_name = info.default_sink_name.as_ref().unwrap().to_string();

        PulseServerInfo {
            default_sink_name,
            default_source_name,
        }
    }
}
