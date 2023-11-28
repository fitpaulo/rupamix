pub mod server_info_wrapper;
pub mod sinksource;
use crate::pulse_wrapper::server_info_wrapper::MyServerInfo;
use crate::pulse_wrapper::sinksource::SinkSource;
use pulse::callbacks::ListResult;
use pulse::context::introspect::Introspector;
use pulse::context::{Context, FlagSet as ContextFlagSet, State};
use pulse::def::Retval;
use pulse::mainloop::standard::{IterateResult, Mainloop};
use pulse::proplist::Proplist;
use pulse::volume::ChannelVolumes;
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::mpsc;
use Message::*;

type Sinks = Rc<RefCell<Vec<Rc<RefCell<SinkSource>>>>>;
type Sources = Rc<RefCell<Vec<Rc<RefCell<SinkSource>>>>>;

enum Message {
    Sink(SinkSource),
    Source(SinkSource),
    Vol(bool),
    ServerInfo(MyServerInfo),
    Empty,
}

pub struct Pulse {
    mainloop: Rc<RefCell<Mainloop>>,
    context: Rc<RefCell<Context>>,
    introspector: Rc<RefCell<Introspector>>,
    sender: mpsc::Sender<Message>,
    receiver: mpsc::Receiver<Message>,
    server_info: Option<MyServerInfo>,
    sinks: Sinks,
    sources: Sources,
}

impl Pulse {
    pub fn connect_to_pulse() -> Option<Pulse> {
        let (sender, receiver) = mpsc::channel();

        let mainloop = Rc::new(RefCell::new(
            Mainloop::new().expect("Failed to create main loop."),
        ));

        let mut proplist = Proplist::new().unwrap();
        proplist
            .set_str(pulse::proplist::properties::APPLICATION_NAME, "RuPaMixa")
            .unwrap();

        let context = Rc::new(RefCell::new(
            Context::new_with_proplist(mainloop.borrow().deref(), "RuPaMixaContext", &proplist)
                .expect("Failed to create new context."),
        ));

        context
            .borrow_mut()
            .connect(None, ContextFlagSet::NOFLAGS, None)
            .expect("Failed to connect to context");

        // wait for context to be ready
        loop {
            match mainloop.borrow_mut().iterate(false) {
                IterateResult::Quit(_) | IterateResult::Err(_) => {
                    eprintln!("Iterate state was not success, quitting...");
                    return None;
                }
                IterateResult::Success(_) => {}
            }
            match context.borrow().get_state() {
                State::Ready => {
                    break;
                }
                State::Failed | State::Terminated => {
                    return None;
                }
                _ => {}
            }
        }

        let introspector = Rc::new(RefCell::new(context.borrow().introspect()));

        Some(Pulse {
            mainloop,
            context,
            introspector,
            sender,
            receiver,
            sinks: Rc::new(RefCell::new(Vec::new())),
            sources: Rc::new(RefCell::new(Vec::new())),
            server_info: None,
        })
    }

    pub fn sync(&mut self) {
        self.get_server_info();
        self.get_source_info();
        self.get_sink_info();
    }

    pub fn print_sources(&self) {
        let mut len_idx = 0;
        let mut len_name = 0;

        for source in self.sources.as_ref().borrow().deref() {
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
        for source in self.sources.as_ref().borrow().deref() {
            if source.borrow().name() == self.server_info.as_ref().unwrap().default_source_name {
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

        for sink in self.sinks.as_ref().borrow().deref() {
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
        for sink in self.sinks.as_ref().borrow().deref() {
            if sink.borrow().name() == self.server_info.as_ref().unwrap().default_sink_name {
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

    pub fn print_sink_volume(&self, idx: Option<u32>, name: Option<String>) {
        let sink;
        if let Some(idx) = idx {
            sink = self.get_sink_by_idx(idx);
        } else if let Some(name) = name {
            sink = self.get_sink_by_name(name);
        } else {
            sink = self.get_default_sink();
        }
        if let Some(sink) = sink {
            sink.borrow().print_volume();
        } else {
            println!("There is no sink data, did you sync it?");
        }
    }

    fn get_sink_by_idx(&self, idx: u32) -> Option<Rc<RefCell<SinkSource>>> {
        for sink in self.sinks.borrow().deref() {
            if sink.borrow().index() == idx {
                return Some(sink.clone());
            }
        }
        None
    }

    fn get_sink_by_name(&self, name: String) -> Option<Rc<RefCell<SinkSource>>> {
        for sink in self.sinks.borrow().deref() {
            if sink.borrow().name() == name {
                return Some(sink.clone());
            }
        }
        None
    }

    fn get_default_sink(&self) -> Option<Rc<RefCell<SinkSource>>> {
        for sink in self.sinks.borrow().deref() {
            if sink.borrow().name() == self.server_info.as_ref().unwrap().default_sink_name {
                return Some(sink.clone());
            }
        }
        None
    }

    fn process_message(&mut self) {
        loop {
            let message = self.receiver.try_recv().unwrap_or(Message::Empty);

            match message {
                Sink(sink) => {
                    // println!("In Sink path");
                    self.update_sinks(sink);
                }
                Source(source) => {
                    // println!("In Source path");
                    self.update_sources(source);
                }
                ServerInfo(info) => {
                    // println!("In serve info path");
                    self.update_server_info(info);
                }
                Vol(_success) => {
                    // println!("In Vol path");
                }
                Empty => {
                    break;
                }
            }
        }
    }

    fn update_server_info(&mut self, info: MyServerInfo) {
        self.server_info = Some(info);
    }

    fn update_sinks(&mut self, sink: SinkSource) {
        self.sinks
            .as_ref()
            .borrow_mut()
            .push(Rc::new(RefCell::new(sink)));
    }

    fn update_sources(&mut self, source: SinkSource) {
        self.sources
            .as_ref()
            .borrow_mut()
            .push(Rc::new(RefCell::new(source)));
    }

    fn get_server_info(&mut self) {
        let sender = self.sender.clone();
        let op = self.introspector.borrow().get_server_info(move |info| {
            let server_info = MyServerInfo::new(info);
            sender.send(ServerInfo(server_info)).unwrap();
        });
        loop {
            // This match must be there
            match self.mainloop.borrow_mut().iterate(false) {
                IterateResult::Quit(_) | IterateResult::Err(_) => {
                    return;
                }
                IterateResult::Success(_) => (),
            }
            match op.get_state() {
                pulse::operation::State::Running => (),
                pulse::operation::State::Cancelled => {
                    return;
                }
                pulse::operation::State::Done => break,
            }
        }
        self.process_message();
    }

    fn get_source_info(&mut self) {
        let sender = self.sender.clone();

        let op = self
            .introspector
            .borrow()
            .get_source_info_list(move |result| match result {
                ListResult::Item(info) => {
                    let name = info.name.as_ref().unwrap().to_string();
                    let idx = info.index;
                    let volume = info.volume;
                    let mute = info.mute;
                    sender
                        .send(Source(SinkSource::new(idx, name, volume, mute)))
                        .expect("Unable to send sinksource.")
                }
                ListResult::Error => {}
                ListResult::End => {}
            });

        loop {
            // This match must be there
            match self.mainloop.borrow_mut().iterate(false) {
                IterateResult::Quit(_) | IterateResult::Err(_) => {
                    return;
                }
                IterateResult::Success(_) => {}
            }
            match op.get_state() {
                pulse::operation::State::Running => (),
                pulse::operation::State::Cancelled => {
                    return;
                }
                pulse::operation::State::Done => break,
            }
        }
        self.process_message();
    }

    fn get_sink_info(&mut self) {
        // println!("In get sink info method");
        let sender = self.sender.clone();

        let op = self
            .introspector
            .borrow()
            .get_sink_info_list(move |result| match result {
                ListResult::Item(info) => {
                    let name = info.name.as_ref().unwrap().to_string();
                    let idx = info.index;
                    let volume = info.volume;
                    let mute = info.mute;
                    sender
                        .send(Sink(SinkSource::new(idx, name, volume, mute)))
                        .expect("Unable to send sinksource.")
                }
                ListResult::Error => {}
                ListResult::End => {}
            });

        loop {
            // This match must be there
            match self.mainloop.borrow_mut().iterate(false) {
                IterateResult::Quit(_) | IterateResult::Err(_) => {
                    return;
                }
                IterateResult::Success(_) => {}
            }
            match op.get_state() {
                pulse::operation::State::Running => (),
                pulse::operation::State::Cancelled => {
                    return;
                }
                pulse::operation::State::Done => break,
            }
        }
        self.process_message();
    }

    pub fn increase_sink_volume(
        &mut self,
        inc: &u8,
        name: Option<String>,
        idx: Option<u32>,
        boost: bool,
    ) {
        let sink;

        if let Some(idx) = idx {
            sink = self.get_sink_by_idx(idx);
        } else if let Some(name) = name {
            sink = self.get_sink_by_name(name);
        } else {
            sink = self.get_default_sink();
        }

        let sink = sink.unwrap();
        sink.borrow_mut().increase_volume(inc, boost);

        let index = sink.borrow().index();
        let volume = sink.borrow().volume();

        self.update_sink_volume(index, volume)
    }

    fn update_sink_volume(&mut self, index: u32, volume: ChannelVolumes) {
        let sender = self.sender.clone();

        let op = self.introspector.borrow_mut().set_sink_volume_by_index(
            index,
            &volume,
            Some(Box::new(move |success| {
                (sender
                    .send(Vol(success))
                    .expect("Unable to send success bool"));
            })),
        );

        loop {
            match self.mainloop.borrow_mut().iterate(false) {
                IterateResult::Quit(_) | IterateResult::Err(_) => {
                    return;
                }
                IterateResult::Success(_) => {}
            }
            match op.get_state() {
                pulse::operation::State::Running => (),
                pulse::operation::State::Cancelled => {
                    return;
                }
                pulse::operation::State::Done => break,
            }
        }
        self.process_message();
    }

    pub fn decrease_sink_volume(&mut self, inc: &u8, name: Option<String>, idx: Option<u32>) {
        let sink;

        if let Some(idx) = idx {
            sink = self.get_sink_by_idx(idx);
        } else if let Some(name) = name {
            sink = self.get_sink_by_name(name);
        } else {
            sink = self.get_default_sink();
        }

        let sink = sink.unwrap();
        sink.borrow_mut().decrease_volume(inc);

        let index = sink.borrow().index();
        let volume = sink.borrow().volume();

        self.update_sink_volume(index, volume)
    }

    pub fn toggle_mute(&mut self, name: Option<String>, idx: Option<u32>) {
        let sink;

        if let Some(idx) = idx {
            sink = self.get_sink_by_idx(idx);
        } else if let Some(name) = name {
            sink = self.get_sink_by_name(name);
        } else {
            sink = self.get_default_sink();
        }

        let sink = sink.unwrap();
        sink.borrow_mut()
            .toggle_mute()
            .expect("Unable to toggle mute");

        let index = sink.borrow().index();
        let volume = sink.borrow().volume();

        self.update_sink_volume(index, volume)
    }

    pub fn shutdown(&mut self) {
        self.mainloop.borrow_mut().quit(Retval(0));
        self.context.borrow_mut().disconnect();
    }
}

impl Drop for Pulse {
    fn drop(&mut self) {
        self.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> Pulse {
        Pulse::connect_to_pulse().unwrap()
    }

    #[test]
    fn checks_update_sereve_gets_server_info() {
        let mut pulse = setup();

        pulse.get_server_info();
        assert!(pulse.server_info.is_some());
    }

    #[test]
    fn checks_get_sinks_builds_a_vec() {
        let mut pulse = setup();
        pulse.get_sink_info();

        pulse.get_sink_info();

        assert!(pulse.sinks.borrow().len() > 0);
    }

    #[test]
    fn checks_get_sources_builds_a_vec() {
        let mut pulse = setup();
        pulse.get_source_info();

        pulse.get_source_info();

        assert!(pulse.sources.borrow().len() > 0);
    }

    #[test]
    fn verify_get_default_sink_returns_a_sink() {
        let mut pulse = setup();
        pulse.sync();

        let default = pulse.get_default_sink();

        assert!(default.is_some())
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

        let default = pulse.get_default_sink().unwrap();

        let initial = default.borrow().get_volume_as_pct();

        pulse.increase_sink_volume(&5, None, None, false);

        // Re-get from the system
        let mut pulse = setup();
        pulse.sync();
        let default = pulse.get_default_sink().unwrap();

        assert_eq!(initial + 5, default.borrow().get_volume_as_pct());
    }

    #[test]
    #[ignore]
    // This requires volume to be 5 or greater, otherwise it will fail
    fn checks_decrease_vol_decreases_vol() {
        let mut pulse = setup();
        pulse.sync();

        let default = pulse.get_default_sink().unwrap();

        let initial = default.borrow().get_volume_as_pct();

        pulse.decrease_sink_volume(&5, None, None);

        // Reg-et from the system
        let mut pulse = setup();
        pulse.sync();
        let default = pulse.get_default_sink().unwrap();

        assert_eq!(initial - 5, default.borrow().get_volume_as_pct());
    }

    #[test]
    #[ignore]
    fn checks_toggle_mute_works() {
        let mut pulse = setup();
        pulse.sync();

        let default = pulse.get_default_sink().unwrap();

        let initial = default.borrow().get_volume_as_pct();

        pulse.toggle_mute(None, None);

        // Re-get from the system
        let mut pulse = setup();
        pulse.sync();
        let default = pulse.get_default_sink().unwrap();
        let muted = default.borrow().get_volume_as_pct();

        assert_eq!(muted, 0);

        pulse.toggle_mute(None, None);

        let mut pulse = setup();
        pulse.sync();
        let default = pulse.get_default_sink().unwrap();

        assert_eq!(initial, default.borrow().get_volume_as_pct());
    }
}
