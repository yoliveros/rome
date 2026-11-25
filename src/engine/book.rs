use std::collections::{BTreeMap, VecDeque};

use crate::engine::types::Trade;

use super::types::{
    EngineError, NewOrder, Order, OrderId, OrderStatus, PlaceOutcome, Price, Quantity, Side,
};

pub struct OrderBook {
    bids: BTreeMap<Price, VecDeque<Order>>,
    asks: BTreeMap<Price, VecDeque<Order>>,
    next_id: OrderId,
    sequence: u64,
}

impl OrderBook {
    pub fn new() -> Self {
        Self {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            next_id: 1,
            sequence: 0,
        }
    }

    pub fn place(&mut self, order: NewOrder) -> Result<PlaceOutcome, EngineError> {
        if order.quantity == 0 {
            return Err(EngineError::InvalidQuantity);
        }

        if order.price == 0 {
            return Err(EngineError::InvalidPrice);
        }

        let mut new_order = Order {
            id: self.next_id,
            sequence: self.sequence,
            user_id: order.user_id,
            side: order.side,
            price: order.price,
            remaining: order.quantity,
        };

        match new_order.side {
            Side::Buy => {
                let trades = self.matching(Side::Buy, &mut new_order);

                self.bids
                    .entry(new_order.price)
                    .or_default()
                    .push_back(new_order);
            }
            Side::Sell => {
                let trades = self.matching(Side::Sell, &mut new_order);

                self.asks
                    .entry(new_order.price)
                    .or_default()
                    .push_back(new_order);
            }
        }

        self.sequence += 1;
        let curr_id = self.next_id;
        self.next_id += 1;

        Ok(PlaceOutcome {
            order_id: curr_id,
            trades: vec![],
            filled: 0,
            remaining: order.quantity,
            status: OrderStatus::Resting,
        })
    }

    fn matching(&mut self, side: Side, order: &mut Order) -> Vec<Trade> {
        match side {
            Side::Buy => {
                if order.remaining <= 0 {
                    return vec![];
                }
                if let Some(mut entry) = self.asks.first_entry() {
                    if entry.key() <= &order.price {
                        let level = entry.get_mut();
                        let maker = level.front_mut().unwrap();
                        let fill = order.remaining.min(maker.remaining);

                        let new_trade = Trade {
                            taker_order_id: order.id,
                            maker_order_id: maker.id,
                            price: maker.price,
                            quantity: fill,
                        };

                        order.remaining -= fill;
                        maker.remaining -= fill;

                        if maker.remaining <= 0 {
                            level.pop_front();

                            if level.is_empty() {
                                entry.remove();
                            }
                        }

                        return vec![new_trade];
                    }
                }

                return vec![];
            }
            Side::Sell => {
                let best_bid = self.bids.last_entry();
                return vec![];
            }
        }
    }

    pub fn cancel(&mut self, id: OrderId) -> Result<Quantity, EngineError> {
        todo!()
    }

    pub fn best_bid(&self) -> Option<Price> {
        self.bids.last_key_value().map(|(key, _value)| *key)
    }

    pub fn best_ask(&self) -> Option<Price> {
        self.asks.first_key_value().map(|(key, _value)| *key)
    }
}
