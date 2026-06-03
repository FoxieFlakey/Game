use std::{
    error::Error,
    pin::Pin,
    rc::Rc,
    sync::atomic::{AtomicBool, AtomicU32, Ordering},
    time::{Duration, Instant},
};

use tokio::{
    signal::unix::{SignalKind, signal},
    sync::Notify,
};

use crate::util::{ErrorWithContext, StringError, sig_safe};

// Maximum of interrupts before hard quit triggered
const WARN_INTERRUPT_COUNT: u32 = 3;
const MAX_INTERRUPT_COUNT: u32 = 5;
static INTERRUPT_COUNT: AtomicU32 = AtomicU32::new(0);

// Different counter for emergency sigint, which bypasses
// game loop, directly handled in signal handler
const WARN_SIG_INTERRUPT_COUNT: u32 = 8;
const MAX_SIG_INTERRUPT_COUNT: u32 = 15;
static SIG_INTERRUPT_COUNT: AtomicU32 = AtomicU32::new(0);

// Give graceful 10 seconds for shutting down
const GRACEFUL_SHUTDOWN_MILISEC: u64 = 10000;
static IS_SHUTTING_DONW: AtomicBool = AtomicBool::new(false);

fn handle_sig_term() {
    if !IS_SHUTTING_DONW.load(Ordering::Relaxed) {
        return;
    }

    sig_safe::write_str_to_stdout(
        "[Signal Handler] Main thread didnt cleanup in time before next SIGTERM, hard quitting\n",
    );
    sig_safe::exit(1);
}

fn handle_sig_hardquit() {
    let current_interrupts = SIG_INTERRUPT_COUNT
        .fetch_add(1, Ordering::Relaxed)
        .saturating_add(1);
    if current_interrupts >= MAX_SIG_INTERRUPT_COUNT {
        sig_safe::write_str_to_stdout(
            "[Signal handler] Main thread does not respond to signal after ",
        );
        let mut buf = itoa::Buffer::new();
        sig_safe::write_str_to_stdout(buf.format(MAX_SIG_INTERRUPT_COUNT));
        sig_safe::write_str_to_stdout(" interrupts. Hard quitting now\n");

        sig_safe::exit(1);
    }

    if current_interrupts >= WARN_SIG_INTERRUPT_COUNT {
        sig_safe::write_str_to_stdout(
            "[Signal handler] Main thread does not respond to signals, hard quiting via signal in ",
        );
        let mut buf = itoa::Buffer::new();
        sig_safe::write_str_to_stdout(
            buf.format(MAX_SIG_INTERRUPT_COUNT.saturating_sub(current_interrupts)),
        );
        sig_safe::write_str_to_stdout(" interrupts\n");
    }
}

pub fn init() -> Result<(), ErrorWithContext<StringError>> {
    // Installing quit handling signal early
    // SAFETY: The handler is only calls async-signal-safe functions and does not panics
    unsafe { signal_hook::low_level::register(signal_hook::consts::SIGINT, handle_sig_hardquit) }
        .map_err(|e| {
        ErrorWithContext::with_message(
            "Cannot install SIGINT handler",
            Box::new(e) as Box<dyn Error>,
        )
    })?;

    // SAFETY: The handler is only calls async-signal-safe functions and does not panics
    unsafe { signal_hook::low_level::register(signal_hook::consts::SIGTERM, handle_sig_term) }
        .map_err(|e| {
            ErrorWithContext::with_message(
                "Cannot install SIGTERM handler",
                Box::new(e) as Box<dyn Error>,
            )
        })?;

    Ok(())
}

pub async fn fail_safe_guard(
    main: impl Fn(
        Box<dyn Fn() -> Pin<Box<dyn Future<Output = ()>>>>,
    )
        -> Pin<Box<dyn Future<Output = Result<(), ErrorWithContext<dyn Error + 'static>>>>>,
) -> Result<(), ErrorWithContext<dyn Error + 'static>> {
    let mut sigint = signal(SignalKind::interrupt()).map_err(|e| {
        ErrorWithContext::with_message("Cannot install SIGINT handler", Box::new(e))
    })?;
    let mut sigterm = signal(SignalKind::terminate()).map_err(|e| {
        ErrorWithContext::with_message("Cannot install SIGTERM handler", Box::new(e))
    })?;
    let quit_notifier = Rc::new(Notify::new());
    let quit_notifier2 = quit_notifier.clone();

    let main_future = main(Box::new(move || {
        let inner_notifier = quit_notifier2.clone();
        Box::pin(async move { inner_notifier.notified().await })
    }));
    tokio::pin!(main_future);

    loop {
        tokio::select! {
            _ = sigint.recv() => {
                let current_interrupts = INTERRUPT_COUNT.fetch_add(1, Ordering::Relaxed).saturating_add(1);
                if current_interrupts >= MAX_INTERRUPT_COUNT {
                    crate::alert!("Unresponsive main loop. Hard quitting now, interrupts exceeds {MAX_INTERRUPT_COUNT}");
                    break;
                }

                if current_interrupts >= WARN_INTERRUPT_COUNT {
                    crate::alert!("Unresponsive main loop. Hard quit will trigger in {} signal count", MAX_INTERRUPT_COUNT.saturating_sub(current_interrupts));
                }

                if current_interrupts == 1 {
                    crate::alert!("SIGINT or Ctrl-C received starting orderly shutdown...");
                }
                quit_notifier.notify_waiters();
            }
            _ = sigterm.recv() => {
                // Sigterm works like trigger one quit and starts timer
                // if timer exhausted and program hasn't cleaned up
                // perform hard exits anyway.
                //
                // SIGINT is not handled in here
                let deadline = Instant::now() + Duration::from_millis(GRACEFUL_SHUTDOWN_MILISEC);
                crate::alert!("SIGTERM received starting orderly shutdown with deadline in {GRACEFUL_SHUTDOWN_MILISEC} ms");

                IS_SHUTTING_DONW.store(true, Ordering::Relaxed);
                quit_notifier.notify_waiters();

                tokio::select! {
                    ret = &mut main_future => {
                        return ret;
                    }

                    _ = tokio::time::sleep_until(deadline.into()) => {
                        crate::fatal!("Program did not shutdown in {GRACEFUL_SHUTDOWN_MILISEC} ms");
                    }
                }
                break;
            }
            ret = &mut main_future => {
                // Game done running
                return ret;
            }
        }
    }

    Err(ErrorWithContext::new_err(StringError::from(
        "Hard quit triggered".to_string(),
    )))
}
