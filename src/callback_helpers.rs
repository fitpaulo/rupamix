use pulse::context::introspect::ServerInfo;
use std::borrow::Cow;

pub fn copy_server<'a>(sref: &'a ServerInfo) -> ServerInfo<'a> {
    let user_name = Some(Cow::from(sref.user_name.as_ref().unwrap().clone()));
    let host_name = Some(Cow::from(sref.host_name.as_ref().unwrap().clone()));
    let server_version = Some(Cow::from(sref.server_version.as_ref().unwrap().clone()));
    let server_name = Some(Cow::from(sref.server_name.as_ref().unwrap().clone()));
    let sample_spec = sref.sample_spec.clone();
    let default_sink_name = Some(Cow::from(sref.default_sink_name.as_ref().unwrap().clone()));
    let default_source_name = Some(Cow::from(
        sref.default_source_name.as_ref().unwrap().clone(),
    ));
    let cookie = sref.cookie;
    let channel_map = sref.channel_map.clone();
    ServerInfo {
        user_name,
        host_name,
        server_version,
        server_name,
        sample_spec,
        default_sink_name,
        default_source_name,
        cookie,
        channel_map,
    }
}
