use pulse::context::introspect::{SinkInfo, SourceInfo};

use crate::pulse_wrappers::device::Device;
use crate::pulse_wrappers::sink_info::PulseSinkInfo;
use crate::pulse_wrappers::source_info::PulseSourceInfo;

use std::cell::RefCell;
use std::rc::Rc;

type Sink = Rc<RefCell<PulseSinkInfo>>;
type Source = Rc<RefCell<PulseSourceInfo>>;

pub enum DeviceError {
    NameNotFound(String),
    IndexNotFound(String),
    DefaultNotFound(String),
    NoSinks(String),
}

impl DeviceError {
    /// These errors get passed up for several layers, so it is easiest to
    /// handle them here.
    pub fn print_err_and_panic(&self) {
        match self {
            DeviceError::NameNotFound(e) => {
                eprintln!("Device NameNotFound Error: {e}");
                panic!("Unable to continue.")
            }
            DeviceError::IndexNotFound(e) => {
                eprintln!("Device IndexNotFound Error: {e}");
                panic!("Unable to continue.")
            }
            DeviceError::DefaultNotFound(e) => {
                eprintln!("Device DefaultNotFound Error: {e}");
                panic!("Unable to continue.")
            }
            DeviceError::NoSinks(e) => {
                eprintln!("Device NoSinks Error: {e}");
                panic!("Unable to continue.")
            }
        }
    }
}

#[derive(Default)]
pub struct DeviceManager {
    sources: Vec<Source>,
    sinks: Vec<Sink>,
    sources_count: u32,
    sinks_count: u32,
    default_sink: Option<Sink>,
    default_source: Option<Source>,
}

impl DeviceManager {
    /// Getter for sources
    pub fn sources(&mut self) -> &[Source] {
        &self.sources
    }

    /// Getter for sinks
    pub fn sinks(&mut self) -> &[Sink] {
        &self.sinks
    }

    /// Getter for default sink
    pub fn default_sink(&mut self) -> Result<Sink, DeviceError> {
        if let Some(default) = self.default_sink.clone() {
            Ok(default)
        } else {
            Err(DeviceError::DefaultNotFound(
                "No defualt is currently set.".to_string(),
            ))
        }
    }

    /// Getter for default source
    pub fn default_source(&mut self) -> Result<Source, DeviceError> {
        if let Some(default) = self.default_source.clone() {
            Ok(default)
        } else {
            Err(DeviceError::DefaultNotFound(
                "No default is currently set.".to_string(),
            ))
        }
    }

    /// Getter for sources count
    pub fn sources_count(&self) -> u32 {
        self.sources_count
    }

    /// Getter for sinks count
    pub fn sinks_count(&self) -> u32 {
        self.sinks_count
    }

    /// Method to reset the device manager members to their default values
    pub fn reset(&mut self) {
        self.sinks = Vec::new();
        self.sources = Vec::new();
        self.default_sink = None;
        self.default_source = None;
        self.sources_count = 0;
        self.sinks_count = 0;
    }

    /// Adds a source into the sources vector and returns the current count
    /// of sources
    pub fn add_source(&mut self, source_info: &SourceInfo) -> u32 {
        self.sources
            .push(Rc::new(RefCell::new(PulseSourceInfo::from(source_info))));
        self.sources_count += 1;
        self.sources_count
    }

    /// Adds a sink into the sinks vector and returns the current count
    /// of sinks
    pub fn add_sink(&mut self, sink_info: &SinkInfo) -> u32 {
        self.sinks
            .push(Rc::new(RefCell::new(PulseSinkInfo::from(sink_info))));
        self.sinks_count += 1;
        self.sinks_count
    }

    /// Make an RC clone and store it here for easy access to the default source
    pub fn set_default_source(&mut self, name: &str) -> Result<(), DeviceError> {
        for source in self.sources() {
            if name == source.borrow().name() {
                self.default_source = Some(source.clone());
                return Ok(());
            }
        }

        Err(DeviceError::NameNotFound(format!(
            "No source found with name: {name}"
        )))
    }

    /// Make an RC clone and store it here for easy access to the default sink
    pub fn set_default_sink(&mut self, name: &str) -> Result<(), DeviceError> {
        for sink in self.sinks() {
            if name == sink.borrow().name() {
                self.default_sink = Some(sink.clone());
                return Ok(());
            }
        }

        Err(DeviceError::NameNotFound(format!(
            "No sink found with name: {name}"
        )))
    }

    /// This method attempts to find a sink with the supplied name
    pub fn get_sink_by_name(&mut self, name: &str) -> Result<Sink, DeviceError> {
        for sink in self.sinks() {
            if name == sink.borrow().name() {
                return Ok(sink.clone());
            }
        }

        Err(DeviceError::NameNotFound(format!(
            "No sink found with name: {name}"
        )))
    }

    /// This method attempts to find a sink with the supplied index
    pub fn get_sink_by_index(&mut self, index: u32) -> Result<Sink, DeviceError> {
        for sink in self.sinks() {
            if index == sink.borrow().index() {
                return Ok(sink.clone());
            }
        }

        Err(DeviceError::NameNotFound(format!(
            "No sink found with index: {index}"
        )))
    }

    /// This is a general method that accepts two optional arguments: index, name
    /// If index is supplied it will attempt to get the sink with that index. If
    /// no index is supplied, but a name is, then it will attempt to find a sink
    /// with that name. Finally, if neither is given, it will get the default
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
            sink = self.default_sink();
        }
        if sink.is_ok() {
            sink
        } else {
            Err(DeviceError::NoSinks(format!(
                "Unable to get a sink for index: {index:?}, name: {name:?}"
            )))
        }
    }

    /// This method attempts to find a source with the supplied name
    pub fn get_source_by_name(&mut self, name: &str) -> Result<Source, DeviceError> {
        for source in self.sources() {
            if name == source.borrow().name() {
                return Ok(source.clone());
            }
        }

        Err(DeviceError::NameNotFound(format!(
            "No source found with name: {name}"
        )))
    }

    /// This method attempts to find a source with the supplied index
    pub fn get_source_by_index(&mut self, index: u32) -> Result<Source, DeviceError> {
        for source in self.sources() {
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

    /// This will print to the comand line the sources in the following format
    //            index :: name
    // --------------------------------
    //                1 :: SourceA
    //      (default) 2 :: SourceB
    // ...
    //                N :: SourceM
    pub fn print_sources(&mut self) -> Result<(), DeviceError> {
        let mut len_idx = 0;
        let mut len_name = 0;

        // need to do this here so that we can make compare later
        // otherwise we'd be borrowing mut twice
        let default = self.default_source()?;

        for source in self.sources() {
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
        let sum = len_idx + len_name + 6; // the 6 is ' -- ' and an opening and closing space

        println!();
        println!("{:>len_idx$} -- {:<len_name$}", "Index", "Name");
        println!("{:-<sum$}", "");
        for source in self.sources() {
            if source.borrow().name() == default.borrow().name() {
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
        Ok(())
    }

    /// This will print to the comand line the sinks in the following format
    //            index :: name
    // --------------------------------
    //                1 :: SinkA
    //      (default) 2 :: SinkB
    // ...
    //                N :: SinkM
    pub fn print_sinks(&mut self) -> Result<(), DeviceError> {
        let mut len_idx = 0;
        let mut len_name = 0;
        let default = self.default_sink()?;

        for sink in self.sinks() {
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
        let sum = len_idx + len_name + 6; // the 6 is ' -- ' and an opening and closing space

        println!();
        println!("{:>len_idx$} -- {:<len_name$}", "Index", "Name");
        println!("{:-<sum$}", "");
        for sink in self.sinks() {
            if sink.borrow().name() == default.borrow().name() {
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
        Ok(())
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
            self.sources.push(Rc::new(RefCell::new(source)));
            self.sources_count += 1;
            self.sources_count
        }

        fn mock_add_sink(&mut self, sink: PulseSinkInfo) -> u32 {
            self.sinks.push(Rc::new(RefCell::new(sink)));
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
        assert!(manager.sources().len() == 1);
    }

    #[test]
    fn test_add_sinks_works() {
        let source = setup_sink();
        let mut manager = DeviceManager::default();

        let sink_count = manager.mock_add_sink(source);

        assert_eq!(sink_count, manager.sinks_count());
        assert!(manager.sinks().len() == 1);
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

        let default = manager.default_sink();

        assert_eq!(NAME, default.ok().unwrap().borrow().name());
    }

    #[test]
    fn set_and_get_default_source() {
        let mut manager = setup_manager();

        let res = manager.set_default_source(NAME);

        assert!(res.is_ok());

        let default = manager.default_source();

        assert_eq!(NAME, default.ok().unwrap().borrow().name());
    }
}
