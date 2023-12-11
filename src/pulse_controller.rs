pub mod device_manager;
pub mod pulse_driver;

use crate::pulse_controller::device_manager::{DeviceError, DeviceManager};
use crate::pulse_controller::pulse_driver::PulseDriver;

use crate::pulse_wrappers::device::Device;
use crate::pulse_wrappers::server_info::PulseServerInfo;
use crate::pulse_wrappers::sink_info::PulseSinkInfo;

use pulse::callbacks::ListResult;
use pulse::volume::ChannelVolumes;

use std::cell::RefCell;
use std::rc::Rc;

pub struct Pulse {
    driver: PulseDriver,
    server_info: Rc<RefCell<PulseServerInfo>>,
    device_manager: Rc<RefCell<DeviceManager>>,
}

impl Default for Pulse {
    fn default() -> Self {
        let driver = PulseDriver::connect_to_pulse().unwrap();

        Pulse {
            driver,
            server_info: Rc::new(RefCell::new(PulseServerInfo::default())),
            device_manager: Rc::new(RefCell::new(DeviceManager::default())),
        }
    }
}

impl Pulse {
    /// This gets a fully ready to use Pulse struct. To achieve that we
    /// created a default with empty server_info and device fields.
    /// To fill those empty fields we call sync to get the current global state of PulseAudio
    pub fn new() -> Pulse {
        let mut pulse = Pulse::default();

        pulse.sync();
        pulse
    }

    /// Return the device manager to a calling controller
    pub fn device_manager(&self) -> Rc<RefCell<DeviceManager>> {
        self.device_manager.clone()
    }

    /// Return the server info to a calling controller
    pub fn server_info(&self) -> Rc<RefCell<PulseServerInfo>> {
        self.server_info.clone()
    }

    /// Get the current state of Pulse Audio
    /// This includes info about the server, the sinks, and the sources
    pub fn sync(&mut self) {
        self.get_server_info();
        // Currently we throw away these Results
        if let Err(res) = self.get_source_info() {
            res.print_err_and_panic()
        } else if let Err(res) = self.get_sink_info() {
            res.print_err_and_panic()
        }
    }

    /// Our access to Pulse state is a oneshot, if the state changes, or if we tried to change it,
    /// we need to ask Pulse for the world state again.
    /// Sync is not idempotent so we need to reset the fields set by sync
    /// before we call sync on it again.
    pub fn update(&mut self) {
        self.device_manager.borrow_mut().reset();
        self.sync();
    }

    /// This calls the device managers print soruces
    pub fn print_sources(&self) {
        if let Err(e) = self.device_manager.borrow_mut().print_sources() {
            e.print_err_and_panic();
        }
    }

    /// This calls the device managers print sinks
    pub fn print_sinks(&self) {
        if let Err(e) = self.device_manager.borrow_mut().print_sinks() {
            e.print_err_and_panic();
        }
    }

    /// Here we want to prink the volume of a specific sink.
    /// Sinks can be specified with either an index or a name.
    /// If neither are supplied, we will print the info from the default
    pub fn print_sink_volume(&self, index: Option<u32>, name: Option<String>) {
        if let Err(e) = self
            .device_manager
            .borrow_mut()
            .print_sink_volume(index, name)
        {
            e.print_err_and_panic()
        }
    }

    /// This method asks the running Pulse server for it's sever info and
    /// stores that data in our thin wrapper around pulse audio's state
    fn get_server_info(&mut self) {
        let server_info = self.server_info.clone();

        let op = self
            .driver
            .introspector
            .borrow()
            .get_server_info(move |info| server_info.borrow_mut().update(info));

        self.driver
            .wait_for_op(op)
            .expect("Wait for op exited prematurely");
    }

    /// Get a list of all pulse audio's sources and store those in our device manager
    fn get_source_info(&mut self) -> Result<(), DeviceError> {
        let manager = self.device_manager.clone();

        let op =
            self.driver
                .introspector
                .borrow()
                .get_source_info_list(move |result| match result {
                    ListResult::Item(info) => {
                        manager.borrow_mut().add_source(info);
                    }
                    ListResult::Error => {}
                    ListResult::End => {}
                });

        self.driver
            .wait_for_op(op)
            .expect("Wait for op exited prematurely");

        self.device_manager
            .borrow_mut()
            .set_default_source(&self.server_info.borrow().default_source_name)
    }

    /// Get a list of all pulse audio's sinks and store those in our device manager
    fn get_sink_info(&mut self) -> Result<(), DeviceError> {
        let manager = self.device_manager.clone();

        let op = self
            .driver
            .introspector
            .borrow()
            .get_sink_info_list(move |result| match result {
                ListResult::Item(info) => {
                    manager.borrow_mut().add_sink(info);
                }
                ListResult::Error => {}
                ListResult::End => {}
            });

        self.driver
            .wait_for_op(op)
            .expect("Wait for op exited prematurely");

        self.device_manager
            .borrow_mut()
            .set_default_sink(&self.server_info.borrow().default_sink_name)
    }

    /// Updates the volume of a particular sink by that sink's index
    /// This method is what actually reaches out to the running server to request
    /// the change in volume
    fn update_sink_volume(&mut self, index: u32, volume: ChannelVolumes) {
        let op = self
            .driver
            .introspector
            .borrow_mut()
            .set_sink_volume_by_index(index, &volume, Some(Box::new(move |_success| ())));

        self.driver
            .wait_for_op(op)
            .expect("Wait for op exited prematurely");
    }

    pub fn set_sink_volume(
        &mut self,
        vol: u8,
        boost: bool,
        index: Option<u32>,
        name: Option<String>,
    ) {
        let mut sink: Option<Rc<RefCell<PulseSinkInfo>>> = None;
        let res = self.device_manager.borrow_mut().get_sink(index, name);

        match res {
            Ok(inner) => sink = Some(inner),
            Err(e) => e.print_err_and_panic(),
        }

        let sink = sink.unwrap();

        sink.borrow_mut().set_volume(vol, boost);

        self.update_sink_volume(sink.borrow().index(), sink.borrow().volume().take());
    }

    /// This method first get the sink by index or name (default if neither are supplied)
    /// It then asks the sink to increase it's volume. This is just a state change in our
    /// representation of the sink, so finally it uses that new rep to call our method that
    /// will interface with the PA server to make the change for real
    pub fn increase_sink_volume(
        &mut self,
        inc: &u8,
        index: Option<u32>,
        name: Option<String>,
        boost: bool,
    ) {
        let mut sink: Option<Rc<RefCell<PulseSinkInfo>>> = None;
        let res = self.device_manager.borrow_mut().get_sink(index, name);

        match res {
            Ok(inner) => sink = Some(inner),
            Err(e) => e.print_err_and_panic(),
        }

        let sink = sink.unwrap();

        sink.borrow_mut().increase_volume(inc, boost);

        self.update_sink_volume(sink.borrow().index(), sink.borrow().volume().take());
    }

    /// This method first get the sink by index or name (default if neither are supplied)
    /// It then asks the sink to decrease it's volume. This is just a state change in our
    /// representation of the sink, so finally it uses that new rep to call our method that
    /// will interface with the PA server to make the change for real
    pub fn decrease_sink_volume(&mut self, inc: &u8, index: Option<u32>, name: Option<String>) {
        let mut sink: Option<Rc<RefCell<PulseSinkInfo>>> = None;
        let res = self.device_manager.borrow_mut().get_sink(index, name);

        match res {
            Ok(inner) => sink = Some(inner),
            Err(e) => e.print_err_and_panic(),
        }

        let sink = sink.unwrap();

        sink.borrow_mut().decrease_volume(inc);

        self.update_sink_volume(sink.borrow().index(), sink.borrow().volume().take());
    }

    /// This method first get the sink by index or name (default if neither are supplied)
    /// It then asks the sink to toggle_mute. This is just a state change in our
    /// representation of the sink, so finally it uses that new rep to call our method that
    /// will interface with the PA server to make the change for real
    pub fn toggle_mute(&mut self, index: Option<u32>, name: Option<String>) {
        let mut sink: Option<Rc<RefCell<PulseSinkInfo>>> = None;
        let res = self.device_manager.borrow_mut().get_sink(index, name);

        match res {
            Ok(inner) => sink = Some(inner),
            Err(e) => e.print_err_and_panic(),
        }

        let sink = sink.unwrap();

        sink.borrow_mut()
            .toggle_mute()
            .expect("Unable to toggle mute");

        self.update_sink_volume(sink.borrow().index(), sink.borrow().volume().take());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    static BOOST: bool = false;
    static INC: u8 = 5;

    fn setup() -> Pulse {
        Pulse::new()
    }

    fn get_default(pulse: &Pulse) -> Rc<RefCell<PulseSinkInfo>> {
        pulse
            .device_manager
            .borrow_mut()
            .default_sink()
            .ok()
            .unwrap()
    }

    // everything below here must be run on a single thread
    // run with the following flags
    // --ignored --test-threads=1
    //  _ _ _ _ _ _ _ _ _ _
    // _ _ _ _ _ _ _ _ _ _

    #[test]
    #[ignore]
    // This requires volume to be 95 or less, otherwise it will fail
    fn checks_increase_vol_increases_vol() {
        let mut pulse = setup();

        // We are taking our sink here, we need to re-init it later
        let default = get_default(&pulse);

        let initial = default.borrow().get_volume_as_pct();

        pulse.increase_sink_volume(&INC, None, None, BOOST);

        // re-init so we can get the sync and compare values
        pulse.update();
        let default = get_default(&pulse);

        assert_eq!(initial + 5, default.borrow().get_volume_as_pct());
    }

    #[test]
    #[ignore]
    // This requires volume to be 5 or greater, otherwise it will fail
    fn checks_decrease_vol_decreases_vol() {
        let mut pulse = setup();

        let default = get_default(&pulse);

        let initial = default.borrow().get_volume_as_pct();

        //Re-init so that decrease can get the sink
        pulse.decrease_sink_volume(&5, None, None);

        // re-init to get the updated system vol
        pulse.update();
        let default = get_default(&pulse);

        assert_eq!(initial - 5, default.borrow().get_volume_as_pct());
    }

    #[test]
    #[ignore]
    fn checks_toggle_mute_works() {
        let mut pulse = setup();

        let default = get_default(&pulse);

        let initial = default.borrow().get_volume_as_pct();

        // Defualt took the sink, re-init
        pulse.toggle_mute(None, None);

        pulse.update();
        let default = get_default(&pulse);

        let muted = default.borrow().get_volume_as_pct();

        assert_eq!(muted, 0);

        // Re-pop sink list
        pulse.toggle_mute(None, None);

        pulse.update();
        let default = get_default(&pulse);

        assert_eq!(initial, default.borrow().get_volume_as_pct());
    }

    #[test]
    #[ignore]
    fn checks_set_volume_works() {
        let mut pulse = setup();

        let default = get_default(&pulse);
        let vol = 100;
        let boost = false;

        let initial = default.borrow().get_volume_as_pct();

        // Defualt took the sink, re-init
        pulse.set_sink_volume(vol, boost, None, None);

        pulse.update();
        let default = get_default(&pulse);

        let new_vol = default.borrow().get_volume_as_pct();

        assert_eq!(vol, new_vol);

        // Re-pop sink list
        pulse.set_sink_volume(initial, boost, None, None);

        pulse.update();
        let default = get_default(&pulse);

        assert_eq!(initial, default.borrow().get_volume_as_pct());
    }

    #[test]
    #[ignore]
    fn checks_set_volume_works_boosted() {
        let mut pulse = setup();

        let default = get_default(&pulse);
        let vol = 120;
        let boost = true;

        let initial = default.borrow().get_volume_as_pct();

        // Defualt took the sink, re-init
        pulse.set_sink_volume(vol, boost, None, None);

        pulse.update();
        let default = get_default(&pulse);

        let new_vol = default.borrow().get_volume_as_pct();

        assert_eq!(vol, new_vol);

        // Re-pop sink list
        pulse.set_sink_volume(initial, boost, None, None);

        pulse.update();
        let default = get_default(&pulse);

        assert_eq!(initial, default.borrow().get_volume_as_pct());
    }
}
