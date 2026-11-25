use rome::engine::{
    OrderBook,
    types::{NewOrder, OrderStatus, Side},
};

#[test]
fn simple_match() {
    let mut order_book = OrderBook::new();

    let dummy_sell = NewOrder {
        user_id: 1,
        side: Side::Sell,
        price: 100,
        quantity: 100,
    };
    let place_sell = order_book.place(dummy_sell);

    let dummy_buy = NewOrder {
        user_id: 2,
        side: Side::Buy,
        price: 100,
        quantity: 100,
    };
    let place_buy = order_book.place(dummy_buy);

    let buy_result = place_buy.ok().unwrap();

    assert_eq!(buy_result.trades.len(), 1);
    assert_eq!(buy_result.trades[0].price, 100);
    assert_eq!(buy_result.trades[0].quantity, 100);
    assert_eq!(buy_result.filled, 100);
    assert_eq!(buy_result.remaining, 0);
    assert_eq!(buy_result.status, OrderStatus::Filled);

    assert_eq!(order_book.best_bid(), None);
    assert_eq!(order_book.best_ask(), None);
}
