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

        let order_id = self.next_id;
        let original_quantity = order.quantity;

        let mut new_order = Order {
            id: order_id,
            sequence: self.sequence,
            user_id: order.user_id,
            side: order.side,
            price: order.price,
            remaining: order.quantity,
        };

        let trades = self.matching(new_order.side, &mut new_order);

        let remaining = new_order.remaining;
        let filled = original_quantity - remaining;

        if remaining > 0 {
            match new_order.side {
                Side::Buy => {
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
        }

        self.sequence += 1;
        self.next_id += 1;

        let status = if remaining == 0 {
            OrderStatus::Filled
        } else if filled > 0 {
            OrderStatus::PartiallyFilled
        } else {
            OrderStatus::Resting
        };

        Ok(PlaceOutcome {
            order_id,
            trades,
            filled,
            remaining,
            status,
        })
    }

    fn matching(&mut self, side: Side, order: &mut Order) -> Vec<Trade> {
        let mut trades = Vec::new();

        while order.remaining > 0 {
            match side {
                Side::Buy => {
                    let Some(mut entry) = self.asks.first_entry() else {
                        break;
                    };

                    if *entry.key() > order.price {
                        break;
                    }

                    let level = entry.get_mut();
                    let maker = level.front_mut().unwrap();
                    let fill = order.remaining.min(maker.remaining);

                    trades.push(Trade {
                        taker_order_id: order.id,
                        maker_order_id: maker.id,
                        price: maker.price,
                        quantity: fill,
                    });

                    order.remaining -= fill;
                    maker.remaining -= fill;

                    if maker.remaining == 0 {
                        level.pop_front();
                    }

                    if level.is_empty() {
                        entry.remove();
                    }
                }
                Side::Sell => {
                    let Some(mut entry) = self.bids.last_entry() else {
                        break;
                    };

                    if *entry.key() < order.price {
                        break;
                    }

                    let level = entry.get_mut();
                    let maker = level.front_mut().unwrap();
                    let fill = order.remaining.min(maker.remaining);

                    trades.push(Trade {
                        taker_order_id: order.id,
                        maker_order_id: maker.id,
                        price: maker.price,
                        quantity: fill,
                    });

                    order.remaining -= fill;
                    maker.remaining -= fill;

                    if maker.remaining == 0 {
                        level.pop_front();
                    }

                    if level.is_empty() {
                        entry.remove();
                    }
                }
            }
        }

        trades
    }

    pub fn cancel(&mut self, id: OrderId) -> Result<Quantity, EngineError> {
        let bid_price = self
            .bids
            .iter()
            .find(|(_, level)| level.iter().any(|order| order.id == id))
            .map(|(price, _)| *price);

        if let Some(price) = bid_price {
            let level = self.bids.get_mut(&price).unwrap();
            let position = level.iter().position(|order| order.id == id).unwrap();
            let order = level.remove(position).unwrap();
            let remaining = order.remaining;

            if level.is_empty() {
                self.bids.remove(&price);
            }

            return Ok(remaining);
        }

        let ask_price = self
            .asks
            .iter()
            .find(|(_, level)| level.iter().any(|order| order.id == id))
            .map(|(price, _)| *price);

        if let Some(price) = ask_price {
            let level = self.asks.get_mut(&price).unwrap();
            let position = level.iter().position(|order| order.id == id).unwrap();
            let order = level.remove(position).unwrap();
            let remaining = order.remaining;

            if level.is_empty() {
                self.asks.remove(&price);
            }

            return Ok(remaining);
        }

        Err(EngineError::UnknownOrder)
    }

    pub fn best_bid(&self) -> Option<Price> {
        self.bids.last_key_value().map(|(key, _value)| *key)
    }

    pub fn best_ask(&self) -> Option<Price> {
        self.asks.first_key_value().map(|(key, _value)| *key)
    }
}
