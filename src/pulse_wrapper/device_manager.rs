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

pub enum DeviceError {
    NameNotFound(String),
    IndexNotFound(String),
    DefaultNotFound(String),
    NoSinks(String),
}

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

    pub fn get_default_sink(&mut self) -> Result<Sink, DeviceError> {
        if let Some(default) = self.default_sink.clone() {
            Ok(default.clone())
        } else {
            Err(DeviceError::DefaultNotFound(
                "No defualt is currently set.".to_string(),
            ))
        }
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

    pub fn reset(&mut self) {
        self.sinks = Rc::new(RefCell::new(Vec::new()));
        self.sources = Rc::new(RefCell::new(Vec::new()));
        self.default_sink = None;
        self.default_source = None;
        self.sources_count = 0;
        self.sinks_count = 0;
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

    pub fn set_default_source(&mut self, name: &str) -> Result<(), DeviceError> {
        for source in self.sources.borrow_mut().clone() {
            if name == source.borrow().name() {
                self.default_source = Some(source.clone());
                return Ok(());
            }
        }

        Err(DeviceError::NameNotFound(format!(
            "No source found with name: {name}"
        )))
    }

    pub fn set_default_sink(&mut self, name: &str) -> Result<(), DeviceError> {
        for sink in self.sinks.borrow_mut().clone() {
            if name == sink.borrow().name() {
                self.default_sink = Some(sink.clone());
                return Ok(());
            }
        }

        Err(DeviceError::NameNotFound(format!(
            "No sink found with name: {name}"
        )))
    }

    pub fn get_sink_by_name(&mut self, name: &str) -> Result<Sink, DeviceError> {
        for sink in self.sinks.borrow_mut().clone() {
            if name == sink.borrow().name() {
                return Ok(sink.clone());
            }
        }

        Err(DeviceError::NameNotFound(format!(
            "No sink found with name: {name}"
        )))
    }

    pub fn get_sink_by_index(&mut self, index: u32) -> Result<Sink, DeviceError> {
        for sink in self.sinks.borrow_mut().clone() {
            if index == sink.borrow().index() {
                return Ok(sink.clone());
            }
        }

        Err(DeviceError::NameNotFound(format!(
            "No sink found with index: {index}"
        )))
    }

    pub fn get_source_by_name(&mut self, name: &str) -> Result<Source, DeviceError> {
        for source in self.sources.borrow_mut().clone() {
            if name == source.borrow().name() {
                return Ok(source.clone());
            }
        }

        Err(DeviceError::NameNotFound(format!(
            "No source found with name: {name}"
        )))
    }

    pub fn get_source_by_index(&mut self, index: u32) -> Result<Source, DeviceError> {
        for source in self.sources.borrow_mut().clone() {
            if index == source.borrow().index() {
                return Ok(source.clone());
            }
        }

        Err(DeviceError::NameNotFound(format!(
            "No source found with index: {index}"
        )))
    }

    pub fn print_sink_volume(
        &mut self,
        index: Option<u32>,
        name: Option<String>,
    ) -> Result<(), DeviceError> {
        let sink = self.get_sink(index, name)?;
        sink.borrow().print_volume();
        Ok(())
    }

    pub fn get_sink(
        &mut self,
        index: Option<u32>,
        name: Option<String>,
    ) -> Result<Sink, DeviceError> {
        let sink;
        if let Some(index) = index {
            sink = self.get_sink_by_index(index);
        } else if let Some(name) = name.clone() {
            sink = self.get_sink_by_name(&name);
        } else {
            sink = self.get_default_sink();
        }
        if sink.is_ok() {
            sink
        } else {
            Err(DeviceError::NoSinks(format!(
                "Unable to get a sink for index: {index:?}, name: {name:?}"
            )))
        }
    }

    pub fn print_sources(&self) {
        let mut len_idx = 0;
        let mut len_name = 0;

        for source in self.sources.as_ref().borrow().clone() {
            let len = source.borrow().index().to_string().len();
            if len > len_idx {
                len_idx = len;
            }
            let len = source.borrow().name().len();
            if len > len_name {
                len_name = len;
            }
        }

        len_idx += 10; // len of '(default) '
        let sum = len_idx + len_name + 6;

        println!();
        println!("{:>len_idx$} -- {:<len_name$}", "Index", "Name");
        println!("{:-<sum$}", "");
        for source in self.sources.as_ref().borrow_mut().clone() {
            if source.borrow().name() == self.default_source.as_ref().unwrap().borrow().name() {
                let idx = format!("(default) {}", source.borrow().index());
                println!("{:>len_idx$} -- {:<len_name$}", idx, source.borrow().name());
            } else {
                println!(
                    "{:>len_idx$} -- {:<len_name$}",
                    source.borrow().index(),
                    source.borrow().name()
                );
            }
        }
    }

    pub fn print_sinks(&self) {
        let mut len_idx = 0;
        let mut len_name = 0;

        for sink in self.sinks.as_ref().borrow().clone() {
            let len = sink.borrow().index().to_string().len();
            if len > len_idx {
                len_idx = len;
            }
            let len = sink.borrow().name().len();
            if len > len_name {
                len_name = len;
            }
        }

        len_idx += 10; // len of '(default) '
        let sum = len_idx + len_name + 6;

        println!();
        println!("{:>len_idx$} -- {:<len_name$}", "Index", "Name");
        println!("{:-<sum$}", "");
        for sink in self.sinks.as_ref().borrow_mut().clone() {
            if sink.borrow().name() == self.default_sink.as_ref().unwrap().borrow().name() {
                let idx = format!("(default) {}", sink.borrow().index());
                println!("{:>len_idx$} -- {:<len_name$}", idx, sink.borrow().name());
            } else {
                println!(
                    "{:>len_idx$} -- {:<len_name$}",
                    sink.borrow().index(),
                    sink.borrow().name()
                );
            }
        }
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

        assert!(sink.is_ok());
        assert_eq!(sink.ok().unwrap().borrow().index(), IDX);
    }

    #[test]
    fn test_get_sink_by_name() {
        let mut manager = setup_manager();

        let sink = manager.get_sink_by_name(NAME);

        assert!(sink.is_ok());
        assert_eq!(sink.ok().unwrap().borrow().name(), NAME);
    }

    #[test]
    fn test_get_source_by_idx() {
        let mut manager = setup_manager();

        let source = manager.get_source_by_index(IDX);

        assert!(source.is_ok());
        assert_eq!(source.ok().unwrap().borrow().index(), IDX);
    }

    #[test]
    fn test_get_source_by_name() {
        let mut manager = setup_manager();

        let source = manager.get_source_by_name(NAME);

        assert!(source.is_ok());
        assert_eq!(source.ok().unwrap().borrow().name(), NAME);
    }

    #[test]
    fn set_and_get_default_sink() {
        let mut manager = setup_manager();

        let res = manager.set_default_sink(NAME);

        assert!(res.is_ok());

        let default = manager.get_default_sink();

        assert_eq!(NAME, default.ok().unwrap().borrow().name());
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