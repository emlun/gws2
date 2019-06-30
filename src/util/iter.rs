use std::iter::FromIterator;

pub trait CollectOrFirstErr<A, E>
where
    Self: Iterator<Item = Result<A, E>>,
{
    fn collect_or_first_err<B>(&mut self) -> Result<B, E>
    where
        B: FromIterator<A>;
}

impl<I, A, E> CollectOrFirstErr<A, E> for I
where
    I: Iterator<Item = Result<A, E>>,
{
    fn collect_or_first_err<B>(&mut self) -> Result<B, E>
    where
        B: FromIterator<A>,
    {
        let mut buffer: Vec<A> = Vec::new();
        for item in self {
            match item {
                Ok(v) => buffer.push(v),
                Err(e) => return Err(e),
            }
        }
        Ok(buffer.into_iter().collect())
    }
}

#[cfg(test)]
mod test_collect_or_first_err {
    use super::CollectOrFirstErr;

    #[test]
    fn first_err_is_returned() {
        assert_eq!(
            vec![Ok(1), Err(2), Err(3)]
                .into_iter()
                .collect_or_first_err::<Vec<i64>>(),
            Err(2),
        );
    }

    #[test]
    fn collection_is_returned_if_all_are_ok() {
        let v: Vec<Result<i64, i64>> = vec![Ok(1), Ok(2), Ok(3)];
        assert_eq!(
            v.into_iter().collect_or_first_err::<Vec<i64>>(),
            Ok(vec![1, 2, 3]),
        );
    }
}
