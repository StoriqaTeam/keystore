use super::super::requests::*;
use super::super::responses::*;
use super::super::utils::{parse_body, response_with_model};
use super::Context;
use super::ControllerFuture;
use failure::Fail;
use futures::prelude::*;
use models::*;

pub fn post_transactions(ctx: &Context) -> ControllerFuture {
    let transactions_service = ctx.transactions_service.clone();
    let maybe_token = ctx.get_auth_token();
    Box::new(
        parse_body::<PostTransactionsRequest>(ctx.body.clone())
            .and_then(move |input| {
                let input_clone = input.clone();
                let tx: UnsignedTransaction = input.into();
                transactions_service.sign(maybe_token, tx).map_err(ectx!(convert => input_clone))
            }).and_then(|raw_transaction| {
                let transaction_response = PostTransactionsResponse { raw: raw_transaction };
                response_with_model(&transaction_response)
            }),
    )
}
