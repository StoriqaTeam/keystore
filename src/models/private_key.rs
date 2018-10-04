use super::currency::Currency;
use super::user::UserId;
use uuid::Uuid;

pub struct PrivateKeyId(Uuid);

pub struct PrivateKey {
    id: PrivateKeyId,
    data: Vec<u8>,
    currency: Currency,
    owner: UserId,
}
