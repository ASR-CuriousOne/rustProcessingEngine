use crate::models::OhlcvData;
use crate::{Broker, Strategy};

pub struct EMAStrategy {
    current_ema: Option<f64>,
    alpha: f64,
}

impl EMAStrategy {
    pub fn new(period: usize) -> Self {
        Self {
            current_ema: None,
            alpha: 2.0 / ((period as f64) + 1.0),
        }
    }
}

impl Strategy for EMAStrategy {
    fn on_bar(&mut self, bar: &OhlcvData, broker: &mut Broker) {
        if let Some(ema) = self.current_ema {
            let new_ema = (bar.close * self.alpha) + (ema * (1.0 - self.alpha));
            self.current_ema = Some(new_ema);

            if bar.close > new_ema && broker.asset_balance == 0.0 {
                broker.buy(bar.close, 1.0);
            } else if bar.close < new_ema && broker.asset_balance > 0.0 {
                broker.sell(bar.close, broker.asset_balance);
            }
        } else {
            self.current_ema = Some(bar.close);
        }
    }
}

pub struct BuyOnGreenStrategy;

impl Strategy for BuyOnGreenStrategy {
    fn on_bar(&mut self, bar: &OhlcvData, broker: &mut Broker) {
        if bar.is_bullish() {
            broker.buy(bar.close, 1.0);
        }
    }
}
