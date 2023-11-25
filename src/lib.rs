use pulse::callbacks::ListResult;
use pulse::context::introspect::{Introspector, SinkInfo};
use pulse::context::{Context, FlagSet as ContextFlagSet, State};
use pulse::def::Retval;
use pulse::mainloop::standard::{IterateResult, Mainloop};
use pulse::proplist::Proplist;
use pulse::volume::{ChannelVolumes, Volume, VolumeDB, VolumeLinear};
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::mpsc;

pub struct Pulse {
    mainloop: Rc<RefCell<Mainloop>>,
    context: Rc<RefCell<Context>>,
    introspector: Rc<RefCell<Introspector>>,
    vol_sender: mpsc::Sender<(u32, ChannelVolumes)>,
    vol_receiver: mpsc::Receiver<(u32, ChannelVolumes)>,
}

impl Pulse {
    pub fn connect_to_pulse() -> Option<Pulse> {
        let (vol_sender, vol_receiver) = mpsc::channel();
        println!("In fn connect to pulse.");
        let mainloop = Rc::new(RefCell::new(
            Mainloop::new().expect("Failed to create main loop."),
        ));
        println!("Mainloop created.");

        let mut proplist = Proplist::new().unwrap();
        proplist
            .set_str(pulse::proplist::properties::APPLICATION_NAME, "RuPaMixa")
            .unwrap();

        println!("Attempting to create the context.");
        let context = Rc::new(RefCell::new(
            Context::new_with_proplist(mainloop.borrow().deref(), "RuPaMixaContext", &proplist)
                .expect("Failed to create new context."),
        ));
        println!("Context created.");

        println!("Connecting to context.");
        context
            .borrow_mut()
            .connect(None, ContextFlagSet::NOFLAGS, None)
            .expect("Failed to connect to context");
        println!("Connected to context.");

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
                    eprintln!("Context state failed/terminated, quitting...");
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
            vol_sender,
            vol_receiver,
        })
    }

    pub fn update_volume(&mut self) {
        let vol_data = self.vol_receiver.recv().unwrap();
        let op = self.introspector.borrow_mut().set_sink_volume_by_index(
            vol_data.0,
            &vol_data.1,
            Some(Box::new(|_| ())),
        );
        loop {
            match self.mainloop.borrow_mut().iterate(false) {
                IterateResult::Quit(_) | IterateResult::Err(_) => {
                    eprintln!("Iterate state was not success, quitting...");
                    return;
                }
                IterateResult::Success(_) => {}
            }
            match op.get_state() {
                pulse::operation::State::Running => (),
                pulse::operation::State::Cancelled => {
                    eprintln!("Operation cancelled.");
                    return;
                }
                pulse::operation::State::Done => break,
            }
        }
    }

    pub fn get_sink_info(&mut self) {
        println!("Getting the server info.");
        let sender = self.vol_sender.clone();
        let op = self.introspector.borrow().get_sink_info_list(move |info| {
            println!("In the get server info callback.");
            let result = print_sink_info(info, 5).expect("Got none instead of some from sink_info");
            sender.send(result);
        });
        loop {
            match self.mainloop.borrow_mut().iterate(false) {
                IterateResult::Quit(_) | IterateResult::Err(_) => {
                    eprintln!("Iterate state was not success, quitting...");
                    return;
                }
                IterateResult::Success(_) => {}
            }
            match op.get_state() {
                pulse::operation::State::Running => (),
                pulse::operation::State::Cancelled => {
                    eprintln!("Operation cancelled.");
                    return;
                }
                pulse::operation::State::Done => break,
            }
        }
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

// Call backs
fn print_sink_info(list_result: ListResult<&SinkInfo>, inc: u32) -> Option<(u32, ChannelVolumes)> {
    println!("Inside print_sink_info");
    match list_result {
        ListResult::Item(sink_info) => {
            let mut channels = sink_info.volume;
            // println!(
            // "Name: {}, idx: {}, channels: {}",
            // sink_info.name.as_ref().unwrap(),
            // sink_info.index,
            // channels.len(),
            // );
            let one_pct_approx = Volume::from(VolumeDB(-120.0));
            for _ in 0..inc {
                channels.increase(one_pct_approx).unwrap();
            }
            let vols = channels.get();
            println!("volume: {} - {}", vols[0], vols[1]);
            Some((sink_info.index, channels))
        }
        ListResult::End | ListResult::Error => None,
    }
}
