use log::{info, trace};
use wfm_rs::shared::OrderType;
use wfm_rs::websocket::{WebsocketConnection, WebsocketMessagePayload};

fn main() {
    pretty_env_logger::init();

    let mut conn = WebsocketConnection::create(None).unwrap();
    conn.send_message(WebsocketMessagePayload::SubscribeMostRecent).unwrap();

    info!("Listening for messages");
    loop {
        if let Ok(msg) = conn.read_message() {
            match msg {
                WebsocketMessagePayload::NewOrder { order } => {
                    if order.order_type == OrderType::Buy {
                        continue;
                    }

                    if let Some(ducats) = order.item.ducats {
                        let ducats_per_plat = ducats as f64 / order.platinum;
                        trace!("{} d/p on {}", ducats_per_plat, order.item.en.item_name);

                        if ducats_per_plat >= 10.0 {
                            info!("{} d/p on {} (x{}) from {}", ducats_per_plat, order.item.en.item_name, order.quantity, order.user.ingame_name);
                            info!("In-game whisper: /w {} Hi! I want to buy: \"{}\" for {} platinum. (warframe.market watcher)", order.user.ingame_name, order.item.en.item_name, order.platinum)
                        }
                    }
                },
                _ => (),
            }
        }
    }
}
