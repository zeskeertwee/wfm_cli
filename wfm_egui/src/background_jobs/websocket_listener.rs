use crossbeam_channel::Sender;
use eframe::egui::Context;
use log::{info, warn, trace};
use wfm_rs::User;
use wfm_rs::websocket::{Status, WebsocketConnection, WebsocketMessagePayload};

pub fn start(user: User, tx: Sender<WebsocketMessagePayload>, context: Context) {
    std::thread::spawn(move || {
        info!("WS listener thread started");
        let mut conn = WebsocketConnection::create(Some(&user)).unwrap();

        conn.send_message(WebsocketMessagePayload::SubscribeMostRecent).unwrap();
        //conn.send_message(WebsocketMessagePayload::SetStatus(Status::Online)).unwrap();

        loop {
            match conn.read_message() {
                Ok(v) => {
                    trace!("Received WS message: {:?}", v);
                    if let Err(e) = tx.send(v) {
                        warn!("Failed to submit WS message: {}", e);
                    }

                    context.request_repaint();
                },
                Err(e) => warn!("Error receiving WS message: {:?}", e),
            }
        }
    });
}