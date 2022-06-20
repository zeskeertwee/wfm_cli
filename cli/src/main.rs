#![feature(drain_filter)]

use colored::*;
use util::{clear_terminal, unix_timestamp};
use wfm_rs::response::{ExistingProfileOrder, Order};
use wfm_rs::shared::OrderType;

mod config;
mod util;

const DATA_PATH_SUFFIX: &str = ".wfm_cli/";
const DATA_CONFIG_FILE: &str = "config.wfm.json";
const ITEMS_CACHE_EXPIRY_S: u64 = 24 * 60 * 60;

enum OrderStatus {
    Undercut {
        below: Vec<Order>,
        pos: (usize, usize)
    },
    Lowest {
        next: Order,
    },
    SharedLowest,
}

struct OrderScanReport {
    status: OrderStatus,
    order: ExistingProfileOrder,
}

#[tokio::main]
async fn main() {
    let config = config::run().await.unwrap();
    let user = config.user();

    let existing_orders = user.get_user_orders().await.unwrap();

    let mut scan_results: Vec<OrderScanReport> = Vec::new();

    for sell_order in existing_orders.sell_orders {
        let mut item_orders = user.get_item_orders(&sell_order.item).await.unwrap();
        item_orders.drain_filter(|v| v.user.status != "ingame" || v.order_type == OrderType::Buy);
        item_orders.sort_by(|a, b| a.platinum.partial_cmp(&b.platinum).unwrap());
        if item_orders[0].platinum < sell_order.platinum {
            let mut pos = (usize::MAX, usize::MAX);
            for (idx, order) in item_orders.iter().enumerate() {
                if order.platinum == sell_order.platinum {
                    pos.0 = idx + 1;

                    break;
                }

                if order.platinum > sell_order.platinum {
                    pos.0 = idx + 1;
                    pos.1 = pos.0;

                    break;
                }
            }

            if pos.1 == usize::MAX {
                pos.1 = pos.0 + item_orders.iter().filter(|v| v.platinum == sell_order.platinum).count();
            }

            println!("{} on {}: LOW {} - YOU {} | POS {}-{}", "[UND]".cyan(), sell_order.item.en.item_name, item_orders[0].platinum, sell_order.platinum, pos.0, pos.1);

            let mut below = Vec::new();
            for i in 0..pos.0 - 1 {
                below.push(item_orders[i].clone());
            }

            scan_results.push(OrderScanReport {
                order: sell_order,
                status: OrderStatus::Undercut { below, pos },
            });
        } else {
            let lowest_orders: Vec<Order> = item_orders.iter().filter(|v| v.platinum == sell_order.platinum).map(|v| v.clone()).collect();
            let self_in_lowest = lowest_orders.iter().any(|v| v.id == sell_order.id);
            let lowest_count = lowest_orders.len();

            if (self_in_lowest && lowest_count == 1) || (!self_in_lowest && lowest_count == 0) {
                let next_order = &item_orders[if self_in_lowest { 1 } else { 0 }];

                println!("{} on {}: YOU {} - NEXT {}", "[LOW]".green(), sell_order.item.en.item_name, sell_order.platinum, next_order.platinum);
                scan_results.push(OrderScanReport {
                    order: sell_order,
                    status: OrderStatus::Lowest { next: next_order.clone() },
                })
            } else {
                println!("{} on {}: YOU {} | POS 1-{}", "[SHR]".yellow(), sell_order.item.en.item_name, sell_order.platinum, lowest_count);
                scan_results.push(OrderScanReport {
                    order: sell_order,
                    status: OrderStatus::SharedLowest,
                })
            }
        }
    }

    if scan_results.is_empty() {
        println!("\nNo issues found with open sell orders!");
        println!("Goodbye!");
        println!("\n\nPress [Enter] to close the program");
        let mut buf = String::new();
        std::io::stdin().read_line(&mut buf);
        return;
    }

    println!("\nFixing {} sell orders", scan_results.len());

    for (idx, report) in scan_results.iter().enumerate() {
        println!("\n{} ({}/{})", report.order.item.en.item_name, idx, scan_results.len());

        match &report.status {
            OrderStatus::Undercut { below, pos } => {
                println!("{}", "Existing orders below you:".cyan());

                for (i, order) in below.iter().enumerate() {
                    println!("{:>6} | PRICE {} | {}", i, order.platinum, order.user.ingame_name);
                }

                println!("{:>6} | PRICE {} | {}", format!("{}-{}", pos.0, pos.1), report.order.platinum, "<--- YOU".red());
            },
            _ => (),
        }
    }
}