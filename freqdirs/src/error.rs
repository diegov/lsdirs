pub fn transpose<T, E>(v: Option<Result<T, E>>) -> Result<Option<T>, E> {
    if let Some(v) = v {
        match v {
            Ok(v) => Ok(Some(v)),
            Err(e) => Err(e),
        }
    } else {
        Ok(None)
    }
}
