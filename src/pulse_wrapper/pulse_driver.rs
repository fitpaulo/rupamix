use pulse::context::introspect::Introspector;
use pulse::context::{Context, FlagSet as ContextFlagSet, State};
use pulse::def::Retval;
use pulse::mainloop::standard::{IterateResult, Mainloop};
use pulse::operation::Operation;
use pulse::proplist::Proplist;
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

pub struct PulseDriver {
    pub mainloop: Rc<RefCell<Mainloop>>,
    pub context: Rc<RefCell<Context>>,
    pub introspector: Rc<RefCell<Introspector>>,
}

impl Drop for PulseDriver {
    fn drop(&mut self) {
        self.shutdown();
    }
}

impl PulseDriver {
    pub fn connect_to_pulse() -> Result<PulseDriver, &'static str> {
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
                    return Err("Iterate state was not success, quitting...");
                }
                IterateResult::Success(_) => {}
            }
            match context.borrow().get_state() {
                State::Ready => {
                    break;
                }
                State::Failed | State::Terminated => {
                    return Err("Context in state failed/teminated");
                }
                _ => {}
            }
        }

        let introspector = Rc::new(RefCell::new(context.borrow().introspect()));

        Ok(PulseDriver {
            mainloop,
            context,
            introspector,
        })
    }

    pub fn wait_for_op<T: ?Sized>(&mut self, op: Operation<T>) -> Result<(), &'static str> {
        loop {
            // This match must be there
            match self.mainloop.borrow_mut().iterate(false) {
                IterateResult::Quit(_) => return Err("Mainloop quit..."),
                IterateResult::Err(_) => {
                    return Err("Error in mainloop");
                }
                IterateResult::Success(_) => (),
            }
            match op.get_state() {
                pulse::operation::State::Running => (),
                pulse::operation::State::Cancelled => {
                    return Err("Operation was calceled");
                }
                pulse::operation::State::Done => break,
            }
        }
        Ok(())
    }

    fn shutdown(&mut self) {
        self.mainloop.borrow_mut().quit(Retval(0));
        self.context.borrow_mut().disconnect();
    }
}
