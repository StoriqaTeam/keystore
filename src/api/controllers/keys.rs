use super::super::error::*;
use super::super::requests::*;
use super::super::responses::*;
use super::super::utils::{parse_body, response_with_model};
use super::Context;
use super::ControllerFuture;
use failure::Fail;
use futures::prelude::*;
use models::*;
use serde_qs;

pub fn get_keys(ctx: &Context, user_id: UserId) -> ControllerFuture {
    let keys_service = ctx.keys_service.clone();
    let maybe_token = ctx.get_auth_token();
    let path_and_query = ctx.uri.path_and_query();
    let path_and_query_clone = ctx.uri.path_and_query();
    Box::new(
        ctx.uri
            .query()
            .ok_or(ectx!(err ErrorContext::RequestMissingQuery, ErrorKind::BadRequest => path_and_query))
            .and_then(|query| {
                serde_qs::from_str::<GetKeysParams>(query).map_err(|e| {
                    let e = format_err!("{}", e);
                    ectx!(err e, ErrorContext::RequestQueryParams, ErrorKind::BadRequest => path_and_query_clone)
                })
            }).into_future()
            .and_then(move |input| {
                let input_clone = input.clone();
                keys_service
                    .list(maybe_token, user_id, input.offset, input.limit)
                    .map_err(ectx!(convert => input_clone))
            }).and_then(|keys| {
                let keys: Vec<KeyResponse> = keys
                    .iter()
                    .map(|key| KeyResponse {
                        id: key.id.clone(),
                        blockchain_address: key.blockchain_address.clone(),
                        currency: key.currency,
                    }).collect();
                response_with_model(&keys)
            }),
    )
}

pub fn post_keys(ctx: &Context, user_id: UserId) -> ControllerFuture {
    let keys_service = ctx.keys_service.clone();
    let maybe_token = ctx.get_auth_token();
    Box::new(
        parse_body::<PostKeysRequest>(ctx.body.clone())
            .and_then(move |input| {
                let input_clone = input.clone();
                keys_service
                    .create(maybe_token, user_id, input.currency, input.id)
                    .map_err(ectx!(convert => input_clone))
            }).and_then(|key| {
                let Key {
                    blockchain_address,
                    currency,
                    id,
                    ..
                } = key;
                let key_response = KeyResponse {
                    id,
                    blockchain_address,
                    currency,
                };
                response_with_model(&key_response)
            }),
    )
}
