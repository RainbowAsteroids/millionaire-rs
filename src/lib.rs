use std::collections::HashMap;
use std::cmp::Ordering;
use std::fmt::{self, Display, Formatter};
use std::hash::{Hash, Hasher};
use rand::Rng;

pub struct Stock<'a> {
    id: i64,
    initial_value: i64,
    name: &'a str,
    value: i64,
    variation: i64,
}

impl<'a> Stock<'a> {
    /// Generates a new stock.
    pub fn new(id: i64, name: &'a str, value: i64, variation: i64) -> Self {
        Self { id, initial_value: value, name, value, variation }
    }

    /// Getter for the current value of the stock.
    pub fn value(&self) -> i64 { self.value }

    /// Getter for the stock's name
    pub fn name(&self) -> &str { self.name }

    /// Getter for the stock's id
    pub fn id(&self) -> i64 { self.id }

    /// Varies the value of the stock and returns how much the stock was varied.
    pub fn vary(&mut self) -> i64 {
        let variation = self.variation;
        let vary = rand::thread_rng().gen_range(-variation..variation);
        self.value += vary;
        vary
    }

    /// Resets the value and balance of the stock. Used when the stock value reaches or 
    /// is less than 0.
    pub fn reset(&mut self) { self.value = self.initial_value; }
}

impl<'a> Hash for Stock<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.name.hash(state);
    }
}

impl<'a> Ord for Stock<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

impl<'a> PartialOrd for Stock<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> PartialEq for Stock<'a> {
    fn eq(&self, other: &Self) -> bool {
        (self.id == other.id) && (self.name == other.name)
    }
}

impl<'a> Eq for Stock<'a> {}

impl<'a> Display for Stock<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}, Value: {}", self.name, self.value)
    }
}

pub struct Player {
    balance: i64,
    income: i64,
    stock_balances: HashMap<i64, i64>,
}

impl Player {
    /// Generates a new `Player`.
    pub fn new(balance: i64, income: i64) -> Self {
        Self { balance, income, stock_balances: HashMap::new() }
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

    pub fn reset_stock(&mut self, stock: &Stock) {
        self.stock_balances.insert(stock.id(), 0);
    }

    /// Increment the balance by the player's income.
    pub fn collect_income(&mut self) { self.balance += self.income }

    /// Returns the balance of the player plus the worth of the player's owned
    /// stock.
    pub fn net_worth(&self, stocks: &[Stock]) -> i64 {
        let mut result = self.balance;
        for s in stocks { result += s.value() * self.stock_balance(s) }
        result
    }
}

