pub type OrderId = u64;
pub type Price = u64;
pub type Quantity = u64;
pub type UserId = u64;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, PartialEq)]
pub enum OrderStatus {
    Filled,
    PartiallyFilled,
    Resting,
}

#[derive(Debug)]
pub enum EngineError {
    InvalidQuantity,
    InvalidPrice,
    SelfTrade,
    UnknownOrder,
}

pub struct NewOrder {
    pub user_id: UserId,
    pub side: Side,
    pub price: Price,
    pub quantity: Quantity,
}

#[derive(Debug)]
pub struct Trade {
    pub maker_order_id: OrderId,
    pub taker_order_id: OrderId,
    pub price: Price,
    pub quantity: Quantity,
}

#[derive(Debug)]
pub struct PlaceOutcome {
    pub order_id: OrderId,
    pub trades: Vec<Trade>,
    pub filled: Quantity,
    pub remaining: Quantity,
    pub status: OrderStatus,
}

pub(crate) struct Order {
    pub(crate) id: OrderId,
    pub(crate) user_id: UserId,
    pub(crate) side: Side,
    pub(crate) price: Price,
    pub(crate) remaining: Quantity,
    pub(crate) sequence: u64,
}
