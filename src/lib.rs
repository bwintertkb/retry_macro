//! # Overview
//! A library which consists of declarative macros which retry the execution of functions upon failure. Sync and async execution is supported (async via tokio).

use std::{error::Error, fmt::Display};

/// If the function fails after n attempts, the returned error will be stored in the RetryError struct
#[derive(Debug)]
pub struct RetryError<T: Error> {
    /// The returned errors are stored in the retries field
    pub retries: Vec<T>,
}

impl<T: Error> Display for RetryError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Retry Error")
    }
}

impl<T: Error> Error for RetryError<T> {}

/// Retry synchronous function without sleep in between retries. Arguments are: number of retries, function, function arguments.
#[macro_export]
macro_rules! retry {
    ($retries: expr, $f: expr, $($params:tt)* ) => {
        {
            (|| {
            let mut errs = Vec::with_capacity($retries);
            for _ in 0..$retries {
                match $f($($params)*) {
                    Ok(res) => return Ok(res),
                    Err(e) => {
                        errs.push(e);
                    }
                }
            }
            Err(RetryError{retries: errs})
            })()
        }
    };
}

/// Retry synchronous function with sleep in between retries. Arguments are: number of retries, sleep time (milliseconds), function, function arguments.
#[macro_export]
macro_rules! retry_sleep {
    ($retries: expr, $time_ms: expr, $f: expr, $($params:tt)* ) => {
        {
            (|| {
            let mut errs = Vec::with_capacity($retries);
            for _ in 0..$retries {
                match $f($($params)*) {
                    Ok(res) => return Ok(res),
                    Err(e) => {
                        errs.push(e);
                        std::thread::sleep(std::time::Duration::from_millis($time_ms))
                    }
                }
            }
            Err(RetryError{retries: errs})
            })()
        }
    };
}

/// Retry asynchronous function without sleep in between retries. Arguments are: number of retries, function, function arguments.
#[macro_export]
macro_rules! retry_async {
    ($retries: expr, $f: expr, $($params:tt)* ) => {
        {
            let r = (async {
            let mut errs = Vec::with_capacity($retries);
            for _ in 0..$retries {
                match $f($($params)*).await {
                    Ok(res) => return Ok(res),
                    Err(e) => {
                        errs.push(e);
                    }
                }
            }
            Err(RetryError {retries: errs})
            }).await;
            r
        }
    };
}

/// Retry asynchronous function with sleep (enable feature tokio) in between retries. Arguments are: number of retries, sleep time (milliseconds), function, function arguments.
#[macro_export]
#[cfg(feature = "tokio")]
macro_rules! retry_async_sleep {
    ($retries: expr, $time_ms: expr, $f: expr, $($params:tt)* ) => {
        {
            let r = (async {
            let mut errs = Vec::with_capacity($retries);
            for _ in 0..$retries {
                match $f($($params)*).await {
                    Ok(res) => return Ok(res),
                    Err(e) => {
                        errs.push(e);
                        tokio::time::sleep(tokio::time::Duration::from_millis($time_ms)).await;
                    }
                }
            }
            Err(RetryError {retries: errs})
            }).await;
            r
        }
    };
}
#[cfg(test)]
mod tests {
    use std::{error::Error, fmt::Display, time::Instant};

    use super::*;

    fn one_arg(arg1: i32) -> Result<i32, TestError> {
        Ok(arg1)
    }

    fn three_arg(arg1: i32, arg2: i32, arg3: i32) -> Result<i32, TestError> {
        Ok(arg1 + arg2 + arg3)
    }

    fn one_arg_vec(_v: Vec<i32>) -> Result<Vec<i32>, TestError> {
        Err(TestError)
    }

    #[derive(Debug)]
    struct TestError;

    impl Display for TestError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Test error")
        }
    }

    impl Error for TestError {}

    fn failing_function(_arg1: i32, _arg2: i32) -> Result<i32, TestError> {
        Err(TestError)
    }

    #[cfg(feature = "tokio")]
    async fn failing_function_async(_arg1: i32, _arg2: i32) -> Result<i32, TestError> {
        tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
        Err(TestError)
    }

    #[test]
    fn test_success_function_1_arg() {
        let actual = retry!(3, one_arg, 5);
        assert!(actual.is_ok());
        assert_eq!(actual.unwrap(), 5);
    }

    #[test]
    fn test_success_function_3_arg() {
        let actual = retry!(3, three_arg, 5, 10, 20);
        assert!(actual.is_ok());
        assert_eq!(actual.unwrap(), 35);
    }

    #[test]
    fn test_fail_function() {
        let actual = retry!(3, failing_function, 1, 2);
        assert!(actual.is_err());
        assert_eq!(actual.unwrap_err().retries.len(), 3);
    }

    #[test]
    fn test_vec_clone_fail_function() {
        let actual = retry!(5, one_arg_vec, vec![1, 2, 3, 4, 5]);
        assert!(actual.is_err());
        assert_eq!(actual.unwrap_err().retries.len(), 5);
    }

    #[test]
    fn test_fail_function_w_sleep() {
        let start_time = Instant::now();
        let actual = retry_sleep!(3, 100, one_arg_vec, vec![1, 2, 3]);
        let elapsed = start_time.elapsed().as_millis();
        assert!(elapsed >= 300);
        assert!(actual.is_err());
        assert_eq!(actual.unwrap_err().retries.len(), 3);
    }

    #[cfg(feature = "tokio")]
    #[tokio::test]
    async fn test_fail_function_async() {
        let actual = retry_async!(3, failing_function_async, 1, 2);
        assert!(actual.is_err());
        assert_eq!(actual.unwrap_err().retries.len(), 3);
    }

    #[cfg(feature = "tokio")]
    #[tokio::test]
    async fn test_fail_function_async_w_sleep() {
        let start_time = Instant::now();
        let actual = retry_async_sleep!(2, 100, failing_function_async, 1, 2);
        let elapsed = start_time.elapsed().as_millis();
        assert!(elapsed >= 200);
        assert!(actual.is_err());
        assert_eq!(actual.unwrap_err().retries.len(), 2);
    }
}
