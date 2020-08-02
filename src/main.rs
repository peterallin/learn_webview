use include_dir::{include_dir, Dir};
use std::time::Duration;

const UI: Dir = include_dir!("ui");

struct State {
    count1: u32,
    count2: u32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let html : &str = UI.get_file("index.html").unwrap().contents_utf8().unwrap();
    let state = State {
        count1: 0,
        count2: 0,
    };
    let wv = web_view::builder()
        .title("My hello world")
        .content(web_view::Content::Html(html))
        .size(640, 480)
        .resizable(true)
        .debug(true)
        .user_data(state)
        .invoke_handler(|wv, msg| {
            if msg == "reset" {
                wv.user_data_mut().count1 = 0;
                wv.user_data_mut().count2 = 0;
            }
            Ok(())
        })
        .build()?;

    let leashes = vec![
        thread_loop(Duration::from_millis(19), wv.handle(), |x| x.count1 += 1),
        thread_loop(Duration::from_millis(1234), wv.handle(), |x| x.count2 += 1),
    ];
    wv.run()?;
    for l in leashes {
        l.kill()?;
    }
    Ok(())
}

fn thread_loop(
    sleep_time: Duration,
    web_view_handle: web_view::Handle<State>,
    mutator: fn(&mut State) -> (),
) -> ThreadLeash {
    let (stop_tx, stop_rx) = std::sync::mpsc::channel();
    let join_handle = std::thread::spawn(move || loop {
        web_view_handle
            .dispatch(move |v| {
                let state = v.user_data_mut();
                mutator(state);
                let call = format!(
                    "setCount1('{}'); setCount2('{}');",
                    state.count1, state.count2
                );
                v.eval(&call).unwrap();
                Ok(())
            })
            .unwrap();
        if let Ok(()) = stop_rx.recv_timeout(sleep_time) {
            break;
        }
    });
    ThreadLeash {
        stop_tx,
        join_handle,
    }
}

struct ThreadLeash {
    join_handle: std::thread::JoinHandle<()>,
    stop_tx: std::sync::mpsc::Sender<()>,
}

impl ThreadLeash {
    fn kill(self) -> Result<(), Box<dyn std::error::Error>> {
        self.stop_tx.send(())?;
        self.join_handle
            .join()
            .map_err(|_| "Failed to join thread")?;
        Ok(())
    }
}
