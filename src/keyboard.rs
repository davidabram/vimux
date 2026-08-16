use std::error::Error;

use futures_util::StreamExt;
use zbus::{Connection, Proxy};

const DESTINATION: &str = "org.freedesktop.a11y.Manager";
const OBJECT_PATH: &str = "/org/freedesktop/a11y/Manager";
const INTERFACE: &str = "org.freedesktop.a11y.KeyboardMonitor";

// XKB keysyms from xkeyboard-config / keysymdef.h.
const ESCAPE_KEYSYM: u32 = 0xff1b;
const F12_KEYSYM: u32 = 0xffc9;

/// Receive and selectively grab F12 through niri's accessibility D-Bus API.
pub async fn test() -> Result<(), Box<dyn Error>> {
    let connection = Connection::session().await?;
    let proxy = Proxy::new(&connection, DESTINATION, OBJECT_PATH, INTERFACE).await?;
    let mut events = proxy.receive_signal("KeyEvent").await?;

    println!("connected to KeyboardMonitor");

    // Watching reports all keys without suppressing them. SetKeyGrabs then makes
    // only F12 suppress normal compositor/application handling.
    let setup_result = async {
        call_no_args(&proxy, "WatchKeyboard").await?;
        let _: () = proxy
            .call(
                "SetKeyGrabs",
                &(Vec::<u32>::new(), vec![(F12_KEYSYM, 0_u32)]),
            )
            .await?;
        Ok::<_, zbus::Error>(())
    }
    .await;

    if let Err(error) = setup_result {
        let _ = release(&proxy).await;
        return Err(error.into());
    }

    println!("grabbed F12");
    println!("press Escape or Ctrl+C to exit");

    let run_result = receive_events(&mut events).await;
    let release_result = release(&proxy).await;

    if let Err(error) = run_result {
        // Still attempt cleanup before reporting the event-stream failure.
        let _ = release_result;
        return Err(error);
    }
    release_result?;

    Ok(())
}

async fn receive_events(events: &mut zbus::proxy::SignalStream<'_>) -> Result<(), Box<dyn Error>> {
    let ctrl_c = tokio::signal::ctrl_c();
    tokio::pin!(ctrl_c);

    loop {
        tokio::select! {
            result = &mut ctrl_c => {
                result?;
                break;
            }
            message = events.next() => {
                let Some(message) = message else {
                    return Err("KeyboardMonitor signal stream ended".into());
                };
                let (released, modifiers, keysym, unicode, keycode):
                    (bool, u32, u32, u32, u16) = message.body().deserialize()?;

                println!("key:");
                println!("  pressed={}", !released);
                println!("  keysym=0x{keysym:x}");
                println!("  keycode={keycode}");
                println!("  modifiers=0x{modifiers:x}");
                println!("  unicode={unicode}");

                if !released && keysym == ESCAPE_KEYSYM {
                    break;
                }
            }
        }
    }

    Ok(())
}

async fn call_no_args(proxy: &Proxy<'_>, method: &str) -> zbus::Result<()> {
    let _: () = proxy.call(method, &()).await?;
    Ok(())
}

async fn release(proxy: &Proxy<'_>) -> zbus::Result<()> {
    // Attempt both operations even if the first one fails. SetKeyGrabs replaces
    // the previous set, so an empty set releases F12.
    let grabs_result = async {
        let _: () = proxy
            .call(
                "SetKeyGrabs",
                &(Vec::<u32>::new(), Vec::<(u32, u32)>::new()),
            )
            .await?;
        Ok::<_, zbus::Error>(())
    }
    .await;
    let unwatch_result = call_no_args(proxy, "UnwatchKeyboard").await;

    grabs_result.and(unwatch_result)
}
