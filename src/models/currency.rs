use diesel::deserialize::{self, FromSql};
use diesel::pg::Pg;
use diesel::serialize::{self, IsNull, Output, ToSql};
use diesel::sql_types::VarChar;
use std::fmt;
use std::io::Write;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, FromSqlRow, AsExpression, Clone, Copy)]
#[sql_type = "VarChar"]
#[serde(rename_all = "lowercase")]
pub enum Currency {
    Eth,
    Stq,
    Btc,
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Currency::Eth => f.write_str("eth"),
            Currency::Stq => f.write_str("stq"),
            Currency::Btc => f.write_str("btc"),
        }
    }
}

impl FromSql<VarChar, Pg> for Currency {
    fn from_sql(data: Option<&[u8]>) -> deserialize::Result<Self> {
        match data {
            Some(b"eth") => Ok(Currency::Eth),
            Some(b"stq") => Ok(Currency::Stq),
            Some(b"btc") => Ok(Currency::Btc),
            Some(v) => Err(format!(
                "Unrecognized enum variant: {:?}",
                String::from_utf8(v.to_vec()).unwrap_or("Non - UTF8 value".to_string())
            ).to_string()
            .into()),
            None => Err("Unexpected null for non-null column".into()),
        }
    }
}

impl ToSql<VarChar, Pg> for Currency {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        match self {
            Currency::Eth => out.write_all(b"eth")?,
            Currency::Stq => out.write_all(b"stq")?,
            Currency::Btc => out.write_all(b"btc")?,
        };
        Ok(IsNull::No)
    }
}
