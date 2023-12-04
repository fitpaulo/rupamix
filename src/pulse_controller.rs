pub mod device_manager;
pub mod device_wrapper;
pub mod pulse_driver;
pub mod server_info_wrapper;
pub mod sink_info;
pub mod source_info;

use crate::pulse_controller::device_manager::{DeviceError, DeviceManager};
use crate::pulse_controller::device_wrapper::Device;
use crate::pulse_controller::pulse_driver::PulseDriver;
use crate::pulse_controller::server_info_wrapper::PulseServerInfo;

use pulse::callbacks::ListResult;
use pulse::volume::ChannelVolumes;

use std::cell::RefCell;
use std::rc::Rc;

pub struct Pulse {
    driver: PulseDriver,
    server_info: Rc<RefCell<PulseServerInfo>>,
    devices: Rc<RefCell<DeviceManager>>,
}

impl Default for Pulse {
    fn default() -> Self {
        let driver = PulseDriver::connect_to_pulse().unwrap();

        Pulse {
            driver,
            server_info: Rc::new(RefCell::new(PulseServerInfo::default())),
            devices: Rc::new(RefCell::new(DeviceManager::default())),
        }
    }
}

impl Pulse {
    pub fn new() -> Pulse {
        let mut pulse = Pulse::default();

        pulse.sync();
        pulse
    }

    pub fn sync(&mut self) {
        self.get_server_info();
        // Currently we throw away these Results
        let _ = self.get_source_info();
        let _ = self.get_sink_info();
    }

    /// Sync is not idempotent so we need to reset it's fields
    /// before we call sync on it again.
    pub fn update(&mut self) {
        self.devices.borrow_mut().reset();
        self.sync();
    }

    pub fn print_sources(&self) {
        self.devices.borrow_mut().print_sources();
    }

    pub fn print_sinks(&self) {
        self.devices.borrow_mut().print_sinks();
    }

    pub fn print_sink_volume(
        &self,
        index: Option<u32>,
        name: Option<String>,
    ) -> Result<(), DeviceError> {
        self.devices.borrow_mut().print_sink_volume(index, name)
    }

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

    fn get_source_info(&mut self) -> Result<(), DeviceError> {
        let manager = self.devices.clone();

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

        self.devices
            .borrow_mut()
            .set_default_source(&self.server_info.borrow().default_source_name)
    }

    fn get_sink_info(&mut self) -> Result<(), DeviceError> {
        let manager = self.devices.clone();

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

        self.devices
            .borrow_mut()
            .set_default_sink(&self.server_info.borrow().default_sink_name)
    }

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

    pub fn increase_sink_volume(
        &mut self,
        inc: &u8,
        name: Option<String>,
        idx: Option<u32>,
        boost: bool,
    ) -> Result<(), &'static str> {
        let sink = self.devices.borrow_mut().get_sink(idx, name).ok().unwrap();

        sink.borrow_mut().increase_volume(inc, boost);

        let index = sink.borrow().index();
        let volume = sink.borrow().volume();

        self.update_sink_volume(index, volume.take());
        Ok(())
    }

    pub fn decrease_sink_volume(
        &mut self,
        inc: &u8,
        name: Option<String>,
        idx: Option<u32>,
    ) -> Result<(), &'static str> {
        let sink = self.devices.borrow_mut().get_sink(idx, name).ok().unwrap();

        sink.borrow_mut().decrease_volume(inc);

        let index = sink.borrow().index();
        let volume = sink.borrow().volume();

        self.update_sink_volume(index, volume.take());
        Ok(())
    }

    pub fn toggle_mute(
        &mut self,
        name: Option<String>,
        idx: Option<u32>,
    ) -> Result<(), &'static str> {
        let sink = self.devices.borrow_mut().get_sink(idx, name).ok().unwrap();

        sink.borrow_mut()
            .toggle_mute()
            .expect("Unable to toggle mute");

        let index = sink.borrow().index();
        let volume = sink.borrow().volume();

        self.update_sink_volume(index, volume.take());
        Ok(())
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
        pulse.sync();

        // We are taking our sink here, we need to re-init it later
        let default = pulse.devices.borrow_mut().get_default_sink().ok().unwrap();

        let initial = default.borrow().get_volume_as_pct();

        let _ = pulse.increase_sink_volume(&INC, None, None, BOOST);

        // re-init so we can get the sync and compare values
        pulse.update();
        let default = pulse.devices.borrow_mut().get_default_sink().ok().unwrap();

        assert_eq!(initial + 5, default.borrow().get_volume_as_pct());
    }

    #[test]
    #[ignore]
    // This requires volume to be 5 or greater, otherwise it will fail
    fn checks_decrease_vol_decreases_vol() {
        let mut pulse = setup();
        pulse.sync();

        let default = pulse.devices.borrow_mut().get_default_sink().ok().unwrap();

        let initial = default.borrow().get_volume_as_pct();

        //Re-init so that decrease can get the sink
        pulse.decrease_sink_volume(&5, None, None).unwrap();

        // re-init to get the updated system vol
        pulse.update();
        let default = pulse.devices.borrow_mut().get_default_sink().ok().unwrap();
        assert_eq!(initial - 5, default.borrow().get_volume_as_pct());
    }

    #[test]
    #[ignore]
    fn checks_toggle_mute_works() {
        let mut pulse = setup();
        pulse.sync();

        let default = pulse.devices.borrow_mut().get_default_sink().ok().unwrap();

        let initial = default.borrow().get_volume_as_pct();

        // Defualt took the sink, re-init
        pulse.toggle_mute(None, None).unwrap();

        pulse.update();
        let default = pulse.devices.borrow_mut().get_default_sink().ok().unwrap();
        let muted = default.borrow().get_volume_as_pct();

        assert_eq!(muted, 0);

        // Re-pop sink list
        pulse.toggle_mute(None, None).unwrap();

        pulse.update();
        let default = pulse.devices.borrow_mut().get_default_sink().ok().unwrap();

        assert_eq!(initial, default.borrow().get_volume_as_pct());
    }
}
