use super::super::requests::*;
use super::super::responses::*;
use super::super::utils::{parse_body, response_with_model};
use super::Context;
use super::ControllerFuture;
use failure::Fail;
use futures::prelude::*;

pub fn get_keys(ctx: &Context) -> ControllerFuture {
    unimplemented!()
    // let users_service = ctx.users_service.clone();
    // Box::new(
    //     parse_body::<PostSessionsRequest>(ctx.body.clone())
    //         .and_then(move |input| {
    //             let input_clone = input.clone();
    //             users_service
    //                 .get_jwt(input.email, input.password)
    //                 .map_err(ectx!(catch => input_clone))
    //         }).and_then(|jwt| {
    //             let model = PostSessionsResponse { token: jwt };
    //             response_with_model(&model)
    //         }),
    // )
}

pub fn post_keys(ctx: &Context) -> ControllerFuture {
    unimplemented!()
    // let users_service = ctx.users_service.clone();
    // Box::new(
    //     parse_body::<PostSessionsOauthRequest>(ctx.body.clone())
    //         .and_then(move |input| {
    //             let input_clone = input.clone();
    //             users_service
    //                 .get_jwt_by_oauth(input.oauth_token, input.oauth_provider)
    //                 .map_err(ectx!(catch => input_clone))
    //         }).and_then(|jwt| {
    //             let model = PostSessionsResponse { token: jwt };
    //             response_with_model(&model)
    //         }),
    // )
}
