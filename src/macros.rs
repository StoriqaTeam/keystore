macro_rules! ectx {
    (err $e:expr $(,$context:expr)* $(=> $($arg:expr),*)*) => {{
        let mut msg = "at ".to_string();
        msg.push_str(&format!("{}:{}", file!(), line!()));
        $(
            $(
                let arg = format!("\nwith args - {}: {:#?}", stringify!($arg), $arg);
                msg.push_str(&arg);
            )*
        )*
        let err = $e.context(msg);
        $(
            let err = err.context($context);
        )*
        err.into()
    }};

    (catch err $e:expr $(,$context:expr)* $(=> $($arg:expr),*)*) => {{
        let e = $e.kind().into();
        ectx!(err $e $(,$context)*, e $(=> $($arg),*)*)
    }};


    (catch $($context:expr),* $(=> $($arg:expr),*)*) => {{
        move |e| {
            ectx!(catch err e $(,$context)* $(=> $($arg),*)*)
        }
    }};

    ($($context:expr),* $(=> $($arg:expr),*)*) => {{
        move |e| {
            ectx!(err e $(,$context)* $(=> $($arg),*)*)
        }
    }};
}

macro_rules! derive_newtype_sql {
    ($mod_name:ident, $sql_type:ty, $type:ty, $constructor:expr) => {
        mod $mod_name {
            use super::*;
            use diesel::deserialize::{self, FromSql};
            use diesel::pg::Pg;
            use diesel::serialize::{self, Output, ToSql};
            use std::io::Write;

            impl FromSql<$sql_type, Pg> for $type {
                fn from_sql(data: Option<&[u8]>) -> deserialize::Result<Self> {
                    FromSql::<$sql_type, Pg>::from_sql(data).map($constructor)
                }
            }

            impl ToSql<$sql_type, Pg> for $type {
                fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
                    ToSql::<$sql_type, Pg>::to_sql(&self.0, out)
                }
            }
        }
    };
}

macro_rules! mask_logs {
    ($type:ty) => {
        impl Debug for $type {
            fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
                f.write_str("********")
            }
        }

        impl Display for $type {
            fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
                f.write_str("********")
            }
        }
    };
}

macro_rules! derive_error_impls {
    () => {
        #[allow(dead_code)]
        impl Error {
            pub fn kind(&self) -> ErrorKind {
                self.inner.get_context().clone()
            }
        }

        impl Fail for Error {
            fn cause(&self) -> Option<&Fail> {
                self.inner.cause()
            }

            fn backtrace(&self) -> Option<&Backtrace> {
                self.inner.backtrace()
            }
        }

        impl Display for Error {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                Display::fmt(&self.inner, f)
            }
        }

        impl From<ErrorKind> for Error {
            fn from(kind: ErrorKind) -> Error {
                Error {
                    inner: Context::new(kind),
                }
            }
        }

        impl From<Context<ErrorKind>> for Error {
            fn from(inner: Context<ErrorKind>) -> Error {
                Error { inner: inner }
            }
        }
    };
}
