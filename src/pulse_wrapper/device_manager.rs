use pulse::context::introspect::{SinkInfo, SourceInfo};

use crate::pulse_wrapper::device_wrapper::Device;
use crate::pulse_wrapper::sink_info::PulseSinkInfo;
use crate::pulse_wrapper::source_info::PulseSourceInfo;

use std::cell::RefCell;
use std::rc::Rc;

type Sink = Rc<RefCell<PulseSinkInfo>>;
type Source = Rc<RefCell<PulseSourceInfo>>;

type Sinks = Rc<RefCell<Vec<Sink>>>;
type Sources = Rc<RefCell<Vec<Source>>>;

#[derive(Default)]
pub struct DeviceManager {
    sources: Sources,
    sinks: Sinks,
    sources_count: u32,
    sinks_count: u32,
    default_sink: Option<Sink>,
    default_source: Option<Source>,
}

impl DeviceManager {
    pub fn sources(&mut self) -> Sources {
        self.sources.clone()
    }

    pub fn sinks(&mut self) -> Sinks {
        self.sinks.clone()
    }

    pub fn default_sink(&mut self) -> Sink {
        self.default_sink.as_ref().unwrap().clone()
    }

    pub fn default_source(&mut self) -> Source {
        self.default_source.as_ref().unwrap().clone()
    }

    pub fn sources_count(&self) -> u32 {
        self.sources_count
    }

    pub fn sinks_count(&self) -> u32 {
        self.sinks_count
    }

    pub fn add_source(&mut self, source_info: &SourceInfo) -> u32 {
        self.sources
            .borrow_mut()
            .push(Rc::new(RefCell::new(PulseSourceInfo::from(source_info))));
        self.sources_count += 1;
        self.sources_count
    }

    pub fn add_sink(&mut self, sink_info: &SinkInfo) -> u32 {
        self.sinks
            .borrow_mut()
            .push(Rc::new(RefCell::new(PulseSinkInfo::from(sink_info))));
        self.sinks_count += 1;
        self.sinks_count
    }

    pub fn set_default_source(&mut self, name: &str) -> Result<(), &'static str> {
        for source in self.sources.borrow_mut().clone() {
            if name == source.borrow().name() {
                self.default_source = Some(source.clone());
                return Ok(());
            }
        }
        Err("Unable to set default source")
    }

    pub fn set_default_sink(&mut self, name: &str) -> Result<(), &'static str> {
        for sink in self.sinks.borrow_mut().clone() {
            if name == sink.borrow().name() {
                self.default_sink = Some(sink.clone());
                return Ok(());
            }
        }
        Err("Unable to set default sink")
    }

    pub fn get_sink_by_name(&mut self, name: &str) -> Option<Sink> {
        for sink in self.sinks.borrow_mut().clone() {
            if name == sink.borrow().name() {
                return Some(sink.clone());
            }
        }
        None
    }

    pub fn get_sink_by_index(&mut self, index: u32) -> Option<Sink> {
        for sink in self.sinks.borrow_mut().clone() {
            if index == sink.borrow().index() {
                return Some(sink.clone());
            }
        }
        None
    }

    pub fn get_source_by_name(&mut self, name: &str) -> Option<Source> {
        for source in self.sources.borrow_mut().clone() {
            if name == source.borrow().name() {
                return Some(source.clone());
            }
        }
        None
    }

    pub fn get_source_by_index(&mut self, index: u32) -> Option<Source> {
        for source in self.sources.borrow_mut().clone() {
            if index == source.borrow().index() {
                return Some(source.clone());
            }
        }
        None
    }
}

#[cfg(test)]
mod tets {
    use pulse::volume::{ChannelVolumes, Volume, VolumeDB};

    use super::*;
    static APPRROX_54_PCT: VolumeDB = VolumeDB(-16.01);
    static NAME: &str = "Test";
    static DESC: &str = "test desc";
    static IDX: u32 = 150;
    static CHANNELS: u8 = 2;

    impl DeviceManager {
        fn mock_add_source(&mut self, source: PulseSourceInfo) -> u32 {
            self.sources
                .borrow_mut()
                .push(Rc::new(RefCell::new(source)));
            self.sources_count += 1;
            self.sources_count
        }

        fn mock_add_sink(&mut self, sink: PulseSinkInfo) -> u32 {
            self.sinks.borrow_mut().push(Rc::new(RefCell::new(sink)));
            self.sinks_count += 1;
            self.sinks_count
        }
    }

    fn setup_sink() -> PulseSinkInfo {
        let name = NAME.to_string();
        let index = IDX;
        let description = DESC.to_string();
        let base_volume = Volume::from(APPRROX_54_PCT);
        let mut volume = ChannelVolumes::default();
        volume.set(CHANNELS, base_volume);

        let base_volume = Rc::new(RefCell::new(base_volume));
        let volume = Rc::new(RefCell::new(volume));
        PulseSinkInfo::new(name, index, description, volume, base_volume)
    }

    fn setup_source() -> PulseSourceInfo {
        let name = NAME.to_string();
        let index = IDX;
        let description = DESC.to_string();
        let base_volume = Volume::from(APPRROX_54_PCT);
        let mut volume = ChannelVolumes::default();
        volume.set(CHANNELS, base_volume);

        let base_volume = Rc::new(RefCell::new(base_volume));
        let volume = Rc::new(RefCell::new(volume));
        PulseSourceInfo::new(name, index, description, volume, base_volume)
    }

    fn setup_manager() -> DeviceManager {
        let sink = setup_sink();
        let source = setup_source();
        let mut manager = DeviceManager::default();

        manager.mock_add_source(source);
        manager.mock_add_sink(sink);
        manager
    }

    #[test]
    fn test_add_sources_works() {
        let source = setup_source();
        let mut manager = DeviceManager::default();

        let source_count = manager.mock_add_source(source);

        assert_eq!(source_count, manager.sources_count());
        assert!(manager.sources().borrow_mut().len() == 1);
    }

    #[test]
    fn test_add_sinks_works() {
        let source = setup_sink();
        let mut manager = DeviceManager::default();

        let sink_count = manager.mock_add_sink(source);

        assert_eq!(sink_count, manager.sinks_count());
        assert!(manager.sinks().borrow_mut().len() == 1);
    }

    #[test]
    fn test_get_sink_by_idx() {
        let mut manager = setup_manager();

        let sink = manager.get_sink_by_index(IDX);

        assert!(sink.is_some());
        assert_eq!(sink.unwrap().borrow().index(), IDX);
    }

    #[test]
    fn test_get_sink_by_name() {
        let mut manager = setup_manager();

        let sink = manager.get_sink_by_name(NAME);

        assert!(sink.is_some());
        assert_eq!(sink.unwrap().borrow().name(), NAME);
    }

    #[test]
    fn test_get_source_by_idx() {
        let mut manager = setup_manager();

        let source = manager.get_source_by_index(IDX);

        assert!(source.is_some());
        assert_eq!(source.unwrap().borrow().index(), IDX);
    }

    #[test]
    fn test_get_source_by_name() {
        let mut manager = setup_manager();

        let source = manager.get_source_by_name(NAME);

        assert!(source.is_some());
        assert_eq!(source.unwrap().borrow().name(), NAME);
    }

    #[test]
    fn set_and_get_default_sink() {
        let mut manager = setup_manager();

        let res = manager.set_default_sink(NAME);

        assert!(res.is_ok());

        let default = manager.default_sink();

        assert_eq!(NAME, default.borrow().name());
    }

    #[test]
    fn set_and_get_default_source() {
        let mut manager = setup_manager();

        let res = manager.set_default_source(NAME);

        assert!(res.is_ok());

        let default = manager.default_source();

        assert_eq!(NAME, default.borrow().name());
    }
}
