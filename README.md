# retry_macro

A library which provides macros that automatically re-execute both synchronous and asynchronous (tokio with sleep) functions upon failure.

## Examples

Here are two simple examples using both the synchronous and asynchronous macros. Note that all function inputs must be **bounded to an identifier** (variable).

### Synchronous

```rust
use retry_macro::{retry, retry_sleep, RetryError};

fn can_fail(input: &str) -> Result<i32, std::num::ParseIntError> {
    input.parse::<i32>()
}

fn main() {
    // Retry the can_fail function 5 times with the input "not_a_number"
    let var = "not_a_number";
    let res = retry!(5, can_fail, var);
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().retries.len(), 5);

    // Retry the can_fail function 5 times with the input "not_a_number", sleep for 100 milliseconds between retries
    let var = "not_a_number";
    let res = retry_sleep!(5, 100, can_fail, var);
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().retries.len(), 5);
}
```

### Asynchronous

```rust
use retry_macro::{retry_async, retry_async_sleep, RetryError};

async fn can_fail_async(input: &str) -> Result<i32, std::num::ParseIntError> {
    tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
    input.parse::<i32>()
}

#[tokio::main]
async fn main() {
    // Retry the can_fail function 5 times with the input "not_a_number"
    let var = "not_a_number";
    let res = retry_async!(5, can_fail_async, var);
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().retries.len(), 5);

    // Retry the can_fail function 5 times with the input "not_a_number", sleep for 100 milliseconds between retries
    let var = "not_a_number";
    let res = retry_async_sleep!(5, 100, can_fail_async, var);
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().retries.len(), 5);
}
```

### License

`retry_macro` is distributed under the [MIT](https://choosealicense.com/licenses/mit/) and [Apache-2.0](https://choosealicense.com/licenses/apache-2.0/) licenses.
