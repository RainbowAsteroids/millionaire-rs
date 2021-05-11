use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;
use std::io::{self, Write};
use millionaire::{self, Stock, Player};

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

fn main() {
    let mut goal = 1_000_000;
    loop {
        let options = ["Play game!", "Quit"];
        match *menu(&options).expect("IO error") {
            "Play game!" => {
                let mut run_game = true;
                let mut player = Player::new(1000, 1000);
                let mut stocks = [
                    Stock::new(0, "Safe stock", 50, 10),
                    Stock::new(1, "Medium stock", 50, 25),
                    Stock::new(2, "Risky stock", 50, 50),
                ];
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
                        println!("You win!");
                        break;
                    }

                    let options = ["Buy stocks", "Sell stocks", 
                                   "Print net worth breakdown", "End turn", "Quit game"];
                    
                    loop {
                        println!();
                        if !breakdown_printed {
                            net_worth_breakdown(&player, &stocks);
                            breakdown_printed = true;
                        } else {
                            println!("Balance: {}\n", player.balance());
                        }
                        
                        match *menu(&options).expect("IO error") {
                            "Buy stocks" => {
                                println!();
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
                                println!();
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
                            "Print net worth breakdown" => { 
                                net_worth_breakdown(&player, &stocks);
                            }
                            "End turn" => { 
                                println!();
                                player.collect_income();
                                break; 
                            }
                            "Quit game" => {
                                print!("Are you sure you want to end the game? (y/N) ");
                                io::stdout().flush().expect("IO Error");
                                let mut choice = String::new();
                                io::stdin().read_line(&mut choice).expect("IO Error");
                                choice.make_ascii_lowercase();

                                if choice.starts_with("y") {
                                    println!();
                                    run_game = false;
                                    break;
                                }
                            }
                            _ => { /* unreachable case */ }
                        }
                    }

                    for s in stocks.iter_mut() {
                        s.vary();
                    }
                }
            }
            "Quit" => {
                println!("Goodbye ;(");
                break;
            }
            _ => { /* unreachable case */ }
        }
    }
}
