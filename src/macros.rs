use std::fmt::Debug;

pub trait IntoResult {
    type Ok;
    type Err: Debug;

    fn into_result(self) -> Result<Self::Ok, Option<Self::Err>>;
}

impl<T, E: Debug> IntoResult for Result<T, E> {
    type Err = E;
    type Ok = T;

    fn into_result(self) -> Result<T, Option<E>> {
        self.map_err(Some)
    }
}

impl<T> IntoResult for Option<T> {
    type Err = ();
    type Ok = T;

    fn into_result(self) -> Result<T, Option<()>> {
        self.ok_or(None)
    }
}

#[macro_export]
macro_rules! unwrap_or_return {
    ($e:expr) => {
        unwrap_or_return!($e, |_| {}, {})
    };
    ($e:expr, $handle:expr) => {
        unwrap_or_return!(
            $e,
            |e| {
                $handle(Some(e));
            },
            {
                $handle(None);
            }
        )
    };
    ($e:expr, $error_handle:expr, $none_handle:block) => {
        match $crate::macros::IntoResult::into_result($e) {
            Ok(v) => v,
            Err(Some(e)) => {
                $error_handle(&e);
                return;
            }
            Err(None) => {
                $none_handle;
                return;
            }
        }
    };
}
