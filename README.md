# Airinfo

#### A library to read battery and status info from Airpods and Beats

## Example 
```rust
use airinfo::find_pods;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pods = find_pods().await?;
    for pod in pods {
        println!("POD: {:#?}", pod);
    }
    Ok(())
}
```
