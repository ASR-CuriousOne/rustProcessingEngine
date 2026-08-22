use crate::StreamingLoader;
use crate::models::OhlcvData;
use std::error::Error;
use std::time::Instant;

pub struct Broker {
    pub cash: f64,
    pub asset_balance: f64,
    pub total_trades: u32,
}

impl Broker {
    pub fn new(initial_cash: f64) -> Self {
        Self {
            cash: initial_cash,
            asset_balance: 0.0,
            total_trades: 0,
        }
    }

    pub fn buy(&mut self, price: f64, quantity: f64) {
        let cost = price * quantity;
        if self.cash >= cost {
            self.cash -= cost;
            self.asset_balance += quantity;
            self.total_trades += 1;
        }
    }

    pub fn sell(&mut self, price: f64, quantity: f64) {
        if self.asset_balance >= quantity {
            self.cash += price * quantity;
            self.asset_balance -= quantity;
            self.total_trades += 1;
        }
    }

    pub fn equity(&self, current_price: f64) -> f64 {
        self.cash + (self.asset_balance * current_price)
    }
}

pub trait Strategy {
    fn on_bar(&mut self, bar: &OhlcvData, broker: &mut Broker);
}

pub struct BacktestEngine<'a> {
    pub broker: Broker,
    strategy: &'a mut dyn Strategy,
}

impl<'a> BacktestEngine<'a> {
    pub fn new(initial_capital: f64, strategy: &'a mut dyn Strategy) -> Self {
        Self {
            broker: Broker::new(initial_capital),
            strategy,
        }
    }

    pub fn run(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let start_time = Instant::now();
        let mut final_price = 0.0;
        let mut bars_processed = 0;

        StreamingLoader::stream_ohlcv(file_path, |candle| {
            self.strategy.on_bar(candle, &mut self.broker);

            final_price = candle.close;
            bars_processed += 1;
        })?;

        let final_equity = self.broker.equity(final_price);
        let elapsed = start_time.elapsed();

        self.print_report(bars_processed, final_equity, elapsed);
        Ok(())
    }

    fn print_report(&self, bars_processed: usize, final_equity: f64, elapsed: std::time::Duration) {
        println!("=========================================");
        println!("           BACKTEST COMPLETE             ");
        println!("=========================================");
        println!("Bars Processed: {}", bars_processed);
        println!("Time Taken:     {:.2?}", elapsed);
        println!("Total Trades:   {}", self.broker.total_trades);
        println!("Final Cash:     ${:.2}", self.broker.cash);
        println!("Final Assets:   {:.4} units", self.broker.asset_balance);
        println!("Final Equity:   ${:.2}", final_equity);
        println!("=========================================");
    }
}
