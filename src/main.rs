use std::collections::HashMap;
use std::fs;
use std::fmt::Display;
use std::hash::Hash;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process;
use millionaire::{self, Player, Stock};
use millionaire::save::{self, Error, Game};

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

fn menu<T: Hash + Display>(options: &[T], cancel: bool) -> Result<Option<&T>, io::Error> {
    loop {
        let mut map = HashMap::new();

        for (idx, t) in options.iter().enumerate() {
            let idx = idx + 1;
            map.insert(idx, t);
            println!("{}. {}", idx, t);
        }

        if cancel { println!("0. Exit"); }
        let choice = number_input("Please choose an option: ")?;
        
        if cancel && choice == 0 { return Ok(None); }
        return match map.get(&choice) {
            Some(t) => Ok(Some(*t)),
            None => {
                println!("Invalid choice, `{}`!\n", choice);
                continue;
            }
        }
    }
}

fn new_number(name: &str, default: Option<i32>) -> Result<i64, io::Error> {
    let suffix = match default {
        Some(s) => format!("(Default {}) ", s),
        None => "".to_string()
    };
    Ok(number_input(&format!("What will the new '{}' be? {}", name, suffix))? as i64)
}

fn default_or_number(name: &str, default: &str) -> Result<Option<i64>, io::Error> {
    let options = ["New value", default];
    Ok(match *menu(&options, false)?.unwrap() {
        "New value" => Some(new_number(name, None)?),
        _ => None,
    })
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

fn run_game(mut game: Game, save_path: PathBuf) {
    let mut run_game = true;
                
    let options = ["Buy stocks", "Sell stocks", "Increase income",
                    "Add a new stock", "Print net worth breakdown", 
                    "End turn", "Quit game"];

    while run_game {
        save::save(&save_path, &game).unwrap();

        for s in game.stocks.iter_mut() {
            if s.value() <= 0 {
                println!("Stock '{}' went bankrupt!", s.name());
                s.reset();
                game.player.reset_stock(s);
            }
        }

        let mut breakdown_printed = false;
        if game.player.net_worth(&game.stocks) > game.goal {
            net_worth_breakdown(&game.player, &game.stocks);
            println!("You win!");
            break;
        }

        loop {
            println!();
            if !breakdown_printed {
                net_worth_breakdown(&game.player, &game.stocks);
                breakdown_printed = true;
            } else {
                println!("Balance: {}\n", game.player.balance());
            }

            let choice = *menu(&options, false).expect("IO error").unwrap();
            println!();
                    
            match choice {
                "Buy stocks" => {
                    if let Some(stock) = menu(&game.stocks, true).expect("IO error") {
                        let prompt = format!(
                                "How much stock would you like to buy? (Max: {}) ",
                                game.player.balance() / stock.value());
                        let amount = number_input(&prompt)
                            .expect("IO Error");
                        if let Err(()) = game.player.buy_stock(stock, amount as i64) {
                            println!("You could not afford that much stock.");
                        }
                    }
                }
                "Sell stocks" => {
                    if let Some(stock) = menu(&game.stocks, true).expect("IO error") {
                        let prompt = format!(
                                "How much stock would you like to sell? (Max: {}) ",
                                game.player.stock_balance(stock));
                        let amount = number_input(&prompt)
                            .expect("IO Error");
                        if let Err(()) = game.player.sell_stock(stock, amount as i64) {
                            println!("You do not have enough stock.");
                    }
                    }
                }
                "Increase income" => {
                    println!("An income increase costs {}.", game.income_upgrade_cost);
                    if double_check(
                        "Are you sure you want to increase your income?", true
                    ).expect("IO Error") {
                        if let Err(()) = game.player.increase_income(game.income_upgrade_cost) {
                            println!("You couldn't afford an income increase.");
                        }
                    }
                }
                "Add a new stock" => {
                    println!("Adding a new stock costs {}", game.add_stock_cost);
                    if double_check(
                        "Are you sure you want to unlock a new stock?", true
                    ).expect("IO error") {
                        if let Err(()) = game.player.withdraw(game.add_stock_cost) {
                            println!("You couldn't afford a new stock.");
                        } else {
                            let name = millionaire::generate_name();
                            let stock = millionaire::generate_stock(
                                game.stocks.len() as i64, 10, 100, 10, 100, name);
                            game.stocks.push(stock);
                        }
                    }
                }
                "Print net worth breakdown" => { 
                    net_worth_breakdown(&game.player, &game.stocks);
                }
                "End turn" => { 
                    game.player.collect_income();
                    break; 
                }
                "Quit game" => {
                    if double_check("Are you sure you want to end the game?", 
                                    false).expect("IO Error") {
                        run_game = false;
                        break;
                    }
                }
                _ => { panic!("unreachable arm in game loop"); }
            }
        }

        for s in game.stocks.iter_mut() {
            s.vary();
        }
    }
    println!();
}

fn main() {
    let path = None;
    
    loop {
        match save::saves_in_folder(path) {
            Ok(_) => {
                break;
            }
            Err(Error::NotFound(p)) => {
                match fs::create_dir(p) {
                    Ok(_) => continue,
                    Err(_) => {
                        eprintln!("A save folder cannot be created.");
                        process::exit(1);
                    }
                }
            }
            Err(Error::PlatformNotSupported) => {
                eprintln!("A save folder cannot be found for this platform.");
                process::exit(1);
            }
            Err(_) => panic!("IO Error"),
        }
    }

    let mut goal = 1_000_000;
    let mut income = 1000;
    let mut initial_balance: Option<i64> = None;
    let mut add_stock_cost = 15000;
    let mut starting_stocks = 3;
    let mut income_upgrade_cost: Option<i64> = None;

    loop {
        let options = ["Play game!", "Load save", "Manage saves", "Edit variables", "Quit"];
        
        let choice = *menu(&options, false).expect("IO error").unwrap();
        println!();

        match choice {
            "Play game!" => {
                let mut stocks = Vec::new();

                for _ in 0..starting_stocks {
                    let name = millionaire::generate_name();
                    let stock = millionaire::generate_stock(stocks.len() as i64, 10, 100, 
                                                            10, 100, name);
                    stocks.push(stock);
                }

                run_game(Game {
                    stocks,
                    player: Player::new(
                        match initial_balance {
                            Some(i) => i,
                            None => income,
                        }, 
                        income
                    ),
                    goal,
                    initial_income: income,
                    add_stock_cost,
                    income_upgrade_cost: match income_upgrade_cost {
                        Some(i) => i,
                        None => income * 10,
                    }
                },
                save::make_path(path).unwrap());
            }
            "Load save" => {
                // Safe unwrap because we verified this function works eariler
                let saves = save::saves_in_folder(path).unwrap();
                if saves.len() == 0 {
                    println!("There are no saved games.");
                } else {
                    let save = menu(&saves, true).expect("IO Error");
                    if let Some(save) = save {
                        let path = &save.path;
                        match save::from_path(path) {
                            Ok(g) => {
                                run_game(g, path.to_path_buf());
                            }
                            Err(_e) => panic!(),
                        }
                    }
                }
            },
            "Manage saves" => {
                // Safe unwrap because we verified this function works eariler
                let saves = save::saves_in_folder(path).unwrap();
                if saves.len() == 0 {
                    println!("There are no saved games.");
                } else {
                    let save = menu(&saves, true).expect("IO Error");
                    if let Some(save) = save {
                        let options = ["Copy save", "Delete save", "Rename save"];
                        if let Some(choice) = menu(&options, true).expect("IO Error") {
                            match *choice {
                                "Copy save" => {
                                    if let Err(_) = save::copy(&save.path) {
                                        println!("There was an error copying the save file!");
                                    }
                                }
                                "Delete save" => {
                                    if let Err(_) = save::delete(&save.path) {
                                        println!("There was an error removing the save file!");
                                    }
                                }
                                "Rename save" => {
                                    let mut new_name = String::new();
                                    print!("What will the new name of the save be? ");
                                    io::stdout().flush().expect("IO Error");
                                    io::stdin().read_line(&mut new_name).expect("IO Error");

                                    match save::rename(&save.path, &new_name) {
                                        Ok(_) => {
                                            println!("Save file renamed!");
                                        }
                                        Err(save::Error::AlreadyExists) => {
                                            println!("A save with the same name already exists!");
                                        }
                                        Err(save::Error::EmptyFileName) => {
                                            println!("That filename was empty.");
                                        }
                                        Err(_) => {
                                            println!("Issue renaming the file.");
                                        }
                                    }
                                }
                                _ => panic!("unreachable arm in manage saves"),
                            }
                        }
                    }
                }
            },
            "Edit variables" => {
                let options = ["Change goal", "Change income", "Change initial balance",
                               "Change add stock cost", "Change number of starting stocks",
                               "Change income upgrade cost"];
                
                match *menu(&options, false).expect("IO Error").unwrap() {
                    "Change goal" => {
                        goal = new_number("goal", Some(1_000_000)).expect("IO Error");
                    },
                    "Change income" => {
                        income = new_number("income", Some(1000)).expect("IO Error");
                    },
                    "Change initial balance" => {
                        initial_balance = default_or_number("initial balance", "Same as income").expect("IO Error");
                    },
                    "Change add stock cost" => {
                        add_stock_cost = new_number("add stock cost", Some(15000)).expect("IO Error");
                    },
                    "Change number of starting stocks" => {
                        starting_stocks = new_number("number of starting stocks", Some(3)).expect("IO Error");
                    },
                    "Change income upgrade cost" => {
                        income_upgrade_cost = default_or_number("income upgrade cost", "Ten times initial income").expect("IO Error");
                    },
                    _ => panic!("unreachable arm in edit variables option"),
                }
            },
            "Quit" => {
                println!("Goodbye ;(");
                break;
            }
            _ => panic!("unreachable arm in the main loop"),
        }
    }
}
