use colored::*;
use util::{clear_terminal, unix_timestamp};
use config::prompt;
use wfm_rs::model::UpdateOrderDescriptor;
use wfm_rs::response::{ExistingProfileOrder, Order};
use wfm_rs::shared::OrderType;
use wfm_rs::User;
use crate::util::press_enter_prompt;

mod config;
mod util;
mod cheap_rivens;

const DATA_PATH_SUFFIX: &str = ".wfm_cli/";
const DATA_CONFIG_FILE: &str = "config.wfm.json";
const ITEMS_CACHE_EXPIRY_S: u64 = 24 * 60 * 60;

enum OrderStatus {
    Undercut {
        below: Vec<Order>,
        pos: (usize, usize)
    },
    Lowest {
        above: Vec<Order>,
    },
    SharedLowest {
        /// the amount of orders at the same price, including the user's order
        count: usize,
    }
}

struct OrderScanReport {
    status: OrderStatus,
    order: ExistingProfileOrder,
    solution: Option<OrderSolution>,
}

#[derive(PartialOrd, PartialEq, Eq, Ord)]
enum OrderSolution {
    ChangePrice {
        new_price: u16,
    },
    RemoveOrder,
    MakeOrderPrivate,
    DoNothing,
}

#[tokio::main]
async fn main() {
    if let Some(arg) = std::env::args().nth(1) {
        if arg.to_lowercase() == "rscan" {
            println!("[{}] Running riven scanner", "OK ".green());
            let riven_type = match std::env::args().nth(2) {
                Some(arg) => arg,
                None => {
                    println!("Please specify a riven type (melee/pistol/rifle/kitgun/zaw/shotgun/archgun)");
                    return;
                }
            };

            match cheap_rivens::run(&riven_type).await {
                Ok(_) => {},
                Err(e) => {
                    println!("[{}] {}", "ERR".red(), e);
                    std::process::exit(1);
                }
            }
            return;
        }
    }

    let config = config::run().await.unwrap();
    let user = config.user();

    let existing_orders = user.get_user_orders().await.unwrap();

    let mut scan_results: Vec<OrderScanReport> = Vec::new();

    for sell_order in existing_orders.sell_orders {
        let mut item_orders = user.get_item_orders(&sell_order.item).await.unwrap();
        //item_orders.drain_filter(|v| v.user.status != "ingame" || v.order_type == OrderType::Buy);
        {
            let mut i = 0;
            while i < item_orders.len() {
                if item_orders[i].user.status != "ingame" || item_orders[i].order_type == OrderType::Buy {
                    item_orders.remove(i);
                } else {
                    i += 1;
                }
            }
        }

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
                solution: None
            });
        } else {
            let lowest_orders: Vec<Order> = item_orders.iter().filter(|v| v.platinum == sell_order.platinum).map(|v| v.clone()).collect();
            let self_in_lowest = lowest_orders.iter().any(|v| v.id == sell_order.id);
            let lowest_count = lowest_orders.len() + if self_in_lowest { 0 } else { 1 };

            if lowest_count == 1 {
                // user has the lowest order
                let next_order = &item_orders[if self_in_lowest { 1 } else { 0 }];

                println!("{} on {}: YOU {} - NEXT {}", "[LOW]".green(), sell_order.item.en.item_name, sell_order.platinum, next_order.platinum);
                let above = item_orders.iter().filter(|v| v.id != sell_order.id).map(|v| v.clone()).collect();

                scan_results.push(OrderScanReport {
                    order: sell_order,
                    status: OrderStatus::Lowest { above },
                    solution: None
                })
            } else {
                println!("{} on {}: YOU {} | POS 1-{}", "[SHR]".yellow(), sell_order.item.en.item_name, sell_order.platinum, lowest_count);
                scan_results.push(OrderScanReport {
                    order: sell_order,
                    status: OrderStatus::SharedLowest { count: lowest_count },
                    solution: None
                })
            }
        }
    }

    if scan_results.is_empty() {
        println!("\nNo issues found with open sell orders!");
        println!("Goodbye!");
        press_enter_prompt();
    }

    println!("\nFixing {} sell orders", scan_results.len());

    let scan_results_len = scan_results.len();
    for (idx, report) in scan_results.iter_mut().enumerate() {
        clear_terminal();
        let status_text = match report.status {
            OrderStatus::Undercut { .. } => "UND".cyan(),
            OrderStatus::Lowest { .. } => "LOW".green(),
            OrderStatus::SharedLowest { .. } => "SHR".yellow(),
        };
        println!("\n[{}] {} ({}/{})", status_text, report.order.item.en.item_name, idx, scan_results_len);

        match &report.status {
            OrderStatus::Undercut { below, pos } => {
                println!("{}", "Existing orders below you:".cyan());

                for (i, order) in below.iter().enumerate() {
                    println!("{:>6} | PRICE {} | {}", i+1, order.platinum, order.user.ingame_name);
                }

                println!("{:>6} | PRICE {} | {}", format!("{}-{}", pos.0, pos.1), report.order.platinum, "<--- YOU".red());
            },
            OrderStatus::Lowest { above } => {
                println!("{}", "Existing orders above you:".cyan());

                println!("{:>6} | PRICE {} | {}", "0", report.order.platinum, "<--- YOU".red());
                for (i, order) in above.iter().enumerate() {
                    println!("{:>6} | PRICE {} | {}", i+1, order.platinum, order.user.ingame_name);
                }
            },
            OrderStatus::SharedLowest { count } => {
                println!("{}", format!("Sharing lowest price of {} with {} other orders", report.order.platinum, count).as_str().cyan());
            }
        }

        report.solution = take_order_solution();
    }

    clear_terminal();
    confirm_solutions(&mut scan_results);
    clear_terminal();
    apply_order_solutions(&user, &scan_results).await;
    press_enter_prompt();
}

fn take_order_solution() -> Option<OrderSolution> {
    loop {
        match prompt("Do you want to [C]hange the order price, [R]emove the order, [D]o nothing, or [P]rivate the order?").to_lowercase().as_str() {
            "c" => {
                let mut new_price;
                loop {
                    match prompt("Enter the new price:").parse::<u16>() {
                        Ok(v) => {
                            new_price = v;
                            break;
                        },
                        Err(_) => {
                            println!("Invalid number for price!");
                        }
                    }
                }
                return Some(OrderSolution::ChangePrice { new_price });
            },
            "r" => {
                return Some(OrderSolution::RemoveOrder);
            },
            "p" => {
                return Some(OrderSolution::MakeOrderPrivate);
            },
            "d" => {
                return Some(OrderSolution::DoNothing);
            },
            _ => {
                println!("Invalid option!");
            }
        }
    }
}

fn confirm_solutions(reports: &mut Vec<OrderScanReport>) -> bool {
    reports.sort_by(|a, b| a.solution.cmp(&b.solution));

    let mut i = 0;
    for report in reports {
        if let Some(solution) = &report.solution {
            match solution {
                OrderSolution::ChangePrice { new_price } => {
                    println!("{} price of {} from {} to {}", "Change".cyan(), report.order.item.en.item_name.cyan(), report.order.platinum, new_price);
                },
                OrderSolution::RemoveOrder => {
                    if i == 0 {
                        println!();
                        i = 1;
                    }
                    println!("{} sell order for {}", "Remove".red(), report.order.item.en.item_name.cyan());
                },
                OrderSolution::MakeOrderPrivate => {
                    if i == 1 {
                        println!();
                        i = 2;
                    }
                    println!("Make sell order for {} {}", report.order.item.en.item_name.cyan(), "private".cyan());
                },
                OrderSolution::DoNothing => {
                    if i == 2 {
                        println!();
                    }
                    println!("Do nothing for {}", report.order.item.en.item_name.cyan());
                },
            }
        } else {
            println!("{}", "No solution, this is a bug!".bright_red());
            return false;
        }
    }

    loop {
        match prompt("Is this OK? (y/n)").to_lowercase().as_str() {
            "y" => return true,
            "n" => return false,
            _ => {
                println!("Invalid option!");
            }
        }
    }
}

async fn apply_order_solutions(user: &User, reports: &Vec<OrderScanReport>) {
    for report in reports {
        let mut update_descriptor = UpdateOrderDescriptor {
            platinum: report.order.platinum as u64,
            quantity: report.order.quantity as u16,
            visible: report.order.visible,
            rank: None,
            subtype: None,
        };

        match report.solution.as_ref().unwrap() {
            OrderSolution::ChangePrice { new_price} => update_descriptor.platinum = *new_price as u64,
            OrderSolution::MakeOrderPrivate => update_descriptor.visible = false,
            OrderSolution::RemoveOrder => {
                match user.remove_order(&report.order).await {
                    Ok(_) => println!("[{}] Successfully removed order {}", "OK ".green(), report.order.item.en.item_name.cyan()),
                    Err(e) => println!("[{}] Error removing order {}: {}", "ERR".red(), report.order.item.en.item_name.cyan(), e.to_string().red())
                }
                continue;
            },
            OrderSolution::DoNothing => continue,
        }

        match user.update_order(&report.order, &update_descriptor).await {
            Ok(_) => println!("[{}] Successfully updated order {}", "OK ".green(), report.order.item.en.item_name.cyan()),
            Err(e) => println!("[{}] Error updating order {}: {}", "ERR".red(), report.order.item.en.item_name.cyan(), e.to_string().red())
        }
    }
}