use std::collections::HashMap;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::fmt::{self, Display, Formatter};
use rand::Rng;

pub struct Stock {
    direction: i64,
    id: i64,
    initial_value: i64,
    name: String,
    value: i64,
    variation: i64,
}

impl Stock {
    /// Generates a new stock.
    pub fn new(id: i64, name: String, value: i64, variation: i64) -> Self {
        Self { direction: 0, id, initial_value: value, name, value, variation }
    }

    /// Getter for the current value of the stock.
    pub fn value(&self) -> i64 { self.value }

    /// Getter for the stock's name
    pub fn name(&self) -> &str { &self.name }

    /// Getter for the stock's id
    pub fn id(&self) -> i64 { self.id }

    /// Varies the value of the stock.
    pub fn vary(&mut self) {
        let random = rand::thread_rng().gen_range(-self.variation..=self.variation);
        // ((x * 3) / 5) == x * 0.6, but no need to cast twice
        self.direction = ((self.direction * 3)/5) + random;
        self.value += self.direction;
    }

    /// Resets the value and balance of the stock. Used when the stock value reaches or 
    /// is less than 0.
    pub fn reset(&mut self) { 
        self.value = self.initial_value;
        self.direction = 0;
    }
}

impl Hash for Stock {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.name.hash(state);
    }
}

impl Ord for Stock {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

impl PartialOrd for Stock {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Stock {
    fn eq(&self, other: &Self) -> bool {
        (self.id == other.id) && (self.name == other.name)
    }
}

impl Eq for Stock {}

pub fn generate_name() -> String {
    let first_names = [
        "Trading", "Rainbow", "Cake", "Power", "Mining", "Spacecraft", "Cargo", "Crab", 
        "Dining", "Computer", "Game", "Security", "Block", "Micro", "Time",
    ];
    let last_names = [
        "Incorporated", "Enterprise", "Solutions", "Company", "Operations", "Factory",
        "Agency", "Firm", "Chain", "Box", "Store", "Market",
    ];

    let first_name = first_names[rand::thread_rng().gen_range(0..first_names.len())];
    let last_name = last_names[rand::thread_rng().gen_range(0..last_names.len())];

    format!("{} {}", first_name, last_name)
}

pub fn generate_stock(id: i64, min_value: i64, max_value: i64, min_variation: i64, 
                      max_variation: i64, name: String) -> Stock {
    let value = rand::thread_rng().gen_range(min_value..=max_value);
    let variation = rand::thread_rng().gen_range(min_variation..=max_variation);

    Stock::new(id, name, value, variation)
}

impl Display for Stock {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}, Value: {}", self.name, self.value)
    }
}

pub struct Player {
    balance: i64,
    income: i64,
    initial_income: i64,
    stock_balances: HashMap<i64, i64>,
}

impl Player {
    /// Generates a new `Player`.
    pub fn new(balance: i64, income: i64) -> Self {
        Self { balance, income, initial_income: income, stock_balances: HashMap::new() }
    }

    /// Getter for the balance
    pub fn balance(&self) -> i64 { self.balance }
    
    /// Gets the amount of stock a player owns
    pub fn stock_balance(&self, stock: &Stock) -> i64 {
        if let Some(b) = self.stock_balances.get(&stock.id()) {
            return *b;
        } else {
            return 0;
        }
    }

    /// Getter for the income
    pub fn income(&self) -> i64 { self.income }

    /// Purchases a stock. Returns `Err(())` if the player had too low of a balance.
    pub fn buy_stock(&mut self, stock: &Stock, amount: i64) -> Result<(), ()> {
        let cost = stock.value() * amount;
        if i64::from(self.balance) < cost { return Err(()) }
        self.balance -= cost;
        let stock_balance = self.stock_balance(stock);
        self.stock_balances.insert(stock.id(), stock_balance + amount);
        Ok(())
    }

    /// Sells a stock. Returns `Err(())` if the player doesn't have enough stock to sell.
    pub fn sell_stock(&mut self, stock: &Stock, amount: i64) -> Result<(), ()> {
        let bal = self.stock_balance(stock);
        if bal < amount { return Err(()) }
        self.stock_balances.insert(stock.id(), bal - amount);
        self.balance += stock.value() * amount;
        Ok(())
    }

    /// Resets a stock balance back to 0.
    pub fn reset_stock(&mut self, stock: &Stock) {
        self.stock_balances.insert(stock.id(), 0);
    }

    /// Increment the balance by the player's income.
    pub fn collect_income(&mut self) { self.balance += self.income }

    /// Increases the income of the player by the initial income amount for the cost of 
    /// 10 times the initial income. Returns an Err(()) if the player didn't have enough
    /// money to increase their income.
    pub fn increase_income(&mut self) -> Result<(), ()> { 
        let cost = self.initial_income * 10;
        if cost > self.balance { return Err(()); }

        self.income += self.initial_income;
        self.balance -= cost;
        Ok(()) 
    }

    /// Returns the balance of the player plus the worth of the player's owned
    /// stock.
    pub fn net_worth(&self, stocks: &[Stock]) -> i64 {
        let mut result = self.balance;
        for s in stocks { result += s.value() * self.stock_balance(s) }
        result
    }

    /// Remove an arbitrary amount of money from the player's balance. Should only be 
    /// used when no other method applies (or when the Player struct has no other state
    /// to manipulate).
    pub fn withdraw(&mut self, amount: i64) -> Result<(), ()> {
        if self.balance < amount { return Err(()); }
        self.balance -= amount;
        Ok(())
    }

    /// Add an arbitrary amount of money to the player's balance. Should only be used
    /// when no other method applies (or when the Player struct has no other state to 
    /// manipulate).
    pub fn deposit(&mut self, amount: i64) { self.balance += amount; }
}

