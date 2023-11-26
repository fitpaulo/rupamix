pub mod server_info_wrapper;
use crate::pulse_wrapper::server_info_wrapper::MyServerInfo;
use pulse::callbacks::ListResult;
use pulse::context::introspect::{Introspector, SinkInfo};
use pulse::context::{Context, FlagSet as ContextFlagSet, State};
use pulse::def::Retval;
use pulse::mainloop::standard::{IterateResult, Mainloop};
use pulse::proplist::Proplist;
use pulse::volume::{ChannelVolumes, Volume, VolumeDB};
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::mpsc;
use Message::*;

#[derive(Debug)]
struct SinkSource(u32, String);

enum Message {
    Sink(Rc<RefCell<Vec<SinkSource>>>),
    Source(Rc<RefCell<Vec<SinkSource>>>),
    Vol(u32, ChannelVolumes),
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
    sinks: Option<Rc<RefCell<Vec<SinkSource>>>>,
    sources: Option<Rc<RefCell<Vec<SinkSource>>>>,
}

impl Pulse {
    pub fn connect_to_pulse() -> Option<Pulse> {
        log::debug!("In fn connect to pulse.");

        let (sender, receiver) = mpsc::channel();

        let mainloop = Rc::new(RefCell::new(
            Mainloop::new().expect("Failed to create main loop."),
        ));
        log::debug!("Mainloop created.");

        let mut proplist = Proplist::new().unwrap();
        proplist
            .set_str(pulse::proplist::properties::APPLICATION_NAME, "RuPaMixa")
            .unwrap();

        log::debug!("Attempting to create the context.");
        let context = Rc::new(RefCell::new(
            Context::new_with_proplist(mainloop.borrow().deref(), "RuPaMixaContext", &proplist)
                .expect("Failed to create new context."),
        ));
        log::debug!("Context created.");

        log::debug!("Connecting to context.");
        context
            .borrow_mut()
            .connect(None, ContextFlagSet::NOFLAGS, None)
            .expect("Failed to connect to context");
        log::debug!("Connected to context.");

        // wait for context to be ready
        loop {
            match mainloop.borrow_mut().iterate(false) {
                IterateResult::Quit(_) | IterateResult::Err(_) => {
                    log::error!("Iterate state was not success, quitting...");
                    return None;
                }
                IterateResult::Success(_) => {}
            }
            match context.borrow().get_state() {
                State::Ready => {
                    break;
                }
                State::Failed | State::Terminated => {
                    log::error!("Context state failed/terminated, quitting...");
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
            sinks: None,
            sources: None,
            server_info: None,
        })
    }

    pub fn sync(&mut self) {
        log::debug!("Syncing server info");
        self.get_server_info();
        log::debug!("Syncing source info");
        self.get_soruce_info();
        log::debug!("Syncing sink info");
        self.get_sink_info();
    }

    pub fn print_sources(&self) {
        let mut len_idx = 0;
        let mut len_name = 0;

        for source in self.sources.as_ref().unwrap().borrow().deref() {
            let len = source.0.to_string().len();
            if len > len_idx {
                len_idx = len;
            }
            let len = source.1.len();
            if len > len_name {
                len_name = len;
            }
        }

        len_idx += 10; // len of '(default) '
        let sum = len_idx + len_name;

        println!("{:>len_idx$} -- {:<len_name$}", "Index", "Name");
        println!("{:-<sum$}", "");
        for source in self.sources.as_ref().unwrap().borrow().deref() {
            if source.1 == self.server_info.as_ref().unwrap().default_source_name {
                let idx = format!("(default) {}", source.0);
                println!("{:>len_idx$} -- {:<len_name$}", idx, source.1);
            } else {
                println!("{:>len_idx$} -- {:<len_name$}", source.0, source.1);
            }
        }
    }

    pub fn print_sinks(&self) {
        let mut len_idx = 0;
        let mut len_name = 0;

        for sink in self.sinks.as_ref().unwrap().borrow().deref() {
            let len = sink.0.to_string().len();
            if len > len_idx {
                len_idx = len;
            }
            let len = sink.1.len();
            if len > len_name {
                len_name = len;
            }
        }

        len_idx += 10; // len of '(default) '
        let sum = len_idx + len_name;

        println!("{:>len_idx$} -- {:<len_name$}", "Index", "Name");
        println!("{:-<sum$}", "");
        for sink in self.sinks.as_ref().unwrap().borrow().deref() {
            if sink.1 == self.server_info.as_ref().unwrap().default_sink_name {
                let idx = format!("(default) {}", sink.0);
                println!("{:>len_idx$} -- {:<len_name$}", idx, sink.1);
            } else {
                println!("{:>len_idx$} -- {:<len_name$}", sink.0, sink.1);
            }
        }
    }

    fn update_server_info(&mut self, info: MyServerInfo) {
        self.server_info = Some(info);
    }

    fn update_sinks(&mut self, sinks: Rc<RefCell<Vec<SinkSource>>>) {
        self.sinks = Some(sinks);
    }

    fn update_sources(&mut self, sources: Rc<RefCell<Vec<SinkSource>>>) {
        self.sources = Some(sources);
    }

    fn process_message(&mut self) {
        loop {
            let message = self.receiver.try_recv().unwrap_or(Message::Empty);

            match message {
                Sink(sink_list) => {
                    // println!("In Sink path");
                    self.update_sinks(sink_list);
                }
                Source(source_list) => {
                    // println!("In Source path");
                    self.update_sources(source_list);
                }
                ServerInfo(info) => {
                    // println!("In serve info path");
                    self.update_server_info(info);
                }
                Vol(index, volume) => {
                    // println!("In Vol path");
                    self.update_volume(index, &volume);
                }
                Empty => {
                    break;
                }
            }
        }
    }

    fn update_volume(&mut self, index: u32, volume: &ChannelVolumes) {
        let op = self.introspector.borrow_mut().set_sink_volume_by_index(
            index,
            volume,
            Some(Box::new(|success| {
                if success {
                    log::info!("Volume updated successfully.");
                } else {
                    log::error!("Failed to update volume!");
                }
            })),
        );

        loop {
            match self.mainloop.borrow_mut().iterate(false) {
                IterateResult::Quit(_) | IterateResult::Err(_) => {
                    log::error!("Iterate state was not success, quitting...");
                    return;
                }
                IterateResult::Success(_) => {}
            }
            match op.get_state() {
                pulse::operation::State::Running => (),
                pulse::operation::State::Cancelled => {
                    log::error!("Operation cancelled.");
                    return;
                }
                pulse::operation::State::Done => break,
            }
        }
    }

    fn get_server_info(&mut self) {
        let sender = self.sender.clone();
        let op = self.introspector.borrow().get_server_info(move |info| {
            let server_info = MyServerInfo::new(info);
            sender.send(ServerInfo(server_info)).unwrap();
        });
        loop {
            // This top match must be there, it must get some upate that makes the second match statement work
            match self.mainloop.borrow_mut().iterate(false) {
                IterateResult::Quit(_) | IterateResult::Err(_) => {
                    log::error!("Iterate state was not success, quitting...");
                    return;
                }
                IterateResult::Success(_) => (),
            }
            match op.get_state() {
                pulse::operation::State::Running => (),
                pulse::operation::State::Cancelled => {
                    log::error!("Operation cancelled.");
                    return;
                }
                pulse::operation::State::Done => break,
            }
        }
        self.process_message();
    }

    fn get_soruce_info(&mut self) {
        // log::debug!("Getting source info.");
        let sources = Rc::new(RefCell::new(Vec::new()));
        let sender = self.sender.clone();

        let op = self
            .introspector
            .borrow()
            .get_source_info_list(move |result| {
                match result {
                    ListResult::Item(info) => {
                        let name = info.name.as_ref().unwrap().to_string();
                        let idx = info.index;
                        sources.borrow_mut().push(SinkSource(idx, name))
                    }
                    ListResult::Error => {}
                    ListResult::End => {}
                }
                log::debug!("The current state of sources is: {:?}", sources);
                sender
                    .send(Source(sources.clone()))
                    .expect("Unable to send Source Message");
            });

        loop {
            // This top loop must be there, it must get some upate that makes the second match statement work
            match self.mainloop.borrow_mut().iterate(false) {
                IterateResult::Quit(_) | IterateResult::Err(_) => {
                    log::error!("Iterate state was not success, quitting...");
                    return;
                }
                IterateResult::Success(_) => {}
            }
            match op.get_state() {
                pulse::operation::State::Running => (),
                pulse::operation::State::Cancelled => {
                    log::error!("Operation cancelled.");
                    return;
                }
                pulse::operation::State::Done => break,
            }
        }
        self.process_message();
    }

    fn get_sink_info(&mut self) {
        println!("In get sink info method");
        let sinks = Rc::new(RefCell::new(Vec::new()));
        let sender = self.sender.clone();

        let op = self
            .introspector
            .borrow()
            .get_sink_info_list(move |result| {
                match result {
                    ListResult::Item(info) => {
                        let name = info.name.as_ref().unwrap().to_string();
                        let idx = info.index;
                        sinks.borrow_mut().push(SinkSource(idx, name))
                    }
                    ListResult::Error => {}
                    ListResult::End => {}
                }
                sender
                    .send(Sink(sinks.clone()))
                    .expect("Unable to send Sink Message.");
            });

        loop {
            // This top loop must be there, it must get some upate that makes the second match statement work
            match self.mainloop.borrow_mut().iterate(false) {
                IterateResult::Quit(_) | IterateResult::Err(_) => {
                    log::error!("Iterate state was not success, quitting...");
                    return;
                }
                IterateResult::Success(_) => {}
            }
            match op.get_state() {
                pulse::operation::State::Running => (),
                pulse::operation::State::Cancelled => {
                    log::error!("Operation cancelled.");
                    return;
                }
                pulse::operation::State::Done => break,
            }
        }
        self.process_message();
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
