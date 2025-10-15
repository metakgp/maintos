use std::error::Error;

pub(crate) type Res<T> = Result<T, Box<dyn Error>>;
