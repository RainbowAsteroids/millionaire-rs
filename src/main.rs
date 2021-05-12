use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;
use std::io::{self, Write};
use millionaire::{self, Player, Stock};

fn number_input(prompt: &str) -> Result<usize, io::Error> {
    loop {
        print!("{}", prompt); io::stdout().flush()?;
        let mut choice = String::new();
        io::stdin().read_line(&mut choice)?;
        let choice = choice.trim();

        let choice: usize = match choice.parse() {
            Ok(i) => i,
            Err(_) => {
                println!("`{}` was not a number!\n", choice);
                continue;
            }
        };

        return Ok(choice);
    }
}

fn menu<T: Hash + Display>(options: &[T]) -> Result<&T, io::Error> {
    loop {
        let mut map = HashMap::new();

        for (idx, t) in options.iter().enumerate() {
            let idx = idx + 1;
            map.insert(idx, t);
            println!("{}. {}", idx, t);
        }
        let choice = number_input("Please choose an option: ").expect("IO Error");
        
        return match map.get(&choice) {
            Some(t) => Ok(*t),
            None => {
                println!("Invalid choice, `{}`!\n", choice);
                continue;
            }
        }
    }
}

fn net_worth_breakdown(player: &Player, stocks: &[Stock]) {
    println!("---");
    println!("Balance: {}", player.balance());
    for s in stocks {
        let value = s.value();
        let stock_balance = player.stock_balance(s);
        println!("Stock: '{}', Balance: {}, Value: {}, Worth: {}", s.name(), stock_balance,
                 value, stock_balance * value);
    }
    println!("\nNet worth: {}", player.net_worth(stocks));
    println!("---");
}

fn double_check(prompt: &str, default: bool) -> Result<bool, io::Error> {
    print!("{} {} ", prompt, if default { "(Y/n)" } else { "(y/N)" });
    io::stdout().flush()?;

    let mut choice = String::new();
    io::stdin().read_line(&mut choice)?;
    choice.make_ascii_lowercase();

    if default {
        Ok(!choice.starts_with("n"))
    } else {
        Ok(choice.starts_with("y"))
    }
}

fn main() {
    let mut goal = 1_000_000;
    let mut income = 1000;
    let mut initial_balance = 1000;
    let mut new_stock_cost = 15000;

    loop {
        let options = ["Play game!", "Quit"];
        match *menu(&options).expect("IO error") {
            "Play game!" => {
                let mut run_game = true;
                let initial_income = income;
                let mut player = Player::new(initial_balance, income);
                
                let mut stocks = Vec::new();

                for _ in 0..3 {
                    let name = millionaire::generate_name();
                    let stock = millionaire::generate_stock(stocks.len() as i64, 10, 100, 
                                                            10, 100, name);
                    stocks.push(stock);
                }
                let options = ["Buy stocks", "Sell stocks", "Increase income",
                               "Add a new stock", "Print net worth breakdown", 
                               "End turn", "Quit game"];

                while run_game {
                    for s in stocks.iter_mut() {
                        if s.value() <= 0 {
                            println!("Stock '{}' went bankrupt!", s.name());
                            s.reset();
                            player.reset_stock(s);
                        }
                    }

                    let mut breakdown_printed = false;
                    if player.net_worth(&stocks) > goal {
                        net_worth_breakdown(&player, &stocks);
                        println!("You win!");
                        break;
                    }

                    
                    loop {
                        println!();
                        if !breakdown_printed {
                            net_worth_breakdown(&player, &stocks);
                            breakdown_printed = true;
                        } else {
                            println!("Balance: {}\n", player.balance());
                        }

                        let choice = *menu(&options).expect("IO error");
                        println!();
                    
                        match choice {
                            "Buy stocks" => {
                                let stock = menu(&stocks).expect("IO error");
                                let prompt = format!(
                                        "How much stock would you like to buy? (Max: {}) ",
                                        player.balance() / stock.value());
                                let amount = number_input(&prompt)
                                    .expect("IO Error");
                                if let Err(()) = player.buy_stock(stock, amount as i64) {
                                    println!("You could not afford that much stock.");
                                }
                            }
                            "Sell stocks" => {
                                let stock = menu(&stocks).expect("IO error");
                                let prompt = format!(
                                        "How much stock would you like to sell? (Max: {}) ",
                                        player.stock_balance(stock));
                                let amount = number_input(&prompt)
                                    .expect("IO Error");
                                if let Err(()) = player.sell_stock(stock, amount as i64) {
                                    println!("You do not have enough stock.");
                                }
                            }
                            "Increase income" => {
                                println!("An income increase costs {}.", initial_income * 10);
                                if double_check(
                                    "Are you sure you want to increase your income?", true
                                ).expect("IO Error") {
                                    if let Err(()) = player.increase_income() {
                                        println!("You couldn't afford an income increase.");
                                    }
                                }
                            }
                            "Add a new stock" => {
                                println!("Adding a new stock costs {}", new_stock_cost);
                                if double_check(
                                    "Are you sure you want to unlock a new stock?", true
                                ).expect("IO error") {
                                    if let Err(()) = player.withdraw(new_stock_cost) {
                                        println!("You couldn't afford a new stock.");
                                    } else {
                                        let name = millionaire::generate_name();
                                        let stock = millionaire::generate_stock(
                                            stocks.len() as i64, 10, 100, 10, 100, name);
                                        stocks.push(stock);
                                    }
                                }
                            }
                            "Print net worth breakdown" => { 
                                net_worth_breakdown(&player, &stocks);
                            }
                            "End turn" => { 
                                player.collect_income();
                                break; 
                            }
                            "Quit game" => {
                                if double_check("Are you sure you want to end the game?", 
                                                false).expect("IO Error") {
                                    run_game = false;
                                    break;
                                }
                            }
                            _ => { panic!("unreachable case in game loop"); }
                        }
                    }

                    for s in stocks.iter_mut() {
                        s.vary();
                    }
                }
                println!();
            }
            "Quit" => {
                println!("Goodbye ;(");
                break;
            }
            _ => { panic!("unreachable case in the main loop"); }
        }
    }
}
